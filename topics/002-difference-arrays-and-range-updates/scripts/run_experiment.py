#!/usr/bin/env python3
"""Run the frozen range-update benchmark protocol and retain raw evidence."""

from __future__ import annotations

import sys

if sys.version_info < (3, 11):
    raise SystemExit(
        "run_experiment.py requires Python 3.11 or newer "
        f"(tomllib, datetime.UTC); running under {sys.version.split()[0]}"
    )

import argparse
import csv
import functools
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
import time
import tomllib
import traceback
from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any, Iterable, Sequence


SCHEMA_VERSION = 1
ASSIGNMENT_SEED = 2_026_083_002
FAMILY_ALPHA = 0.05
FAMILY_SIZE = 18
PRACTICAL_RATIO = 1.05
PILOT_BLOCKS = 4
MAIN_BLOCKS = 12
AA_BLOCKS = 12
DEFAULT_WARMUPS = 1
QUICK_WARMUPS = 0
QUICK_SAMPLES = 1
DEFAULT_TIMEOUT_SECONDS = 120.0
PERF_STAT_EVENTS = (
    "cycles",
    "instructions",
    "branches",
    "branch-misses",
    "cache-references",
    "cache-misses",
    "page-faults",
    "context-switches",
)
PERF_STAT_SEPARATOR = ","
WORK_FIELDS = (
    "validation_checks",
    "base_reads",
    "base_writes",
    "range_element_updates",
    "boundary_updates",
    "difference_steps",
    "scan_steps",
    "event_records",
    "sort_comparisons_count_pass",
    "vec_constructions",
    "allocated_scalar_slots",
)
# Windows lacks SIGHUP; name-based lookup keeps this module importable.
EXTERNAL_SIGNALS = {
    resolved
    for name in ("SIGINT", "SIGTERM", "SIGHUP")
    if (resolved := getattr(signal, name, None)) is not None
}

SCRIPT = Path(__file__).resolve()
TOPIC_DIR = SCRIPT.parents[1]
REPO_ROOT = SCRIPT.parents[3]
BENCH_SOURCE = TOPIC_DIR / "benches" / "range_updates.rs"


@dataclass(frozen=True)
class Contrast:
    contrast_id: str
    b_algorithm: str
    cell: str
    n: int
    updates: int
    pattern: str
    order: str
    max_span: int
    samples: int
    inner: int = 1


CELLS = (
    ("tiny_point_sorted", 64, 8, "point", "sorted", 1, 101, 1),
    ("small_full_repeated", 4_096, 256, "full", "reverse", 4_096, 31, 1),
    ("cache_short_shuffled", 65_536, 2_048, "uniform", "shuffled", 32, 7, 1),
    ("cache_clustered", 65_536, 2_048, "clustered", "shuffled", 1_024, 7, 1),
    ("large_sparse_sorted", 1_048_576, 8, "point", "sorted", 1, 3, 1),
    ("large_wide_shuffled", 262_144, 64, "uniform", "shuffled", 131_072, 3, 1),
)
CONTRASTS = tuple(
    Contrast(
        f"{candidate}_vs_in_place__{cell}",
        candidate,
        cell,
        n,
        updates,
        pattern,
        order,
        max_span,
        samples,
        inner,
    )
    for cell, n, updates, pattern, order, max_span, samples, inner in CELLS
    for candidate in ("reference", "dense", "sorted_events")
)
AA_CONTRAST = Contrast(
    "aa_in_place__cache_short_shuffled",
    "in_place",
    "cache_short_shuffled",
    65_536,
    2_048,
    "uniform",
    "shuffled",
    32,
    7,
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
    cell: str
    n: int
    updates: int
    pattern: str
    order: str
    max_span: int
    seed: int
    samples: int
    inner: int
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
    cell: str
    n: int
    updates: int
    pattern: str
    order: str
    max_span: int
    seed: int
    samples: int
    inner: int
    status: str
    invalid_reason: str
    external_interruption: bool
    returncode: int | None
    timed_out: bool
    position_ns: int | None
    zero_sample_count: int
    generator_version: str
    input_hash: str
    updates_hash: str
    output_hash: str
    work_sha256: str
    parser_status: str
    canary_status: str
    stdout_sha256: str
    stderr_sha256: str
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
                os.killpg(child.pid, signum)
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


def child_affinity_preexec(cpus: frozenset[int] | None) -> Any:
    """Build a `preexec_fn` that pins the child before it execs the target.

    A `taskset` wrapper applies the mask inside its own process and then execs,
    so a parent that reads the child's affinity right after `Popen` can observe
    the inherited mask instead of the requested one. Applying the mask in the
    forked child closes that window: `Popen` returns only after the child execs,
    so the mask is already in place when the parent validates it.

    `preexec_fn` runs between `fork` and `exec` and is only safe in a
    single-threaded parent, which this runner is.
    """
    if cpus is None:
        return None

    def apply() -> None:
        os.sched_setaffinity(0, set(cpus))

    return apply


def run_capture_group(
    argv: Sequence[str],
    *,
    cwd: Path,
    env: dict[str, str],
    timeout: float,
    relay: ExternalSignalRelay | None = None,
    cpus: frozenset[int] | None = None,
) -> tuple[int | None, bytes, bytes, bool]:
    """Run a wrapper command and, on timeout, kill its whole process group.

    Killing only the direct child leaves grandchildren running: a timed-out
    profiler wrapper such as `perf stat -- <harness>` dies while the harness
    it launched keeps consuming CPU and memory. A new session gives the
    wrapper its own process group, so the timeout kill reaches every
    descendant.
    """
    try:
        process = subprocess.Popen(
            list(argv),
            cwd=cwd,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
            preexec_fn=child_affinity_preexec(cpus),
        )
    except (OSError, subprocess.SubprocessError) as error:
        # A caller classifies a `None` return code as a failed collection, so a
        # spawn or `preexec_fn` failure stays inside the retained record instead
        # of escaping this helper. commentlint: allow(JUDGE)
        return None, b"", f"spawn failure: {error!r}".encode("utf-8"), False
    with process:
        if relay is not None:
            relay.child = process
        try:
            stdout, stderr = process.communicate(timeout=timeout)
            return process.returncode, stdout, stderr, False
        except subprocess.TimeoutExpired:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            stdout, stderr = process.communicate()
            return process.returncode, stdout, stderr, True
        finally:
            if relay is not None:
                relay.child = None


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
    if build_env.get(env_key):
        findings.append({"source": env_key, "value": build_env[env_key]})
    if build_env.get("CARGO_BUILD_TARGET") not in (None, "", host_triple):
        findings.append({"source": "CARGO_BUILD_TARGET", "value": build_env["CARGO_BUILD_TARGET"]})

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
                # cfg(...) tables remain flagged because this check cannot
                # resolve their match set.
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


def validate_source_snapshot_records(records: Sequence[dict[str, Any]]) -> list[str]:
    errors = []
    paths = [record.get("path") for record in records]
    if len(paths) != len(set(paths)):
        errors.append("source manifest contains duplicate paths")
    for record in records:
        relative_text = record.get("path")
        if not isinstance(relative_text, str):
            errors.append("source manifest path is not a string")
            continue
        relative = Path(relative_text)
        if relative.is_absolute() or ".." in relative.parts:
            errors.append(f"source manifest path escapes the repository: {relative_text!r}")
            continue
        source = REPO_ROOT / relative
        if not source.is_file():
            errors.append(f"source manifest file is missing: {relative_text}")
            continue
        if source.stat().st_size != record.get("bytes"):
            errors.append(f"source manifest byte count changed: {relative_text}")
        if sha256_file(source) != record.get("sha256"):
            errors.append(f"source manifest hash changed: {relative_text}")
    return errors


def verify_source_snapshot(
    run_dir: Path, records: Sequence[dict[str, Any]]
) -> dict[str, Any]:
    snapshot_root = run_dir / "source-snapshot"
    errors = validate_source_snapshot_records(records)
    expected_paths = {str(record["path"]) for record in records}
    observed_paths = {
        path.relative_to(snapshot_root).as_posix()
        for path in snapshot_root.rglob("*")
        if path.is_file()
    }
    if observed_paths != expected_paths:
        errors.append(
            "source snapshot file set differs from the source manifest: "
            f"missing={sorted(expected_paths - observed_paths)!r}, "
            f"extra={sorted(observed_paths - expected_paths)!r}"
        )
    verified = []
    for record in records:
        relative = str(record["path"])
        snapshot = snapshot_root / relative
        if not snapshot.is_file():
            continue
        observed_bytes = snapshot.stat().st_size
        observed_sha256 = sha256_file(snapshot)
        if observed_bytes != record["bytes"]:
            errors.append(f"source snapshot byte count mismatch: {relative}")
        if observed_sha256 != record["sha256"]:
            errors.append(f"source snapshot hash mismatch: {relative}")
        verified.append(
            {
                "path": relative,
                "bytes": observed_bytes,
                "sha256": observed_sha256,
            }
        )
    return {
        "status": "COMPLETE" if not errors else "FAILED",
        "expected_file_count": len(records),
        "verified_file_count": len(verified),
        "source_tree_sha256": source_tree_digest(records),
        "errors": errors,
        "files": verified,
    }


def capture_source_snapshot(
    run_dir: Path, records: Sequence[dict[str, Any]]
) -> dict[str, Any]:
    prerequisite_errors = validate_source_snapshot_records(records)
    if prerequisite_errors:
        raise ExperimentIncomplete(
            f"source changed before snapshot capture: {prerequisite_errors}"
        )
    snapshot_root = run_dir / "source-snapshot"
    for record in records:
        relative = str(record["path"])
        source = REPO_ROOT / relative
        data = source.read_bytes()
        if len(data) != record["bytes"] or sha256_bytes(data) != record["sha256"]:
            raise ExperimentIncomplete(f"source changed while snapshotting: {relative}")
        write_bytes_once(snapshot_root / relative, data)
    verification = verify_source_snapshot(run_dir, records)
    if verification["status"] != "COMPLETE":
        raise ExperimentIncomplete(
            f"source snapshot verification failed: {verification['errors']}"
        )
    return verification


def git_metadata(run_dir: Path, env: dict[str, str]) -> dict[str, Any]:
    git = executable("git")
    status_argv = [
        git,
        "status",
        "--porcelain=v1",
        "--untracked-files=all",
        "--",
        ".",
        ":(exclude)topics/002-difference-arrays-and-range-updates/measurements/runs",
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


def frequency_policy_metadata() -> dict[str, Any]:
    if sys.platform.startswith("linux"):
        names = (
            "scaling_governor",
            "scaling_driver",
            "scaling_min_freq",
            "scaling_max_freq",
        )
        records = []
        for path in sorted(Path("/sys/devices/system/cpu").glob("cpu*/cpufreq/*")):
            if path.name not in names:
                continue
            try:
                records.append(
                    {"path": str(path), "status": "COMPLETE", "value": path.read_text().strip()}
                )
            except OSError as error:
                records.append({"path": str(path), "status": "ERROR", "error": repr(error)})
        return {
            "status": "COMPLETE" if any(row["status"] == "COMPLETE" for row in records) else "UNAVAILABLE",
            "platform": "linux",
            "records": records,
            "reason": None if records else "no cpufreq policy files were exposed",
        }
    if sys.platform == "darwin":
        sysctl = Path("/usr/sbin/sysctl")
        records = []
        for key in ("hw.cpufrequency", "hw.cpufrequency_min", "hw.cpufrequency_max"):
            try:
                completed = subprocess.run(
                    [str(sysctl), "-n", key],
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    check=False,
                    timeout=5,
                )
                records.append(
                    {
                        "key": key,
                        "returncode": completed.returncode,
                        "stdout": completed.stdout.decode(
                            "utf-8", errors="replace"
                        ).strip(),
                        "stderr": completed.stderr.decode(
                            "utf-8", errors="replace"
                        ).strip(),
                    }
                )
            except (OSError, subprocess.SubprocessError) as error:
                records.append({"key": key, "status": "ERROR", "error": repr(error)})
        available = any(
            row.get("returncode") == 0 and row.get("stdout") for row in records
        )
        return {
            "status": "COMPLETE" if available else "UNAVAILABLE",
            "platform": "macos",
            "records": records,
            "reason": (
                None
                if available
                else "macOS did not expose the documented frequency sysctl values; "
                "no governor claim is made"
            ),
        }
    return {
        "status": "UNAVAILABLE",
        "platform": sys.platform,
        "records": [],
        "reason": "no frequency-policy probe is defined for this platform",
    }


DARWIN_HARDWARE_FIELDS = (
    "chip_type",
    "machine_model",
    "machine_name",
    "model_number",
    "number_processors",
    "physical_memory",
)


def darwin_hardware_metadata() -> dict[str, Any]:
    argv = ["/usr/sbin/system_profiler", "SPHardwareDataType", "-json"]
    if sys.platform != "darwin":
        return {
            "status": "UNAVAILABLE",
            "command": argv,
            "returncode": None,
            "fields": {},
            "reason": "system_profiler hardware identity is defined only for Darwin",
        }
    try:
        completed = subprocess.run(
            argv,
            cwd=REPO_ROOT,
            env=safe_base_environment(),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=15,
        )
    except (OSError, subprocess.SubprocessError) as error:
        return {
            "status": "UNAVAILABLE",
            "command": argv,
            "returncode": None,
            "fields": {},
            "reason": f"system_profiler invocation failed: {type(error).__name__}",
        }

    fields: dict[str, str | int] = {}
    parse_error: str | None = None
    try:
        document = json.loads(completed.stdout)
        records = document.get("SPHardwareDataType") if isinstance(document, dict) else None
        hardware = records[0] if isinstance(records, list) and records else None
        if not isinstance(hardware, dict):
            raise ValueError("SPHardwareDataType did not contain a hardware record")
        fields = {
            name: value
            for name in DARWIN_HARDWARE_FIELDS
            if isinstance((value := hardware.get(name)), (str, int))
            and not isinstance(value, bool)
            and (not isinstance(value, str) or bool(value.strip()))
        }
    except (json.JSONDecodeError, ValueError) as error:
        parse_error = type(error).__name__

    complete = (
        completed.returncode == 0
        and parse_error is None
        and set(fields) == set(DARWIN_HARDWARE_FIELDS)
    )
    if completed.returncode != 0:
        reason = f"system_profiler exited with code {completed.returncode}"
    elif parse_error is not None:
        reason = f"system_profiler JSON could not be sanitized: {parse_error}"
    elif not complete:
        missing = sorted(set(DARWIN_HARDWARE_FIELDS) - set(fields))
        reason = f"system_profiler omitted required sanitized fields: {missing}"
    else:
        reason = None
    return {
        "status": "COMPLETE" if complete else "UNAVAILABLE",
        "command": argv,
        "returncode": completed.returncode,
        "fields": fields,
        "reason": reason,
    }


def host_metadata() -> dict[str, Any]:
    cpu_description = platform.processor()
    hardware_identity = darwin_hardware_metadata()
    if sys.platform == "darwin":
        chip_type = hardware_identity.get("fields", {}).get("chip_type")
        if isinstance(chip_type, str) and chip_type.strip():
            cpu_description = chip_type
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
        "hardware_identity": hardware_identity,
        "frequency_policy": frequency_policy_metadata(),
        "python": sys.version,
    }


def tool_output(argv: Sequence[str], env: dict[str, str]) -> str:
    completed = run_capture(argv, cwd=REPO_ROOT, env=env)
    if completed.returncode != 0:
        raise RuntimeError(f"tool identity command failed: {' '.join(argv)}")
    return completed.stdout.decode("utf-8", errors="replace").strip()


def rustup_dispatched_tool(tool_name: str, build_env: dict[str, str]) -> dict[str, Any] | None:
    """Resolve the toolchain binary that a rustup proxy dispatches to.

    On rustup-managed hosts the PATH entries for rustc and cargo are the
    shared rustup proxy, so hashing them does not identify the compiler that
    runs. `rustup which` reports the dispatched binary for the toolchain
    active in the repository.
    """
    rustup = shutil.which("rustup", path=build_env.get("PATH"))
    if rustup is None:
        return None
    completed = run_capture(
        [rustup, "which", tool_name], cwd=REPO_ROOT, env=build_env, timeout=15.0
    )
    if completed.returncode != 0:
        return None
    resolved = Path(completed.stdout.decode("utf-8", errors="replace").strip())
    if not resolved.is_file():
        return None
    return {"path": str(resolved), "sha256": sha256_file(resolved)}


def source_allocator_boundary() -> dict[str, Any]:
    declarations = []
    for path in sorted(TOPIC_DIR.rglob("*.rs")):
        for line_number, line in enumerate(
            path.read_text(encoding="utf-8", errors="replace").splitlines(), start=1
        ):
            if "global_allocator" in line:
                declarations.append(
                    {
                        "path": path.relative_to(REPO_ROOT).as_posix(),
                        "line": line_number,
                        "text": line.strip(),
                    }
                )
    return {
        "global_allocator_declarations": declarations,
        "source_defines_global_allocator": bool(declarations),
        "boundary": (
            "No #[global_allocator] declaration appears in the topic Rust source. The build "
            "therefore uses the Rust target default. This does not by itself identify a "
            "concrete allocator implementation."
            if not declarations
            else "The source declares a global allocator; inspect the recorded declarations."
        ),
    }


def linked_library_metadata(
    run_dir: Path, binary: Path, environment: dict[str, str]
) -> dict[str, Any]:
    if sys.platform == "darwin" and Path("/usr/bin/otool").is_file():
        argv = ["/usr/bin/otool", "-L", str(binary)]
    elif sys.platform.startswith("linux") and shutil.which("ldd") is not None:
        argv = [str(Path(shutil.which("ldd") or "ldd").resolve()), str(binary)]
    else:
        return {
            "status": "UNAVAILABLE",
            "reason": "neither macOS otool -L nor Linux ldd is available",
        }
    try:
        completed = run_capture(argv, cwd=REPO_ROOT, env=environment, timeout=30.0)
        timed_out = False
    except subprocess.TimeoutExpired as error:
        completed = subprocess.CompletedProcess(
            argv,
            returncode=124,
            stdout=error.stdout or b"",
            stderr=error.stderr or b"",
        )
        timed_out = True
    stdout_path = run_dir / "metadata" / "linked-libraries.stdout"
    stderr_path = run_dir / "metadata" / "linked-libraries.stderr"
    write_bytes_once(stdout_path, completed.stdout)
    write_bytes_once(stderr_path, completed.stderr)
    return {
        "status": "COMPLETE" if completed.returncode == 0 and not timed_out else "FAILED",
        "argv": argv,
        "returncode": completed.returncode,
        "timed_out": timed_out,
        "stdout_sha256": sha256_bytes(completed.stdout),
        "stderr_sha256": sha256_bytes(completed.stderr),
        "boundary": (
            "Linked-library identity constrains the system boundary. It does not prove which "
            "concrete allocator services every allocation."
        ),
    }


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
        "topic-002-difference-arrays-and-range-updates",
        "--bench",
        "range_updates",
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
            and target.get("name") == "range_updates"
            and "bench" in target.get("kind", [])
            and record.get("executable")
        ):
            artifacts.append(Path(record["executable"]).resolve())
    unique_artifacts = sorted(set(artifacts))
    if len(unique_artifacts) != 1 or not unique_artifacts[0].is_file():
        raise RuntimeError(f"expected one resolved benchmark executable, found {unique_artifacts}")

    source_binary = unique_artifacts[0]
    binary_hash = sha256_file(source_binary)
    retained_binary = run_dir / "artifacts" / f"range_updates-{binary_hash}"
    retained_binary.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source_binary, retained_binary)
    if sha256_file(retained_binary) != binary_hash:
        raise RuntimeError("retained benchmark image hash does not match source image")

    allocator = source_allocator_boundary()
    linked_libraries = linked_library_metadata(run_dir, retained_binary, build_env)

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
        "benchmark_source": str(BENCH_SOURCE),
        "benchmark_source_sha256": sha256_file(BENCH_SOURCE),
        "runner_source": str(SCRIPT),
        "runner_source_sha256": sha256_file(SCRIPT),
        "cargo_sha256": sha256_file(Path(cargo)),
        "rustc_sha256": sha256_file(Path(rustc)),
        "dispatched_cargo": rustup_dispatched_tool("cargo", build_env),
        "dispatched_rustc": rustup_dispatched_tool("rustc", build_env),
        "runner_configuration": runner_check,
        "allocator": allocator,
        "linked_libraries": linked_libraries,
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


def phase_warmups(phase: str) -> int:
    return QUICK_WARMUPS if phase == "quick" else DEFAULT_WARMUPS


def phase_samples(phase: str, contrast: Contrast) -> int:
    return QUICK_SAMPLES if phase == "quick" else contrast.samples


def positions_for_block(
    phase: str,
    contrast: Contrast,
    block_index: int,
    template: str,
    retry_index: int,
) -> list[Position]:
    positions = []
    for position_index, label in enumerate(template, start=1):
        algorithm = "in_place" if label == "A" else contrast.b_algorithm
        positions.append(
            Position(
                phase=phase,
                contrast_id=contrast.contrast_id,
                block_index=block_index,
                template=template,
                position_index=position_index,
                label=label,
                algorithm=algorithm,
                cell=contrast.cell,
                n=contrast.n,
                updates=contrast.updates,
                pattern=contrast.pattern,
                order=contrast.order,
                max_span=contrast.max_span,
                seed=ASSIGNMENT_SEED,
                samples=phase_samples(phase, contrast),
                inner=contrast.inner,
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


def validate_work_shape(value: Any) -> list[str]:
    if not isinstance(value, dict):
        return ["work is not an object"]
    errors = []
    if tuple(sorted(value)) != tuple(sorted(WORK_FIELDS)):
        errors.append("work counter fields differ from the frozen schema")
    for field in WORK_FIELDS:
        observed = value.get(field)
        if not isinstance(observed, int) or isinstance(observed, bool) or observed < 0:
            errors.append(f"work.{field} is not a nonnegative integer")
    return errors


def validate_timing_samples(
    value: Any, expected_count: int
) -> tuple[list[str], int | None, int]:
    if (
        not isinstance(value, list)
        or len(value) != expected_count
        or any(
            not isinstance(sample, int) or isinstance(sample, bool) or sample < 0
            for sample in value
        )
    ):
        return ["invalid raw timing sample array"], None, 0
    zero_count = sum(sample == 0 for sample in value)
    position_ns = int(statistics.median(value))
    if position_ns <= 0:
        return ["position median is not positive and cannot enter log analysis"], position_ns, zero_count
    return [], position_ns, zero_count


class FixtureLcg:
    def __init__(self, seed: int) -> None:
        self.state = seed & ((1 << 64) - 1)

    def next_u64(self) -> int:
        self.state = (
            self.state * 6_364_136_223_846_793_005 + 1_442_695_040_888_963_407
        ) & ((1 << 64) - 1)
        return self.state

    def below(self, upper: int) -> int:
        if upper <= 0:
            raise ValueError("fixture generator upper bound must be positive")
        return self.next_u64() % upper

    def mixed_i64(self) -> int:
        selector = self.next_u64() & 7
        if selector == 0:
            return -(1 << 63)
        if selector == 1:
            return (1 << 63) - 1
        if selector == 2:
            return -1
        if selector == 3:
            return 0
        if selector == 4:
            return 1
        value = self.next_u64()
        return value if value < (1 << 63) else value - (1 << 64)

    def nonzero_mixed_i64(self, update_index: int) -> int:
        fixed = (1, -1, 7, -13, (1 << 63) - 1, -(1 << 63))
        if update_index % 8 < len(fixed):
            return fixed[update_index % 8]
        value = self.next_u64()
        signed = value if value < (1 << 63) else value - (1 << 64)
        return signed if signed != 0 else 1


def fnv1a_update(current: int, data: bytes) -> int:
    for byte in data:
        current ^= byte
        current = (current * 0x0000_0100_0000_01B3) & ((1 << 64) - 1)
    return current


@functools.lru_cache(maxsize=None)
def fixture_metrics(
    n: int,
    updates_count: int,
    pattern: str,
    order: str,
    max_span: int,
    seed: int,
) -> dict[str, int | str]:
    rng = FixtureLcg(seed)
    input_hash = fnv1a_update(
        0xCBF2_9CE4_8422_2325, n.to_bytes(8, "little", signed=False)
    )
    for _ in range(n):
        value = rng.mixed_i64()
        input_hash = fnv1a_update(input_hash, value.to_bytes(8, "little", signed=True))

    updates: list[tuple[int, int, int]] = []
    for update_index in range(updates_count):
        if pattern == "point":
            start = rng.below(n)
            end = start + 1
        elif pattern == "full":
            start, end = 0, n
        elif pattern == "uniform":
            start = rng.below(n)
            available = min(n - start, max_span)
            end = start + 1 + rng.below(available)
        elif pattern == "clustered":
            hot_width = min(n, 1_024)
            hot_start = (n - hot_width) // 2
            bucket_count = min(hot_width, 32) + 1
            first_bucket = rng.below(bucket_count)
            second_bucket = rng.below(bucket_count)
            if second_bucket == first_bucket:
                second_bucket = (
                    first_bucket + 1 if first_bucket + 1 < bucket_count else first_bucket - 1
                )
            first = hot_start + first_bucket * hot_width // (bucket_count - 1)
            second = hot_start + second_bucket * hot_width // (bucket_count - 1)
            start = min(first, second)
            end = max(first, second)
            end = min(end, start + max_span)
        else:
            raise ValueError(f"unsupported frozen fixture pattern: {pattern}")
        updates.append((start, end, rng.nonzero_mixed_i64(update_index)))

    if order == "sorted":
        updates.sort(key=lambda update: (update[0], update[1]))
    elif order == "reverse":
        updates.sort(key=lambda update: (update[0], update[1]))
        updates.reverse()
    elif order == "shuffled":
        for index in range(len(updates) - 1, 0, -1):
            other = rng.below(index + 1)
            updates[index], updates[other] = updates[other], updates[index]
    else:
        raise ValueError(f"unsupported frozen fixture order: {order}")

    updates_hash = fnv1a_update(
        0xCBF2_9CE4_8422_2325, updates_count.to_bytes(8, "little", signed=False)
    )
    for start, end, delta in updates:
        updates_hash = fnv1a_update(updates_hash, start.to_bytes(8, "little", signed=False))
        updates_hash = fnv1a_update(updates_hash, end.to_bytes(8, "little", signed=False))
        updates_hash = fnv1a_update(updates_hash, delta.to_bytes(8, "little", signed=True))
    return {
        "input_hash": f"{input_hash:016x}",
        "updates_hash": f"{updates_hash:016x}",
        "range_element_updates": sum(end - start for start, end, _ in updates),
        "end_before_n": sum(end < n for _, end, _ in updates),
    }


def validate_work_counts(position: Position, work: Any) -> list[str]:
    if not isinstance(work, dict) or any(
        not isinstance(work.get(field), int) or isinstance(work.get(field), bool)
        for field in WORK_FIELDS
    ):
        return []
    n = position.n
    updates = position.updates
    fixture = fixture_metrics(
        position.n,
        position.updates,
        position.pattern,
        position.order,
        position.max_span,
        position.seed,
    )
    common = {
        "validation_checks": updates,
        "base_reads": n,
        "base_writes": n,
    }
    if position.algorithm == "reference":
        expected = {
            **common,
            "boundary_updates": 0,
            "difference_steps": 0,
            "scan_steps": 0,
            "event_records": 0,
            "sort_comparisons_count_pass": 0,
            "vec_constructions": 1,
            "allocated_scalar_slots": n,
            "range_element_updates": fixture["range_element_updates"],
        }
    elif position.algorithm == "dense":
        expected = {
            **common,
            "range_element_updates": 0,
            "boundary_updates": 2 * updates,
            "difference_steps": 0,
            "scan_steps": n,
            "event_records": 0,
            "sort_comparisons_count_pass": 0,
            "vec_constructions": 2,
            "allocated_scalar_slots": 2 * n + 1,
        }
    elif position.algorithm == "in_place":
        expected = {
            **common,
            "range_element_updates": 0,
            "difference_steps": max(n - 1, 0),
            "scan_steps": max(n - 1, 0),
            "event_records": 0,
            "sort_comparisons_count_pass": 0,
            "vec_constructions": 1,
            "allocated_scalar_slots": n,
            "boundary_updates": updates + fixture["end_before_n"],
        }
    elif position.algorithm == "sorted_events":
        expected = {
            **common,
            "range_element_updates": 0,
            "difference_steps": 0,
            "scan_steps": n,
            "vec_constructions": 2,
            "allocated_scalar_slots": n + 4 * updates,
            "boundary_updates": updates + fixture["end_before_n"],
            "event_records": updates + fixture["end_before_n"],
        }
    else:
        return [f"unknown candidate in work canary: {position.algorithm}"]

    errors = [
        f"work.{field} mismatch: expected {expected_value}, got {work.get(field)!r}"
        for field, expected_value in expected.items()
        if work.get(field) != expected_value
    ]
    if position.algorithm == "sorted_events":
        events = work["event_records"]
        if work["boundary_updates"] != events:
            errors.append("sorted_events boundary_updates must equal event_records")
    return errors


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
            range_text, _, stride_text = part.partition(":")
            stride = 1
            if stride_text:
                stride = int(stride_text)
                if stride <= 0:
                    raise ValueError(f"non-positive CPU-range stride: {part}")
            start_text, end_text = range_text.split("-", 1)
            start = int(start_text)
            end = int(end_text)
            if end < start:
                raise ValueError(f"descending CPU range: {part}")
            cpus.update(range(start, end + 1, stride))
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
        self.output_hashes: dict[tuple[Any, ...], str] = {}
        self.fixture_hashes: dict[tuple[Any, ...], tuple[str, str, str]] = {}
        self.work_hashes: dict[tuple[Any, ...], str] = {}
        self.cpu_list = cpu_list
        self.cpus: frozenset[int] | None = None
        if cpu_list is not None:
            requested = parse_cpu_list(cpu_list)
            if not sys.platform.startswith("linux"):
                raise RuntimeError("--cpu-list requires Linux affinity control and validation")
            granted = set(os.sched_getaffinity(0))
            if not requested.issubset(granted):
                raise RuntimeError(
                    f"requested CPUs {sorted(requested)} exceed granted CPUs {sorted(granted)}"
                )
            self.cpus = frozenset(requested)

    def harness_argv(self, position: Position, warmups: int, samples: int) -> list[str]:
        return [
            str(self.binary),
            "--bench",
            "--candidate",
            position.algorithm,
            "--workload",
            position.cell,
            "--pattern",
            position.pattern,
            "--order",
            position.order,
            "--len",
            str(position.n),
            "--updates",
            str(position.updates),
            "--max-span",
            str(position.max_span),
            "--seed",
            str(position.seed),
            "--warmup",
            str(warmups),
            "--samples",
            str(samples),
            "--inner",
            str(position.inner),
        ]

    def run_position(self, position: Position, warmups: int, samples: int) -> AttemptResult:
        if self.relay is not None:
            self.relay.raise_if_pending()
        attempt_id = attempt_identifier(position)
        attempt_dir = self.run_dir / "raw" / "attempts" / attempt_id
        attempt_dir.mkdir(parents=True, exist_ok=False)
        harness_argv = self.harness_argv(position, warmups, samples)
        expanded_argv = harness_argv
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
            "requested_affinity": sorted(self.cpus) if self.cpus is not None else None,
            "affinity_application": (
                "sched_setaffinity in the forked child before exec"
                if self.cpus is not None
                else "inherited from the runner"
            ),
            "reset": {
                "action": "launch a fresh process and reconstruct the deterministic fixture",
                "washout_seconds": 0,
                "cross_process_host_state_reset": False,
            },
        }
        write_json_once(attempt_dir / "launch.json", launch)

        timed_out = False
        returncode: int | None = None
        stdout = b""
        stderr = b""
        observed_affinity: list[int] | None = None
        spawn_error: str | None = None
        started_at = utc_now()
        started_ns = time.time_ns()
        try:
            process = subprocess.Popen(
                expanded_argv,
                cwd=REPO_ROOT,
                env=self.environment,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                start_new_session=True,
                preexec_fn=child_affinity_preexec(self.cpus),
            )
        except (OSError, subprocess.SubprocessError) as error:
            # `preexec_fn` failures raise `SubprocessError`, which is not an `OSError`
            # subclass, so CPU-offline errors after validation are recorded here
            # rather than escaping before the attempt exists. commentlint: allow(JUDGE)
            spawn_error = repr(error)
        else:
            with process:
                if self.relay is not None:
                    self.relay.child = process
                if hasattr(os, "sched_getaffinity"):
                    try:
                        observed_affinity = sorted(os.sched_getaffinity(process.pid))
                    except (OSError, ProcessLookupError):
                        observed_affinity = None
                try:
                    stdout, stderr = process.communicate(timeout=self.timeout_seconds)
                    returncode = process.returncode
                except subprocess.TimeoutExpired:
                    timed_out = True
                    try:
                        os.killpg(process.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    stdout, stderr = process.communicate()
                    returncode = process.returncode
                finally:
                    if self.relay is not None:
                        self.relay.child = None
        ended_ns = time.time_ns()
        ended_at = utc_now()
        write_bytes_once(attempt_dir / "stdout.jsonl", stdout)
        write_bytes_once(attempt_dir / "stderr.txt", stderr)

        stdout_hash = sha256_bytes(stdout)
        stderr_hash = sha256_bytes(stderr)
        binary_after = sha256_file(self.binary)
        invalid_reasons: list[str] = []
        parsed: dict[str, Any] | None = None
        parser_status = "NOT_RUN"
        if binary_before != self.binary_hash or binary_after != self.binary_hash:
            invalid_reasons.append("executable hash changed")
        if spawn_error is not None:
            invalid_reasons.append(f"process spawn failure: {spawn_error}")
        elif timed_out:
            invalid_reasons.append("timeout")
        elif returncode != 0:
            invalid_reasons.append(f"nonzero return code {returncode}")
        if not timed_out and returncode == 0:
            lines = [
                line
                for line in stdout.decode("utf-8", errors="replace").splitlines()
                if line.strip()
            ]
            if len(lines) != 1:
                parser_status = "INVALID"
                invalid_reasons.append(f"expected one JSON line, found {len(lines)}")
            else:
                try:
                    candidate = json.loads(lines[0])
                    if not isinstance(candidate, dict):
                        raise TypeError("top-level JSON is not an object")
                    parsed = candidate
                    parser_status = "PARSED"
                except (json.JSONDecodeError, TypeError) as error:
                    parser_status = "INVALID"
                    invalid_reasons.append(f"JSON parse failure: {error}")

        expected_echo = {
            "schema_version": SCHEMA_VERSION,
            "candidate": position.algorithm,
            "workload": position.cell,
            "pattern": position.pattern,
            "order": position.order,
            "len": position.n,
            "updates": position.updates,
            "max_span": position.max_span,
            "seed": position.seed,
            "warmup": warmups,
            "samples": samples,
            "inner": position.inner,
        }
        position_ns: int | None = None
        zero_sample_count = 0
        generator_version = ""
        input_hash = ""
        updates_hash = ""
        output_hash = ""
        work_hash = ""
        if parsed is not None:
            expected_fields = set(expected_echo) | {
                "generator_version",
                "input_hash",
                "updates_hash",
                "sample_ns",
                "output_hash",
                "work",
            }
            if set(parsed) != expected_fields:
                invalid_reasons.append("top-level JSON fields differ from the frozen schema")
            for field, expected in expected_echo.items():
                if parsed.get(field) != expected:
                    invalid_reasons.append(
                        f"echo mismatch for {field}: expected {expected!r}, got {parsed.get(field)!r}"
                    )
            sample_ns = parsed.get("sample_ns")
            sample_errors, position_ns, zero_sample_count = validate_timing_samples(
                sample_ns, samples
            )
            invalid_reasons.extend(sample_errors)
            generator_version = parsed.get("generator_version", "")
            if generator_version != "topic002-fixture-v1":
                invalid_reasons.append(
                    "generator_version must equal 'topic002-fixture-v1'"
                )
            input_hash = parsed.get("input_hash", "")
            updates_hash = parsed.get("updates_hash", "")
            output_hash = parsed.get("output_hash", "")
            for field, value in (
                ("input_hash", input_hash),
                ("updates_hash", updates_hash),
                ("output_hash", output_hash),
            ):
                if (
                    not isinstance(value, str)
                    or len(value) != 16
                    or value.lower() != value
                ):
                    invalid_reasons.append(
                        f"{field} is not 16 lowercase hexadecimal characters"
                    )
                    continue
                try:
                    int(value, 16)
                except ValueError:
                    invalid_reasons.append(f"{field} is not hexadecimal")
            expected_fixture = fixture_metrics(
                position.n,
                position.updates,
                position.pattern,
                position.order,
                position.max_span,
                position.seed,
            )
            if input_hash != expected_fixture["input_hash"]:
                invalid_reasons.append(
                    "input_hash differs from the independent Python fixture generator"
                )
            if updates_hash != expected_fixture["updates_hash"]:
                invalid_reasons.append(
                    "updates_hash differs from the independent Python fixture generator"
                )
            work = parsed.get("work")
            invalid_reasons.extend(validate_work_shape(work))
            invalid_reasons.extend(validate_work_counts(position, work))
            if isinstance(work, dict):
                work_hash = sha256_bytes(
                    json.dumps(work, separators=(",", ":"), sort_keys=True).encode("utf-8")
                )
                work_key = (
                    position.algorithm,
                    position.cell,
                    position.n,
                    position.updates,
                    position.pattern,
                    position.order,
                    position.max_span,
                    position.seed,
                )
                prior_work = self.work_hashes.setdefault(work_key, work_hash)
                if prior_work != work_hash:
                    invalid_reasons.append("work counters changed for an identical candidate and fixture")
            cell_key = (
                position.cell,
                position.n,
                position.updates,
                position.pattern,
                position.order,
                position.max_span,
                position.seed,
            )
            fixture_identity = (generator_version, input_hash, updates_hash)
            if all(fixture_identity):
                prior_fixture = self.fixture_hashes.setdefault(cell_key, fixture_identity)
                if prior_fixture != fixture_identity:
                    invalid_reasons.append("fixture identity changed within one frozen cell")
            if output_hash:
                prior_output = self.output_hashes.setdefault(cell_key, output_hash)
                if prior_output != output_hash:
                    invalid_reasons.append("output hash differs across candidates in one fixture cell")
            if self.cpu_list is not None:
                requested_cpus = parse_cpu_list(self.cpu_list)
                if observed_affinity is None or set(observed_affinity) != requested_cpus:
                    invalid_reasons.append(
                        f"observed affinity {observed_affinity!r} does not match "
                        f"requested CPUs {sorted(requested_cpus)!r}"
                    )

        if parsed is not None and not any("parse" in reason or "echo" in reason for reason in invalid_reasons):
            parser_status = "VALID"
        canary_status = "PASS" if parsed is not None and not invalid_reasons else "FAIL"
        parser_document = {
            "status": parser_status,
            "expected_echo": expected_echo,
            "observed_fields": sorted(parsed) if parsed is not None else None,
            "errors": invalid_reasons,
        }
        write_json_once(attempt_dir / "parser.json", parser_document)
        write_json_once(
            attempt_dir / "canary.json",
            {
                "status": canary_status,
                "bench_oracle_contract": (
                    "A zero exit means the bench compared the candidate output byte-for-byte "
                    "with the independent reference before warmup and timing."
                ),
                "output_hash": output_hash,
                "generator_version": generator_version,
                "input_hash": input_hash,
                "updates_hash": updates_hash,
                "zero_sample_count": zero_sample_count,
                "work_sha256": work_hash,
                "observed_affinity": observed_affinity,
            },
        )

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
            cell=position.cell,
            n=position.n,
            updates=position.updates,
            pattern=position.pattern,
            order=position.order,
            max_span=position.max_span,
            seed=position.seed,
            samples=samples,
            inner=position.inner,
            status="VALID" if not invalid_reasons else "INVALID",
            invalid_reason="; ".join(invalid_reasons),
            external_interruption=external_interruption,
            returncode=returncode,
            timed_out=timed_out,
            position_ns=position_ns,
            zero_sample_count=zero_sample_count,
            generator_version=generator_version,
            input_hash=input_hash,
            updates_hash=updates_hash,
            output_hash=output_hash,
            work_sha256=work_hash,
            parser_status=parser_status,
            canary_status=canary_status,
            stdout_sha256=stdout_hash,
            stderr_sha256=stderr_hash,
            executable_sha256_before=binary_before,
            executable_sha256_after=binary_after,
            attempt_directory=attempt_dir.relative_to(self.run_dir).as_posix(),
        )
        result_document = {
            "schema_version": SCHEMA_VERSION,
            "started_at": started_at,
            "ended_at": ended_at,
            "started_ns_since_epoch": started_ns,
            "ended_ns_since_epoch": ended_ns,
            "duration_ns": ended_ns - started_ns,
            "attempt": asdict(result),
            "observed_affinity": observed_affinity,
            "spawn_error": spawn_error,
            "retained_file_hashes": {
                "launch.json": sha256_file(attempt_dir / "launch.json"),
                "stdout.jsonl": stdout_hash,
                "stderr.txt": stderr_hash,
                "parser.json": sha256_file(attempt_dir / "parser.json"),
                "canary.json": sha256_file(attempt_dir / "canary.json"),
            },
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


def sensitivity_diagnostics(values: Sequence[float]) -> dict[str, Any]:
    count = len(values)
    midpoint = count // 2
    x_mean = (count + 1) / 2.0
    denominator = sum((index - x_mean) ** 2 for index in range(1, count + 1))
    slope = sum(
        (index - x_mean) * (value - statistics.fmean(values))
        for index, value in enumerate(values, start=1)
    ) / denominator
    pairwise_slopes = sorted(
        (values[right] - values[left]) / (right - left)
        for left in range(count)
        for right in range(left + 1, count)
    )
    positives = sum(value > 0.0 for value in values)
    negatives = sum(value < 0.0 for value in values)
    nonzero = positives + negatives
    tail = min(positives, negatives)
    sign_p = (
        min(
            1.0,
            2.0 * sum(math.comb(nonzero, index) for index in range(tail + 1)) / (2**nonzero),
        )
        if nonzero
        else 1.0
    )
    ordered = sorted(values)
    median_interval_lower_index = 2
    median_interval_upper_index = 9
    median_interval_coverage = 1.0 - 2.0 * sum(
        math.comb(count, index) for index in range(median_interval_lower_index + 1)
    ) / (2**count)
    return {
        "residual_time_order": {
            "ols_log_contrast_slope_per_block": slope,
            "theil_sen_log_contrast_slope_per_block": statistics.median(pairwise_slopes),
            "first_half_geometric_ratio": math.exp(statistics.fmean(values[:midpoint])),
            "second_half_geometric_ratio": math.exp(statistics.fmean(values[midpoint:])),
            "decision_role": "diagnostic only; does not alter the fixed primary interval",
        },
        "nonparametric_sensitivity": {
            "median_log_contrast": statistics.median(values),
            "median_ratio": math.exp(statistics.median(values)),
            "positive_count": positives,
            "negative_count": negatives,
            "zero_count": count - nonzero,
            "exact_two_sided_sign_test_p": sign_p,
            "distribution_free_median_interval": {
                "lower_order_statistic_zero_based_index": median_interval_lower_index,
                "upper_order_statistic_zero_based_index": median_interval_upper_index,
                "coverage": median_interval_coverage,
                "lower_log_ratio": ordered[median_interval_lower_index],
                "upper_log_ratio": ordered[median_interval_upper_index],
                "lower_ratio": math.exp(ordered[median_interval_lower_index]),
                "upper_ratio": math.exp(ordered[median_interval_upper_index]),
                "assumptions": "12 iid continuous complete-block log contrasts",
            },
            "decision_role": "sensitivity diagnostic only; no post-data replacement of the primary estimand",
        },
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


def reliability_summary(executor: HarnessExecutor, phase: str) -> dict[str, Any]:
    phase_attempts = [attempt for attempt in executor.attempts if attempt.phase == phase]
    grouped: dict[tuple[str, str], dict[str, Any]] = {}
    for attempt in phase_attempts:
        key = (attempt.algorithm, attempt.cell)
        row = grouped.setdefault(
            key,
            {
                "candidate": attempt.algorithm,
                "cell": attempt.cell,
                "attempt_count": 0,
                "valid_attempt_count": 0,
                "invalid_attempt_count": 0,
                "timed_out_attempt_count": 0,
                "external_interruption_count": 0,
                "zero_sample_count": 0,
                "failure_reason_counts": {},
            },
        )
        row["attempt_count"] += 1
        row["valid_attempt_count"] += int(attempt.status == "VALID")
        row["invalid_attempt_count"] += int(attempt.status != "VALID")
        row["timed_out_attempt_count"] += int(attempt.timed_out)
        row["external_interruption_count"] += int(attempt.external_interruption)
        row["zero_sample_count"] += attempt.zero_sample_count
        if attempt.invalid_reason:
            for reason in attempt.invalid_reason.split("; "):
                counts = row["failure_reason_counts"]
                counts[reason] = counts.get(reason, 0) + 1
    return {
        "phase": phase,
        "attempt_count": len(phase_attempts),
        "valid_attempt_count": sum(attempt.status == "VALID" for attempt in phase_attempts),
        "invalid_attempt_count": sum(attempt.status != "VALID" for attempt in phase_attempts),
        "zero_sample_count": sum(attempt.zero_sample_count for attempt in phase_attempts),
        "by_candidate_and_cell": [grouped[key] for key in sorted(grouped)],
        "boundary": (
            "Counts include valid, partial, failed, timed-out, interrupted, and retried "
            "attempts. Timing inference still uses only complete valid blocks."
        ),
    }


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
        "reliability": reliability_summary(executor, "pilot"),
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
            interpretation = "IN_PLACE_PRACTICALLY_FASTER_IN_CELL"
        else:
            interpretation = "NO_PRACTICALLY_MEANINGFUL_WINNER_JUSTIFIED"
        results.append(
            {
                "contrast_id": contrast.contrast_id,
                "ratio_orientation": "B_over_in_place",
                "block_log_contrasts": values,
                "analysis": analysis,
                "sensitivity_diagnostics": sensitivity_diagnostics(values),
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
        "reliability": reliability_summary(executor, "main"),
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
            attempt.cell,
            attempt.n,
            attempt.updates,
            attempt.pattern,
            attempt.order,
            attempt.max_span,
            attempt.seed,
            attempt.samples,
            attempt.inner,
        )
        for attempt in aa_attempts
        if attempt.label == "A"
    }
    b_parameters = {
        (
            attempt.algorithm,
            attempt.cell,
            attempt.n,
            attempt.updates,
            attempt.pattern,
            attempt.order,
            attempt.max_span,
            attempt.seed,
            attempt.samples,
            attempt.inner,
        )
        for attempt in aa_attempts
        if attempt.label == "B"
    }
    mechanical_checks = {
        "valid_attempt_count_is_48": len(aa_attempts) == AA_BLOCKS * 4
        and all(attempt.status == "VALID" for attempt in aa_attempts),
        "all_candidates_are_in_place": all(
            attempt.algorithm == "in_place" for attempt in aa_attempts
        ),
        "executable_hashes_identical": len(
            {
                attempt.executable_sha256_before
                for attempt in aa_attempts
            }
            | {attempt.executable_sha256_after for attempt in aa_attempts}
        )
        == 1,
        "output_hashes_identical": len({attempt.output_hash for attempt in aa_attempts}) == 1,
        "generator_version_identical": {
            attempt.generator_version for attempt in aa_attempts
        }
        == {"topic002-fixture-v1"},
        "input_hashes_identical": len({attempt.input_hash for attempt in aa_attempts}) == 1,
        "updates_hashes_identical": len({attempt.updates_hash for attempt in aa_attempts}) == 1,
        "work_hashes_identical": len({attempt.work_sha256 for attempt in aa_attempts}) == 1,
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
            "bonferroni_two_sided_family_interval": analyze_values(
                values, FAMILY_ALPHA / FAMILY_SIZE
            ),
            "maximum_absolute_block_log_contrast": max(abs(value) for value in values),
            "label_by_position_log_time_summaries": label_position_summaries,
            "position_log_time_summaries": position_summaries,
            "interpretation": (
                "Diagnostic only. This run does not prove absence of bias, establish a noise "
                "floor, or alter the primary family."
            ),
        },
        "reliability": reliability_summary(executor, "aa"),
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
        "reliability": reliability_summary(executor, "quick"),
    }


def profile_positions() -> list[Position]:
    specifications = (
        ("reference", "cache_short_shuffled", 65_536, 2_048, "uniform", "shuffled", 32),
        ("in_place", "cache_short_shuffled", 65_536, 2_048, "uniform", "shuffled", 32),
        ("dense", "cache_short_shuffled", 65_536, 2_048, "uniform", "shuffled", 32),
        ("sorted_events", "cache_short_shuffled", 65_536, 2_048, "uniform", "shuffled", 32),
    )
    return [
        Position(
            phase="profile",
            contrast_id=f"profile_{algorithm}__{cell}",
            block_index=index,
            template="P",
            position_index=1,
            label=algorithm,
            algorithm=algorithm,
            cell=cell,
            n=n,
            updates=updates,
            pattern=pattern,
            order=order,
            max_span=max_span,
            seed=ASSIGNMENT_SEED,
            samples=3,
            inner=1,
        )
        for index, (algorithm, cell, n, updates, pattern, order, max_span) in enumerate(
            specifications, start=1
        )
    ]


TIMED_LOOP_SYMBOL_PATHS = ("range_updates::main", "range_updates::run")
CLOCK_SYMBOL_PATHS = ("Instant::now", "Instant::elapsed")
CLOCK_C_SYMBOLS = ("mach_absolute_time", "clock_gettime")


def symbol_fragment_variants(path: str) -> tuple[bytes, ...]:
    """Return the demangled and legacy-mangled spellings of a symbol path.

    `objdump --demangle` prints `range_updates::main`, while `otool -tvV` has no
    Rust demangler and prints the mangled `_ZN13range_updates4main17h...E`. A
    bare identifier survives mangling as a substring, but a `::`-separated path
    does not, because the mangled form encodes each component as its length
    followed by its text.
    """
    components = path.split("::")
    mangled = "".join(f"{len(component)}{component}" for component in components)
    return (path.encode("ascii"), mangled.encode("ascii"))


def is_disassembly_symbol_header(line: bytes) -> bool:
    """Report whether a disassembly line opens a function.

    `objdump` writes `<address> <symbol>:` and indents instructions; `otool`
    writes `_symbol:` at column 0 and separates an instruction's address from
    its mnemonic with a tab. A line that starts at column 0, ends with a colon,
    and holds no tab is a header in both formats.
    """
    return bool(line) and line.endswith(b":") and line[:1] not in (b" ", b"\t") and b"\t" not in line


def find_symbol_fragment(stdout: bytes, paths: Sequence[str]) -> tuple[str, bytes] | None:
    for path in paths:
        for variant in symbol_fragment_variants(path):
            if variant in stdout:
                return path, variant
    return None


def disassembly_symbol_region(stdout: bytes, fragment: bytes) -> bytes | None:
    """Return one symbol's disassembly, from its header to the next header."""
    start = stdout.find(fragment)
    if start < 0:
        return None
    header_start = stdout.rfind(b"\n", 0, start) + 1
    cursor = stdout.find(b"\n", start)
    while cursor != -1:
        line_end = stdout.find(b"\n", cursor + 1)
        line = stdout[cursor + 1 : line_end if line_end != -1 else len(stdout)]
        if is_disassembly_symbol_header(line):
            return stdout[header_start : cursor + 1]
        if line_end == -1:
            break
        cursor = line_end
    return stdout[header_start:]


def collect_disassembly(
    run_dir: Path,
    binary: Path,
    environment: dict[str, str],
    relay: ExternalSignalRelay | None,
) -> dict[str, Any]:
    if sys.platform == "darwin" and Path("/usr/bin/otool").is_file():
        argv = ["/usr/bin/otool", "-tvV", str(binary)]
    else:
        selected = shutil.which("llvm-objdump") or shutil.which("objdump")
        if selected is None:
            return {"status": "UNAVAILABLE", "reason": "no supported disassembler found"}
        argv = [str(Path(selected).resolve()), "-d", "--demangle", str(binary)]
    returncode, stdout, stderr, timed_out = run_capture_group(
        argv,
        cwd=REPO_ROOT,
        env=environment,
        timeout=120.0,
        relay=relay,
    )
    write_bytes_once(run_dir / "profiles" / "linked-image-assembly.stdout", stdout)
    write_bytes_once(run_dir / "profiles" / "linked-image-assembly.stderr", stderr)
    symbol_fragments = (
        b"apply_reference",
        b"apply_dense_sidecar",
        b"apply_in_place_difference",
        b"apply_sorted_events",
    )
    symbol_checks = {
        fragment.decode("ascii"): fragment in stdout for fragment in symbol_fragments
    }
    timed_loop_match = find_symbol_fragment(stdout, TIMED_LOOP_SYMBOL_PATHS)
    timed_loop_region = (
        disassembly_symbol_region(stdout, timed_loop_match[1])
        if timed_loop_match is not None
        else None
    )
    clock_symbols = [
        path
        for path in CLOCK_SYMBOL_PATHS
        if any(variant in stdout for variant in symbol_fragment_variants(path))
    ]
    clock_symbols.extend(name for name in CLOCK_C_SYMBOLS if name.encode("ascii") in stdout)
    retention_checks = {
        "timed_loop_symbol_present": timed_loop_match is not None,
        "timed_loop_region_isolated": timed_loop_region is not None,
        "clock_symbols_present": clock_symbols,
    }
    complete = (
        returncode == 0
        and not timed_out
        and all(symbol_checks.values())
        and retention_checks["timed_loop_symbol_present"]
        and retention_checks["timed_loop_region_isolated"]
        and bool(retention_checks["clock_symbols_present"])
    )
    return {
        "status": "COMPLETE" if complete else "FAILED",
        "argv": argv,
        "cwd": str(REPO_ROOT),
        "environment": environment,
        "returncode": returncode,
        "timed_out": timed_out,
        "binary_sha256": sha256_file(binary),
        "stdout_sha256": sha256_bytes(stdout),
        "stderr_sha256": sha256_bytes(stderr),
        "candidate_symbol_fragments_present": symbol_checks,
        "retention_checks": retention_checks,
        "timed_loop_symbol": (timed_loop_match[0] if timed_loop_match is not None else None),
        "timed_loop_symbol_spelling": (
            timed_loop_match[1].decode("ascii") if timed_loop_match is not None else None
        ),
        "timed_loop_region_sha256": (
            sha256_bytes(timed_loop_region) if timed_loop_region is not None else None
        ),
        "consumer_boundary": (
            "This check establishes retention only: all four candidate symbols, the timed "
            "loop's symbol and its isolated region, and at least one clock symbol are "
            "present in the linked image, and the region's hash is retained so a later run "
            "can detect that those instructions changed. It does not establish the ordering "
            "`BENCHMARK.md` asks for. In this image the candidate is reached through a "
            "function pointer and the clock reads are inlined, so the timed loop's region "
            "names neither a candidate symbol nor a clock symbol as a call operand; a "
            "predicate over the whole disassembly would instead be satisfied by unrelated "
            "indirect calls and clock uses elsewhere in the image. `black_box` emits no "
            "instructions. The timed call site, its clock ordering, and the result consumer "
            "are therefore pinned by the retained source-tree and benchmark-source hashes, "
            "not by this disassembly."
        ),
    }


def profile_target_output_check(
    stdout: bytes,
    position: Position,
    expected_identity: dict[str, str],
) -> dict[str, Any]:
    lines = [
        line
        for line in stdout.decode("utf-8", errors="replace").splitlines()
        if line.strip()
    ]
    checks: dict[str, bool] = {"exactly_one_nonempty_line": len(lines) == 1}
    parsed: dict[str, Any] | None = None
    if len(lines) == 1:
        try:
            candidate = json.loads(lines[0])
            if isinstance(candidate, dict):
                parsed = candidate
        except json.JSONDecodeError:
            pass
    checks["json_object"] = parsed is not None
    if parsed is None:
        return {"status": "MISMATCH", "checks": checks}

    expected_echo = {
        "schema_version": SCHEMA_VERSION,
        "candidate": position.algorithm,
        "workload": position.cell,
        "pattern": position.pattern,
        "order": position.order,
        "len": position.n,
        "updates": position.updates,
        "max_span": position.max_span,
        "seed": position.seed,
        "warmup": DEFAULT_WARMUPS,
        "samples": position.samples,
        "inner": position.inner,
    }
    expected_fields = set(expected_echo) | {
        "generator_version",
        "input_hash",
        "updates_hash",
        "sample_ns",
        "output_hash",
        "work",
    }
    checks["exact_schema_fields"] = set(parsed) == expected_fields
    checks["position_echo"] = all(parsed.get(key) == value for key, value in expected_echo.items())
    checks["generator_version"] = (
        parsed.get("generator_version") == expected_identity["generator_version"]
    )
    checks["input_hash"] = parsed.get("input_hash") == expected_identity["input_hash"]
    checks["updates_hash"] = parsed.get("updates_hash") == expected_identity["updates_hash"]
    checks["output_hash"] = parsed.get("output_hash") == expected_identity["output_hash"]
    sample_errors, position_ns, _ = validate_timing_samples(
        parsed.get("sample_ns"), position.samples
    )
    checks["timing_samples"] = not sample_errors and position_ns is not None
    work = parsed.get("work")
    checks["work_shape"] = not validate_work_shape(work)
    checks["work_counts"] = not validate_work_counts(position, work)
    if isinstance(work, dict):
        work_hash = sha256_bytes(
            json.dumps(work, separators=(",", ":"), sort_keys=True).encode("utf-8")
        )
    else:
        work_hash = ""
    checks["work_hash"] = work_hash == expected_identity["work_sha256"]
    observed = {
        "generator_version": parsed.get("generator_version"),
        "input_hash": parsed.get("input_hash"),
        "updates_hash": parsed.get("updates_hash"),
        "output_hash": parsed.get("output_hash"),
        "work_sha256": work_hash,
    }
    return {
        "status": "MATCH" if all(checks.values()) else "MISMATCH",
        "checks": checks,
        "observed_identity": observed,
        "expected_identity": expected_identity,
    }


def macos_time_denial_check(stderr: bytes) -> dict[str, Any]:
    try:
        lines = stderr.decode("utf-8", errors="strict").splitlines()
    except UnicodeDecodeError:
        lines = []
    expected_denial = "time: sysctl kern.clockrate: Operation not permitted"
    fields = lines[0].split() if len(lines) == 2 else []
    labels_match = len(fields) == 6 and fields[1::2] == ["real", "user", "sys"]
    numeric_fields_match = False
    if labels_match:
        try:
            values = [float(fields[index]) for index in (0, 2, 4)]
            numeric_fields_match = all(math.isfinite(value) and value >= 0.0 for value in values)
        except ValueError:
            numeric_fields_match = False
    checks = {
        "exactly_two_lines": len(lines) == 2,
        "time_summary_shape": labels_match and numeric_fields_match,
        "exact_permission_denial": len(lines) == 2 and lines[1] == expected_denial,
        "no_other_text": len(lines) == 2,
    }
    return {
        "status": "MATCH" if all(checks.values()) else "MISMATCH",
        "checks": checks,
        "expected_denial": expected_denial,
        "observed_lines": lines,
    }


def parse_perf_stat_counters(stderr: bytes) -> dict[str, Any]:
    """Classify each requested `perf stat -x,` counter row.

    `perf stat` reports the target's exit status, so a zero return code says
    nothing about whether the PMU delivered the requested events: an
    unsupported or never-scheduled event still prints a row whose value field
    is `<not supported>` or `<not counted>`. Virtualized hosts and restricted
    `perf_event_paranoid` settings both produce that shape.

    Column order for `-x` output is value, unit, event, counter run time in
    nanoseconds, then the percentage of measurement time the counter ran.
    """
    rows: dict[str, dict[str, Any]] = {}
    malformed: list[str] = []
    for raw_line in stderr.decode("utf-8", errors="replace").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        fields = line.split(PERF_STAT_SEPARATOR)
        if len(fields) < 3:
            continue
        value_text, _unit, event_text = fields[0].strip(), fields[1].strip(), fields[2].strip()
        event = event_text.split(":", 1)[0]
        if event not in PERF_STAT_EVENTS:
            continue
        run_time_ns: int | None = None
        running_percent: float | None = None
        if len(fields) >= 4 and fields[3].strip():
            try:
                run_time_ns = int(fields[3].strip())
            except ValueError:
                malformed.append(f"{event}: unparsable counter run time {fields[3]!r}")
        if len(fields) >= 5 and fields[4].strip():
            try:
                running_percent = float(fields[4].strip())
            except ValueError:
                malformed.append(f"{event}: unparsable running percentage {fields[4]!r}")
        counted: bool
        value: int | None = None
        if value_text.startswith("<"):
            counted = False
        else:
            counted = True
            try:
                value = int(float(value_text))
            except ValueError:
                counted = False
                malformed.append(f"{event}: unparsable counter value {value_text!r}")
        rows[event] = {
            "event": event,
            "raw_value": value_text,
            "value": value,
            "counted": counted,
            "run_time_ns": run_time_ns,
            "running_percent": running_percent,
        }

    missing = [event for event in PERF_STAT_EVENTS if event not in rows]
    unsupported = sorted(
        event for event, row in rows.items() if row["raw_value"] == "<not supported>"
    )
    not_counted = sorted(
        event
        for event, row in rows.items()
        if not row["counted"] and row["raw_value"] != "<not supported>"
    )
    scaled = sorted(
        event
        for event, row in rows.items()
        if row["counted"]
        and row["running_percent"] is not None
        and row["running_percent"] < 100.0
    )
    zero_running = sorted(
        event
        for event, row in rows.items()
        if row["counted"] and (row["run_time_ns"] == 0 or row["running_percent"] == 0.0)
    )
    complete = not (missing or unsupported or not_counted or zero_running or malformed)
    return {
        "requested_events": list(PERF_STAT_EVENTS),
        "counters": [rows[event] for event in PERF_STAT_EVENTS if event in rows],
        "missing_events": missing,
        "unsupported_events": unsupported,
        "not_counted_events": not_counted,
        "multiplexed_events": scaled,
        "zero_running_events": zero_running,
        "malformed_rows": malformed,
        "all_requested_counted": complete,
    }


def classify_dynamic_profile(
    tool_name: str,
    returncode: int,
    timed_out: bool,
    stdout: bytes,
    stderr: bytes,
    position: Position,
    expected_identity: dict[str, str],
    binary_unchanged: bool,
    platform_name: str,
) -> tuple[str, str | None, dict[str, Any], dict[str, Any], dict[str, Any] | None]:
    target_output = profile_target_output_check(stdout, position, expected_identity)
    denial = macos_time_denial_check(stderr)
    counters = parse_perf_stat_counters(stderr) if tool_name == "perf-stat" else None
    if timed_out:
        return "FAILED", "dynamic profile timed out", target_output, denial, counters
    if returncode == 0 and binary_unchanged and target_output["status"] == "MATCH":
        if counters is None or counters["all_requested_counted"]:
            return "COMPLETE", None, target_output, denial, counters
        if counters["malformed_rows"]:
            return (
                "FAILED",
                (
                    "perf stat emitted rows this runner cannot parse: "
                    f"{'; '.join(counters['malformed_rows'])}"
                ),
                target_output,
                denial,
                counters,
            )
        gaps = []
        for label in ("missing", "unsupported", "not_counted", "zero_running"):
            events = counters[f"{label}_events"]
            if events:
                gaps.append(f"{label}={sorted(events)}")
        return (
            "UNAVAILABLE",
            (
                "the target completed and its output was verified, but perf stat did not "
                f"deliver every requested counter ({', '.join(gaps)}); a zero perf exit "
                "status reports the target, not counter availability"
            ),
            target_output,
            denial,
            counters,
        )
    if (
        platform_name == "darwin"
        and tool_name == "time-l"
        and returncode == 1
        and binary_unchanged
        and target_output["status"] == "MATCH"
        and denial["status"] == "MATCH"
    ):
        return (
            "UNAVAILABLE",
            (
                "macOS /usr/bin/time -l could not read kern.clockrate in this execution "
                "environment; the separately retained target output completed, but the "
                "partial time output is not resource evidence"
            ),
            target_output,
            denial,
            counters,
        )

    return (
        "FAILED",
        f"dynamic profile tool exited with code {returncode} or failed integrity checks",
        target_output,
        denial,
        counters,
    )


def collect_dynamic_profile(
    run_dir: Path,
    executor: HarnessExecutor,
    position: Position,
    canary: AttemptResult,
) -> dict[str, Any]:
    harness_argv = executor.harness_argv(position, DEFAULT_WARMUPS, position.samples)
    if sys.platform.startswith("linux") and shutil.which("perf") is not None:
        tool = str(Path(shutil.which("perf") or "perf").resolve())
        argv = [
            tool,
            "stat",
            "-x",
            PERF_STAT_SEPARATOR,
            "-e",
            ",".join(PERF_STAT_EVENTS),
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
    returncode, stdout, stderr, timed_out = run_capture_group(
        argv,
        cwd=REPO_ROOT,
        env=executor.environment,
        timeout=executor.timeout_seconds,
        relay=executor.relay,
        cpus=executor.cpus,
    )
    write_bytes_once(profile_dir / "stdout.jsonl", stdout)
    write_bytes_once(profile_dir / "stderr.txt", stderr)
    binary_after = sha256_file(executor.binary)
    expected_identity = {
        "generator_version": canary.generator_version,
        "input_hash": canary.input_hash,
        "updates_hash": canary.updates_hash,
        "output_hash": canary.output_hash,
        "work_sha256": canary.work_sha256,
    }
    status, unavailable_or_failure_reason, target_output_check, denial_check, counter_check = (
        classify_dynamic_profile(
        tool_name,
        returncode,
        timed_out,
        stdout,
        stderr,
        position,
        expected_identity,
        binary_after == executor.binary_hash,
        sys.platform,
        )
    )
    record = {
        "status": status,
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
        "binary_sha256_after": binary_after,
        "binary_hash_unchanged": binary_after == executor.binary_hash,
        "stdout_sha256": sha256_bytes(stdout),
        "stderr_sha256": sha256_bytes(stderr),
        "target_output_check": target_output_check,
        "macos_time_denial_check": denial_check,
        "counter_check": counter_check,
        "counter_scope": "WHOLE_PROCESS",
        "counter_scope_boundary": (
            "The tool wraps the whole harness process, so every counter and resource total "
            "covers process startup, fixture generation, the eager canary, hashing, the "
            "candidate-specific work-count pass, warmup, the timed calls, and output "
            "formatting. The timed calls are one part of that total. These numbers are "
            "whole-process totals for the recorded argv; they do not attribute a cache, "
            "branch, or instruction mechanism to the candidate, and they are not comparable "
            "to the per-call timing distribution. Scoping counters to the candidate "
            "interval requires the harness to gate collection itself, which this runner "
            "does not do."
        ),
        "unavailable_or_failure_reason": unavailable_or_failure_reason,
        "failure_boundary": (
            "A failed target, timeout, malformed completed-target output, or unrecognized "
            "tool error is FAILED. A recognized host permission denial after verified target "
            "completion is UNAVAILABLE, as is a completed target whose requested counters "
            "were unsupported, never counted, or absent. Neither status is timing or "
            "mechanism evidence."
        ),
    }
    write_json_once(profile_dir / "profile.json", record)
    return record


def run_profile_phase(executor: HarnessExecutor) -> dict[str, Any]:
    positions = profile_positions()
    measured_attempts = []
    dynamic_profiles = []
    for position in positions:
        result = executor.run_position(position, DEFAULT_WARMUPS, position.samples)
        measured_attempts.append(result.attempt_id)
        if result.status != "VALID":
            raise ExperimentIncomplete(f"profile canary failed: {result.attempt_id}")
        dynamic_profiles.append(
            collect_dynamic_profile(executor.run_dir, executor, position, result)
        )
    disassembly = collect_disassembly(
        executor.run_dir, executor.binary, executor.environment, executor.relay
    )
    dynamic_statuses = [record["status"] for record in dynamic_profiles]
    complete = disassembly.get("status") == "COMPLETE" and all(
        status in ("COMPLETE", "UNAVAILABLE") for status in dynamic_statuses
    )
    return {
        "status": "COMPLETE" if complete else "INCOMPLETE",
        "measured_attempts": measured_attempts,
        "dynamic_profiles": dynamic_profiles,
        "linked_image_disassembly": disassembly,
        "reliability": reliability_summary(executor, "profile"),
        "mechanism_boundary": (
            "Counters and resource totals are whole-process figures that include fixture "
            "generation, the eager canary, hashing, work-count passes, warmup, and output "
            "formatting alongside the timed calls, so they do not establish a cache, "
            "branch, allocation, or bandwidth mechanism for a candidate. Unsupported, "
            "never-counted, or permission-denied counters leave that mechanism inferred. "
            "The disassembly establishes symbol and region retention, not the ordering of "
            "the timed call, its clock reads, and its result consumer."
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
                phase_warmups(phase),
                phase_samples(phase, contrast),
            )


def execute_phase(executor: HarnessExecutor, phase: str) -> dict[str, Any]:
    if phase in ("quick", "pilot", "main", "aa"):
        run_block_phase(executor, phase)
    elif phase == "profile":
        result = run_profile_phase(executor)
        write_json_once(executor.run_dir / "analysis" / "profile.json", result)
        if result.get("status") != "COMPLETE":
            raise ExperimentIncomplete(
                "profile requires valid canaries, linked-image assembly, and no failed "
                "supported counter or resource collection"
            )
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
    write_json_once(executor.run_dir / "analysis" / f"{phase}.json", result)
    if result.get("status") != "COMPLETE":
        raise ExperimentIncomplete(
            f"{phase} analysis reported status {result.get('status')!r}"
        )
    return result


def phase_sequence(requested: str) -> list[str]:
    if requested in ("all", "plan"):
        return ["pilot", "main", "aa", "profile"]
    if requested == "self-check":
        return []
    return [requested]


def protocol_document(
    requested_phase: str, rustflags: str, timeout: float, cpu_list: str | None
) -> dict[str, Any]:
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
        "quick_warmups_per_position": QUICK_WARMUPS,
        "quick_timed_subsamples_per_position": QUICK_SAMPLES,
        "timed_subsamples_per_position": {
            contrast.contrast_id: contrast.samples for contrast in CONTRASTS
        },
        "timeout_seconds": timeout,
        "rustflags": rustflags,
        "requested_cpu_list": cpu_list,
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


def create_run_directory(requested: Path) -> Path:
    if not requested.is_absolute():
        raise ValueError("--output-dir must be an absolute path")
    run_dir = requested.resolve()
    if run_dir == REPO_ROOT or run_dir.is_relative_to(REPO_ROOT):
        raise ValueError("--output-dir must be outside the repository")
    if run_dir.exists():
        raise ValueError("--output-dir must not already exist")
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
    source_records = source_tree_manifest()
    source_record_errors = validate_source_snapshot_records(source_records)
    if source_record_errors:
        errors.extend(
            f"source snapshot precondition failed: {error}" for error in source_record_errors
        )
    if len(source_records) == 0 or len(source_tree_digest(source_records)) != 64:
        errors.append("source snapshot plan is empty or has an invalid digest")
    host = host_metadata()
    if "frequency_policy" not in host or host["frequency_policy"].get("status") not in (
        "COMPLETE",
        "UNAVAILABLE",
    ):
        errors.append("host metadata is missing a structured frequency-policy result")
    hardware_identity = host.get("hardware_identity", {})
    expected_hardware_command = [
        "/usr/sbin/system_profiler",
        "SPHardwareDataType",
        "-json",
    ]
    if hardware_identity.get("command") != expected_hardware_command:
        errors.append("host hardware identity command is not the frozen sanitized probe")
    if sys.platform == "darwin":
        hardware_fields = hardware_identity.get("fields", {})
        if hardware_identity.get("status") != "COMPLETE":
            errors.append(f"Darwin exact hardware identity is unavailable: {hardware_identity}")
        if not isinstance(hardware_fields, dict) or set(hardware_fields) != set(
            DARWIN_HARDWARE_FIELDS
        ):
            errors.append("Darwin hardware identity does not contain exactly the allowed fields")
        elif any(
            not isinstance(hardware_fields[field], (str, int))
            or isinstance(hardware_fields[field], bool)
            or (isinstance(hardware_fields[field], str) and not hardware_fields[field].strip())
            for field in DARWIN_HARDWARE_FIELDS
        ):
            errors.append("Darwin hardware identity contains an empty or invalid allowed field")
        else:
            generic_cpu_names = {"arm", "arm64", "aarch64", "unknown", ""}
            chip_type = str(hardware_fields["chip_type"]).strip()
            if chip_type.lower() in generic_cpu_names or host.get("cpu_description") != chip_type:
                errors.append("Darwin cpu_description does not contain the exact chip model")
        serialized_hardware_identity = json.dumps(hardware_identity, sort_keys=True).lower()
        if any(
            forbidden in serialized_hardware_identity
            for forbidden in ("serial_number", "platform_uuid", "provisioning_udid")
        ):
            errors.append("Darwin hardware identity retained a forbidden device identifier")
    elif hardware_identity.get("status") != "UNAVAILABLE":
        errors.append("non-Darwin system_profiler identity must be explicitly unavailable")
    allocator = source_allocator_boundary()
    if "source_defines_global_allocator" not in allocator or "boundary" not in allocator:
        errors.append("allocator source-boundary metadata is incomplete")
    if allocator.get("source_defines_global_allocator"):
        errors.append("topic source unexpectedly declares #[global_allocator]")
    accepted_errors, accepted_median, accepted_zeros = validate_timing_samples([0, 1, 2], 3)
    if accepted_errors or accepted_median != 1 or accepted_zeros != 1:
        errors.append(
            "nonnegative timing sample acceptance check failed: "
            f"{accepted_errors}, {accepted_median}, {accepted_zeros}"
        )
    for rejected_samples in ([0, 0, 0], [0, 0, 1]):
        rejected_errors, rejected_median, _ = validate_timing_samples(rejected_samples, 3)
        if not rejected_errors or rejected_median != 0:
            errors.append(
                "zero-median timing sample rejection check failed: "
                f"{rejected_samples}, {rejected_errors}, {rejected_median}"
            )
    profile_probe = profile_positions()[0]
    profile_fixture = fixture_metrics(
        profile_probe.n,
        profile_probe.updates,
        profile_probe.pattern,
        profile_probe.order,
        profile_probe.max_span,
        profile_probe.seed,
    )
    profile_work = {
        "validation_checks": profile_probe.updates,
        "base_reads": profile_probe.n,
        "base_writes": profile_probe.n,
        "range_element_updates": profile_fixture["range_element_updates"],
        "boundary_updates": 0,
        "difference_steps": 0,
        "scan_steps": 0,
        "event_records": 0,
        "sort_comparisons_count_pass": 0,
        "vec_constructions": 1,
        "allocated_scalar_slots": profile_probe.n,
    }
    profile_identity = {
        "generator_version": "topic002-fixture-v1",
        "input_hash": str(profile_fixture["input_hash"]),
        "updates_hash": str(profile_fixture["updates_hash"]),
        "output_hash": "0123456789abcdef",
        "work_sha256": sha256_bytes(
            json.dumps(profile_work, separators=(",", ":"), sort_keys=True).encode("utf-8")
        ),
    }
    profile_payload = {
        "schema_version": SCHEMA_VERSION,
        "candidate": profile_probe.algorithm,
        "workload": profile_probe.cell,
        "pattern": profile_probe.pattern,
        "order": profile_probe.order,
        "len": profile_probe.n,
        "updates": profile_probe.updates,
        "max_span": profile_probe.max_span,
        "seed": profile_probe.seed,
        "warmup": DEFAULT_WARMUPS,
        "samples": profile_probe.samples,
        "inner": profile_probe.inner,
        "generator_version": profile_identity["generator_version"],
        "input_hash": profile_identity["input_hash"],
        "updates_hash": profile_identity["updates_hash"],
        "sample_ns": [1, 2, 3],
        "output_hash": profile_identity["output_hash"],
        "work": profile_work,
    }
    profile_stdout = (json.dumps(profile_payload, separators=(",", ":")) + "\n").encode()
    profile_denial = (
        "        0.02 real         0.00 user         0.00 sys\n"
        "time: sysctl kern.clockrate: Operation not permitted\n"
    ).encode()
    profile_match = classify_dynamic_profile(
        "time-l",
        1,
        False,
        profile_stdout,
        profile_denial,
        profile_probe,
        profile_identity,
        True,
        "darwin",
    )
    profile_near_misses = {
        "extra_stderr": classify_dynamic_profile(
            "time-l",
            1,
            False,
            profile_stdout,
            profile_denial + b"unexpected\n",
            profile_probe,
            profile_identity,
            True,
            "darwin",
        )[0],
        "changed_output_hash": classify_dynamic_profile(
            "time-l",
            1,
            False,
            profile_stdout.replace(b"0123456789abcdef", b"fedcba9876543210"),
            profile_denial,
            profile_probe,
            profile_identity,
            True,
            "darwin",
        )[0],
        "changed_binary_hash": classify_dynamic_profile(
            "time-l",
            1,
            False,
            profile_stdout,
            profile_denial,
            profile_probe,
            profile_identity,
            False,
            "darwin",
        )[0],
        "non_darwin_platform": classify_dynamic_profile(
            "time-l",
            1,
            False,
            profile_stdout,
            profile_denial,
            profile_probe,
            profile_identity,
            True,
            "linux",
        )[0],
    }
    if profile_match[0] != "UNAVAILABLE":
        errors.append(f"exact macOS time denial classification failed: {profile_match[0]}")
    if any(status != "FAILED" for status in profile_near_misses.values()):
        errors.append(f"macOS time denial near-miss classification failed: {profile_near_misses}")
    if balanced_templates(4, "self-check").count("ABBA") != 2:
        errors.append("four-block template assignment is unbalanced")
    if balanced_templates(12, "self-check").count("ABBA") != 6:
        errors.append("12-block template assignment is unbalanced")
    expected_position_count = FAMILY_SIZE * (PILOT_BLOCKS + MAIN_BLOCKS) * 4 + AA_BLOCKS * 4
    actual_position_count = len(planned_schedule(["pilot", "main", "aa"]))
    if actual_position_count != expected_position_count:
        errors.append(
            f"planned position count mismatch: expected {expected_position_count}, got {actual_position_count}"
        )
    known_quantile = student_t_quantile(0.975, 10)
    if abs(known_quantile - 2.228_138_852) > 1.0e-8:
        errors.append(f"Student t quantile check failed: {known_quantile}")
    median_probe = sensitivity_diagnostics([float(index) for index in range(12)])
    median_interval = median_probe["nonparametric_sensitivity"][
        "distribution_free_median_interval"
    ]
    if (
        median_interval["lower_order_statistic_zero_based_index"] != 2
        or median_interval["upper_order_statistic_zero_based_index"] != 9
        or abs(median_interval["coverage"] - 0.961_425_781_25) > 1.0e-15
    ):
        errors.append(f"distribution-free median interval check failed: {median_interval}")
    return {
        "status": "PASS" if not errors else "FAIL",
        "errors": errors,
        "primary_contrast_count": len(CONTRASTS),
        "full_nonprofile_position_count": actual_position_count,
        "student_t_0.975_df10": known_quantile,
        "tool_identities": tool_identities,
        "hardware_identity": hardware_identity,
        "frequency_policy": host["frequency_policy"],
        "allocator": allocator,
        "source_snapshot_plan": {
            "file_count": len(source_records),
            "source_tree_sha256": source_tree_digest(source_records),
            "errors": source_record_errors,
        },
        "timing_sample_checks": {
            "accepted_nonnegative_samples": [0, 1, 2],
            "accepted_median_ns": accepted_median,
            "accepted_zero_sample_count": accepted_zeros,
            "rejected_zero_median_samples": [[0, 0, 0], [0, 0, 1]],
        },
        "profile_unavailable_classifier": {
            "exact_match_status": profile_match[0],
            "target_output_check": profile_match[2],
            "denial_check": profile_match[3],
            "near_miss_statuses": profile_near_misses,
        },
    }


def parse_cli() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "phase",
        choices=("plan", "self-check", "quick", "pilot", "main", "aa", "profile", "all"),
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        help=(
            "required for collection phases: absolute, initially nonexistent, and outside "
            "the repository"
        ),
    )
    parser.add_argument("--timeout-seconds", type=float, default=DEFAULT_TIMEOUT_SECONDS)
    parser.add_argument("--rustflags", default="-C target-cpu=native")
    parser.add_argument(
        "--cpu-list",
        help="Linux CPU list to pin the harness to; the runner verifies it is within the granted affinity",
    )
    return parser.parse_args()


def run_experiment(args: argparse.Namespace) -> int:
    if not math.isfinite(args.timeout_seconds) or args.timeout_seconds <= 0.0:
        raise ValueError("--timeout-seconds must be a finite value greater than zero")
    protocol = protocol_document(args.phase, args.rustflags, args.timeout_seconds, args.cpu_list)
    if args.phase == "plan":
        print(json.dumps(protocol, indent=2, sort_keys=True))
        return 0
    if args.phase == "self-check":
        result = self_check()
        print(json.dumps(result, indent=2, sort_keys=True))
        return 0 if result["status"] == "PASS" else 1
    if args.output_dir is None:
        raise ValueError("--output-dir is required for collection phases")
    run_dir = create_run_directory(args.output_dir)
    relay = ExternalSignalRelay()
    relay.install()
    status: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "status": "STARTED",
        "started_at": utc_now(),
        "requested_phase": args.phase,
        "completed_phases": [],
    }
    failure: BaseException | None = None
    executor: HarnessExecutor | None = None
    try:
        replace_json(run_dir / "run-status.json", status)
        write_json_once(run_dir / "protocol.json", protocol)
        initial_source = source_tree_manifest(run_dir)
        write_json_once(run_dir / "metadata" / "source-tree-before.json", initial_source)
        initial_source_digest = source_tree_digest(initial_source)
        initial_snapshot = capture_source_snapshot(run_dir, initial_source)
        write_json_once(
            run_dir / "metadata" / "source-snapshot-initial.json", initial_snapshot
        )
        build_env = safe_base_environment()
        relay.raise_if_pending()
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
        final_snapshot = verify_source_snapshot(run_dir, initial_source)
        write_json_once(
            run_dir / "metadata" / "source-snapshot-final.json", final_snapshot
        )
        if final_snapshot["status"] != "COMPLETE":
            raise ExperimentIncomplete(
                f"source snapshot changed or failed verification: {final_snapshot['errors']}"
            )
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
        if executor is not None:
            status["attempt_count"] = len(executor.attempts)
            status["block_record_count"] = len(executor.blocks)
            attempted_phases = sorted({attempt.phase for attempt in executor.attempts})
            write_json_once(
                run_dir / "analysis" / "reliability-incomplete.json",
                {
                    "status": "INCOMPLETE",
                    "phases": {
                        phase_name: reliability_summary(executor, phase_name)
                        for phase_name in attempted_phases
                    },
                },
            )
        write_text_once(run_dir / "failure.txt", traceback.format_exc())
    try:
        try:
            replace_json(run_dir / "run-status.json", status)
            manifest_digest, manifest_count = checksum_manifest(run_dir)
            if failure is None:
                relay.raise_if_pending()
        except BaseException as error:
            if failure is not None:
                raise
            failure = error
            status["status"] = "INCOMPLETE"
            status["ended_at"] = utc_now()
            status["failure_type"] = type(error).__name__
            status["failure"] = str(error)
            write_text_once(run_dir / "failure.txt", traceback.format_exc())
            replace_json(run_dir / "run-status.json", status)
            (run_dir / "manifest.sha256").unlink(missing_ok=True)
            manifest_digest, manifest_count = checksum_manifest(run_dir)
    finally:
        relay.restore()
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
