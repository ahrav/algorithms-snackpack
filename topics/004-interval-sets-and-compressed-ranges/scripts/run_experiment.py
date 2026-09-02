#!/usr/bin/env python3
"""Collect the frozen Topic 004 interval-set experiment and retain every attempt."""

from __future__ import annotations

import sys

if sys.version_info < (3, 11):
    raise SystemExit("run_experiment.py requires Python 3.11 or newer")

import argparse
import csv
import errno
import hashlib
import json
import math
import os
import platform
import random
import shutil
import signal
import statistics
import subprocess
import time
import tomllib
import traceback
from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any, Iterable, Sequence

SCHEMA_VERSION = 1
ASSIGNMENT_SEED = 2_026_090_104
FAMILY_ALPHA = 0.05
FAMILY_SIZE = 32
PRACTICAL_RATIO = 1.05
PILOT_BLOCKS = 4
MAIN_BLOCKS = 12
AA_BLOCKS = 12
DEFAULT_WARMUPS = 1
DEFAULT_SAMPLES = 3
DEFAULT_INNER = 1
DEFAULT_TIMEOUT_SECONDS = 180.0
GENERATOR_VERSION = "topic004-fixture-v1"
MAX_ATTEMPTS_PER_POSITION = 1
PROFILE_REPETITIONS = 64
PILOT_SENSITIVITY_MULTIPLIERS = (0.5, 1.0, 1.5, 2.0)
OPERATION_PHASES = ("build", "build_membership")
WORK_FIELDS = (
    "input_intervals",
    "unique_input_intervals",
    "duplicate_intervals",
    "sort_comparisons_count_pass",
    "merge_comparisons",
    "output_runs",
    "membership_queries",
    "canonical_binary_search_comparisons",
    "result_scalar_slots",
)
EXPECTED_LINKED_SYMBOL_SUBSTRINGS = (
    "topic004_oracle_workload",
    "topic004_flat_workload",
    "topic004_packed_workload",
    "topic004_events_workload",
    "topic004_btree_workload",
    "topic004_timed_loop",
    "topic004_consume_result",
    "topic004_clock_now",
)
EXTERNAL_SIGNALS = {
    resolved
    for name in ("SIGINT", "SIGTERM", "SIGHUP")
    if (resolved := getattr(signal, name, None)) is not None
}

SCRIPT = Path(__file__).resolve()
TOPIC_DIR = SCRIPT.parents[1]
REPO_ROOT = SCRIPT.parents[3]
BENCH_SOURCE = TOPIC_DIR / "benches" / "interval_sets.rs"

CANDIDATES = ("oracle", "flat", "packed", "events", "btree")
BASELINE = "flat"
COMPARATORS = ("oracle", "packed", "events", "btree")


@dataclass(frozen=True)
class Cell:
    name: str
    input_intervals: int
    unique_intervals: int
    duplicate_intervals: int
    order: str
    output_runs: int
    membership_queries: int
    shape: str


CELLS = (
    Cell("tiny_sparse_sorted_unique", 8, 8, 0, "sorted", 8, 64, "tiny_sparse"),
    Cell(
        "cache_clustered_shuffled_duplicates",
        320,
        256,
        64,
        "shuffled",
        8,
        1_024,
        "eight_overlapping_clusters",
    ),
    Cell(
        "large_sparse_reverse_unique",
        4_096,
        4_096,
        0,
        "reverse",
        4_096,
        4_096,
        "large_sparse",
    ),
    Cell(
        "large_adjacent_shuffled_duplicates",
        4_608,
        4_096,
        512,
        "shuffled",
        1,
        4_096,
        "adjacent_coalescing",
    ),
)
CELL_BY_NAME = {cell.name: cell for cell in CELLS}


@dataclass(frozen=True)
class Contrast:
    contrast_id: str
    b_candidate: str
    cell: str
    operation_phase: str


CONTRASTS = tuple(
    Contrast(
        f"{candidate}_vs_flat__{cell.name}__{operation_phase}",
        candidate,
        cell.name,
        operation_phase,
    )
    for operation_phase in OPERATION_PHASES
    for cell in CELLS
    for candidate in COMPARATORS
)
AA_CONTRAST = Contrast(
    "aa_flat__cache_clustered_shuffled_duplicates__build_membership",
    "flat",
    "cache_clustered_shuffled_duplicates",
    "build_membership",
)
PROFILE_SPECS = (
    ("oracle", "tiny_sparse_sorted_unique", "build"),
    ("flat", "large_sparse_reverse_unique", "build"),
    ("packed", "large_sparse_reverse_unique", "build_membership"),
    ("events", "cache_clustered_shuffled_duplicates", "build"),
    ("btree", "large_adjacent_shuffled_duplicates", "build_membership"),
)


@dataclass(frozen=True)
class Position:
    phase: str
    contrast_id: str
    block_index: int
    template: str
    position_index: int
    label: str
    candidate: str
    cell: str
    operation_phase: str
    seed: int
    warmups: int
    samples: int
    inner: int


@dataclass
class Attempt:
    attempt_id: str
    phase: str
    contrast_id: str
    block_index: int
    template: str
    position_index: int
    label: str
    candidate: str
    cell: str
    operation_phase: str
    seed: int
    status: str
    invalid_reason: str
    returncode: int | None
    timed_out: bool
    position_ns: int | None
    zero_samples: int
    fixture_hash: str
    output_hash: str
    work_sha256: str
    canary_sha256: str
    external_interruption: bool
    stdout_sha256: str
    stderr_sha256: str
    executable_sha256_before: str
    executable_sha256_after: str
    attempt_directory: str


class ExperimentIncomplete(RuntimeError):
    """The frozen schedule or an evidence gate did not complete."""


class ExternalSignalRelay:
    """Forward external termination signals to the active attempt group.

    A signal can otherwise terminate the orchestrator before it writes the
    partial attempt and final checksum manifest. Signals that arrive without
    a child in flight are held until the next safe checkpoint.
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
                os.killpg(child.pid, signum)
                return
            except ProcessLookupError:
                pass
        self.pending.append(signum)

    def adopt_child(self, child: subprocess.Popen[bytes]) -> None:
        self.child = child
        while self.pending:
            signum = self.pending[0]
            if child.poll() is not None:
                return
            try:
                os.killpg(child.pid, signum)
            except ProcessLookupError:
                return
            self.pending.pop(0)

    def release_child(self) -> None:
        self.child = None

    def raise_if_pending(self) -> None:
        if self.pending:
            names = ", ".join(signal.Signals(signum).name for signum in self.pending)
            raise ExperimentIncomplete(
                f"external signal received with no attempt in flight: {names}"
            )


def utc_now() -> str:
    return datetime.now(UTC).isoformat(timespec="microseconds")


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def binary_digest(path: Path) -> str:
    try:
        return sha256_file(path)
    except OSError as error:
        return f"UNREADABLE:{error.__class__.__name__}"


def write_bytes_once(path: Path, value: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("xb") as stream:
        stream.write(value)


def write_text_once(path: Path, value: str) -> None:
    write_bytes_once(path, value.encode("utf-8"))


def write_json_once(path: Path, value: Any) -> None:
    write_text_once(path, json.dumps(value, indent=2, sort_keys=True) + "\n")


def replace_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(path.name + ".tmp")
    temporary.write_text(
        json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    temporary.replace(path)


def stable_seed(label: str) -> int:
    digest = hashlib.sha256(f"{ASSIGNMENT_SEED}:{label}".encode()).digest()
    return int.from_bytes(digest[:8], "little")


def balanced_templates(count: int, label: str) -> list[str]:
    if count <= 0 or count % 2:
        raise ValueError("complete-block template count must be positive and even")
    templates = ["ABBA"] * (count // 2) + ["BAAB"] * (count // 2)
    random.Random(stable_seed(label)).shuffle(templates)
    return templates


def block_seed(phase: str, contrast_id: str, block_index: int) -> int:
    return stable_seed(f"fixture:{phase}:{contrast_id}:{block_index}")


def positions_for_block(
    phase: str,
    contrast: Contrast,
    block_index: int,
    template: str,
    warmups: int,
    samples: int,
) -> list[Position]:
    if template not in ("ABBA", "BAAB"):
        raise ValueError(f"unsupported template {template!r}")
    seed = stable_seed(
        f"fixture:{phase}:{contrast.cell}:{contrast.operation_phase}:{block_index}"
    )
    positions = []
    for index, label in enumerate(template, start=1):
        candidate = BASELINE if label == "A" else contrast.b_candidate
        positions.append(
            Position(
                phase,
                contrast.contrast_id,
                block_index,
                template,
                index,
                label,
                candidate,
                contrast.cell,
                contrast.operation_phase,
                seed,
                warmups,
                samples,
                DEFAULT_INNER,
            )
        )
    return positions


def planned_schedule(phases: Sequence[str]) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for phase in phases:
        if phase in ("pilot", "main"):
            count = PILOT_BLOCKS if phase == "pilot" else MAIN_BLOCKS
            for contrast in CONTRASTS:
                for block_index, template in enumerate(
                    balanced_templates(count, f"{phase}:{contrast.contrast_id}"),
                    start=1,
                ):
                    rows.extend(
                        asdict(position)
                        for position in positions_for_block(
                            phase,
                            contrast,
                            block_index,
                            template,
                            DEFAULT_WARMUPS,
                            DEFAULT_SAMPLES,
                        )
                    )
        elif phase == "aa":
            for block_index, template in enumerate(
                balanced_templates(AA_BLOCKS, "aa"), start=1
            ):
                rows.extend(
                    asdict(position)
                    for position in positions_for_block(
                        phase,
                        AA_CONTRAST,
                        block_index,
                        template,
                        DEFAULT_WARMUPS,
                        DEFAULT_SAMPLES,
                    )
                )
        elif phase == "quick":
            for operation_phase in OPERATION_PHASES:
                for cell in CELLS:
                    for candidate in CANDIDATES:
                        rows.append(
                            asdict(
                                Position(
                                    phase,
                                    f"quick__{candidate}__{cell.name}__{operation_phase}",
                                    1,
                                    "Q",
                                    1,
                                    "Q",
                                    candidate,
                                    cell.name,
                                    operation_phase,
                                    stable_seed(
                                        f"fixture:{phase}:{cell.name}:{operation_phase}:1"
                                    ),
                                    0,
                                    1,
                                    DEFAULT_INNER,
                                )
                            )
                        )
    return rows


def phase_sequence(requested: str) -> list[str]:
    if requested in ("plan", "all"):
        return ["pilot", "main", "aa", "profile"]
    if requested == "self-check":
        return []
    return [requested]


def protocol_document(requested: str, timeout: float, rustflags: str) -> dict[str, Any]:
    phases = phase_sequence(requested)
    schedule = planned_schedule([phase for phase in phases if phase != "profile"])
    return {
        "schema_version": SCHEMA_VERSION,
        "frozen_before_collection": True,
        "requested_mode": requested,
        "phase_sequence": phases,
        "assignment_seed": ASSIGNMENT_SEED,
        "generator_version": GENERATOR_VERSION,
        "pilot_blocks_per_contrast": PILOT_BLOCKS,
        "main_blocks_per_contrast": MAIN_BLOCKS,
        "aa_blocks": AA_BLOCKS,
        "primary_family_size": FAMILY_SIZE,
        "familywise_alpha": FAMILY_ALPHA,
        "multiplicity": "Bonferroni simultaneous paired-t intervals",
        "practical_ratio_boundary": PRACTICAL_RATIO,
        "analysis_unit": "one complete four-position ABBA or BAAB block contrast",
        "subsample_unit": "one timed batch within a fresh process position",
        "interference_boundary": (
            "fresh processes reset candidate state; caches, thermals, allocator, kernel, "
            "and frequency state may persist"
        ),
        "cells": [asdict(cell) for cell in CELLS],
        "primary_contrasts": [asdict(contrast) for contrast in CONTRASTS],
        "planned_schedule": schedule,
        "planned_position_count": len(schedule),
        "dynamic_profile_specs": [
            {"candidate": candidate, "cell": cell, "operation_phase": operation_phase}
            for candidate, cell, operation_phase in PROFILE_SPECS
        ],
        "timeout_seconds": timeout,
        "attempts_per_position": MAX_ATTEMPTS_PER_POSITION,
        "retry_policy": "none",
        "rustflags": rustflags,
        "invalid_attempt_policy": (
            "retain every attempt; any signal, timeout, nonzero exit, parse, canary, fixture, "
            "sample, source, reset, or image failure makes its complete block invalid; "
            "never replace or omit it"
        ),
        "stopping": "fixed block counts; no outcome-dependent extension or peeking",
    }


def run_capture(
    argv: Sequence[str],
    *,
    cwd: Path = REPO_ROOT,
    env: dict[str, str] | None = None,
    timeout: float = 60.0,
) -> dict[str, Any]:
    started = time.monotonic_ns()
    try:
        result = subprocess.run(
            list(argv),
            cwd=cwd,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout,
            check=False,
        )
        return {
            "argv": list(argv),
            "returncode": result.returncode,
            "timed_out": False,
            "elapsed_ns": time.monotonic_ns() - started,
            "stdout": result.stdout.decode("utf-8", "replace"),
            "stderr": result.stderr.decode("utf-8", "replace"),
        }
    except subprocess.TimeoutExpired as error:
        return {
            "argv": list(argv),
            "returncode": None,
            "timed_out": True,
            "elapsed_ns": time.monotonic_ns() - started,
            "stdout": (error.stdout or b"").decode("utf-8", "replace"),
            "stderr": (error.stderr or b"").decode("utf-8", "replace"),
        }


def run_capture_group(
    argv: Sequence[str],
    *,
    cwd: Path,
    env: dict[str, str],
    timeout: float,
    relay: ExternalSignalRelay | None,
) -> tuple[int | None, bytes, bytes, bool, str | None]:
    """Run one retained attempt and kill its whole process group on timeout."""

    try:
        process = subprocess.Popen(
            list(argv),
            cwd=cwd,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
        )
    except (OSError, subprocess.SubprocessError) as error:
        return None, b"", b"", False, f"spawn failure: {error!r}"
    with process:
        if relay is not None:
            relay.adopt_child(process)
        try:
            stdout, stderr = process.communicate(timeout=timeout)
            return process.returncode, stdout, stderr, False, None
        except subprocess.TimeoutExpired:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            stdout, stderr = process.communicate()
            return process.returncode, stdout, stderr, True, None
        finally:
            if relay is not None:
                relay.release_child()


def executable(name: str) -> str:
    value = shutil.which(name)
    if value is None:
        raise RuntimeError(f"required executable {name!r} was not found")
    return str(Path(value).absolute())


def cargo_config_paths(build_env: dict[str, str]) -> list[Path]:
    paths: list[Path] = []
    current = REPO_ROOT.resolve()
    while True:
        paths.extend(
            (current / ".cargo" / "config.toml", current / ".cargo" / "config")
        )
        if current.parent == current:
            break
        current = current.parent
    cargo_home = build_env.get("CARGO_HOME")
    if cargo_home is None and "HOME" in build_env:
        cargo_home = str(Path(build_env["HOME"]) / ".cargo")
    if cargo_home is not None:
        paths.extend((Path(cargo_home) / "config.toml", Path(cargo_home) / "config"))
    return [path for path in paths if path.is_file()]


def inspect_runner_configuration(
    host_triple: str, build_env: dict[str, str]
) -> dict[str, Any]:
    """Fail closed when direct native replay could bypass Cargo configuration."""

    environment_key = (
        "CARGO_TARGET_" + host_triple.upper().replace("-", "_") + "_RUNNER"
    )
    findings: list[dict[str, str]] = []
    if build_env.get(environment_key):
        findings.append(
            {"source": environment_key, "value": build_env[environment_key]}
        )
    if build_env.get("CARGO_BUILD_TARGET") not in (None, "", host_triple):
        findings.append(
            {"source": "CARGO_BUILD_TARGET", "value": build_env["CARGO_BUILD_TARGET"]}
        )
    parsed_paths: list[dict[str, str]] = []
    for path in cargo_config_paths(build_env):
        parsed_paths.append({"path": str(path), "sha256": sha256_file(path)})
        with path.open("rb") as stream:
            document = tomllib.load(stream)
        target = document.get("target", {})
        if isinstance(target, dict):
            for key, table in target.items():
                if not isinstance(table, dict):
                    continue
                if key != host_triple and not key.startswith("cfg("):
                    continue
                for setting in ("runner", "linker"):
                    if setting in table:
                        findings.append(
                            {
                                "source": f"{path}:target.{key}.{setting}",
                                "value": repr(table[setting]),
                            }
                        )
        build = document.get("build", {})
        if isinstance(build, dict):
            if build.get("target") not in (None, host_triple):
                findings.append(
                    {"source": f"{path}:build.target", "value": repr(build["target"])}
                )
            for key in ("rustc", "rustc-wrapper", "rustc-workspace-wrapper"):
                if build.get(key):
                    findings.append(
                        {"source": f"{path}:build.{key}", "value": repr(build[key])}
                    )
    return {
        "host_triple": host_triple,
        "environment_key_checked": environment_key,
        "config_paths_checked": parsed_paths,
        "ambiguous_or_non_native_findings": findings,
    }


def build_environment(rustflags: str) -> dict[str, str]:
    allowed = (
        "PATH",
        "HOME",
        "TMPDIR",
        "LANG",
        "LC_ALL",
        "CARGO_HOME",
        "RUSTUP_HOME",
        "RUSTUP_TOOLCHAIN",
        "SDKROOT",
        "DEVELOPER_DIR",
    )
    environment = {key: os.environ[key] for key in allowed if key in os.environ}
    environment["RUSTFLAGS"] = rustflags
    environment["CARGO_TERM_COLOR"] = "never"
    environment["CARGO_INCREMENTAL"] = "0"
    environment["RUST_BACKTRACE"] = "1"
    environment["LANG"] = "C"
    environment["LC_ALL"] = "C"
    environment["TZ"] = "UTC"
    return environment


def benchmark_environment() -> dict[str, str]:
    allowed = (
        "PATH",
        "TMPDIR",
        "LANG",
        "LC_ALL",
        "DYLD_LIBRARY_PATH",
        "LD_LIBRARY_PATH",
    )
    environment = {key: os.environ[key] for key in allowed if key in os.environ}
    environment["RUST_BACKTRACE"] = "1"
    environment["LC_ALL"] = "C"
    environment["LANG"] = "C"
    environment["TZ"] = "UTC"
    return environment


def safe_recorded_environment(env: dict[str, str]) -> dict[str, str]:
    # The build and benchmark environments are already strict allowlists.
    return dict(sorted(env.items()))


def source_paths() -> Iterable[Path]:
    # `rust-toolchain.toml` selects the toolchain and profile used for builds,
    # so it affects the digest.
    roots = [
        REPO_ROOT / "Cargo.toml",
        REPO_ROOT / "Cargo.lock",
        REPO_ROOT / "rust-toolchain.toml",
    ]
    for path in roots:
        if path.is_file():
            yield path
    for path in sorted(TOPIC_DIR.rglob("*")):
        if not path.is_file():
            continue
        relative = path.relative_to(TOPIC_DIR)
        if "runs" in relative.parts or "__pycache__" in relative.parts:
            continue
        if path.suffix in (".pyc", ".trace"):
            continue
        yield path


def source_manifest() -> list[dict[str, Any]]:
    records = []
    for path in source_paths():
        relative = path.relative_to(REPO_ROOT)
        records.append(
            {
                "path": relative.as_posix(),
                "bytes": path.stat().st_size,
                "sha256": sha256_file(path),
            }
        )
    return records


def source_digest(records: Sequence[dict[str, Any]]) -> str:
    payload = json.dumps(list(records), sort_keys=True, separators=(",", ":")).encode()
    return sha256_bytes(payload)


def capture_source_snapshot(run_dir: Path, records: Sequence[dict[str, Any]]) -> None:
    for record in records:
        relative = Path(record["path"])
        source = REPO_ROOT / relative
        destination = run_dir / "source-snapshot" / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        data = source.read_bytes()
        if sha256_bytes(data) != record["sha256"]:
            raise ExperimentIncomplete(f"source changed while copying {relative}")
        write_bytes_once(destination, data)


def git_metadata(env: dict[str, str]) -> dict[str, Any]:
    commands = {
        "head": ["git", "rev-parse", "HEAD"],
        "branch": ["git", "branch", "--show-current"],
        "status": ["git", "status", "--porcelain=v1", "--untracked-files=all"],
        "worktrees": ["git", "worktree", "list", "--porcelain"],
    }
    metadata = {name: run_capture(argv, env=env) for name, argv in commands.items()}
    for name, argv in {
        "dirty_diff": ["git", "diff", "--binary", "--no-ext-diff"],
        "staged_diff": ["git", "diff", "--binary", "--no-ext-diff", "--cached"],
    }.items():
        capture = run_capture(argv, env=env)
        output = capture.pop("stdout")
        capture["stdout_bytes"] = len(output.encode())
        capture["stdout_sha256"] = sha256_bytes(output.encode())
        metadata[name] = capture
    return metadata


def sanitized_darwin_hardware() -> dict[str, Any]:
    command = ["/usr/sbin/system_profiler", "SPHardwareDataType", "-json"]
    result = run_capture(command, timeout=30.0)
    output: dict[str, Any] = {
        "argv": command,
        "returncode": result["returncode"],
        "timed_out": result["timed_out"],
    }
    if result["returncode"] != 0 or result["timed_out"]:
        output["status"] = "UNAVAILABLE"
        output["stderr_sha256"] = sha256_bytes(result["stderr"].encode())
        return output
    try:
        parsed = json.loads(result["stdout"])
        rows = parsed.get("SPHardwareDataType", [])
        row = rows[0] if rows else {}
        allowed = (
            "chip_type",
            "machine_model",
            "machine_name",
            "model_number",
            "number_processors",
            "physical_memory",
        )
        output["status"] = "COMPLETE"
        output["fields"] = {key: row[key] for key in allowed if key in row}
    except (IndexError, KeyError, TypeError, json.JSONDecodeError) as error:
        output["status"] = "UNAVAILABLE"
        output["parse_error"] = str(error)
        output["stdout_sha256"] = sha256_bytes(result["stdout"].encode())
    return output


def host_metadata() -> dict[str, Any]:
    value: dict[str, Any] = {
        "captured_at": utc_now(),
        "platform": platform.platform(),
        "system": platform.system(),
        "release": platform.release(),
        "machine": platform.machine(),
        "python": sys.version,
        "cpu_count": os.cpu_count(),
        "allocator": "Rust system allocator; allocator calls were not independently traced",
        "affinity_policy": "unpinned and uncontrolled",
        "frequency_policy": "uncontrolled and unobserved",
    }
    value["uname"] = platform.uname()._asdict()
    if platform.system() == "Darwin":
        value["sw_vers"] = run_capture(["/usr/bin/sw_vers"])
        value["hardware"] = sanitized_darwin_hardware()
    return value


def toolchain_metadata(env: dict[str, str]) -> dict[str, Any]:
    metadata = {
        "rustc": run_capture([executable("rustc"), "-Vv"], env=env),
        "cargo": run_capture([executable("cargo"), "-V"], env=env),
        "python": sys.version,
        "xctrace": (
            run_capture(["/usr/bin/xcrun", "xctrace", "version"])
            if Path("/usr/bin/xcrun").exists()
            else {"status": "UNAVAILABLE"}
        ),
    }
    perf = shutil.which("perf")
    metadata["perf"] = (
        run_capture([perf, "--version"], env=env)
        if perf is not None
        else {"status": "UNAVAILABLE"}
    )
    return metadata


def build_benchmark(
    run_dir: Path,
    rustflags: str,
    relay: ExternalSignalRelay | None,
) -> tuple[Path, dict[str, Any]]:
    """Resolve exactly one native benchmark image from Cargo JSON."""

    env = build_environment(rustflags)
    rustc_verbose = run_capture([executable("rustc"), "-Vv"], env=env)
    host_lines = [
        line
        for line in rustc_verbose["stdout"].splitlines()
        if line.startswith("host: ")
    ]
    if rustc_verbose["returncode"] != 0 or len(host_lines) != 1:
        raise ExperimentIncomplete("rustc -Vv did not report exactly one host triple")
    host_triple = host_lines[0].split(":", 1)[1].strip()
    runner_configuration = inspect_runner_configuration(host_triple, env)
    if runner_configuration["ambiguous_or_non_native_findings"]:
        raise ExperimentIncomplete(
            "direct native benchmark execution is ambiguous: "
            f"{runner_configuration['ambiguous_or_non_native_findings']}"
        )

    target_dir = run_dir / "build" / "target"
    target_dir.mkdir(parents=True, exist_ok=False)
    manifest_path = (TOPIC_DIR / "Cargo.toml").resolve()
    argv = [
        executable("cargo"),
        "bench",
        "--package",
        "topic-004-interval-sets-and-compressed-ranges",
        "--bench",
        "interval_sets",
        "--no-run",
        "--locked",
        "--message-format=json-render-diagnostics",
        "--target-dir",
        str(target_dir),
    ]
    returncode, stdout, stderr, timed_out, spawn_error = run_capture_group(
        argv,
        cwd=REPO_ROOT,
        env=env,
        timeout=900.0,
        relay=relay,
    )
    write_bytes_once(run_dir / "build" / "stdout.jsonl", stdout)
    write_bytes_once(run_dir / "build" / "stderr.txt", stderr)

    parse_errors: list[str] = []
    matches: list[dict[str, Any]] = []
    for index, line in enumerate(stdout.splitlines(), start=1):
        if not line.strip():
            continue
        try:
            message = json.loads(line)
        except json.JSONDecodeError as error:
            parse_errors.append(f"line {index}: {error}")
            continue
        target = message.get("target", {})
        profile = message.get("profile", {})
        executable_value = message.get("executable")
        if (
            message.get("reason") == "compiler-artifact"
            and message.get("manifest_path") == str(manifest_path)
            and isinstance(target, dict)
            and target.get("name") == "interval_sets"
            and "bench" in target.get("kind", [])
            and isinstance(profile, dict)
            and profile.get("opt_level") == "3"
            and profile.get("debug_assertions") is False
            and profile.get("overflow_checks") is False
            and isinstance(executable_value, str)
        ):
            matches.append(message)

    metadata: dict[str, Any] = {
        "argv": argv,
        "cwd": str(REPO_ROOT),
        "returncode": returncode,
        "timed_out": timed_out,
        "spawn_error": spawn_error,
        "stdout_sha256": sha256_bytes(stdout),
        "stderr_sha256": sha256_bytes(stderr),
        "parse_errors": parse_errors,
        "recorded_environment": safe_recorded_environment(env),
        "manifest_path": str(manifest_path),
        "target_dir": str(target_dir),
        "target_dir_was_empty": True,
        "expected_profile_fields": {
            "opt_level": "3",
            "debug_assertions": False,
            "overflow_checks": False,
        },
        "matched_artifact_count": len(matches),
        "rustc_verbose": rustc_verbose,
        "cargo_version": run_capture([executable("cargo"), "-V"], env=env),
        "runner_configuration": runner_configuration,
        "cargo_config_files": runner_configuration["config_paths_checked"],
    }
    if (
        returncode != 0
        or timed_out
        or spawn_error is not None
        or parse_errors
        or len(matches) != 1
    ):
        write_json_once(run_dir / "build" / "metadata.json", metadata)
        raise ExperimentIncomplete(
            "Cargo did not produce exactly one matching interval_sets benchmark artifact"
        )

    message = matches[0]
    executable_path = Path(message["executable"])
    if not executable_path.is_absolute() or not executable_path.is_file():
        metadata["resolved_executable_error"] = repr(message["executable"])
        write_json_once(run_dir / "build" / "metadata.json", metadata)
        raise ExperimentIncomplete(
            "Cargo artifact executable is not an absolute existing file"
        )
    executable_path = executable_path.resolve()
    image_hash = sha256_file(executable_path)
    retained = run_dir / "artifacts" / f"interval_sets-{image_hash}"
    retained.parent.mkdir(parents=True, exist_ok=True)
    write_bytes_once(retained, executable_path.read_bytes())
    retained.chmod(0o755)
    if sha256_file(retained) != image_hash:
        raise ExperimentIncomplete(
            "retained linked image hash differs from Cargo artifact"
        )
    metadata.update(
        {
            "cargo_artifact_message": message,
            "cargo_executable": str(executable_path),
            "retained_executable": str(retained),
            "binary_bytes": retained.stat().st_size,
            "binary_sha256": image_hash,
            "native_direct_replay_argv_suffix": ["--bench"],
            "replay_boundary": (
                "the runner inspection found no effective target runner or non-native target; "
                "the retained native image is invoked directly with Cargo's --bench argument"
            ),
        }
    )
    write_json_once(run_dir / "build" / "metadata.json", metadata)
    return retained, metadata


def median_int(values: Sequence[int]) -> int:
    if not values:
        raise ValueError("cannot take median of an empty sequence")
    ordered = sorted(values)
    middle = len(ordered) // 2
    if len(ordered) % 2:
        return ordered[middle]
    return (ordered[middle - 1] + ordered[middle]) // 2


def attempt_id(position: Position) -> str:
    return (
        f"{position.phase}-{position.contrast_id}-b{position.block_index:02d}-"
        f"p{position.position_index}-{position.label}-{position.candidate}"
    )


def validated_samples(record: Any, samples: int) -> tuple[list[int], list[str]]:
    if not isinstance(record, dict):
        return [], []
    values = record.get("sample_ns")
    if not isinstance(values, list) or len(values) != samples:
        return [], [f"sample_ns must have exactly {samples} entries"]
    if any(
        not isinstance(value, int) or isinstance(value, bool) or value < 0
        for value in values
    ):
        return [], ["sample_ns entries must be nonnegative integers"]
    if median_int(values) <= 0:
        return values, ["median sample_ns must be positive for log analysis"]
    return values, []


def valid_hex64(value: Any) -> bool:
    return (
        isinstance(value, str)
        and len(value) == 16
        and all(character in "0123456789abcdef" for character in value)
    )


def validate_record(position: Position, record: Any) -> list[str]:
    errors: list[str] = []
    if not isinstance(record, dict):
        return ["record is not a JSON object"]
    expected_fields = {
        "schema_version",
        "candidate",
        "workload",
        "phase",
        "seed",
        "warmup",
        "samples",
        "inner",
        "generator_version",
        "fixture_hash",
        "sample_ns",
        "output_hash",
        "work",
        "canaries",
    }
    if set(record) != expected_fields:
        errors.append("top-level fields differ from the frozen schema")
    expected_scalars = {
        "schema_version": SCHEMA_VERSION,
        "candidate": position.candidate,
        "workload": position.cell,
        "phase": position.operation_phase,
        "seed": position.seed,
        "warmup": position.warmups,
        "samples": position.samples,
        "inner": position.inner,
        "generator_version": GENERATOR_VERSION,
    }
    for name, expected in expected_scalars.items():
        if record.get(name) != expected:
            errors.append(f"{name} was {record.get(name)!r}, expected {expected!r}")
    for name in ("fixture_hash", "output_hash"):
        if not valid_hex64(record.get(name)):
            errors.append(f"{name} must be 16 lowercase hexadecimal characters")
    _, sample_errors = validated_samples(record, position.samples)
    errors.extend(sample_errors)

    cell = CELL_BY_NAME[position.cell]
    work = record.get("work")
    if not isinstance(work, dict):
        errors.append("work must be an object")
    else:
        if set(work) != set(WORK_FIELDS):
            errors.append("work fields differ from the frozen schema")
        for name in WORK_FIELDS:
            value = work.get(name)
            if not isinstance(value, int) or isinstance(value, bool) or value < 0:
                errors.append(f"work.{name} must be a nonnegative integer")
        expected_exact = {
            "input_intervals": cell.input_intervals * position.inner,
            "unique_input_intervals": cell.unique_intervals * position.inner,
            "duplicate_intervals": cell.duplicate_intervals * position.inner,
            "output_runs": cell.output_runs * position.inner,
            "membership_queries": (
                cell.membership_queries * position.inner
                if position.operation_phase == "build_membership"
                else 0
            ),
        }
        for name, expected in expected_exact.items():
            if work.get(name) != expected:
                errors.append(
                    f"work.{name} was {work.get(name)!r}, expected {expected}"
                )
        membership_comparisons = work.get("canonical_binary_search_comparisons")
        if position.operation_phase == "build" and membership_comparisons != 0:
            errors.append("build phase must report zero canonical search comparisons")
        if (
            position.operation_phase == "build_membership"
            and isinstance(membership_comparisons, int)
            and membership_comparisons <= 0
        ):
            errors.append(
                "build_membership must report positive canonical search comparisons"
            )

    canaries = record.get("canaries")
    expected_canaries = {
        "oracle_match": True,
        "domain_end": 1 << 32,
        "packed_max_decode": 1 << 32,
        "membership_match": True,
    }
    if canaries != expected_canaries:
        errors.append(f"canaries were {canaries!r}, expected {expected_canaries!r}")
    return errors


class Executor:
    def __init__(
        self,
        run_dir: Path,
        binary: Path,
        timeout: float,
        relay: ExternalSignalRelay | None,
    ) -> None:
        self.run_dir = run_dir
        self.binary = binary
        self.binary_hash = sha256_file(binary)
        self.timeout = timeout
        self.relay = relay
        self.env = benchmark_environment()
        self.attempts: list[Attempt] = []
        self.records: dict[str, dict[str, Any]] = {}
        self.blocks: list[dict[str, Any]] = []
        self.fixture_hashes: dict[tuple[str, int], str] = {}
        self.output_hashes: dict[tuple[str, str, int], str] = {}

    def harness_argv(self, position: Position) -> list[str]:
        return [
            str(self.binary),
            "--bench",
            "--candidate",
            position.candidate,
            "--workload",
            position.cell,
            "--phase",
            position.operation_phase,
            "--seed",
            str(position.seed),
            "--warmup",
            str(position.warmups),
            "--samples",
            str(position.samples),
            "--inner",
            str(position.inner),
        ]

    def run_position(self, position: Position) -> Attempt:
        if self.relay is not None:
            self.relay.raise_if_pending()
        identifier = attempt_id(position)
        directory = self.run_dir / "attempts" / identifier
        directory.mkdir(parents=True, exist_ok=False)
        command = self.harness_argv(position)
        before = binary_digest(self.binary)
        started = utc_now()
        started_ns = time.time_ns()
        write_json_once(
            directory / "command.json",
            {
                "argv": command,
                "cwd": str(REPO_ROOT),
                "effective_environment": safe_recorded_environment(self.env),
                "fresh_process": True,
                "automatic_retry_count": 0,
                "expected_binary_sha256": self.binary_hash,
                "prelaunch_binary_sha256": before,
                "parser_schema_version": SCHEMA_VERSION,
                "parser_sha256": sha256_file(SCRIPT),
                "reset": {
                    "action": "fresh process reconstructs the deterministic immutable fixture",
                    "washout_seconds": 0,
                    "shared_host_state_reset": False,
                },
            },
        )
        returncode, stdout, stderr, timed_out, launch_error = run_capture_group(
            command,
            cwd=REPO_ROOT,
            env=self.env,
            timeout=self.timeout,
            relay=self.relay,
        )
        ended_ns = time.time_ns()
        ended = utc_now()
        after = binary_digest(self.binary)
        write_bytes_once(directory / "stdout.jsonl", stdout)
        write_bytes_once(directory / "stderr.txt", stderr)

        record: dict[str, Any] | None = None
        errors: list[str] = []
        if launch_error is not None:
            errors.append(launch_error)
        if timed_out:
            errors.append("process timed out")
        if returncode != 0:
            errors.append(f"process return code was {returncode!r}")
        if before != self.binary_hash or after != self.binary_hash:
            errors.append("linked image hash changed")
        lines = [
            line
            for line in stdout.decode("utf-8", "replace").splitlines()
            if line.strip()
        ]
        if len(lines) != 1:
            errors.append(
                f"stdout contained {len(lines)} nonempty records, expected one"
            )
        else:
            try:
                parsed = json.loads(lines[0])
                if isinstance(parsed, dict):
                    record = parsed
                else:
                    errors.append("JSON record was not an object")
            except json.JSONDecodeError as error:
                errors.append(f"JSON parse failed: {error}")
        if record is not None:
            errors.extend(validate_record(position, record))

        samples, _ = validated_samples(record, position.samples)
        fixture_hash = record.get("fixture_hash", "") if record is not None else ""
        output_hash = record.get("output_hash", "") if record is not None else ""
        if valid_hex64(fixture_hash):
            fixture_key = (position.cell, position.seed)
            previous = self.fixture_hashes.setdefault(fixture_key, fixture_hash)
            if previous != fixture_hash:
                errors.append("fixture hash changed for an identical cell and seed")
        if valid_hex64(output_hash):
            output_key = (position.cell, position.operation_phase, position.seed)
            previous = self.output_hashes.setdefault(output_key, output_hash)
            if previous != output_hash:
                errors.append(
                    "output hash differs across candidates for one fixture and phase"
                )
        work = record.get("work") if record is not None else None
        canaries = record.get("canaries") if record is not None else None
        work_hash = (
            sha256_bytes(
                json.dumps(work, sort_keys=True, separators=(",", ":")).encode()
            )
            if isinstance(work, dict)
            else ""
        )
        canary_hash = (
            sha256_bytes(
                json.dumps(canaries, sort_keys=True, separators=(",", ":")).encode()
            )
            if isinstance(canaries, dict)
            else ""
        )
        external_interruption = bool(
            returncode is not None
            and returncode < 0
            and -returncode in {int(member) for member in EXTERNAL_SIGNALS}
        )
        attempt = Attempt(
            attempt_id=identifier,
            phase=position.phase,
            contrast_id=position.contrast_id,
            block_index=position.block_index,
            template=position.template,
            position_index=position.position_index,
            label=position.label,
            candidate=position.candidate,
            cell=position.cell,
            operation_phase=position.operation_phase,
            seed=position.seed,
            status="VALID" if not errors else "INVALID",
            invalid_reason="; ".join(errors),
            returncode=returncode,
            timed_out=timed_out,
            position_ns=median_int(samples) if samples else None,
            zero_samples=sum(value == 0 for value in samples),
            fixture_hash=fixture_hash,
            output_hash=output_hash,
            work_sha256=work_hash,
            canary_sha256=canary_hash,
            external_interruption=external_interruption,
            stdout_sha256=sha256_bytes(stdout),
            stderr_sha256=sha256_bytes(stderr),
            executable_sha256_before=before,
            executable_sha256_after=after,
            attempt_directory=str(directory),
        )
        write_json_once(
            directory / "parser.json",
            {
                "status": "VALID" if record is not None and not errors else "INVALID",
                "errors": errors,
                "observed_fields": sorted(record) if record is not None else None,
                "parser_schema_version": SCHEMA_VERSION,
                "parser_sha256": sha256_file(SCRIPT),
            },
        )
        write_json_once(
            directory / "attempt.json",
            {
                "position": asdict(position),
                "attempt": asdict(attempt),
                "started_at": started,
                "ended_at": ended,
                "started_ns_since_epoch": started_ns,
                "ended_ns_since_epoch": ended_ns,
                "duration_ns": ended_ns - started_ns,
                "parsed_record": record,
            },
        )
        self.attempts.append(attempt)
        if record is not None:
            self.records[identifier] = record
        return attempt

    def run_block(
        self,
        phase: str,
        contrast: Contrast,
        block_index: int,
        template: str,
    ) -> None:
        positions = positions_for_block(
            phase,
            contrast,
            block_index,
            template,
            DEFAULT_WARMUPS,
            DEFAULT_SAMPLES,
        )
        attempts: list[Attempt] = []
        for position in positions:
            attempt = self.run_position(position)
            attempts.append(attempt)
            if attempt.status != "VALID":
                break
        errors = [
            attempt.invalid_reason for attempt in attempts if attempt.status != "VALID"
        ]
        if len(attempts) != 4:
            errors.append(f"block retained {len(attempts)} of four required positions")
        fixture_hashes = {
            attempt.fixture_hash for attempt in attempts if attempt.fixture_hash
        }
        output_hashes = {
            attempt.output_hash for attempt in attempts if attempt.output_hash
        }
        if len(fixture_hashes) != 1:
            errors.append("fixture hashes differ or are missing within the block")
        if len(output_hashes) != 1:
            errors.append("output hashes differ or are missing within the block")
        if phase == "aa":
            if {attempt.candidate for attempt in attempts} != {"flat"}:
                errors.append("A/A did not execute the identical flat artifact")
            work_hashes = {attempt.work_sha256 for attempt in attempts}
            if len(work_hashes) != 1:
                errors.append("A/A work counters differ across labels")
        block = {
            "phase": phase,
            "contrast_id": contrast.contrast_id,
            "block_index": block_index,
            "template": template,
            "status": "VALID" if not errors else "INVALID",
            "errors": errors,
            "attempt_ids": [attempt.attempt_id for attempt in attempts],
            "fixture_hash": (
                next(iter(fixture_hashes)) if len(fixture_hashes) == 1 else None
            ),
            "output_hash": (
                next(iter(output_hashes)) if len(output_hashes) == 1 else None
            ),
        }
        self.blocks.append(block)
        write_json_once(
            self.run_dir
            / "blocks"
            / f"{phase}-{contrast.contrast_id}-b{block_index:02d}.json",
            block,
        )
        if errors:
            raise ExperimentIncomplete(
                f"invalid block {phase}/{contrast.contrast_id}/{block_index}: {errors}"
            )

    def write_attempts_table(self) -> None:
        attempts_path = self.run_dir / "attempts.csv"
        with attempts_path.open("x", newline="", encoding="utf-8") as stream:
            writer = csv.DictWriter(
                stream, fieldnames=list(Attempt.__dataclass_fields__)
            )
            writer.writeheader()
            writer.writerows(asdict(attempt) for attempt in self.attempts)

    def write_blocks_table(self) -> None:
        blocks_path = self.run_dir / "blocks.csv"
        fields = (
            "phase",
            "contrast_id",
            "block_index",
            "template",
            "status",
            "errors",
            "attempt_ids",
            "fixture_hash",
            "output_hash",
        )
        with blocks_path.open("x", newline="", encoding="utf-8") as stream:
            writer = csv.DictWriter(stream, fieldnames=fields)
            writer.writeheader()
            for block in self.blocks:
                row = dict(block)
                row["errors"] = json.dumps(row["errors"], separators=(",", ":"))
                row["attempt_ids"] = json.dumps(
                    row["attempt_ids"], separators=(",", ":")
                )
                writer.writerow(row)

    def write_tables(self) -> None:
        existing: list[str] = []
        for write_table in (self.write_attempts_table, self.write_blocks_table):
            try:
                write_table()
            except FileExistsError as error:
                existing.append(str(error.filename))
        if existing:
            raise FileExistsError(
                errno.EEXIST,
                "table already written",
                ", ".join(sorted(existing)),
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
        coefficient = (
            -(a + iteration) * (qab + iteration) * x / ((a + even) * (qap + even))
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
        raise ValueError(
            "at least two independent complete-block contrasts are required"
        )
    count = len(values)
    degrees = count - 1
    mean = statistics.fmean(values)
    deviation = statistics.stdev(values)
    error = deviation / math.sqrt(count)
    critical = student_t_quantile(1.0 - alpha / 2.0, degrees)
    half_width = critical * error
    return {
        "independent_unit_count": count,
        "degrees_of_freedom": degrees,
        "alpha": alpha,
        "mean_log_ratio": mean,
        "standard_deviation_log_ratio": deviation,
        "standard_error_log_ratio": error,
        "student_t_critical": critical,
        "lower_log_ratio": mean - half_width,
        "upper_log_ratio": mean + half_width,
        "point_ratio": math.exp(mean),
        "lower_ratio": math.exp(mean - half_width),
        "upper_ratio": math.exp(mean + half_width),
    }


def main_sensitivity(values: Sequence[float]) -> dict[str, Any]:
    if len(values) != MAIN_BLOCKS:
        raise ValueError(
            "distribution-free sensitivity is frozen for twelve main blocks"
        )
    ordered = sorted(values)
    lower_index = 2
    upper_index = 9
    coverage = 1.0 - 2.0 * sum(
        math.comb(len(values), index) for index in range(lower_index + 1)
    ) / (2 ** len(values))
    midpoint = len(values) // 2
    x_mean = (len(values) + 1) / 2.0
    denominator = sum((index - x_mean) ** 2 for index in range(1, len(values) + 1))
    mean = statistics.fmean(values)
    slope = (
        sum(
            (index - x_mean) * (value - mean)
            for index, value in enumerate(values, start=1)
        )
        / denominator
    )
    return {
        "distribution_free_median_interval": {
            "coverage": coverage,
            "lower_order_statistic_zero_based_index": lower_index,
            "upper_order_statistic_zero_based_index": upper_index,
            "lower_ratio": math.exp(ordered[lower_index]),
            "upper_ratio": math.exp(ordered[upper_index]),
            "assumptions": "twelve iid continuous complete-block log contrasts",
        },
        "time_order": {
            "ols_log_contrast_slope_per_block": slope,
            "first_half_ratio": math.exp(statistics.fmean(values[:midpoint])),
            "second_half_ratio": math.exp(statistics.fmean(values[midpoint:])),
        },
        "decision_role": "diagnostic only",
    }


def attempts_for_block(
    executor: Executor, phase: str, contrast_id: str, block: int
) -> list[Attempt]:
    return sorted(
        (
            attempt
            for attempt in executor.attempts
            if attempt.phase == phase
            and attempt.contrast_id == contrast_id
            and attempt.block_index == block
        ),
        key=lambda attempt: attempt.position_index,
    )


def attempt_value(attempt: Attempt) -> int:
    if attempt.position_ns is None or attempt.position_ns <= 0:
        raise ExperimentIncomplete(
            f"missing positive timing value in {attempt.attempt_id}"
        )
    return attempt.position_ns


def block_contrasts(executor: Executor, phase: str, contrast: Contrast) -> list[float]:
    count = (
        PILOT_BLOCKS
        if phase == "pilot"
        else (AA_BLOCKS if phase == "aa" else MAIN_BLOCKS)
    )
    values = []
    for block_index in range(1, count + 1):
        attempts = attempts_for_block(
            executor, phase, contrast.contrast_id, block_index
        )
        if len(attempts) != 4 or any(attempt.status != "VALID" for attempt in attempts):
            raise ExperimentIncomplete(
                f"{phase}/{contrast.contrast_id}/{block_index} is not a complete valid block"
            )
        a_raw = [attempt_value(attempt) for attempt in attempts if attempt.label == "A"]
        b_raw = [attempt_value(attempt) for attempt in attempts if attempt.label == "B"]
        if len(a_raw) != 2 or len(b_raw) != 2:
            raise ExperimentIncomplete(
                "complete block does not contain two A and two B positions"
            )
        a_values = [math.log(value) for value in a_raw]
        b_values = [math.log(value) for value in b_raw]
        values.append(statistics.fmean(b_values) - statistics.fmean(a_values))
    return values


def primary_block_contrasts(
    executor: Executor, phase: str, contrast: Contrast
) -> list[float]:
    return block_contrasts(executor, phase, contrast)


def reliability(executor: Executor, phase: str) -> dict[str, Any]:
    attempts = [attempt for attempt in executor.attempts if attempt.phase == phase]
    return {
        "attempt_count": len(attempts),
        "valid_attempts": sum(attempt.status == "VALID" for attempt in attempts),
        "invalid_attempts": sum(attempt.status != "VALID" for attempt in attempts),
        "timeouts": sum(attempt.timed_out for attempt in attempts),
        "zero_samples": sum(attempt.zero_samples for attempt in attempts),
        "external_interruptions": sum(
            attempt.external_interruption for attempt in attempts
        ),
        "returncodes": sorted(
            {attempt.returncode for attempt in attempts},
            key=lambda value: -999 if value is None else value,
        ),
    }


def analyze_pilot(executor: Executor) -> dict[str, Any]:
    rows = []
    for contrast in CONTRASTS:
        values = primary_block_contrasts(executor, "pilot", contrast)
        pilot_sd = statistics.stdev(values)
        row = asdict(contrast)
        row.update(
            {
                "independent_unit_count": len(values),
                "block_log_contrasts": values,
                "pilot_standard_deviation_log_ratio": pilot_sd,
                "pilot_point_ratio": math.exp(statistics.fmean(values)),
                "prospective_12_block_interval_width_sensitivity": (
                    prospective_interval_width_sensitivity(pilot_sd)
                ),
                "decision_role": "variance and feasibility only; excluded from main estimate",
            }
        )
        rows.append(row)
    return {
        "status": "COMPLETE",
        "phase": "pilot",
        "results": rows,
        "reliability": reliability(executor, "pilot"),
    }


def prospective_interval_width_sensitivity(pilot_sd: float) -> list[dict[str, float]]:
    """Freeze prospective simultaneous interval widths without changing sample size."""
    per_contrast_alpha = FAMILY_ALPHA / FAMILY_SIZE
    critical = student_t_quantile(1.0 - per_contrast_alpha / 2.0, MAIN_BLOCKS - 1)
    rows = []
    for multiplier in PILOT_SENSITIVITY_MULTIPLIERS:
        assumed_sd = multiplier * pilot_sd
        half_width = critical * assumed_sd / math.sqrt(MAIN_BLOCKS)
        rows.append(
            {
                "pilot_sd_multiplier": multiplier,
                "assumed_standard_deviation_log_ratio": assumed_sd,
                "half_width_log_ratio": half_width,
                "multiplicative_half_width": math.exp(half_width),
                "null_centered_lower_ratio": math.exp(-half_width),
                "null_centered_upper_ratio": math.exp(half_width),
                "full_ratio_span": math.exp(2.0 * half_width),
            }
        )
    return rows


def classify_interval(interval: dict[str, Any]) -> str:
    if interval["upper_ratio"] < 1.0 / PRACTICAL_RATIO:
        return "B_TIME_AT_MOST_1_OVER_1_05_OF_A"
    if interval["lower_ratio"] > PRACTICAL_RATIO:
        return "B_TIME_AT_LEAST_1_05_TIMES_A"
    return "NO_JUSTIFIED_SYMMETRIC_1_05_FACTOR_SEPARATION"


def analyze_main(executor: Executor) -> dict[str, Any]:
    per_alpha = FAMILY_ALPHA / FAMILY_SIZE
    rows = []
    for contrast in CONTRASTS:
        total = primary_block_contrasts(executor, "main", contrast)
        interval = analyze_values(total, per_alpha)
        row = asdict(contrast)
        row.update(
            {
                "estimand": "candidate/flat geometric mean of complete-block time ratios",
                "block_log_contrasts": total,
                "simultaneous_interval": interval,
                "classification": classify_interval(interval),
                "sensitivity": main_sensitivity(total),
            }
        )
        rows.append(row)
    return {
        "status": "COMPLETE",
        "phase": "main",
        "family_size": FAMILY_SIZE,
        "familywise_alpha": FAMILY_ALPHA,
        "per_contrast_alpha": per_alpha,
        "results": rows,
        "reliability": reliability(executor, "main"),
    }


def run_direct_phase(executor: Executor, phase: str) -> None:
    rows = [row for row in planned_schedule([phase])]
    for row in rows:
        attempt = executor.run_position(Position(**row))
        if attempt.status != "VALID":
            raise ExperimentIncomplete(f"invalid {phase} attempt {attempt.attempt_id}")


def analyze_direct(executor: Executor, phase: str) -> dict[str, Any]:
    attempts = [attempt for attempt in executor.attempts if attempt.phase == phase]
    if not attempts or any(attempt.status != "VALID" for attempt in attempts):
        raise ExperimentIncomplete(f"{phase} does not contain only valid attempts")
    result: dict[str, Any] = {
        "status": "COMPLETE",
        "phase": phase,
        "reliability": reliability(executor, phase),
    }
    return result


def analyze_aa(executor: Executor) -> dict[str, Any]:
    values = primary_block_contrasts(executor, "aa", AA_CONTRAST)
    attempts = [attempt for attempt in executor.attempts if attempt.phase == "aa"]
    hashes = {attempt.executable_sha256_before for attempt in attempts} | {
        attempt.executable_sha256_after for attempt in attempts
    }
    mechanical_errors = []
    if hashes != {executor.binary_hash}:
        mechanical_errors.append("linked-image hashes differed")
    if {attempt.candidate for attempt in attempts} != {"flat"}:
        mechanical_errors.append("labels did not execute the same flat candidate")
    templates = [
        block["template"] for block in executor.blocks if block["phase"] == "aa"
    ]
    if templates.count("ABBA") != 6 or templates.count("BAAB") != 6:
        mechanical_errors.append(
            "A/A templates were not balanced six ABBA and six BAAB"
        )
    for block_index in range(1, AA_BLOCKS + 1):
        block_attempts = attempts_for_block(
            executor, "aa", AA_CONTRAST.contrast_id, block_index
        )
        if len({attempt.fixture_hash for attempt in block_attempts}) != 1:
            mechanical_errors.append(f"block {block_index} fixture hashes differed")
        if len({attempt.output_hash for attempt in block_attempts}) != 1:
            mechanical_errors.append(f"block {block_index} output hashes differed")
        work_records = [
            executor.records[attempt.attempt_id].get("work")
            for attempt in block_attempts
        ]
        if any(work != work_records[0] for work in work_records[1:]):
            mechanical_errors.append(f"block {block_index} work records differed")
    label_by_position = []
    for label in ("A", "B"):
        for position_index in range(1, 5):
            selected = [
                attempt.position_ns
                for attempt in attempts
                if attempt.label == label
                and attempt.position_index == position_index
                and attempt.position_ns is not None
            ]
            label_by_position.append(
                {
                    "label": label,
                    "position_index": position_index,
                    "count": len(selected),
                    "geometric_mean_ns": (
                        math.exp(
                            statistics.fmean(math.log(value) for value in selected)
                        )
                        if selected
                        else None
                    ),
                }
            )
    result = {
        "status": "COMPLETE" if not mechanical_errors else "INCOMPLETE",
        "phase": "aa",
        "mechanical_integrity": {
            "status": "PASS" if not mechanical_errors else "FAIL",
            "errors": mechanical_errors,
            "identical_candidate": "flat",
            "linked_image_sha256": executor.binary_hash,
            "parser_schema_version": SCHEMA_VERSION,
            "template_balance": {
                "ABBA": templates.count("ABBA"),
                "BAAB": templates.count("BAAB"),
            },
        },
        "null_diagnostic": {
            "primary_path_bonferroni_interval": analyze_values(
                values, FAMILY_ALPHA / FAMILY_SIZE
            ),
            "unadjusted_95_percent_interval": analyze_values(values, 0.05),
            "block_log_contrasts": values,
            "label_by_position": label_by_position,
            "claim_boundary": (
                "one identical-artifact campaign can expose asymmetry; it does not estimate "
                "the pipeline false-positive rate or define a noise floor"
            ),
        },
        "reliability": reliability(executor, "aa"),
    }
    if mechanical_errors:
        raise ExperimentIncomplete(
            f"A/A mechanical integrity failed: {mechanical_errors}"
        )
    return result


def run_block_phase(executor: Executor, phase: str) -> None:
    if phase == "pilot":
        count = PILOT_BLOCKS
        contrasts = CONTRASTS
    elif phase == "main":
        count = MAIN_BLOCKS
        contrasts = CONTRASTS
    elif phase == "aa":
        count = AA_BLOCKS
        contrasts = (AA_CONTRAST,)
    else:
        raise ValueError(f"not a block phase: {phase}")
    for contrast in contrasts:
        label = "aa" if phase == "aa" else f"{phase}:{contrast.contrast_id}"
        for block_index, template in enumerate(
            balanced_templates(count, label), start=1
        ):
            executor.run_block(phase, contrast, block_index, template)


def collect_disassembly(executor: Executor) -> dict[str, Any]:
    directory = executor.run_dir / "profile" / "linked-image"
    directory.mkdir(parents=True, exist_ok=False)
    before = sha256_file(executor.binary)
    commands: list[tuple[str, list[str]]]
    if platform.system() == "Darwin":
        commands = [
            ("symbols", ["/usr/bin/nm", "-anC", str(executor.binary)]),
            ("disassembly", ["/usr/bin/otool", "-tvV", str(executor.binary)]),
        ]
    else:
        nm = shutil.which("nm")
        objdump = shutil.which("objdump")
        if nm is None or objdump is None:
            raise ExperimentIncomplete(
                "nm and objdump are required for linked-image evidence"
            )
        commands = [
            ("symbols", [nm, "-anC", str(executor.binary)]),
            ("disassembly", [objdump, "-Cd", str(executor.binary)]),
        ]
    records = {}
    capture_text: dict[str, str] = {}
    for name, argv in commands:
        returncode, stdout, stderr, timed_out, spawn_error = run_capture_group(
            argv,
            cwd=REPO_ROOT,
            env=executor.env,
            timeout=executor.timeout,
            relay=executor.relay,
        )
        write_bytes_once(directory / f"{name}.txt", stdout)
        write_bytes_once(directory / f"{name}.stderr.txt", stderr)
        records[name] = {
            "argv": argv,
            "returncode": returncode,
            "timed_out": timed_out,
            "spawn_error": spawn_error,
            "stdout_bytes": len(stdout),
            "stdout_sha256": sha256_bytes(stdout),
            "stderr_sha256": sha256_bytes(stderr),
        }
        capture_text[name] = stdout.decode("utf-8", "replace")
        if returncode != 0 or timed_out or spawn_error is not None or not stdout:
            raise ExperimentIncomplete(f"linked-image {name} collection failed")
    missing_symbols = {
        name: [
            symbol
            for symbol in EXPECTED_LINKED_SYMBOL_SUBSTRINGS
            if symbol not in capture_text[name]
        ]
        for name in ("symbols", "disassembly")
    }
    if any(missing_symbols.values()):
        raise ExperimentIncomplete(
            f"linked-image evidence omitted required paths: {missing_symbols}"
        )
    after = sha256_file(executor.binary)
    if before != executor.binary_hash or after != executor.binary_hash:
        raise ExperimentIncomplete("linked image changed during disassembly")
    return {
        "status": "COMPLETE",
        "binary_sha256": executor.binary_hash,
        "commands": records,
        "required_symbol_substrings": EXPECTED_LINKED_SYMBOL_SUBSTRINGS,
        "missing_symbol_substrings": missing_symbols,
        "claim_boundary": "observed final-image code shape; not dynamic mechanism attribution",
    }


def profile_artifact_bytes(path: Path | None) -> int:
    if path is None or not path.exists():
        return 0
    if path.is_file():
        return path.stat().st_size
    return sum(item.stat().st_size for item in path.rglob("*") if item.is_file())


def dynamic_target_record(
    stdout: bytes,
    candidate: str,
    cell: str,
    operation_phase: str,
    seed: int,
) -> tuple[dict[str, Any] | None, list[str]]:
    records = []
    for line in stdout.decode("utf-8", "replace").splitlines():
        try:
            parsed = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(parsed, dict) and parsed.get("schema_version") == SCHEMA_VERSION:
            records.append(parsed)
    if len(records) != 1:
        return None, [
            f"dynamic target emitted {len(records)} harness records, expected one"
        ]
    position = Position(
        "profile",
        f"dynamic__{candidate}__{cell}",
        1,
        "D",
        1,
        "D",
        candidate,
        cell,
        operation_phase,
        seed,
        1,
        1,
        PROFILE_REPETITIONS,
    )
    record = records[0]
    return record, validate_record(position, record)


def dynamic_profile_attempt(
    executor: Executor,
    candidate: str,
    cell: str,
    operation_phase: str,
    ordinal: int,
) -> dict[str, Any]:
    directory = (
        executor.run_dir
        / "profile"
        / "dynamic"
        / f"{ordinal:02d}-{candidate}-{cell}-{operation_phase}"
    )
    directory.mkdir(parents=True, exist_ok=False)
    seed = block_seed("profile", f"{candidate}:{cell}:{operation_phase}", 1)
    target_args = [
        str(executor.binary),
        "--bench",
        "--candidate",
        candidate,
        "--workload",
        cell,
        "--phase",
        operation_phase,
        "--seed",
        str(seed),
        "--warmup",
        "1",
        "--samples",
        "1",
        "--inner",
        str(PROFILE_REPETITIONS),
    ]
    artifact: Path | None = None
    tool: str | None = None
    if platform.system() == "Darwin" and Path("/usr/bin/xcrun").exists():
        artifact = directory / "time-profiler.trace"
        tool = "xctrace-time-profiler"
        argv = [
            "/usr/bin/xcrun",
            "xctrace",
            "record",
            "--template",
            "Time Profiler",
            "--time-limit",
            "30s",
            "--output",
            str(artifact),
            "--no-prompt",
            "--target-stdout",
            "-",
            "--launch",
            "--",
            *target_args,
        ]
    elif shutil.which("perf"):
        artifact = directory / "perf.data"
        tool = "perf-record"
        argv = [
            shutil.which("perf") or "perf",
            "record",
            "-o",
            str(artifact),
            "--",
            *target_args,
        ]
    else:
        argv = []
    started = utc_now()
    before = binary_digest(executor.binary)
    if tool is None:
        timed_out = False
        returncode = None
        stdout = b""
        stderr = b"no supported dynamic sampling tool was available\n"
        launch_error = None
    else:
        returncode, stdout, stderr, timed_out, launch_error = run_capture_group(
            argv,
            cwd=REPO_ROOT,
            env=executor.env,
            timeout=executor.timeout,
            relay=executor.relay,
        )
    after = binary_digest(executor.binary)
    write_bytes_once(directory / "stdout.txt", stdout)
    write_bytes_once(directory / "stderr.txt", stderr)
    artifact_bytes = profile_artifact_bytes(artifact)
    artifact_exists = artifact is not None and artifact.exists()
    image_unchanged = before == executor.binary_hash == after
    target_record, target_record_errors = dynamic_target_record(
        stdout, candidate, cell, operation_phase, seed
    )
    captured = (
        tool is not None
        and launch_error is None
        and returncode == 0
        and not timed_out
        and artifact_bytes > 0
        and image_unchanged
        and not target_record_errors
    )
    record = {
        "status": "CAPTURED" if captured else "ATTEMPTED_UNAVAILABLE",
        "candidate": candidate,
        "cell": cell,
        "operation_phase": operation_phase,
        "seed": seed,
        "profile_repetitions": PROFILE_REPETITIONS,
        "tool": tool,
        "started_at": started,
        "ended_at": utc_now(),
        "argv": argv,
        "target_argv": target_args,
        "effective_environment": safe_recorded_environment(executor.env),
        "returncode": returncode,
        "timed_out": timed_out,
        "launch_error": launch_error,
        "stdout_sha256": sha256_bytes(stdout),
        "stderr_sha256": sha256_bytes(stderr),
        "artifact": str(artifact) if artifact is not None else None,
        "artifact_exists": artifact_exists,
        "artifact_bytes": artifact_bytes,
        "target_record": target_record,
        "target_record_errors": target_record_errors,
        "binary_sha256_before": before,
        "binary_sha256_after": after,
        "expected_binary_sha256": executor.binary_hash,
        "claim_boundary": (
            "a failed or permission-denied profile attempt provides no dynamic mechanism evidence"
        ),
    }
    write_json_once(directory / "attempt.json", record)
    return record


def run_profile(executor: Executor) -> dict[str, Any]:
    canary_attempts = []
    for ordinal, (candidate, cell, operation_phase) in enumerate(
        PROFILE_SPECS, start=1
    ):
        canary = Position(
            "profile",
            f"profile_canary__{candidate}__{cell}__{operation_phase}",
            ordinal,
            "P",
            1,
            "P",
            candidate,
            cell,
            operation_phase,
            block_seed("profile", f"{candidate}:{cell}:{operation_phase}", 1),
            1,
            1,
            DEFAULT_INNER,
        )
        attempt = executor.run_position(canary)
        canary_attempts.append(attempt.attempt_id)
        if attempt.status != "VALID":
            raise ExperimentIncomplete(
                f"profile canary failed: {attempt.invalid_reason}"
            )
    linked_image = collect_disassembly(executor)
    dynamic = [
        dynamic_profile_attempt(executor, candidate, cell, operation_phase, ordinal)
        for ordinal, (candidate, cell, operation_phase) in enumerate(
            PROFILE_SPECS, start=1
        )
    ]
    result = {
        "status": "COMPLETE",
        "canary_attempts": canary_attempts,
        "linked_image": linked_image,
        "dynamic": dynamic,
        "dynamic_captured_count": sum(
            attempt["status"] == "CAPTURED" for attempt in dynamic
        ),
        "dynamic_attempted_unavailable_count": sum(
            attempt["status"] == "ATTEMPTED_UNAVAILABLE" for attempt in dynamic
        ),
        "canary_reliability": reliability(executor, "profile"),
        "evidence_boundary": (
            "elapsed time is measured; logical payload and work are derived; assembly is "
            "observed; cache, branch, allocation, or instruction mechanisms require a "
            "successful applicable dynamic profile or counter"
        ),
    }
    return result


def freeze_phase_evidence(run_dir: Path, phase: str) -> dict[str, Any]:
    """Checksum and verify the raw phase inputs before statistical reduction."""

    selected: list[Path] = []
    attempts_root = run_dir / "attempts"
    if attempts_root.is_dir():
        selected.extend(
            path
            for path in attempts_root.rglob("*")
            if path.is_file()
            and path.relative_to(attempts_root).parts[0].startswith(f"{phase}-")
        )
    blocks_root = run_dir / "blocks"
    if blocks_root.is_dir():
        selected.extend(
            path for path in blocks_root.glob(f"{phase}-*.json") if path.is_file()
        )
    if phase == "profile":
        profile_root = run_dir / "profile"
        if profile_root.is_dir():
            selected.extend(path for path in profile_root.rglob("*") if path.is_file())
    selected = sorted(set(selected))
    if not selected:
        raise ExperimentIncomplete(
            f"phase {phase} has no retained raw evidence to freeze"
        )
    lines = []
    for path in selected:
        relative = path.relative_to(run_dir).as_posix()
        if "\n" in relative or "\r" in relative:
            raise ExperimentIncomplete("newline in phase evidence path")
        lines.append(f"{sha256_file(path)}  {relative}\n")
    payload = "".join(lines).encode()
    manifest = run_dir / "manifests" / f"{phase}-preanalysis.sha256"
    write_bytes_once(manifest, payload)
    for line in manifest.read_text(encoding="utf-8").splitlines():
        digest, separator, relative = line.partition("  ")
        if separator != "  " or sha256_file(run_dir / relative) != digest:
            raise ExperimentIncomplete(
                f"phase {phase} evidence manifest failed verification"
            )
    return {
        "path": str(manifest),
        "sha256": sha256_file(manifest),
        "member_count": len(selected),
        "verification": "PASS",
    }


def execute_phase(executor: Executor, phase: str) -> dict[str, Any]:
    if phase in ("pilot", "main", "aa"):
        run_block_phase(executor, phase)
        phase_manifest = freeze_phase_evidence(executor.run_dir, phase)
        if phase == "pilot":
            result = analyze_pilot(executor)
        elif phase == "main":
            result = analyze_main(executor)
        else:
            result = analyze_aa(executor)
    elif phase == "quick":
        run_direct_phase(executor, phase)
        phase_manifest = freeze_phase_evidence(executor.run_dir, phase)
        result = analyze_direct(executor, phase)
    elif phase == "profile":
        result = run_profile(executor)
        phase_manifest = freeze_phase_evidence(executor.run_dir, phase)
    else:
        raise ValueError(f"unknown phase {phase!r}")
    result["preanalysis_phase_manifest"] = phase_manifest
    write_json_once(executor.run_dir / "analysis" / f"{phase}.json", result)
    return result


def checksum_manifest(run_dir: Path) -> tuple[str, int]:
    paths = sorted(
        path
        for path in run_dir.rglob("*")
        if path.is_file() and path.name != "manifest.sha256"
    )
    lines = []
    for path in paths:
        relative = path.relative_to(run_dir).as_posix()
        if "\n" in relative or "\r" in relative:
            raise ExperimentIncomplete(
                "newline in evidence path cannot be represented safely"
            )
        lines.append(f"{sha256_file(path)}  {relative}\n")
    payload = "".join(lines).encode("utf-8")
    manifest = run_dir / "manifest.sha256"
    write_bytes_once(manifest, payload)
    retained = manifest.read_bytes()
    if retained != payload:
        raise ExperimentIncomplete("final checksum manifest bytes changed after write")
    for line in retained.decode("utf-8").splitlines():
        digest, separator, relative = line.partition("  ")
        if separator != "  " or sha256_file(run_dir / relative) != digest:
            raise ExperimentIncomplete("final checksum manifest failed verification")
    return sha256_bytes(payload), len(paths)


def create_run_directory(requested: Path) -> Path:
    if not requested.is_absolute():
        raise ValueError("--output-dir must be absolute")
    resolved = requested.resolve(strict=False)
    repository = REPO_ROOT.resolve()
    try:
        common = Path(os.path.commonpath([resolved, repository]))
    except ValueError:
        common = Path("/")
    if common == repository:
        raise ValueError("--output-dir must be outside the repository")
    if resolved.exists():
        raise ValueError("--output-dir must not already exist")
    if not resolved.parent.is_dir():
        raise ValueError("--output-dir parent must already exist")
    resolved.mkdir()
    return resolved


def self_check() -> dict[str, Any]:
    errors = []
    if len(CELLS) != 4 or len({cell.name for cell in CELLS}) != 4:
        errors.append("expected four uniquely named workload cells")
    if len(CONTRASTS) != FAMILY_SIZE:
        errors.append("primary contrast family is not exactly 32")
    if set(COMPARATORS) != {"oracle", "packed", "events", "btree"}:
        errors.append("flat-baseline comparator set changed")
    if {contrast.operation_phase for contrast in CONTRASTS} != set(OPERATION_PHASES):
        errors.append("primary family does not cover both frozen operation phases")
    if PROFILE_SPECS != (
        ("oracle", "tiny_sparse_sorted_unique", "build"),
        ("flat", "large_sparse_reverse_unique", "build"),
        ("packed", "large_sparse_reverse_unique", "build_membership"),
        ("events", "cache_clustered_shuffled_duplicates", "build"),
        ("btree", "large_adjacent_shuffled_duplicates", "build_membership"),
    ):
        errors.append("five representative dynamic profile specifications changed")
    if DEFAULT_TIMEOUT_SECONDS != 180.0:
        errors.append("per-process timeout changed from 180 seconds")
    if MAX_ATTEMPTS_PER_POSITION != 1:
        errors.append("retry policy changed from no retries")
    if PILOT_SENSITIVITY_MULTIPLIERS != (0.5, 1.0, 1.5, 2.0):
        errors.append("pilot sensitivity multipliers changed")
    if balanced_templates(PILOT_BLOCKS, "self-pilot").count("ABBA") != 2:
        errors.append("pilot templates are not balanced")
    if balanced_templates(MAIN_BLOCKS, "self-main").count("ABBA") != 6:
        errors.append("main templates are not balanced")
    all_schedule = planned_schedule(["pilot", "main", "aa"])
    expected_positions = (
        FAMILY_SIZE * PILOT_BLOCKS * 4 + FAMILY_SIZE * MAIN_BLOCKS * 4 + AA_BLOCKS * 4
    )
    if len(all_schedule) != expected_positions:
        errors.append(
            f"planned {len(all_schedule)} positions, expected {expected_positions}"
        )
    identifiers = []
    for row in all_schedule:
        identifiers.append(attempt_id(Position(**row)))
    if len(identifiers) != len(set(identifiers)):
        errors.append("planned attempt identifiers are not unique")
    quick_schedule = planned_schedule(["quick"])
    if len(quick_schedule) != len(CELLS) * len(CANDIDATES) * len(OPERATION_PHASES):
        errors.append("quick schedule does not cover every candidate, cell, and phase")
    quantile = student_t_quantile(0.975, 11)
    if not math.isclose(quantile, 2.200985, rel_tol=0.0, abs_tol=1.0e-5):
        errors.append(
            f"Student t implementation returned unexpected t(11,.975)={quantile}"
        )
    parser_position = Position(
        "self-check",
        "self-check",
        1,
        "Q",
        1,
        "Q",
        "flat",
        "tiny_sparse_sorted_unique",
        "build_membership",
        1,
        1,
        3,
        1,
    )
    parser_record = {
        "schema_version": 1,
        "candidate": "flat",
        "workload": "tiny_sparse_sorted_unique",
        "phase": "build_membership",
        "seed": 1,
        "warmup": 1,
        "samples": 3,
        "inner": 1,
        "generator_version": GENERATOR_VERSION,
        "fixture_hash": "0000000000000001",
        "sample_ns": [1, 2, 3],
        "output_hash": "0000000000000002",
        "work": {
            "input_intervals": 8,
            "unique_input_intervals": 8,
            "duplicate_intervals": 0,
            "sort_comparisons_count_pass": 7,
            "merge_comparisons": 7,
            "output_runs": 8,
            "membership_queries": 64,
            "canonical_binary_search_comparisons": 192,
            "result_scalar_slots": 16,
        },
        "canaries": {
            "oracle_match": True,
            "domain_end": 1 << 32,
            "packed_max_decode": 1 << 32,
            "membership_match": True,
        },
    }
    parser_errors = validate_record(parser_position, parser_record)
    if parser_errors:
        errors.append(f"frozen parser rejected its self-check record: {parser_errors}")
    if not BENCH_SOURCE.is_file():
        errors.append(f"benchmark source is missing: {BENCH_SOURCE}")
    if not (TOPIC_DIR / "src" / "lib.rs").is_file():
        errors.append("library source is missing")
    return {
        "status": "PASS" if not errors else "FAIL",
        "errors": errors,
        "assignment_seed": ASSIGNMENT_SEED,
        "cell_count": len(CELLS),
        "contrast_count": len(CONTRASTS),
        "dynamic_profile_count": len(PROFILE_SPECS),
        "timeout_seconds": DEFAULT_TIMEOUT_SECONDS,
        "attempts_per_position": MAX_ATTEMPTS_PER_POSITION,
        "pilot_sensitivity_multipliers": PILOT_SENSITIVITY_MULTIPLIERS,
        "planned_position_count": len(all_schedule),
        "expected_position_count": expected_positions,
        "student_t_975_df11": quantile,
    }


def parse_cli() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "mode",
        choices=(
            "plan",
            "self-check",
            "quick",
            "pilot",
            "main",
            "aa",
            "profile",
            "all",
        ),
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        help="required for collection: absolute, outside Git, and initially nonexistent",
    )
    parser.add_argument(
        "--timeout-seconds", type=float, default=DEFAULT_TIMEOUT_SECONDS
    )
    parser.add_argument("--rustflags", default="-C target-cpu=native")
    return parser.parse_args()


def run_experiment(args: argparse.Namespace) -> int:
    if not math.isfinite(args.timeout_seconds) or args.timeout_seconds <= 0:
        raise ValueError("--timeout-seconds must be finite and positive")
    if (
        args.mode not in ("plan", "self-check")
        and args.timeout_seconds != DEFAULT_TIMEOUT_SECONDS
    ):
        raise ValueError("collection modes require the frozen 180-second timeout")
    protocol = protocol_document(args.mode, args.timeout_seconds, args.rustflags)
    if args.mode == "plan":
        print(json.dumps(protocol, indent=2, sort_keys=True))
        return 0
    if args.mode == "self-check":
        result = self_check()
        print(json.dumps(result, indent=2, sort_keys=True))
        return 0 if result["status"] == "PASS" else 1
    if args.output_dir is None:
        raise ValueError("--output-dir is required for collection modes")
    run_dir = create_run_directory(args.output_dir)
    relay = ExternalSignalRelay()
    relay.install()
    status: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "status": "STARTED",
        "started_at": utc_now(),
        "requested_mode": args.mode,
        "completed_phases": [],
    }
    failure: BaseException | None = None
    executor: Executor | None = None
    initial_source: list[dict[str, Any]] = []
    try:
        replace_json(run_dir / "run-status.json", status)
        write_json_once(run_dir / "protocol.json", protocol)
        initial_source = source_manifest()
        write_json_once(
            run_dir / "metadata" / "source-tree-before.json", initial_source
        )
        capture_source_snapshot(run_dir, initial_source)
        env = build_environment(args.rustflags)
        write_json_once(run_dir / "metadata" / "host.json", host_metadata())
        write_json_once(
            run_dir / "metadata" / "toolchain.json", toolchain_metadata(env)
        )
        write_json_once(run_dir / "metadata" / "git.json", git_metadata(env))
        relay.raise_if_pending()
        binary, build = build_benchmark(run_dir, args.rustflags, relay)
        post_build_source = source_manifest()
        write_json_once(
            run_dir / "metadata" / "source-tree-after-build.json", post_build_source
        )
        if source_digest(post_build_source) != source_digest(initial_source):
            raise ExperimentIncomplete("source tree changed during the build")
        executor = Executor(run_dir, binary, args.timeout_seconds, relay)
        write_json_once(
            run_dir / "metadata" / "benchmark-environment.json",
            {
                "cwd": str(REPO_ROOT),
                "recorded_environment": safe_recorded_environment(executor.env),
                "binary_sha256": build["binary_sha256"],
            },
        )
        for phase in phase_sequence(args.mode):
            execute_phase(executor, phase)
            relay.raise_if_pending()
            status["completed_phases"].append(phase)
            status["last_progress_at"] = utc_now()
            replace_json(run_dir / "run-status.json", status)
        final_source = source_manifest()
        write_json_once(run_dir / "metadata" / "source-tree-after.json", final_source)
        if source_digest(final_source) != source_digest(initial_source):
            raise ExperimentIncomplete("source tree changed during collection")
        if sha256_file(binary) != build["binary_sha256"]:
            raise ExperimentIncomplete(
                "retained linked image changed during collection"
            )
        relay.raise_if_pending()
        executor.write_tables()
        status.update(
            {
                "status": "COMPLETE",
                "ended_at": utc_now(),
                "attempt_count": len(executor.attempts),
                "block_count": len(executor.blocks),
                "source_tree_sha256": source_digest(initial_source),
                "source_tree_sampled_at": [
                    "before build",
                    "after build",
                    "after the final phase",
                ],
                "source_attestation_limit": (
                    "endpoint hashing proves the tree matched at each sampled instant; it "
                    "cannot exclude an edit made and reverted between two samples, and the "
                    "image is not built from the archived snapshot"
                ),
                "binary_sha256": executor.binary_hash,
            }
        )
    except BaseException as error:
        failure = error
        status.update(
            {
                "status": "INCOMPLETE",
                "ended_at": utc_now(),
                "failure_type": type(error).__name__,
                "failure": str(error),
            }
        )
        if executor is not None:
            status["attempt_count"] = len(executor.attempts)
            status["block_count"] = len(executor.blocks)
            write_json_once(
                run_dir / "analysis" / "reliability-incomplete.json",
                {
                    phase: reliability(executor, phase)
                    for phase in sorted(
                        {attempt.phase for attempt in executor.attempts}
                    )
                },
            )
            try:
                executor.write_tables()
            except FileExistsError:
                pass
        write_text_once(run_dir / "failure.txt", traceback.format_exc())
    try:
        try:
            replace_json(run_dir / "run-status.json", status)
            manifest_hash, manifest_count = checksum_manifest(run_dir)
            if failure is None:
                relay.raise_if_pending()
        except BaseException as error:
            if failure is not None:
                raise
            failure = error
            status.update(
                {
                    "status": "INCOMPLETE",
                    "ended_at": status.get("ended_at") or utc_now(),
                    "failure_type": type(error).__name__,
                    "failure": str(error),
                    "manifest_verification": "FAILED",
                }
            )
            try:
                write_text_once(run_dir / "failure.txt", traceback.format_exc())
            except FileExistsError:
                write_text_once(
                    run_dir / "manifest-failure.txt", traceback.format_exc()
                )
            replace_json(run_dir / "run-status.json", status)
            (run_dir / "manifest.sha256").unlink(missing_ok=True)
            manifest_hash, manifest_count = checksum_manifest(run_dir)
    finally:
        relay.restore()
    print(
        json.dumps(
            {
                "run_directory": str(run_dir),
                "status": status["status"],
                "manifest_sha256": manifest_hash,
                "manifest_file_count": manifest_count,
            },
            sort_keys=True,
        )
    )
    return 1 if failure is not None else 0


def main() -> int:
    try:
        return run_experiment(parse_cli())
    except (OSError, RuntimeError, ValueError) as error:
        print(f"run_experiment.py: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
