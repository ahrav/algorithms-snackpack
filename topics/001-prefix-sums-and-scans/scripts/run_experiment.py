#!/usr/bin/env python3
"""Run the frozen prefix-scan benchmark protocol and retain raw evidence."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import math
import os
import platform
import random
import shutil
import signal
import socket
import statistics
import subprocess
import sys
import time
import tomllib
import traceback
import uuid
from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any, Iterable, Sequence


SCHEMA_VERSION = 1
ASSIGNMENT_SEED = 2_026_082_901
FAMILY_ALPHA = 0.05
FAMILY_SIZE = 12
PRACTICAL_RATIO = 1.05
PILOT_BLOCKS = 4
MAIN_BLOCKS = 12
AA_BLOCKS = 12
BLOCK_SIZE = 16_384
DEFAULT_WARMUPS = 1
DEFAULT_SAMPLES = 3
DEFAULT_TIMEOUT_SECONDS = 120.0
# Windows lacks SIGHUP; name-based lookup keeps this module importable.
EXTERNAL_SIGNALS = {
    resolved
    for name in ("SIGINT", "SIGTERM", "SIGHUP")
    if (resolved := getattr(signal, name, None)) is not None
}

SCRIPT = Path(__file__).resolve()
TOPIC_DIR = SCRIPT.parents[1]
REPO_ROOT = SCRIPT.parents[3]
RUNS_DIR = TOPIC_DIR / "measurements" / "runs"


@dataclass(frozen=True)
class Contrast:
    contrast_id: str
    b_algorithm: str
    n: int
    block_size: int = 1
    workers: int = 1


CONTRASTS = (
    Contrast("ref_linear_n64", "reference", 64),
    Contrast("ref_linear_n512", "reference", 512),
    Contrast("ref_linear_n4096", "reference", 4_096),
    Contrast("blocked_linear_n4096_b16384", "blocked", 4_096, BLOCK_SIZE),
    Contrast("blocked_linear_n262144_b16384", "blocked", 262_144, BLOCK_SIZE),
    Contrast("blocked_linear_n4194304_b16384", "blocked", 4_194_304, BLOCK_SIZE),
    Contrast("parallel2_linear_n262144", "parallel", 262_144, workers=2),
    Contrast("parallel4_linear_n262144", "parallel", 262_144, workers=4),
    Contrast("parallel8_linear_n262144", "parallel", 262_144, workers=8),
    Contrast("parallel2_linear_n4194304", "parallel", 4_194_304, workers=2),
    Contrast("parallel4_linear_n4194304", "parallel", 4_194_304, workers=4),
    Contrast("parallel8_linear_n4194304", "parallel", 4_194_304, workers=8),
)

AA_CONTRAST = Contrast("aa_linear_n262144", "linear", 262_144)
AUDIT_PATTERNS = ("zero", "constant", "ascending", "alternating", "mixed")
AUDIT_ALGORITHMS = (
    ("linear", 1, 1),
    ("blocked", BLOCK_SIZE, 1),
    ("parallel", 1, 4),
)


@dataclass(frozen=True)
class Position:
    phase: str
    contrast_id: str
    block_index: int
    template: str
    position_index: int
    label: str
    algorithm: str
    n: int
    pattern: str
    seed: int
    block_size: int
    workers: int
    retry_index: int = 0


@dataclass
class AttemptResult:
    attempt_id: str
    phase: str
    contrast_id: str
    block_index: int
    retry_index: int
    template: str
    position_index: int
    label: str
    algorithm: str
    n: int
    pattern: str
    seed: int
    block_size: int
    workers: int
    status: str
    invalid_reason: str
    external_interruption: bool
    returncode: int | None
    timed_out: bool
    position_ns: int | None
    input_checksum: str
    output_checksum: str
    executable_sha256_before: str
    executable_sha256_after: str
    attempt_directory: str


class ExperimentIncomplete(RuntimeError):
    """The fixed schedule could not produce every required complete block."""


class ExternalSignalRelay:
    """Turn external signals delivered to the runner into recorded child outcomes.

    The declared retry rule acts on a harness child terminated by SIGINT,
    SIGTERM, or SIGHUP, but those signals often land on the runner instead:
    a terminal Ctrl-C targets the whole process group, and automation
    usually signals the runner pid. Without a handler, SIGTERM and SIGHUP
    terminate Python outright and SIGINT raises KeyboardInterrupt out of
    the attempt in flight, so the started attempt is never recorded and the
    whole-block retry cannot happen. The relay forwards the signal to the
    active child, whose recorded exit then drives the normal
    record-and-retry path. A signal that arrives with no child in flight is
    held and raised as ExperimentIncomplete at the next safe checkpoint, so
    the run ends with a recorded INCOMPLETE status instead of vanishing.
    The caller keeps the relay installed through final status and manifest
    writing; restoring default handlers earlier lets a late signal terminate
    the runner mid-manifest and leave the bundle unverified.
    """

    def __init__(self) -> None:
        self.child: subprocess.Popen[bytes] | None = None
        self.pending: list[int] = []
        self._previous: dict[int, Any] = {}

    def install(self) -> None:
        for signum in EXTERNAL_SIGNALS:
            self._previous[int(signum)] = signal.signal(signum, self._relay)

    def restore(self) -> None:
        for signum, previous in self._previous.items():
            signal.signal(signum, previous)
        self._previous.clear()

    def _relay(self, signum: int, frame: Any) -> None:
        del frame
        child = self.child
        if child is not None and child.poll() is None:
            try:
                child.send_signal(signum)
                return
            except ProcessLookupError:
                pass
        self.pending.append(signum)

    def raise_if_pending(self) -> None:
        if self.pending:
            names = ", ".join(signal.Signals(signum).name for signum in self.pending)
            raise ExperimentIncomplete(
                f"external signal received with no attempt in flight: {names}"
            )


def utc_now() -> str:
    return datetime.now(UTC).isoformat(timespec="microseconds")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_bytes_once(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("xb") as stream:
        stream.write(data)


def write_text_once(path: Path, text: str) -> None:
    write_bytes_once(path, text.encode("utf-8"))


def write_json_once(path: Path, value: Any) -> None:
    write_text_once(path, json.dumps(value, indent=2, sort_keys=True) + "\n")


def replace_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    temporary.replace(path)


def run_capture(
    argv: Sequence[str],
    *,
    cwd: Path,
    env: dict[str, str],
    timeout: float | None = None,
) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        list(argv),
        cwd=cwd,
        env=env,
        input=None,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
        timeout=timeout,
    )


def safe_base_environment() -> dict[str, str]:
    inherited_names = (
        "PATH",
        "HOME",
        "TMPDIR",
        "CARGO_HOME",
        "RUSTUP_HOME",
        "RUSTUP_TOOLCHAIN",
        "SDKROOT",
        "DEVELOPER_DIR",
    )
    result = {name: os.environ[name] for name in inherited_names if name in os.environ}
    result.update(
        {
            "LANG": "C",
            "LC_ALL": "C",
            "TZ": "UTC",
            "CARGO_INCREMENTAL": "0",
            "CARGO_TERM_COLOR": "never",
        }
    )
    return result


def benchmark_environment() -> dict[str, str]:
    return {
        "LANG": "C",
        "LC_ALL": "C",
        "TZ": "UTC",
        "RUST_BACKTRACE": "1",
    }


def executable(name: str) -> str:
    resolved = shutil.which(name)
    if resolved is None:
        raise RuntimeError(f"required executable not found: {name}")
    return str(Path(resolved).absolute())


def cargo_config_paths(build_env: dict[str, str]) -> list[Path]:
    paths: list[Path] = []
    current = REPO_ROOT.resolve()
    while True:
        paths.extend((current / ".cargo" / "config.toml", current / ".cargo" / "config"))
        if current.parent == current:
            break
        current = current.parent
    cargo_home = build_env.get("CARGO_HOME")
    if cargo_home is None and "HOME" in build_env:
        cargo_home = str(Path(build_env["HOME"]) / ".cargo")
    if cargo_home is not None:
        paths.extend((Path(cargo_home) / "config.toml", Path(cargo_home) / "config"))
    return [path for path in paths if path.is_file()]


def inspect_runner_configuration(host_triple: str, build_env: dict[str, str]) -> dict[str, Any]:
    env_key = "CARGO_TARGET_" + host_triple.upper().replace("-", "_") + "_RUNNER"
    findings: list[dict[str, str]] = []
    if os.environ.get(env_key):
        findings.append({"source": env_key, "value": os.environ[env_key]})
    if os.environ.get("CARGO_BUILD_TARGET") not in (None, "", host_triple):
        findings.append({"source": "CARGO_BUILD_TARGET", "value": os.environ["CARGO_BUILD_TARGET"]})

    parsed_paths: list[str] = []
    for path in cargo_config_paths(build_env):
        parsed_paths.append(str(path))
        with path.open("rb") as stream:
            document = tomllib.load(stream)
        target = document.get("target", {})
        if isinstance(target, dict):
            for key, table in target.items():
                if not (isinstance(table, dict) and "runner" in table):
                    continue
                # cfg(...) tables remain flagged because this check cannot
                # resolve their match set.
                if key != host_triple and not key.startswith("cfg("):
                    continue
                findings.append(
                    {"source": f"{path}:target.{key}.runner", "value": repr(table["runner"])}
                )
        build = document.get("build", {})
        if isinstance(build, dict) and "target" in build:
            value = build["target"]
            if value != host_triple:
                findings.append({"source": f"{path}:build.target", "value": repr(value)})
        if isinstance(build, dict):
            # `rustc` overrides bypass the PATH-resolved compiler hash
            # recorded in build metadata.
            for override in ("rustc", "rustc-wrapper", "rustc-workspace-wrapper"):
                if build.get(override):
                    findings.append(
                        {"source": f"{path}:build.{override}", "value": repr(build[override])}
                    )

    return {
        "host_triple": host_triple,
        "environment_key_checked": env_key,
        "config_paths_checked": parsed_paths,
        "ambiguous_or_non_native_findings": findings,
    }


def source_paths(exclude_dir: Path | None = None) -> Iterable[Path]:
    fixed = (
        REPO_ROOT / "Cargo.toml",
        REPO_ROOT / "Cargo.lock",
        REPO_ROOT / "rust-toolchain.toml",
        REPO_ROOT / "README.md",
        REPO_ROOT / "DOCUMENTATION.md",
        REPO_ROOT / "METHODOLOGY.md",
    )
    for path in fixed:
        if path.is_file():
            yield path
    for path in sorted(TOPIC_DIR.rglob("*")):
        if not path.is_file():
            continue
        if "__pycache__" in path.parts or path.suffix == ".pyc":
            continue
        if "measurements/runs" in path.relative_to(TOPIC_DIR).as_posix():
            continue
        if exclude_dir is not None and path.is_relative_to(exclude_dir):
            continue
        yield path


def source_tree_manifest(exclude_dir: Path | None = None) -> list[dict[str, Any]]:
    records = []
    for path in source_paths(exclude_dir):
        records.append(
            {
                "path": path.relative_to(REPO_ROOT).as_posix(),
                "bytes": path.stat().st_size,
                "sha256": sha256_file(path),
            }
        )
    return records


def source_tree_digest(records: Sequence[dict[str, Any]]) -> str:
    canonical = json.dumps(records, separators=(",", ":"), sort_keys=True).encode("utf-8")
    return sha256_bytes(canonical)


def git_metadata(run_dir: Path, env: dict[str, str]) -> dict[str, Any]:
    git = executable("git")
    status_argv = [
        git,
        "status",
        "--porcelain=v1",
        "--untracked-files=all",
        "--",
        ".",
        ":(exclude)topics/001-prefix-sums-and-scans/measurements/runs",
    ]
    if run_dir.resolve().is_relative_to(REPO_ROOT.resolve()):
        relative_run_dir = run_dir.resolve().relative_to(REPO_ROOT.resolve()).as_posix()
        status_argv.append(f":(exclude){relative_run_dir}")
    commands = {
        "head": [git, "rev-parse", "HEAD"],
        "status": status_argv,
        "diff": [git, "diff", "--binary", "HEAD"],
    }
    results: dict[str, Any] = {}
    for name, argv in commands.items():
        completed = run_capture(argv, cwd=REPO_ROOT, env=env)
        write_bytes_once(run_dir / "metadata" / f"git-{name}.stdout", completed.stdout)
        write_bytes_once(run_dir / "metadata" / f"git-{name}.stderr", completed.stderr)
        if completed.returncode != 0:
            raise RuntimeError(f"git metadata command failed: {' '.join(argv)}")
        results[name] = completed.stdout.decode("utf-8", errors="replace")
    return {
        "head": results["head"].strip(),
        "status_sha256": sha256_bytes(results["status"].encode("utf-8")),
        "diff_sha256": sha256_bytes(results["diff"].encode("utf-8")),
    }


def host_metadata() -> dict[str, Any]:
    cpu_description = platform.processor()
    if sys.platform == "darwin":
        try:
            completed = subprocess.run(
                ["/usr/sbin/sysctl", "-n", "machdep.cpu.brand_string"],
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
                check=False,
                timeout=5,
            )
            if completed.returncode == 0:
                cpu_description = completed.stdout.decode().strip()
        except (OSError, subprocess.SubprocessError):
            pass
    elif Path("/proc/cpuinfo").is_file():
        for line in Path("/proc/cpuinfo").read_text(encoding="utf-8", errors="replace").splitlines():
            if line.startswith("model name"):
                cpu_description = line.split(":", 1)[1].strip()
                break

    affinity: list[int] | None = None
    if hasattr(os, "sched_getaffinity"):
        affinity = sorted(os.sched_getaffinity(0))
    cpuset_effective_path = Path("/sys/fs/cgroup/cpuset.cpus.effective")
    cpuset_effective = None
    if cpuset_effective_path.is_file():
        cpuset_effective = cpuset_effective_path.read_text(encoding="utf-8").strip()
    return {
        "hostname": socket.gethostname(),
        "platform": platform.platform(),
        "system": platform.system(),
        "release": platform.release(),
        "machine": platform.machine(),
        "cpu_description": cpu_description,
        "logical_cpu_count": os.cpu_count(),
        "runner_process_affinity": affinity,
        "cpuset_cpus_effective": cpuset_effective,
        "python": sys.version,
    }


def tool_output(argv: Sequence[str], env: dict[str, str]) -> str:
    completed = run_capture(argv, cwd=REPO_ROOT, env=env)
    if completed.returncode != 0:
        raise RuntimeError(f"tool identity command failed: {' '.join(argv)}")
    return completed.stdout.decode("utf-8", errors="replace").strip()


def build_benchmark(run_dir: Path, rustflags: str) -> tuple[Path, dict[str, Any]]:
    build_env = safe_base_environment()
    build_env["RUSTFLAGS"] = rustflags
    cargo = executable("cargo")
    rustc = executable("rustc")
    rustc_verbose = tool_output([rustc, "-vV"], build_env)
    host_lines = [line for line in rustc_verbose.splitlines() if line.startswith("host: ")]
    if len(host_lines) != 1:
        raise RuntimeError("rustc -vV did not report exactly one host triple")
    host_triple = host_lines[0].split(":", 1)[1].strip()
    runner_check = inspect_runner_configuration(host_triple, build_env)
    if runner_check["ambiguous_or_non_native_findings"]:
        raise RuntimeError(f"direct native execution is ambiguous: {runner_check['ambiguous_or_non_native_findings']}")

    argv = [
        cargo,
        "build",
        "--release",
        "--package",
        "prefix-sums-and-scans",
        "--bench",
        "prefix_sums",
        "--message-format=json-render-diagnostics",
    ]
    started = utc_now()
    completed = run_capture(argv, cwd=REPO_ROOT, env=build_env)
    ended = utc_now()
    write_bytes_once(run_dir / "metadata" / "build.stdout", completed.stdout)
    write_bytes_once(run_dir / "metadata" / "build.stderr", completed.stderr)
    if completed.returncode != 0:
        raise RuntimeError("Cargo benchmark build failed; raw output is retained")

    artifacts: list[Path] = []
    for line in completed.stdout.splitlines():
        try:
            record = json.loads(line)
        except json.JSONDecodeError:
            continue
        target = record.get("target", {})
        if (
            record.get("reason") == "compiler-artifact"
            and target.get("name") == "prefix_sums"
            and "bench" in target.get("kind", [])
            and record.get("executable")
        ):
            artifacts.append(Path(record["executable"]).resolve())
    unique_artifacts = sorted(set(artifacts))
    if len(unique_artifacts) != 1 or not unique_artifacts[0].is_file():
        raise RuntimeError(f"expected one resolved benchmark executable, found {unique_artifacts}")

    source_binary = unique_artifacts[0]
    binary_hash = sha256_file(source_binary)
    retained_binary = run_dir / "artifacts" / f"prefix_sums-{binary_hash}"
    retained_binary.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source_binary, retained_binary)
    if sha256_file(retained_binary) != binary_hash:
        raise RuntimeError("retained benchmark image hash does not match source image")

    metadata = {
        "started_at": started,
        "ended_at": ended,
        "argv": argv,
        "cwd": str(REPO_ROOT),
        "environment": build_env,
        "cargo_version": tool_output([cargo, "-V"], build_env),
        "rustc_verbose": rustc_verbose,
        "rustflags": rustflags,
        "cargo_profile": "release",
        "source_binary": str(source_binary),
        "retained_binary": str(retained_binary),
        "binary_sha256": binary_hash,
        "cargo_sha256": sha256_file(Path(cargo)),
        "rustc_sha256": sha256_file(Path(rustc)),
        "runner_configuration": runner_check,
    }
    write_json_once(run_dir / "metadata" / "build.json", metadata)
    return retained_binary, metadata


def stable_random(label: str) -> random.Random:
    digest = hashlib.sha256(f"{ASSIGNMENT_SEED}:{label}".encode("utf-8")).digest()
    return random.Random(int.from_bytes(digest[:8], "big"))


def balanced_templates(block_count: int, label: str) -> list[str]:
    if block_count <= 0 or block_count % 2 != 0:
        raise ValueError("complete-block template count must be positive and even")
    templates = ["ABBA"] * (block_count // 2) + ["BAAB"] * (block_count // 2)
    stable_random(label).shuffle(templates)
    return templates


def positions_for_block(
    phase: str,
    contrast: Contrast,
    block_index: int,
    template: str,
    retry_index: int,
) -> list[Position]:
    positions = []
    for position_index, label in enumerate(template, start=1):
        algorithm = "linear" if label == "A" else contrast.b_algorithm
        positions.append(
            Position(
                phase=phase,
                contrast_id=contrast.contrast_id,
                block_index=block_index,
                template=template,
                position_index=position_index,
                label=label,
                algorithm=algorithm,
                n=contrast.n,
                pattern="mixed",
                seed=ASSIGNMENT_SEED,
                block_size=contrast.block_size,
                workers=contrast.workers,
                retry_index=retry_index,
            )
        )
    return positions


def planned_schedule(phases: Sequence[str]) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for phase in phases:
        if phase in ("pilot", "main"):
            count = PILOT_BLOCKS if phase == "pilot" else MAIN_BLOCKS
            for contrast in CONTRASTS:
                templates = balanced_templates(count, f"{phase}:{contrast.contrast_id}")
                for block_index, template in enumerate(templates, start=1):
                    rows.extend(asdict(position) for position in positions_for_block(
                        phase, contrast, block_index, template, 0
                    ))
        elif phase == "aa":
            templates = balanced_templates(AA_BLOCKS, "aa")
            for block_index, template in enumerate(templates, start=1):
                rows.extend(asdict(position) for position in positions_for_block(
                    phase, AA_CONTRAST, block_index, template, 0
                ))
        elif phase == "quick":
            for contrast in CONTRASTS:
                rows.extend(asdict(position) for position in positions_for_block(
                    phase, contrast, 1, "ABBA", 0
                ))
        elif phase == "distribution":
            index = 0
            for pattern in AUDIT_PATTERNS:
                for algorithm, block_size, workers in AUDIT_ALGORITHMS:
                    index += 1
                    rows.append(
                        asdict(
                            Position(
                                phase=phase,
                                contrast_id=f"distribution_{pattern}_{algorithm}",
                                block_index=index,
                                template="D",
                                position_index=1,
                                label=algorithm,
                                algorithm=algorithm,
                                n=262_144,
                                pattern=pattern,
                                seed=ASSIGNMENT_SEED,
                                block_size=block_size,
                                workers=workers,
                            )
                        )
                    )
        elif phase == "profile":
            continue
        else:
            raise ValueError(f"unknown planned phase: {phase}")
    return rows


def attempt_identifier(position: Position) -> str:
    return (
        f"{position.phase}-{position.contrast_id}-b{position.block_index:02d}"
        f"-r{position.retry_index}-p{position.position_index}-{position.label}"
    )


def expected_work(position: Position) -> dict[str, int | bool]:
    n = position.n
    base = {
        "input_elements_read": n,
        "output_element_writes": n,
        "output_element_reads": 0,
        "auxiliary_element_writes": 0,
        "auxiliary_element_reads": 0,
        "requested_worker_threads": 0,
        "effective_worker_threads": 0,
        "scoped_thread_spawns": 0,
        "scoped_thread_joins": 0,
        "thread_runtime_allocations_counted": False,
    }
    if position.algorithm == "reference":
        additions = n * (n + 1) // 2
        base.update(
            wrapping_adds=additions,
            input_elements_read=additions,
            explicit_vec_allocations=int(n > 0),
        )
        return base
    if position.algorithm == "linear":
        base.update(
            wrapping_adds=max(n - 1, 0),
            explicit_vec_allocations=int(n > 0),
        )
        return base
    if position.algorithm == "blocked":
        q = 0 if n == 0 else 1 + (n - 1) // position.block_size
        first = min(n, position.block_size)
        adjusted = n - first
        additions = 0 if q == 0 else n - 1 if q == 1 else 2 * n - first - 2
        base.update(
            wrapping_adds=additions,
            output_element_writes=n + adjusted,
            output_element_reads=adjusted,
            auxiliary_element_writes=q,
            auxiliary_element_reads=q,
            explicit_vec_allocations=2 * int(n > 0),
        )
        return base
    if position.algorithm == "parallel":
        k = min(position.workers, n)
        base["requested_worker_threads"] = position.workers
        base["effective_worker_threads"] = k
        if k == 0:
            base.update(
                wrapping_adds=0,
                input_elements_read=0,
                output_element_writes=0,
                explicit_vec_allocations=0,
            )
            return base
        if k == 1:
            base.update(
                wrapping_adds=max(n - 1, 0),
                explicit_vec_allocations=1,
            )
            return base
        first = n // k + int(n % k != 0)
        adjusted = n - first
        thread_events = 2 * k - 1
        base.update(
            wrapping_adds=2 * n - first - 2,
            output_element_writes=2 * n + adjusted,
            output_element_reads=n + adjusted,
            auxiliary_element_writes=thread_events,
            auxiliary_element_reads=thread_events,
            explicit_vec_allocations=k + 6,
            scoped_thread_spawns=thread_events,
            scoped_thread_joins=thread_events,
        )
        return base
    raise ValueError(f"unknown algorithm for work count: {position.algorithm}")


def write_attempt_csv(run_dir: Path, attempts: Sequence[AttemptResult]) -> None:
    path = run_dir / "attempts.csv"
    temporary = path.with_suffix(".csv.tmp")
    temporary.parent.mkdir(parents=True, exist_ok=True)
    fields = list(AttemptResult.__dataclass_fields__)
    with temporary.open("w", encoding="utf-8", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=fields)
        writer.writeheader()
        for attempt in attempts:
            writer.writerow(asdict(attempt))
    temporary.replace(path)


def write_block_csv(run_dir: Path, blocks: Sequence[dict[str, Any]]) -> None:
    path = run_dir / "blocks.csv"
    temporary = path.with_suffix(".csv.tmp")
    temporary.parent.mkdir(parents=True, exist_ok=True)
    fields = (
        "phase",
        "contrast_id",
        "block_index",
        "retry_index",
        "template",
        "status",
        "invalid_reason",
        "attempt_ids",
        "a_mean_log_ns",
        "b_mean_log_ns",
        "log_contrast",
        "ratio",
    )
    with temporary.open("w", encoding="utf-8", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=fields)
        writer.writeheader()
        for block in blocks:
            writer.writerow({field: block.get(field, "") for field in fields})
    temporary.replace(path)


def parse_cpu_list(value: str) -> set[int]:
    cpus: set[int] = set()
    for part in value.split(","):
        part = part.strip()
        if not part:
            raise ValueError("empty CPU-list component")
        if "-" in part:
            start_text, end_text = part.split("-", 1)
            start = int(start_text)
            end = int(end_text)
            if end < start:
                raise ValueError(f"descending CPU range: {part}")
            cpus.update(range(start, end + 1))
        else:
            cpus.add(int(part))
    if not cpus:
        raise ValueError("CPU list is empty")
    return cpus


class HarnessExecutor:
    def __init__(
        self,
        run_dir: Path,
        binary: Path,
        binary_hash: str,
        timeout_seconds: float,
        cpu_list: str | None,
        relay: ExternalSignalRelay | None = None,
    ) -> None:
        self.run_dir = run_dir
        self.binary = binary
        self.binary_hash = binary_hash
        self.timeout_seconds = timeout_seconds
        self.relay = relay
        self.environment = benchmark_environment()
        self.attempts: list[AttemptResult] = []
        self.blocks: list[dict[str, Any]] = []
        self.input_checksums: dict[tuple[int, str, int], str] = {}
        self.output_checksums: dict[tuple[int, str, int], str] = {}
        self.cpu_list = cpu_list
        self.taskset: str | None = None
        if cpu_list is not None:
            requested = parse_cpu_list(cpu_list)
            if not sys.platform.startswith("linux"):
                raise RuntimeError("--cpu-list requires Linux taskset and affinity validation")
            self.taskset = executable("taskset")
            granted = set(os.sched_getaffinity(0))
            if not requested.issubset(granted):
                raise RuntimeError(f"requested CPUs {sorted(requested)} exceed granted CPUs {sorted(granted)}")

    def harness_argv(self, position: Position, warmups: int, samples: int) -> list[str]:
        attempt_id = attempt_identifier(position)
        return [
            str(self.binary),
            "--algorithm",
            position.algorithm,
            "--label",
            position.label,
            "--attempt-id",
            attempt_id,
            "--contrast-id",
            position.contrast_id,
            "--phase",
            position.phase,
            "--n",
            str(position.n),
            "--pattern",
            position.pattern,
            "--seed",
            str(position.seed),
            "--block-size",
            str(position.block_size),
            "--workers",
            str(position.workers),
            "--warmups",
            str(warmups),
            "--samples",
            str(samples),
        ]

    def run_position(self, position: Position, warmups: int, samples: int) -> AttemptResult:
        if self.relay is not None:
            self.relay.raise_if_pending()
        attempt_id = attempt_identifier(position)
        attempt_dir = self.run_dir / "raw" / "attempts" / attempt_id
        attempt_dir.mkdir(parents=True, exist_ok=False)
        harness_argv = self.harness_argv(position, warmups, samples)
        expanded_argv = harness_argv
        if self.taskset is not None:
            expanded_argv = [self.taskset, "-c", str(self.cpu_list), *harness_argv]
        binary_before = sha256_file(self.binary)
        launch = {
            "schema_version": SCHEMA_VERSION,
            "started_at": utc_now(),
            "position": asdict(position),
            "harness_argv": harness_argv,
            "expanded_argv": expanded_argv,
            "cwd": str(REPO_ROOT),
            "environment": self.environment,
            "timeout_seconds": self.timeout_seconds,
            "executable_sha256_before": binary_before,
            "expected_work": expected_work(position),
        }
        write_json_once(attempt_dir / "launch.json", launch)

        timed_out = False
        returncode: int | None = None
        stdout = b""
        stderr = b""
        started_ns = time.time_ns()
        # A Popen handle (rather than run_capture) lets the signal relay
        # forward external signals to the child while communicate() blocks;
        # the forwarded signal then appears as the child's exit status.
        with subprocess.Popen(
            expanded_argv,
            cwd=REPO_ROOT,
            env=self.environment,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        ) as process:
            if self.relay is not None:
                self.relay.child = process
            try:
                stdout, stderr = process.communicate(timeout=self.timeout_seconds)
                returncode = process.returncode
            except subprocess.TimeoutExpired:
                process.kill()
                stdout, stderr = process.communicate()
                returncode = process.returncode
                timed_out = True
            finally:
                if self.relay is not None:
                    self.relay.child = None
        ended_ns = time.time_ns()
        write_bytes_once(attempt_dir / "stdout.jsonl", stdout)
        write_bytes_once(attempt_dir / "stderr.txt", stderr)

        binary_after = sha256_file(self.binary)
        invalid_reasons: list[str] = []
        parsed: dict[str, Any] | None = None
        if binary_before != self.binary_hash or binary_after != self.binary_hash:
            invalid_reasons.append("executable hash changed")
        if timed_out:
            invalid_reasons.append("timeout")
        elif returncode != 0:
            invalid_reasons.append(f"nonzero return code {returncode}")
        if not timed_out and returncode == 0:
            lines = [line for line in stdout.decode("utf-8", errors="replace").splitlines() if line.strip()]
            if len(lines) != 1:
                invalid_reasons.append(f"expected one JSON line, found {len(lines)}")
            else:
                try:
                    candidate = json.loads(lines[0])
                    if not isinstance(candidate, dict):
                        raise TypeError("top-level JSON is not an object")
                    parsed = candidate
                except (json.JSONDecodeError, TypeError) as error:
                    invalid_reasons.append(f"JSON parse failure: {error}")

        expected_echo = {
            "schema_version": SCHEMA_VERSION,
            "attempt_id": attempt_id,
            "contrast_id": position.contrast_id,
            "phase": position.phase,
            "label": position.label,
            "algorithm": position.algorithm,
            "n": position.n,
            "pattern": position.pattern,
            "seed": position.seed,
            "block_size": position.block_size,
            "workers": position.workers,
            "warmups": warmups,
            "samples": samples,
            "semantic_ok": True,
        }
        if parsed is not None:
            for field, expected in expected_echo.items():
                if parsed.get(field) != expected:
                    invalid_reasons.append(
                        f"echo mismatch for {field}: expected {expected!r}, got {parsed.get(field)!r}"
                    )
            sample_ns = parsed.get("sample_ns")
            if (
                not isinstance(sample_ns, list)
                or len(sample_ns) != samples
                or any(not isinstance(value, int) or value <= 0 for value in sample_ns)
            ):
                invalid_reasons.append("invalid raw timing sample array")
            if not isinstance(parsed.get("position_ns"), int) or parsed.get("position_ns", 0) <= 0:
                invalid_reasons.append("invalid position outcome")
            if parsed.get("work") != expected_work(position):
                invalid_reasons.append("work-count canary mismatch")
            for field in ("input_checksum", "output_checksum"):
                value = parsed.get(field)
                if not isinstance(value, str) or len(value) != 16:
                    invalid_reasons.append(f"invalid {field}")
                else:
                    try:
                        int(value, 16)
                    except ValueError:
                        invalid_reasons.append(f"non-hex {field}")

            key = (position.n, position.pattern, position.seed)
            input_checksum = parsed.get("input_checksum")
            output_checksum = parsed.get("output_checksum")
            if isinstance(input_checksum, str):
                prior = self.input_checksums.setdefault(key, input_checksum)
                if prior != input_checksum:
                    invalid_reasons.append("input checksum differs within workload cell")
            if isinstance(output_checksum, str):
                prior = self.output_checksums.setdefault(key, output_checksum)
                if prior != output_checksum:
                    invalid_reasons.append("output checksum differs within workload cell")

        external_interruption = bool(
            returncode is not None
            and returncode < 0
            and -returncode in {int(member) for member in EXTERNAL_SIGNALS}
        )
        result = AttemptResult(
            attempt_id=attempt_id,
            phase=position.phase,
            contrast_id=position.contrast_id,
            block_index=position.block_index,
            retry_index=position.retry_index,
            template=position.template,
            position_index=position.position_index,
            label=position.label,
            algorithm=position.algorithm,
            n=position.n,
            pattern=position.pattern,
            seed=position.seed,
            block_size=position.block_size,
            workers=position.workers,
            status="VALID" if not invalid_reasons else "INVALID",
            invalid_reason="; ".join(invalid_reasons),
            external_interruption=external_interruption,
            returncode=returncode,
            timed_out=timed_out,
            position_ns=parsed.get("position_ns") if parsed is not None else None,
            input_checksum=parsed.get("input_checksum", "") if parsed is not None else "",
            output_checksum=parsed.get("output_checksum", "") if parsed is not None else "",
            executable_sha256_before=binary_before,
            executable_sha256_after=binary_after,
            attempt_directory=attempt_dir.relative_to(self.run_dir).as_posix(),
        )
        result_document = {
            "schema_version": SCHEMA_VERSION,
            "started_ns_since_epoch": started_ns,
            "ended_ns_since_epoch": ended_ns,
            "duration_ns": ended_ns - started_ns,
            "attempt": asdict(result),
            "parsed_output": parsed,
        }
        write_json_once(attempt_dir / "result.json", result_document)
        self.attempts.append(result)
        write_attempt_csv(self.run_dir, self.attempts)
        return result

    def run_block(
        self,
        phase: str,
        contrast: Contrast,
        block_index: int,
        template: str,
        warmups: int,
        samples: int,
    ) -> None:
        retry_index = 0
        while True:
            positions = positions_for_block(
                phase, contrast, block_index, template, retry_index
            )
            results: list[AttemptResult] = []
            failure: AttemptResult | None = None
            for position in positions:
                result = self.run_position(position, warmups, samples)
                results.append(result)
                if result.status != "VALID":
                    failure = result
                    break

            record: dict[str, Any] = {
                "phase": phase,
                "contrast_id": contrast.contrast_id,
                "block_index": block_index,
                "retry_index": retry_index,
                "template": template,
                "status": "VALID" if failure is None and len(results) == 4 else "INVALID",
                "invalid_reason": "" if failure is None else failure.invalid_reason,
                "attempt_ids": ";".join(result.attempt_id for result in results),
            }
            if record["status"] == "VALID":
                a_values = [result.position_ns for result in results if result.label == "A"]
                b_values = [result.position_ns for result in results if result.label == "B"]
                if len(a_values) != 2 or len(b_values) != 2 or None in a_values or None in b_values:
                    raise RuntimeError("complete block does not have two valid values per label")
                a_mean = statistics.fmean(math.log(value) for value in a_values if value is not None)
                b_mean = statistics.fmean(math.log(value) for value in b_values if value is not None)
                log_contrast = b_mean - a_mean
                record.update(
                    a_mean_log_ns=a_mean,
                    b_mean_log_ns=b_mean,
                    log_contrast=log_contrast,
                    ratio=math.exp(log_contrast),
                )
            self.blocks.append(record)
            write_block_csv(self.run_dir, self.blocks)

            if record["status"] == "VALID":
                return
            if failure is not None and failure.external_interruption and retry_index == 0:
                retry_index = 1
                continue
            raise ExperimentIncomplete(
                f"{phase} {contrast.contrast_id} block {block_index} is incomplete: "
                f"{record['invalid_reason']}"
            )


def beta_continued_fraction(a: float, b: float, x: float) -> float:
    maximum_iterations = 300
    epsilon = 3.0e-14
    floor = 1.0e-300
    qab = a + b
    qap = a + 1.0
    qam = a - 1.0
    c = 1.0
    d = 1.0 - qab * x / qap
    if abs(d) < floor:
        d = floor
    d = 1.0 / d
    result = d
    for iteration in range(1, maximum_iterations + 1):
        even = 2 * iteration
        coefficient = iteration * (b - iteration) * x / ((qam + even) * (a + even))
        d = 1.0 + coefficient * d
        if abs(d) < floor:
            d = floor
        c = 1.0 + coefficient / c
        if abs(c) < floor:
            c = floor
        d = 1.0 / d
        result *= d * c
        coefficient = -(a + iteration) * (qab + iteration) * x / (
            (a + even) * (qap + even)
        )
        d = 1.0 + coefficient * d
        if abs(d) < floor:
            d = floor
        c = 1.0 + coefficient / c
        if abs(c) < floor:
            c = floor
        d = 1.0 / d
        delta = d * c
        result *= delta
        if abs(delta - 1.0) <= epsilon:
            return result
    raise ArithmeticError("incomplete-beta continued fraction did not converge")


def regularized_incomplete_beta(a: float, b: float, x: float) -> float:
    if not 0.0 <= x <= 1.0:
        raise ValueError("regularized beta input must be in [0, 1]")
    if x == 0.0:
        return 0.0
    if x == 1.0:
        return 1.0
    factor = math.exp(
        math.lgamma(a + b)
        - math.lgamma(a)
        - math.lgamma(b)
        + a * math.log(x)
        + b * math.log1p(-x)
    )
    if x < (a + 1.0) / (a + b + 2.0):
        return factor * beta_continued_fraction(a, b, x) / a
    return 1.0 - factor * beta_continued_fraction(b, a, 1.0 - x) / b


def student_t_cdf(value: float, degrees_of_freedom: int) -> float:
    if degrees_of_freedom <= 0:
        raise ValueError("degrees of freedom must be positive")
    if value == 0.0:
        return 0.5
    x = degrees_of_freedom / (degrees_of_freedom + value * value)
    tail = 0.5 * regularized_incomplete_beta(degrees_of_freedom / 2.0, 0.5, x)
    return 1.0 - tail if value > 0.0 else tail


def student_t_quantile(probability: float, degrees_of_freedom: int) -> float:
    if not 0.0 < probability < 1.0:
        raise ValueError("probability must be in (0, 1)")
    if probability == 0.5:
        return 0.0
    if probability < 0.5:
        return -student_t_quantile(1.0 - probability, degrees_of_freedom)
    low = 0.0
    high = 1.0
    while student_t_cdf(high, degrees_of_freedom) < probability:
        high *= 2.0
        if high > 1.0e9:
            raise ArithmeticError("Student t quantile bracket failed")
    for _ in range(100):
        midpoint = (low + high) / 2.0
        if student_t_cdf(midpoint, degrees_of_freedom) < probability:
            low = midpoint
        else:
            high = midpoint
    return (low + high) / 2.0


def analyze_values(values: Sequence[float], alpha: float) -> dict[str, float | int]:
    if len(values) < 2:
        raise ValueError("at least two independent values are required")
    count = len(values)
    degrees_of_freedom = count - 1
    mean = statistics.fmean(values)
    standard_deviation = statistics.stdev(values)
    standard_error = standard_deviation / math.sqrt(count)
    critical = student_t_quantile(1.0 - alpha / 2.0, degrees_of_freedom)
    half_width = critical * standard_error
    return {
        "independent_unit_count": count,
        "degrees_of_freedom": degrees_of_freedom,
        "alpha": alpha,
        "mean_log_ratio": mean,
        "standard_deviation_log_ratio": standard_deviation,
        "standard_error_log_ratio": standard_error,
        "student_t_critical": critical,
        "lower_log_ratio": mean - half_width,
        "upper_log_ratio": mean + half_width,
        "point_ratio": math.exp(mean),
        "lower_ratio": math.exp(mean - half_width),
        "upper_ratio": math.exp(mean + half_width),
    }


def valid_block_values(
    blocks: Sequence[dict[str, Any]], phase: str, contrast_id: str
) -> list[float]:
    matching = [
        block
        for block in blocks
        if block["phase"] == phase
        and block["contrast_id"] == contrast_id
        and block["status"] == "VALID"
    ]
    matching.sort(key=lambda block: block["block_index"])
    return [float(block["log_contrast"]) for block in matching]


def analyze_pilot(executor: HarnessExecutor) -> dict[str, Any]:
    results = []
    family_alpha = FAMILY_ALPHA / FAMILY_SIZE
    critical = student_t_quantile(1.0 - family_alpha / 2.0, MAIN_BLOCKS - 1)
    for contrast in CONTRASTS:
        values = valid_block_values(executor.blocks, "pilot", contrast.contrast_id)
        if len(values) != PILOT_BLOCKS:
            raise ExperimentIncomplete(
                f"pilot {contrast.contrast_id} has {len(values)} valid blocks, expected {PILOT_BLOCKS}"
            )
        observed_sd = statistics.stdev(values)
        sensitivity = []
        for multiplier in (0.5, 1.0, 1.5, 2.0):
            half_width = critical * observed_sd * multiplier / math.sqrt(MAIN_BLOCKS)
            sensitivity.append(
                {
                    "pilot_sd_multiplier": multiplier,
                    "log_half_width": half_width,
                    "multiplicative_half_width": math.exp(half_width),
                }
            )
        results.append(
            {
                "contrast_id": contrast.contrast_id,
                "pilot_block_count": len(values),
                "pilot_log_contrasts": values,
                "observed_pilot_sd": observed_sd,
                "fixed_main_block_count": MAIN_BLOCKS,
                "family_adjusted_student_t_critical_df11": critical,
                "sensitivity": sensitivity,
                "excluded_from_primary_analysis": True,
            }
        )
    return {
        "status": "COMPLETE",
        "assignment_seed": ASSIGNMENT_SEED,
        "pilot_blocks_per_contrast": PILOT_BLOCKS,
        "primary_results": results,
    }


def analyze_main(executor: HarnessExecutor) -> dict[str, Any]:
    results = []
    per_contrast_alpha = FAMILY_ALPHA / FAMILY_SIZE
    faster_boundary = 1.0 / PRACTICAL_RATIO
    for contrast in CONTRASTS:
        values = valid_block_values(executor.blocks, "main", contrast.contrast_id)
        if len(values) != MAIN_BLOCKS:
            raise ExperimentIncomplete(
                f"main {contrast.contrast_id} has {len(values)} valid blocks, expected {MAIN_BLOCKS}"
            )
        analysis = analyze_values(values, per_contrast_alpha)
        lower = float(analysis["lower_ratio"])
        upper = float(analysis["upper_ratio"])
        if upper < faster_boundary:
            interpretation = "B_PRACTICALLY_FASTER_IN_CELL"
        elif lower > PRACTICAL_RATIO:
            interpretation = "LINEAR_PRACTICALLY_FASTER_IN_CELL"
        else:
            interpretation = "NO_PRACTICALLY_MEANINGFUL_WINNER_JUSTIFIED"
        results.append(
            {
                "contrast_id": contrast.contrast_id,
                "ratio_orientation": "B_over_linear",
                "block_log_contrasts": values,
                "analysis": analysis,
                "practical_ratio_boundary": PRACTICAL_RATIO,
                "interpretation": interpretation,
                "repository_policy_verdict": None,
            }
        )
    return {
        "status": "COMPLETE",
        "family": [contrast.contrast_id for contrast in CONTRASTS],
        "familywise_alpha": FAMILY_ALPHA,
        "multiplicity": "two-sided Bonferroni",
        "per_contrast_alpha": per_contrast_alpha,
        "fixed_blocks_per_contrast": MAIN_BLOCKS,
        "primary_results": results,
        "model_boundary": (
            "Paired Student t intervals assume iid, approximately normal complete-block "
            "log contrasts. One host and build do not estimate host or build variation."
        ),
    }


def attempts_in_valid_blocks(
    executor: HarnessExecutor, phase: str
) -> tuple[list[AttemptResult], list[AttemptResult]]:
    """Split a phase's attempts into members of VALID blocks and the rest.

    A block invalidated by an external interruption is retried, and its
    recorded attempts stay in ``executor.attempts`` as evidence. Mechanical
    integrity checks reason over the attempts that constitute the valid
    blocks; the remainder is reported separately, never silently dropped.
    """
    valid_ids = {
        attempt_id
        for block in executor.blocks
        if block["phase"] == phase and block["status"] == "VALID"
        for attempt_id in block["attempt_ids"].split(";")
        if attempt_id
    }
    phase_attempts = [attempt for attempt in executor.attempts if attempt.phase == phase]
    included = [attempt for attempt in phase_attempts if attempt.attempt_id in valid_ids]
    excluded = [attempt for attempt in phase_attempts if attempt.attempt_id not in valid_ids]
    return included, excluded


def analyze_aa(executor: HarnessExecutor) -> dict[str, Any]:
    values = valid_block_values(executor.blocks, "aa", AA_CONTRAST.contrast_id)
    aa_attempts, excluded_attempts = attempts_in_valid_blocks(executor, "aa")
    a_parameters = {
        (
            attempt.algorithm,
            attempt.n,
            attempt.pattern,
            attempt.seed,
            attempt.block_size,
            attempt.workers,
        )
        for attempt in aa_attempts
        if attempt.label == "A"
    }
    b_parameters = {
        (
            attempt.algorithm,
            attempt.n,
            attempt.pattern,
            attempt.seed,
            attempt.block_size,
            attempt.workers,
        )
        for attempt in aa_attempts
        if attempt.label == "B"
    }
    mechanical_checks = {
        "valid_attempt_count_is_48": len(aa_attempts) == AA_BLOCKS * 4
        and all(attempt.status == "VALID" for attempt in aa_attempts),
        "all_algorithms_are_linear": all(attempt.algorithm == "linear" for attempt in aa_attempts),
        "executable_hashes_identical": len(
            {
                attempt.executable_sha256_before
                for attempt in aa_attempts
            }
            | {attempt.executable_sha256_after for attempt in aa_attempts}
        )
        == 1,
        "input_checksums_identical": len({attempt.input_checksum for attempt in aa_attempts}) == 1,
        "output_checksums_identical": len({attempt.output_checksum for attempt in aa_attempts}) == 1,
        "label_counts_balanced": sum(attempt.label == "A" for attempt in aa_attempts)
        == sum(attempt.label == "B" for attempt in aa_attempts)
        == AA_BLOCKS * 2,
        "template_counts_balanced": sum(
            block["template"] == "ABBA"
            for block in executor.blocks
            if block["phase"] == "aa" and block["status"] == "VALID"
        )
        == sum(
            block["template"] == "BAAB"
            for block in executor.blocks
            if block["phase"] == "aa" and block["status"] == "VALID"
        )
        == AA_BLOCKS // 2,
        "candidate_parameters_identical_across_labels": a_parameters == b_parameters
        and len(a_parameters) == 1,
        "schema_v1_parser_path_passed_for_every_attempt": bool(aa_attempts)
        and all(attempt.status == "VALID" for attempt in aa_attempts),
    }
    if len(values) != AA_BLOCKS:
        raise ExperimentIncomplete(f"A/A has {len(values)} valid blocks, expected {AA_BLOCKS}")
    label_position_summaries = []
    for label in ("A", "B"):
        for position_index in range(1, 5):
            group = [
                math.log(attempt.position_ns)
                for attempt in aa_attempts
                if attempt.label == label
                and attempt.position_index == position_index
                and attempt.position_ns is not None
            ]
            label_position_summaries.append(
                {
                    "label": label,
                    "position_index": position_index,
                    "count": len(group),
                    "mean_log_ns": statistics.fmean(group) if group else None,
                    "minimum_log_ns": min(group) if group else None,
                    "maximum_log_ns": max(group) if group else None,
                }
            )
    position_summaries = []
    for position_index in range(1, 5):
        group = [
            math.log(attempt.position_ns)
            for attempt in aa_attempts
            if attempt.position_index == position_index and attempt.position_ns is not None
        ]
        position_summaries.append(
            {
                "position_index": position_index,
                "count": len(group),
                "mean_log_ns": statistics.fmean(group),
                "geometric_mean_ns": math.exp(statistics.fmean(group)),
            }
        )
    return {
        "status": "COMPLETE" if all(mechanical_checks.values()) else "INVALID",
        "mechanical_integrity": mechanical_checks,
        "attempts_excluded_from_mechanical_checks": [
            {
                "attempt_id": attempt.attempt_id,
                "status": attempt.status,
                "invalid_reason": attempt.invalid_reason,
                "external_interruption": attempt.external_interruption,
                "retry_index": attempt.retry_index,
            }
            for attempt in excluded_attempts
        ],
        "null_calibration_diagnostics": {
            "block_log_contrasts": values,
            "unadjusted_two_sided_95_percent_interval": analyze_values(values, 0.05),
            "maximum_absolute_block_log_contrast": max(abs(value) for value in values),
            "label_by_position_log_time_summaries": label_position_summaries,
            "position_log_time_summaries": position_summaries,
            "interpretation": (
                "Diagnostic only. This run does not prove absence of bias, establish a noise "
                "floor, or alter the primary family."
            ),
        },
    }


def analyze_distribution(executor: HarnessExecutor) -> dict[str, Any]:
    attempts = [attempt for attempt in executor.attempts if attempt.phase == "distribution"]
    if len(attempts) != len(AUDIT_PATTERNS) * len(AUDIT_ALGORITHMS):
        raise ExperimentIncomplete("distribution audit is missing attempts")
    return {
        "status": "COMPLETE" if all(attempt.status == "VALID" for attempt in attempts) else "INCOMPLETE",
        "confirmatory": False,
        "n": 262_144,
        "observations": [
            {
                "pattern": attempt.pattern,
                "algorithm": attempt.algorithm,
                "position_ns": attempt.position_ns,
                "attempt_id": attempt.attempt_id,
            }
            for attempt in attempts
        ],
        "boundary": "Descriptive workload audit. It is outside the 12-contrast primary family.",
    }


def analyze_quick(executor: HarnessExecutor) -> dict[str, Any]:
    attempts, excluded_attempts = attempts_in_valid_blocks(executor, "quick")
    return {
        "status": "COMPLETE" if attempts and all(attempt.status == "VALID" for attempt in attempts) else "INCOMPLETE",
        "confirmatory": False,
        "promotable": False,
        "attempt_count": len(attempts),
        "observations": [asdict(attempt) for attempt in attempts],
        "attempts_excluded_from_mechanical_checks": [asdict(attempt) for attempt in excluded_attempts],
    }


def profile_positions() -> list[Position]:
    specifications = (
        ("reference", 4_096, 1, 1),
        ("linear", 4_194_304, 1, 1),
        ("blocked", 4_194_304, BLOCK_SIZE, 1),
        ("parallel", 4_194_304, 1, 4),
    )
    return [
        Position(
            phase="profile",
            contrast_id=f"profile_{algorithm}_n{n}",
            block_index=index,
            template="P",
            position_index=1,
            label=algorithm,
            algorithm=algorithm,
            n=n,
            pattern="mixed",
            seed=ASSIGNMENT_SEED,
            block_size=block_size,
            workers=workers,
        )
        for index, (algorithm, n, block_size, workers) in enumerate(specifications, start=1)
    ]


def collect_disassembly(run_dir: Path, binary: Path, environment: dict[str, str]) -> dict[str, Any]:
    if sys.platform == "darwin" and Path("/usr/bin/otool").is_file():
        argv = ["/usr/bin/otool", "-tvV", str(binary)]
    else:
        selected = shutil.which("llvm-objdump") or shutil.which("objdump")
        if selected is None:
            return {"status": "UNAVAILABLE", "reason": "no supported disassembler found"}
        argv = [str(Path(selected).resolve()), "-d", "--demangle", str(binary)]
    completed = run_capture(argv, cwd=REPO_ROOT, env=environment, timeout=120.0)
    write_bytes_once(run_dir / "profiles" / "linked-image-assembly.stdout", completed.stdout)
    write_bytes_once(run_dir / "profiles" / "linked-image-assembly.stderr", completed.stderr)
    return {
        "status": "COMPLETE" if completed.returncode == 0 else "FAILED",
        "argv": argv,
        "cwd": str(REPO_ROOT),
        "environment": environment,
        "returncode": completed.returncode,
        "binary_sha256": sha256_file(binary),
        "stdout_sha256": sha256_bytes(completed.stdout),
        "stderr_sha256": sha256_bytes(completed.stderr),
    }


def collect_dynamic_profile(
    run_dir: Path,
    executor: HarnessExecutor,
    position: Position,
) -> dict[str, Any]:
    harness_argv = executor.harness_argv(position, DEFAULT_WARMUPS, DEFAULT_SAMPLES)
    if executor.taskset is not None:
        harness_argv = [executor.taskset, "-c", str(executor.cpu_list), *harness_argv]
    if sys.platform.startswith("linux") and shutil.which("perf") is not None:
        tool = str(Path(shutil.which("perf") or "perf").resolve())
        argv = [
            tool,
            "stat",
            "-x",
            ",",
            "-e",
            "cycles,instructions,branches,branch-misses,cache-references,cache-misses",
            "--",
            *harness_argv,
        ]
        tool_name = "perf-stat"
    elif sys.platform == "darwin" and Path("/usr/bin/time").is_file():
        argv = ["/usr/bin/time", "-l", *harness_argv]
        tool_name = "time-l"
    else:
        return {
            "status": "UNAVAILABLE",
            "algorithm": position.algorithm,
            "reason": "no supported dynamic counter or resource tool found",
        }

    profile_id = f"{position.block_index:02d}-{position.algorithm}"
    profile_dir = run_dir / "profiles" / profile_id
    profile_dir.mkdir(parents=True, exist_ok=False)
    started = utc_now()
    try:
        completed = run_capture(
            argv,
            cwd=REPO_ROOT,
            env=executor.environment,
            timeout=executor.timeout_seconds,
        )
        returncode: int | None = completed.returncode
        stdout = completed.stdout
        stderr = completed.stderr
        timed_out = False
    except subprocess.TimeoutExpired as error:
        returncode = None
        stdout = error.stdout or b""
        stderr = error.stderr or b""
        timed_out = True
    write_bytes_once(profile_dir / "stdout.jsonl", stdout)
    write_bytes_once(profile_dir / "stderr.txt", stderr)
    record = {
        "status": "COMPLETE" if returncode == 0 and not timed_out else "FAILED",
        "tool": tool_name,
        "algorithm": position.algorithm,
        "started_at": started,
        "ended_at": utc_now(),
        "argv": argv,
        "cwd": str(REPO_ROOT),
        "environment": executor.environment,
        "returncode": returncode,
        "timed_out": timed_out,
        "binary_sha256_before": executor.binary_hash,
        "binary_sha256_after": sha256_file(executor.binary),
        "stdout_sha256": sha256_bytes(stdout),
        "stderr_sha256": sha256_bytes(stderr),
    }
    write_json_once(profile_dir / "profile.json", record)
    return record


def collect_macos_sample(executor: HarnessExecutor) -> dict[str, Any]:
    sample_tool = Path("/usr/bin/sample")
    if sys.platform != "darwin" or not sample_tool.is_file():
        return {"status": "UNAVAILABLE", "reason": "/usr/bin/sample is unavailable"}
    position = Position(
        phase="profile",
        contrast_id="profile_macos_sample_parallel4_n4194304",
        block_index=99,
        template="S",
        position_index=1,
        label="parallel",
        algorithm="parallel",
        n=4_194_304,
        pattern="mixed",
        seed=ASSIGNMENT_SEED,
        block_size=1,
        workers=4,
    )
    harness_argv = executor.harness_argv(position, 1, 256)
    if executor.taskset is not None:
        harness_argv = [executor.taskset, "-c", str(executor.cpu_list), *harness_argv]
    profile_dir = executor.run_dir / "profiles" / "macos-sample-parallel4"
    profile_dir.mkdir(parents=True, exist_ok=False)
    profile_output = profile_dir / "sample.txt"
    launch = {
        "schema_version": SCHEMA_VERSION,
        "status": "STARTED",
        "started_at": utc_now(),
        "harness_argv": harness_argv,
        "cwd": str(REPO_ROOT),
        "environment": executor.environment,
        "warmups": 1,
        "samples": 256,
        "sample_duration_seconds": 1,
        "binary_sha256_before": sha256_file(executor.binary),
        "sample_tool": str(sample_tool),
        "sample_tool_sha256": sha256_file(sample_tool),
    }
    write_json_once(profile_dir / "launch.json", launch)
    # The context manager reaps the harness child even when sampling raises
    # before communicate(), so no process handle or pipe descriptor leaks.
    with subprocess.Popen(
        harness_argv,
        cwd=REPO_ROOT,
        env=executor.environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    ) as process_handle:
        time.sleep(0.05)
        sample_argv = [str(sample_tool), str(process_handle.pid), "1", "-file", str(profile_output)]
        try:
            sample_completed = run_capture(
                sample_argv,
                cwd=REPO_ROOT,
                env=executor.environment,
                timeout=15.0,
            )
            sample_timed_out = False
        except subprocess.TimeoutExpired as error:
            sample_completed = subprocess.CompletedProcess(
                sample_argv,
                returncode=124,
                stdout=error.stdout or b"",
                stderr=error.stderr or b"",
            )
            sample_timed_out = True
        harness_timed_out = False
        try:
            harness_stdout, harness_stderr = process_handle.communicate(timeout=executor.timeout_seconds)
        except subprocess.TimeoutExpired:
            harness_timed_out = True
            process_handle.kill()
            harness_stdout, harness_stderr = process_handle.communicate()
    write_bytes_once(profile_dir / "harness.stdout.jsonl", harness_stdout)
    write_bytes_once(profile_dir / "harness.stderr.txt", harness_stderr)
    write_bytes_once(profile_dir / "sample.stdout", sample_completed.stdout)
    write_bytes_once(profile_dir / "sample.stderr", sample_completed.stderr)
    record = {
        "status": (
            "COMPLETE"
            if sample_completed.returncode == 0
            and process_handle.returncode == 0
            and profile_output.is_file()
            and not sample_timed_out
            and not harness_timed_out
            else "FAILED"
        ),
        "ended_at": utc_now(),
        "harness_pid": process_handle.pid,
        "harness_returncode": process_handle.returncode,
        "harness_timed_out": harness_timed_out,
        "sample_argv": sample_argv,
        "sample_returncode": sample_completed.returncode,
        "sample_timed_out": sample_timed_out,
        "profile_output_present": profile_output.is_file(),
        "profile_output_sha256": sha256_file(profile_output) if profile_output.is_file() else None,
        "binary_sha256_after": sha256_file(executor.binary),
        "failure_boundary": (
            "A missing process, permission failure, unsupported sampler, timeout, or nonzero "
            "exit remains a retained profile failure. It does not become timing evidence."
        ),
    }
    write_json_once(profile_dir / "result.json", record)
    return record


def run_profile_phase(executor: HarnessExecutor) -> dict[str, Any]:
    positions = profile_positions()
    measured_attempts = []
    dynamic_profiles = []
    for position in positions:
        result = executor.run_position(position, DEFAULT_WARMUPS, DEFAULT_SAMPLES)
        measured_attempts.append(result.attempt_id)
        if result.status != "VALID":
            raise ExperimentIncomplete(f"profile canary failed: {result.attempt_id}")
        dynamic_profiles.append(collect_dynamic_profile(executor.run_dir, executor, position))
    disassembly = collect_disassembly(executor.run_dir, executor.binary, executor.environment)
    macos_sample = collect_macos_sample(executor)
    return {
        "status": "COMPLETE",
        "measured_attempts": measured_attempts,
        "dynamic_profiles": dynamic_profiles,
        "linked_image_disassembly": disassembly,
        "macos_sampling_profile": macos_sample,
        "mechanism_boundary": (
            "Elapsed time and resource totals do not establish a cache or bandwidth mechanism. "
            "Unsupported or permission-denied counters leave that mechanism inferred."
        ),
    }


def run_block_phase(executor: HarnessExecutor, phase: str) -> None:
    if phase == "pilot":
        block_count = PILOT_BLOCKS
    elif phase == "main":
        block_count = MAIN_BLOCKS
    elif phase == "aa":
        block_count = AA_BLOCKS
    elif phase == "quick":
        block_count = 1
    else:
        raise ValueError(f"not a block phase: {phase}")

    contrasts = (AA_CONTRAST,) if phase == "aa" else CONTRASTS
    for contrast in contrasts:
        if phase == "quick":
            templates = ["ABBA"]
        else:
            templates = balanced_templates(block_count, f"{phase}:{contrast.contrast_id}" if phase != "aa" else "aa")
        for block_index, template in enumerate(templates, start=1):
            executor.run_block(
                phase,
                contrast,
                block_index,
                template,
                0 if phase == "quick" else DEFAULT_WARMUPS,
                1 if phase == "quick" else DEFAULT_SAMPLES,
            )


def run_distribution_phase(executor: HarnessExecutor) -> None:
    index = 0
    for pattern in AUDIT_PATTERNS:
        for algorithm, block_size, workers in AUDIT_ALGORITHMS:
            index += 1
            position = Position(
                phase="distribution",
                contrast_id=f"distribution_{pattern}_{algorithm}",
                block_index=index,
                template="D",
                position_index=1,
                label=algorithm,
                algorithm=algorithm,
                n=262_144,
                pattern=pattern,
                seed=ASSIGNMENT_SEED,
                block_size=block_size,
                workers=workers,
            )
            result = executor.run_position(position, DEFAULT_WARMUPS, DEFAULT_SAMPLES)
            if result.status != "VALID":
                raise ExperimentIncomplete(f"distribution audit attempt failed: {result.attempt_id}")


def execute_phase(executor: HarnessExecutor, phase: str) -> dict[str, Any]:
    if phase in ("quick", "pilot", "main", "aa"):
        run_block_phase(executor, phase)
    elif phase == "distribution":
        run_distribution_phase(executor)
    elif phase == "profile":
        result = run_profile_phase(executor)
        write_json_once(executor.run_dir / "analysis" / "profile.json", result)
        return result
    else:
        raise ValueError(f"unknown execution phase: {phase}")

    if phase == "quick":
        result = analyze_quick(executor)
    elif phase == "pilot":
        result = analyze_pilot(executor)
    elif phase == "main":
        result = analyze_main(executor)
    elif phase == "aa":
        result = analyze_aa(executor)
    else:
        result = analyze_distribution(executor)
    write_json_once(executor.run_dir / "analysis" / f"{phase}.json", result)
    if result.get("status") != "COMPLETE":
        raise ExperimentIncomplete(
            f"{phase} analysis reported status {result.get('status')!r}"
        )
    return result


def phase_sequence(requested: str) -> list[str]:
    if requested in ("all", "plan"):
        return ["pilot", "main", "aa", "distribution", "profile"]
    if requested == "self-check":
        return []
    return [requested]


def protocol_document(requested_phase: str, rustflags: str, timeout: float) -> dict[str, Any]:
    phases = phase_sequence(requested_phase)
    return {
        "schema_version": SCHEMA_VERSION,
        "frozen_before_collection": True,
        "requested_phase": requested_phase,
        "phase_sequence": phases,
        "assignment_seed": ASSIGNMENT_SEED,
        "pilot_blocks_per_contrast": PILOT_BLOCKS,
        "main_blocks_per_contrast": MAIN_BLOCKS,
        "aa_blocks_total": AA_BLOCKS,
        "primary_family_size": FAMILY_SIZE,
        "familywise_alpha": FAMILY_ALPHA,
        "interval": "two-sided paired Student t on complete-block log contrasts",
        "multiplicity": "Bonferroni",
        "practical_ratio_boundary": PRACTICAL_RATIO,
        "warmups_per_position": DEFAULT_WARMUPS,
        "timed_subsamples_per_position": DEFAULT_SAMPLES,
        "timeout_seconds": timeout,
        "rustflags": rustflags,
        "primary_contrasts": [asdict(contrast) for contrast in CONTRASTS],
        "planned_schedule": planned_schedule([phase for phase in phases if phase != "profile"]),
        "retry_rule": (
            "One whole-block retry only after external SIGINT, SIGTERM, or SIGHUP. "
            "No retry for treatment, timeout, parse, checksum, reset, or artifact failures."
        ),
        "quick_evidence_boundary": "Exploratory, non-promotable, one block, no warmup, one subsample.",
    }


def checksum_manifest(run_dir: Path) -> tuple[str, int]:
    excluded = {"manifest.sha256"}
    records = []
    for path in sorted(run_dir.rglob("*")):
        if path.is_file() and path.name not in excluded:
            records.append((path.relative_to(run_dir).as_posix(), sha256_file(path)))
    text = "".join(f"{digest}  {relative}\n" for relative, digest in records)
    write_text_once(run_dir / "manifest.sha256", text)
    for relative, expected in records:
        if sha256_file(run_dir / relative) != expected:
            raise RuntimeError(f"bundle checksum verification failed: {relative}")
    return sha256_bytes(text.encode("utf-8")), len(records)


def create_run_directory(requested: Path | None, phase: str) -> Path:
    if requested is not None:
        run_dir = requested.expanduser().resolve()
    else:
        stamp = datetime.now(UTC).strftime("%Y%m%dT%H%M%SZ")
        run_dir = RUNS_DIR / f"{stamp}-{phase}-{uuid.uuid4().hex[:8]}"
    run_dir.mkdir(parents=True, exist_ok=False)
    return run_dir


def self_check() -> dict[str, Any]:
    errors = []
    tool_identities: dict[str, dict[str, Any]] = {}
    identity_environment = safe_base_environment()
    for tool_name, arguments in (("rustc", ["-vV"]), ("cargo", ["-V"])):
        tool_path = executable(tool_name)
        completed = run_capture(
            [tool_path, *arguments],
            cwd=REPO_ROOT,
            env=identity_environment,
            timeout=15.0,
        )
        output = completed.stdout.decode("utf-8", errors="replace").strip()
        tool_identities[tool_name] = {
            "selected_path": tool_path,
            "symlink_target": str(Path(tool_path).resolve()),
            "sha256": sha256_file(Path(tool_path)),
            "returncode": completed.returncode,
            "stdout": output,
            "stderr": completed.stderr.decode("utf-8", errors="replace").strip(),
        }
        if completed.returncode != 0 or not output:
            errors.append(f"{tool_name} identity invocation failed through {tool_path}")
    rustc_output = tool_identities.get("rustc", {}).get("stdout", "")
    if not any(line.startswith("host: ") for line in str(rustc_output).splitlines()):
        errors.append("rustc identity output is missing the host triple")
    if len(CONTRASTS) != FAMILY_SIZE:
        errors.append("primary family size mismatch")
    if len({contrast.contrast_id for contrast in CONTRASTS}) != len(CONTRASTS):
        errors.append("duplicate contrast ID")
    if balanced_templates(4, "self-check").count("ABBA") != 2:
        errors.append("four-block template assignment is unbalanced")
    if balanced_templates(12, "self-check").count("ABBA") != 6:
        errors.append("12-block template assignment is unbalanced")
    expected_position_count = FAMILY_SIZE * (PILOT_BLOCKS + MAIN_BLOCKS) * 4 + AA_BLOCKS * 4 + 15
    actual_position_count = len(planned_schedule(["pilot", "main", "aa", "distribution"]))
    if actual_position_count != expected_position_count:
        errors.append(
            f"planned position count mismatch: expected {expected_position_count}, got {actual_position_count}"
        )
    known_quantile = student_t_quantile(0.975, 10)
    if abs(known_quantile - 2.228_138_852) > 1.0e-8:
        errors.append(f"Student t quantile check failed: {known_quantile}")
    return {
        "status": "PASS" if not errors else "FAIL",
        "errors": errors,
        "primary_contrast_count": len(CONTRASTS),
        "full_nonprofile_position_count": actual_position_count,
        "student_t_0.975_df10": known_quantile,
        "tool_identities": tool_identities,
    }


def parse_cli() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "phase",
        choices=("plan", "self-check", "quick", "pilot", "main", "aa", "distribution", "profile", "all"),
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        help="new evidence directory; default is measurements/runs/<timestamp>-<phase>-<id>",
    )
    parser.add_argument("--timeout-seconds", type=float, default=DEFAULT_TIMEOUT_SECONDS)
    parser.add_argument("--rustflags", default="-C target-cpu=native")
    parser.add_argument(
        "--cpu-list",
        help="Linux taskset CPU list; the runner verifies it is within the granted affinity",
    )
    return parser.parse_args()


def run_experiment(args: argparse.Namespace) -> int:
    if not math.isfinite(args.timeout_seconds) or args.timeout_seconds <= 0.0:
        raise ValueError("--timeout-seconds must be a finite value greater than zero")
    protocol = protocol_document(args.phase, args.rustflags, args.timeout_seconds)
    if args.phase == "plan":
        print(json.dumps(protocol, indent=2, sort_keys=True))
        return 0
    if args.phase == "self-check":
        result = self_check()
        print(json.dumps(result, indent=2, sort_keys=True))
        return 0 if result["status"] == "PASS" else 1

    run_dir = create_run_directory(args.output_dir, args.phase)
    status: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "status": "STARTED",
        "started_at": utc_now(),
        "requested_phase": args.phase,
        "completed_phases": [],
    }
    replace_json(run_dir / "run-status.json", status)
    write_json_once(run_dir / "protocol.json", protocol)
    initial_source = source_tree_manifest(run_dir)
    write_json_once(run_dir / "metadata" / "source-tree-before.json", initial_source)
    initial_source_digest = source_tree_digest(initial_source)
    build_env = safe_base_environment()
    relay = ExternalSignalRelay()
    relay.install()
    failure: BaseException | None = None
    try:
        write_json_once(run_dir / "metadata" / "host.json", host_metadata())
        write_json_once(run_dir / "metadata" / "git.json", git_metadata(run_dir, build_env))
        binary, build = build_benchmark(run_dir, args.rustflags)
        executor = HarnessExecutor(
            run_dir,
            binary,
            build["binary_sha256"],
            args.timeout_seconds,
            args.cpu_list,
            relay,
        )
        write_json_once(
            run_dir / "metadata" / "benchmark-environment.json",
            {
                "cwd": str(REPO_ROOT),
                "environment": executor.environment,
                "requested_cpu_list": args.cpu_list,
            },
        )
        for phase in phase_sequence(args.phase):
            execute_phase(executor, phase)
            relay.raise_if_pending()
            status["completed_phases"].append(phase)
            status["last_progress_at"] = utc_now()
            replace_json(run_dir / "run-status.json", status)

        final_source = source_tree_manifest(run_dir)
        write_json_once(run_dir / "metadata" / "source-tree-after.json", final_source)
        if source_tree_digest(final_source) != initial_source_digest:
            raise ExperimentIncomplete("source tree changed during evidence collection")
        if sha256_file(binary) != build["binary_sha256"]:
            raise ExperimentIncomplete("retained benchmark image changed during evidence collection")
        relay.raise_if_pending()
        status["status"] = "COMPLETE"
        status["ended_at"] = utc_now()
        status["attempt_count"] = len(executor.attempts)
        status["block_record_count"] = len(executor.blocks)
    except BaseException as error:
        failure = error
        status["status"] = "INCOMPLETE"
        status["ended_at"] = utc_now()
        status["failure_type"] = type(error).__name__
        status["failure"] = str(error)
        write_text_once(run_dir / "failure.txt", traceback.format_exc())
    replace_json(run_dir / "run-status.json", status)
    manifest_digest, manifest_count = checksum_manifest(run_dir)
    print(
        json.dumps(
            {
                "run_directory": str(run_dir),
                "status": status["status"],
                "manifest_sha256": manifest_digest,
                "manifest_file_count": manifest_count,
            },
            sort_keys=True,
        )
    )
    relay.restore()
    if failure is not None:
        return 1
    return 0


def main() -> int:
    try:
        return run_experiment(parse_cli())
    except (OSError, RuntimeError, ValueError) as error:
        print(f"run_experiment.py: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
