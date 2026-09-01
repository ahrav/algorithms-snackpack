#!/usr/bin/env python3
"""Collect the frozen Topic 003 bitmap experiment and retain every attempt."""

from __future__ import annotations

import sys

if sys.version_info < (3, 11):
    raise SystemExit("run_experiment.py requires Python 3.11 or newer")

import argparse
import csv
import hashlib
import json
import math
import os
import platform
import random
import shutil
import statistics
import subprocess
import time
import traceback
from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any, Iterable, Sequence


SCHEMA_VERSION = 1
ASSIGNMENT_SEED = 2_026_083_103
FAMILY_ALPHA = 0.05
FAMILY_SIZE = 21
PRACTICAL_RATIO = 1.05
PILOT_BLOCKS = 4
MAIN_BLOCKS = 12
AA_BLOCKS = 12
REFERENCE_ATTEMPTS_PER_CELL = 4
PRIMARY_COMPONENT = "total"
DESCRIPTIVE_COMPONENTS = ("build", "contains", "intersection")
SAMPLE_FIELDS = (
    "total_sample_ns",
    "build_sample_ns",
    "contains_sample_ns",
    "intersection_sample_ns",
)
DEFAULT_WARMUPS = 1
DEFAULT_SAMPLES = 3
DEFAULT_TIMEOUT_SECONDS = 180.0
GENERATOR_VERSION = "topic003-density-fixtures-v1"
MAX_ATTEMPTS_PER_POSITION = 1
PROFILE_REPETITIONS = 64
PILOT_SENSITIVITY_MULTIPLIERS = (0.5, 1.0, 1.5, 2.0)
EXPECTED_LINKED_SYMBOL_SUBSTRINGS = (
    "topic003_reference_workload",
    "topic003_sorted_workload",
    "topic003_dense_workload",
    "topic003_adaptive_workload",
    "topic003_timed_loop",
    "topic003_consume_result",
    "topic003_clock_now",
)

SCRIPT = Path(__file__).resolve()
TOPIC_DIR = SCRIPT.parents[1]
REPO_ROOT = SCRIPT.parents[3]
BENCH_SOURCE = TOPIC_DIR / "benches" / "bitmap_sets.rs"

CANDIDATES = ("reference", "sorted", "dense", "adaptive")
OPTIMIZED = ("sorted", "dense", "adaptive")
PAIR_SPECS = (
    ("sorted", "dense"),
    ("sorted", "adaptive"),
    ("dense", "adaptive"),
)


@dataclass(frozen=True)
class Cell:
    name: str
    universe: int
    unique_a: int
    unique_b: int
    overlap: int | None
    probes: int
    hit_percent: int
    intersections: int
    build_repetitions: int
    shape: str


CELLS = (
    Cell("tiny_sparse_shuffled", 1_024, 24, 24, 8, 256, 50, 256, 128, "tiny_duplicate_once"),
    Cell("wide_sparse_low_overlap", 1 << 24, 4_096, 4_096, 128, 4_096, 25, 64, 4, "uniform_256_chunks_duplicate_10pct"),
    Cell("skewed_sparse", 1 << 24, 64, 65_536, 32, 4_096, 50, 64, 1, "uniform_64_to_1"),
    Cell("dense_local_high_overlap", 65_536, 49_152, 49_152, 40_960, 4_096, 90, 64, 1, "single_chunk_duplicate_5pct"),
    Cell("dense_wide_medium_overlap", 1 << 18, 131_072, 131_072, 65_536, 4_096, 50, 64, 1, "four_uniform_chunks"),
    Cell("long_runs", 1 << 20, 32_768, 32_768, 16_384, 4_096, 50, 64, 2, "64_runs_len_512_shift_256"),
    Cell("mixed_chunk_shapes", 1 << 20, 193_304, 193_304, 65_798, 4_096, 50, 64, 1, "fixed_16_chunk_manifest"),
)
CELL_BY_NAME = {cell.name: cell for cell in CELLS}

EXPECTED_RAW_COUNTS = {
    "tiny_sparse_shuffled": (48, 48),
    "wide_sparse_low_overlap": (4_505, 4_505),
    "skewed_sparse": (64, 65_536),
    "dense_local_high_overlap": (51_609, 51_609),
    "dense_wide_medium_overlap": (131_072, 131_072),
    "long_runs": (32_768, 32_768),
    "mixed_chunk_shapes": (193_304, 193_304),
}

MIXED_CARDINALITIES_A = {
    0: 24,
    1: 4_095,
    2: 4_096,
    3: 4_097,
    4: 32_768,
    5: 49_152,
    6: 16_384,
    7: 16_384,
    8: 256,
    9: 16_384,
    10: 16_384,
    11: 256,
    12: 16_384,
    13: 16_384,
    14: 256,
}
MIXED_CARDINALITIES_B = {
    0: 24,
    1: 4_095,
    2: 4_096,
    3: 4_097,
    4: 32_768,
    5: 49_152,
    6: 16_384,
    7: 16_384,
    8: 16_384,
    9: 256,
    10: 256,
    11: 16_384,
    12: 16_384,
    13: 16_384,
    15: 256,
}
MIXED_PAIR_KINDS = {
    0: ("array", "array"),
    1: ("array", "array"),
    2: ("array", "array"),
    3: ("bitmap", "bitmap"),
    4: ("bitmap", "bitmap"),
    5: ("run", "run"),
    6: ("run", "run"),
    7: ("bitmap", "bitmap"),
    8: ("array", "bitmap"),
    9: ("bitmap", "array"),
    10: ("run", "array"),
    11: ("array", "run"),
    12: ("run", "bitmap"),
    13: ("bitmap", "run"),
    14: ("array", None),
    15: (None, "array"),
}


def expected_chunk_cardinalities(cell: str, operand: str) -> dict[int, int]:
    if cell == "tiny_sparse_shuffled":
        return {0: 24}
    if cell == "wide_sparse_low_overlap":
        if operand == "a":
            return {key: 16 for key in range(256)}
        return {key: (17 if key < 128 else 15) for key in range(256)}
    if cell == "skewed_sparse":
        if operand == "a":
            return {key: 1 for key in range(64)}
        return {
            key: (257 if key < 32 else 255 if key < 64 else 256)
            for key in range(256)
        }
    if cell == "dense_local_high_overlap":
        return {0: 49_152}
    if cell == "dense_wide_medium_overlap":
        return {key: 32_768 for key in range(4)}
    if cell == "long_runs":
        return {key: 2_048 for key in range(16)}
    if cell == "mixed_chunk_shapes":
        return dict(MIXED_CARDINALITIES_A if operand == "a" else MIXED_CARDINALITIES_B)
    raise ValueError(f"unknown cell {cell!r}")


@dataclass(frozen=True)
class Contrast:
    contrast_id: str
    a_candidate: str
    b_candidate: str
    cell: str


CONTRASTS = tuple(
    Contrast(f"{a}_vs_{b}__{cell.name}", a, b, cell.name)
    for cell in CELLS
    for a, b in PAIR_SPECS
)
AA_CONTRAST = Contrast(
    "aa_adaptive__mixed_chunk_shapes", "adaptive", "adaptive", "mixed_chunk_shapes"
)
PROFILE_SPECS = (
    ("sorted", "wide_sparse_low_overlap"),
    ("dense", "dense_local_high_overlap"),
    ("adaptive", "mixed_chunk_shapes"),
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
    seed: int
    warmups: int
    samples: int


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
    seed: int
    status: str
    invalid_reason: str
    returncode: int | None
    timed_out: bool
    position_ns: int | None
    build_ns: int | None
    contains_ns: int | None
    intersection_ns: int | None
    zero_total_samples: int
    fixture_hash: str
    expected_hash: str
    payload_bytes: int | None
    stdout_sha256: str
    stderr_sha256: str
    executable_sha256_before: str
    executable_sha256_after: str
    attempt_directory: str


class ExperimentIncomplete(RuntimeError):
    """The frozen schedule or an evidence gate did not complete."""


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
    temporary.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
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
    seed = block_seed(phase, contrast.contrast_id, block_index)
    positions = []
    for index, label in enumerate(template, start=1):
        candidate = contrast.a_candidate if label == "A" else contrast.b_candidate
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
                seed,
                warmups,
                samples,
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
                    balanced_templates(count, f"{phase}:{contrast.contrast_id}"), start=1
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
        elif phase == "reference":
            for cell in CELLS:
                for index in range(1, REFERENCE_ATTEMPTS_PER_CELL + 1):
                    rows.append(
                        asdict(
                            Position(
                                phase,
                                f"reference__{cell.name}",
                                index,
                                "R",
                                1,
                                "R",
                                "reference",
                                cell.name,
                                block_seed(phase, cell.name, index),
                                DEFAULT_WARMUPS,
                                DEFAULT_SAMPLES,
                            )
                        )
                    )
        elif phase == "quick":
            for cell in CELLS:
                for candidate in CANDIDATES:
                    rows.append(
                        asdict(
                            Position(
                                phase,
                                f"quick__{candidate}__{cell.name}",
                                1,
                                "Q",
                                1,
                                "Q",
                                candidate,
                                cell.name,
                                block_seed(phase, f"{candidate}:{cell.name}", 1),
                                0,
                                1,
                            )
                        )
                    )
    return rows


def phase_sequence(requested: str) -> list[str]:
    if requested in ("plan", "all"):
        return ["pilot", "main", "reference", "aa", "profile"]
    if requested == "main":
        return ["main", "reference"]
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
        "reference_attempts_per_cell": REFERENCE_ATTEMPTS_PER_CELL,
        "primary_family_size": FAMILY_SIZE,
        "familywise_alpha": FAMILY_ALPHA,
        "multiplicity": "Bonferroni simultaneous paired-t intervals",
        "practical_ratio_boundary": PRACTICAL_RATIO,
        "analysis_unit": "one complete four-position ABBA or BAAB block contrast",
        "subsample_unit": "one timed composite workload call within a process position",
        "interference_boundary": (
            "fresh processes reset candidate state; caches, thermals, allocator, kernel, "
            "and frequency state may persist"
        ),
        "cells": [asdict(cell) for cell in CELLS],
        "primary_contrasts": [asdict(contrast) for contrast in CONTRASTS],
        "planned_schedule": schedule,
        "planned_position_count": len(schedule),
        "dynamic_profile_specs": [
            {"candidate": candidate, "cell": cell}
            for candidate, cell in PROFILE_SPECS
        ],
        "timeout_seconds": timeout,
        "attempts_per_position": MAX_ATTEMPTS_PER_POSITION,
        "retry_policy": "none",
        "rustflags": rustflags,
        "invalid_attempt_policy": (
            "retain every attempt; any timeout, nonzero exit, parse, canary, fixture, "
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


def executable(name: str) -> str:
    value = shutil.which(name)
    if value is None:
        raise RuntimeError(f"required executable {name!r} was not found")
    return value


def build_environment(rustflags: str) -> dict[str, str]:
    allowed = (
        "PATH",
        "HOME",
        "TMPDIR",
        "LANG",
        "LC_ALL",
        "CARGO_HOME",
        "RUSTUP_HOME",
        "SDKROOT",
        "DEVELOPER_DIR",
    )
    environment = {key: os.environ[key] for key in allowed if key in os.environ}
    environment["RUSTFLAGS"] = rustflags
    environment["CARGO_TERM_COLOR"] = "never"
    environment["RUST_BACKTRACE"] = "1"
    return environment


def benchmark_environment() -> dict[str, str]:
    allowed = ("PATH", "TMPDIR", "LANG", "LC_ALL", "DYLD_LIBRARY_PATH", "LD_LIBRARY_PATH")
    environment = {key: os.environ[key] for key in allowed if key in os.environ}
    environment["RUST_BACKTRACE"] = "1"
    environment["LC_ALL"] = "C"
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
        "xctrace": run_capture(["/usr/bin/xcrun", "xctrace", "version"])
        if Path("/usr/bin/xcrun").exists()
        else {"status": "UNAVAILABLE"},
    }
    perf = shutil.which("perf")
    metadata["perf"] = (
        run_capture([perf, "--version"], env=env)
        if perf is not None
        else {"status": "UNAVAILABLE"}
    )
    return metadata


def build_benchmark(run_dir: Path, rustflags: str) -> tuple[Path, dict[str, Any]]:
    env = build_environment(rustflags)
    argv = [
        executable("cargo"),
        "build",
        "--package",
        "topic-003-density-adaptive-bitmaps",
        "--release",
        "--bench",
        "bitmap_sets",
        "--message-format=json-render-diagnostics",
    ]
    result = subprocess.run(
        argv,
        cwd=REPO_ROOT,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    write_bytes_once(run_dir / "build" / "stdout.jsonl", result.stdout)
    write_bytes_once(run_dir / "build" / "stderr.txt", result.stderr)
    executable_path: Path | None = None
    parse_errors = []
    for line in result.stdout.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError as error:
            parse_errors.append(str(error))
            continue
        target = message.get("target", {})
        candidate = message.get("executable")
        if target.get("name") == "bitmap_sets" and candidate:
            executable_path = Path(candidate).resolve()
    metadata = {
        "argv": argv,
        "returncode": result.returncode,
        "stdout_sha256": sha256_bytes(result.stdout),
        "stderr_sha256": sha256_bytes(result.stderr),
        "parse_errors": parse_errors,
        "recorded_environment": safe_recorded_environment(env),
    }
    if result.returncode != 0 or executable_path is None or not executable_path.is_file():
        write_json_once(run_dir / "build" / "metadata.json", metadata)
        raise ExperimentIncomplete("cargo did not produce the bitmap_sets benchmark image")
    retained = run_dir / "build" / "bitmap_sets"
    write_bytes_once(retained, executable_path.read_bytes())
    retained.chmod(0o755)
    metadata.update(
        {
            "cargo_executable": str(executable_path),
            "retained_executable": str(retained),
            "binary_bytes": retained.stat().st_size,
            "binary_sha256": sha256_file(retained),
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


def validated_sample_arrays(
    record: Any, samples: int
) -> tuple[dict[str, list[int]], list[str]]:
    arrays: dict[str, list[int]] = {}
    errors: list[str] = []
    if not isinstance(record, dict):
        return arrays, errors
    for name in SAMPLE_FIELDS:
        value = record.get(name)
        if not isinstance(value, list) or len(value) != samples:
            errors.append(f"{name} must have exactly {samples} entries")
            continue
        if any(not isinstance(item, int) or isinstance(item, bool) or item < 0 for item in value):
            errors.append(f"{name} entries must be nonnegative integers")
            continue
        arrays[name] = value
    return arrays, errors


def validate_record(
    position: Position, record: Any, profile_repetitions: int = 1
) -> list[str]:
    errors: list[str] = []
    if not isinstance(record, dict):
        return ["record is not a JSON object"]
    expected_scalars = {
        "schema_version": SCHEMA_VERSION,
        "candidate": position.candidate,
        "workload": position.cell,
        "generator_version": GENERATOR_VERSION,
        "seed": position.seed,
        "warmups": position.warmups,
        "samples": position.samples,
        "profile_repetitions": profile_repetitions,
    }
    for name, expected in expected_scalars.items():
        if record.get(name) != expected:
            errors.append(f"{name} was {record.get(name)!r}, expected {expected!r}")
    cell = CELL_BY_NAME[position.cell]
    cell_scalars = {
        "universe": cell.universe,
        "dense_logical_words": (cell.universe + 63) // 64,
        "unique_a": cell.unique_a,
        "unique_b": cell.unique_b,
        "probe_count": cell.probes,
        "intersection_calls": cell.intersections,
        "build_repetitions": cell.build_repetitions,
    }
    for name, expected in cell_scalars.items():
        if record.get(name) != expected:
            errors.append(f"{name} was {record.get(name)!r}, expected {expected!r}")
    if cell.overlap is not None and record.get("overlap") != cell.overlap:
        errors.append(f"overlap was {record.get('overlap')!r}, expected {cell.overlap}")
    expected_raw_a, expected_raw_b = EXPECTED_RAW_COUNTS[cell.name]
    exact_fixture_scalars = {
        "raw_a": expected_raw_a,
        "raw_b": expected_raw_b,
        "duplicate_a": expected_raw_a - cell.unique_a,
        "duplicate_b": expected_raw_b - cell.unique_b,
        "probe_count_a": cell.probes // 2,
        "probe_count_b": cell.probes // 2,
        "probe_hits": cell.probes * cell.hit_percent // 100,
        "probe_hits_a": (cell.probes * cell.hit_percent // 100) // 2,
        "probe_hits_b": (cell.probes * cell.hit_percent // 100) // 2,
        "probe_misses": cell.probes - cell.probes * cell.hit_percent // 100,
    }
    for name, expected in exact_fixture_scalars.items():
        if record.get(name) != expected:
            errors.append(f"{name} was {record.get(name)!r}, expected {expected}")
    for name in ("fixture_hash", "expected_hash"):
        value = record.get(name)
        if not isinstance(value, str) or len(value) != 16:
            errors.append(f"{name} must be a 16-character lowercase hexadecimal string")
        elif any(character not in "0123456789abcdef" for character in value):
            errors.append(f"{name} is not lowercase hexadecimal")
    for name in ("payload_bytes_a", "payload_bytes_b"):
        value = record.get(name)
        if not isinstance(value, int) or isinstance(value, bool) or value < 0:
            errors.append(f"{name} must be a nonnegative integer")
    arrays, array_errors = validated_sample_arrays(record, position.samples)
    errors.extend(array_errors)
    if len(arrays) == 4:
        for index, total in enumerate(arrays["total_sample_ns"]):
            component_total = (
                arrays["build_sample_ns"][index]
                + arrays["contains_sample_ns"][index]
                + arrays["intersection_sample_ns"][index]
            )
            if total != component_total:
                errors.append(
                    f"sample {index} total {total} does not equal components {component_total}"
                )
        if median_int(arrays["total_sample_ns"]) <= 0:
            errors.append("median total sample must be positive for log analysis")
    work = record.get("work")
    if not isinstance(work, dict):
        errors.append("work must be an object")
    else:
        required = (
            "raw_values_read",
            "unique_values",
            "duplicate_values",
            "membership_probes",
            "membership_hits",
            "intersection_calls",
            "build_calls",
            "expected_intersection",
            "occupied_chunks_a",
            "occupied_chunks_b",
        )
        for name in required:
            value = work.get(name)
            if not isinstance(value, int) or isinstance(value, bool) or value < 0:
                errors.append(f"work.{name} must be a nonnegative integer")
        exact_work = {
            "raw_values_read": profile_repetitions
            * cell.build_repetitions
            * (expected_raw_a + expected_raw_b),
            "unique_values": profile_repetitions
            * cell.build_repetitions
            * (cell.unique_a + cell.unique_b),
            "duplicate_values": profile_repetitions
            * cell.build_repetitions
            * ((expected_raw_a - cell.unique_a) + (expected_raw_b - cell.unique_b)),
            "membership_probes": profile_repetitions * cell.probes,
            "membership_hits": profile_repetitions
            * (cell.probes * cell.hit_percent // 100),
            "intersection_calls": profile_repetitions * cell.intersections,
            "build_calls": profile_repetitions * cell.build_repetitions * 2,
            "expected_intersection": cell.overlap,
            "occupied_chunks_a": len(expected_chunk_cardinalities(cell.name, "a")),
            "occupied_chunks_b": len(expected_chunk_cardinalities(cell.name, "b")),
        }
        for name, expected in exact_work.items():
            if work.get(name) != expected:
                errors.append(f"work.{name} was {work.get(name)!r}, expected {expected}")
    containers_a, container_errors_a = validate_container_records(
        record.get("adaptive_containers_a"),
        expected_chunk_cardinalities(cell.name, "a"),
        "adaptive_containers_a",
    )
    containers_b, container_errors_b = validate_container_records(
        record.get("adaptive_containers_b"),
        expected_chunk_cardinalities(cell.name, "b"),
        "adaptive_containers_b",
    )
    errors.extend(container_errors_a)
    errors.extend(container_errors_b)
    if position.candidate in ("reference", "sorted"):
        expected_payload_a = 4 * cell.unique_a
        expected_payload_b = 4 * cell.unique_b
    elif position.candidate == "dense":
        expected_payload_a = 8 * ((cell.universe + 63) // 64)
        expected_payload_b = expected_payload_a
    elif not container_errors_a and not container_errors_b:
        expected_payload_a = sum(
            record.get("payload_bytes", -1) for record in containers_a.values()
        )
        expected_payload_b = sum(
            record.get("payload_bytes", -1) for record in containers_b.values()
        )
    else:
        expected_payload_a = None
        expected_payload_b = None
    if expected_payload_a is not None and record.get("payload_bytes_a") != expected_payload_a:
        errors.append(
            f"payload_bytes_a was {record.get('payload_bytes_a')!r}, "
            f"expected {expected_payload_a}"
        )
    if expected_payload_b is not None and record.get("payload_bytes_b") != expected_payload_b:
        errors.append(
            f"payload_bytes_b was {record.get('payload_bytes_b')!r}, "
            f"expected {expected_payload_b}"
        )
    if cell.name == "mixed_chunk_shapes" and not container_errors_a and not container_errors_b:
        actual_pairs = {
            key: (
                containers_a.get(key, {}).get("kind"),
                containers_b.get(key, {}).get("kind"),
            )
            for key in range(16)
        }
        if actual_pairs != MIXED_PAIR_KINDS:
            errors.append(
                f"mixed ordered container-pair manifest was {actual_pairs!r}, "
                f"expected {MIXED_PAIR_KINDS!r}"
            )
    return errors


def validate_container_records(
    value: Any, expected_cardinalities: dict[int, int], field: str
) -> tuple[dict[int, dict[str, Any]], list[str]]:
    errors: list[str] = []
    if not isinstance(value, list):
        return {}, [f"{field} must be an array"]
    by_key: dict[int, dict[str, Any]] = {}
    for index, record in enumerate(value):
        if not isinstance(record, dict):
            errors.append(f"{field}[{index}] must be an object")
            continue
        key = record.get("key")
        cardinality = record.get("cardinality")
        run_count = record.get("run_count")
        payload = record.get("payload_bytes")
        kind = record.get("kind")
        if not isinstance(key, int) or isinstance(key, bool) or not 0 <= key <= 65_535:
            errors.append(f"{field}[{index}].key is invalid")
            continue
        if key in by_key:
            errors.append(f"{field} repeats key {key}")
        by_key[key] = record
        if not isinstance(cardinality, int) or isinstance(cardinality, bool) or cardinality <= 0:
            errors.append(f"{field}[{index}].cardinality is invalid")
            continue
        if not isinstance(run_count, int) or isinstance(run_count, bool) or not 1 <= run_count <= cardinality:
            errors.append(f"{field}[{index}].run_count is invalid")
            continue
        baseline_kind = "array" if cardinality <= 4_096 else "bitmap"
        baseline_payload = 2 * cardinality if cardinality <= 4_096 else 8_192
        run_payload = 2 + 4 * run_count
        expected_kind = "run" if run_payload < baseline_payload else baseline_kind
        expected_payload = run_payload if expected_kind == "run" else baseline_payload
        if kind != expected_kind:
            errors.append(
                f"{field}[{index}].kind was {kind!r}, expected {expected_kind!r}"
            )
        if payload != expected_payload:
            errors.append(
                f"{field}[{index}].payload_bytes was {payload!r}, expected {expected_payload}"
            )
    observed_cardinalities = {
        key: record.get("cardinality") for key, record in by_key.items()
    }
    if observed_cardinalities != expected_cardinalities:
        errors.append(
            f"{field} chunk cardinalities were {observed_cardinalities!r}, "
            f"expected {expected_cardinalities!r}"
        )
    if list(by_key) != sorted(by_key):
        errors.append(f"{field} keys were not strictly ascending")
    return by_key, errors


class Executor:
    def __init__(self, run_dir: Path, binary: Path, timeout: float) -> None:
        self.run_dir = run_dir
        self.binary = binary
        self.binary_hash = sha256_file(binary)
        self.timeout = timeout
        self.env = benchmark_environment()
        self.attempts: list[Attempt] = []
        self.records: dict[str, dict[str, Any]] = {}
        self.blocks: list[dict[str, Any]] = []

    def run_position(self, position: Position) -> Attempt:
        identifier = attempt_id(position)
        directory = self.run_dir / "attempts" / identifier
        directory.mkdir(parents=True, exist_ok=False)
        command = [
            str(self.binary),
            "--candidate",
            position.candidate,
            "--workload",
            position.cell,
            "--seed",
            str(position.seed),
            "--warmups",
            str(position.warmups),
            "--samples",
            str(position.samples),
            "--profile-repetitions",
            "1",
        ]
        before = binary_digest(self.binary)
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
                "orchestrator_sha256": sha256_file(SCRIPT),
            },
        )
        started = utc_now()
        timed_out = False
        launch_error: str | None = None
        interrupted: BaseException | None = None
        returncode: int | None
        stdout: bytes
        stderr: bytes
        try:
            result = subprocess.run(
                command,
                cwd=REPO_ROOT,
                env=self.env,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                timeout=self.timeout,
                check=False,
            )
            returncode = result.returncode
            stdout = result.stdout
            stderr = result.stderr
        except subprocess.TimeoutExpired as error:
            timed_out = True
            returncode = None
            stdout = error.stdout or b""
            stderr = error.stderr or b""
        except OSError as error:
            launch_error = f"process launch failed: {error!r}"
            returncode = None
            stdout = b""
            stderr = b""
        except BaseException as error:
            # `subprocess.run` kills and reaps the child before it propagates,
            # so the position is finished; retain it, then re-raise below.
            interrupted = error
            launch_error = f"process wait interrupted: {error!r}"
            returncode = None
            stdout = b""
            stderr = b""
        ended = utc_now()
        after = binary_digest(self.binary)
        write_bytes_once(directory / "stdout.txt", stdout)
        write_bytes_once(directory / "stderr.txt", stderr)
        record: dict[str, Any] | None = None
        errors: list[str] = []
        lines = [line for line in stdout.decode("utf-8", "replace").splitlines() if line.strip()]
        if launch_error is not None:
            errors.append(launch_error)
        if timed_out:
            errors.append("process timed out")
        if returncode != 0:
            errors.append(f"process return code was {returncode!r}")
        if before != self.binary_hash or after != self.binary_hash:
            errors.append("linked image hash changed")
        if len(lines) != 1:
            errors.append(f"stdout contained {len(lines)} nonempty records, expected one")
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
        sample_arrays, _ = validated_sample_arrays(record, position.samples)
        total_values = sample_arrays.get("total_sample_ns", [])
        build_values = sample_arrays.get("build_sample_ns", [])
        contains_values = sample_arrays.get("contains_sample_ns", [])
        intersection_values = sample_arrays.get("intersection_sample_ns", [])
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
            seed=position.seed,
            status="VALID" if not errors else "INVALID",
            invalid_reason="; ".join(errors),
            returncode=returncode,
            timed_out=timed_out,
            position_ns=median_int(total_values) if total_values else None,
            build_ns=median_int(build_values) if build_values else None,
            contains_ns=median_int(contains_values) if contains_values else None,
            intersection_ns=median_int(intersection_values) if intersection_values else None,
            zero_total_samples=sum(value == 0 for value in total_values),
            fixture_hash=record.get("fixture_hash", "") if record is not None else "",
            expected_hash=record.get("expected_hash", "") if record is not None else "",
            payload_bytes=(
                record.get("payload_bytes_a", 0) + record.get("payload_bytes_b", 0)
                if record is not None
                and isinstance(record.get("payload_bytes_a"), int)
                and isinstance(record.get("payload_bytes_b"), int)
                else None
            ),
            stdout_sha256=sha256_bytes(stdout),
            stderr_sha256=sha256_bytes(stderr),
            executable_sha256_before=before,
            executable_sha256_after=after,
            attempt_directory=str(directory),
        )
        write_json_once(
            directory / "attempt.json",
            {
                "position": asdict(position),
                "attempt": asdict(attempt),
                "started_at": started,
                "ended_at": ended,
                "parsed_record": record,
            },
        )
        self.attempts.append(attempt)
        if record is not None:
            self.records[identifier] = record
        if interrupted is not None:
            raise interrupted
        return attempt

    def run_block(self, phase: str, contrast: Contrast, block_index: int, template: str) -> None:
        positions = positions_for_block(
            phase,
            contrast,
            block_index,
            template,
            DEFAULT_WARMUPS,
            DEFAULT_SAMPLES,
        )
        attempts = [self.run_position(position) for position in positions]
        errors = [attempt.invalid_reason for attempt in attempts if attempt.status != "VALID"]
        fixture_hashes = {attempt.fixture_hash for attempt in attempts}
        expected_hashes = {attempt.expected_hash for attempt in attempts}
        if len(fixture_hashes) != 1:
            errors.append("fixture hashes differ within the complete block")
        if len(expected_hashes) != 1:
            errors.append("expected hashes differ within the complete block")
        if phase == "aa" and {attempt.candidate for attempt in attempts} != {"adaptive"}:
            errors.append("A/A did not execute the identical adaptive artifact")
        block = {
            "phase": phase,
            "contrast_id": contrast.contrast_id,
            "block_index": block_index,
            "template": template,
            "status": "VALID" if not errors else "INVALID",
            "errors": errors,
            "attempt_ids": [attempt.attempt_id for attempt in attempts],
            "fixture_hash": next(iter(fixture_hashes)) if len(fixture_hashes) == 1 else None,
            "expected_hash": next(iter(expected_hashes)) if len(expected_hashes) == 1 else None,
        }
        self.blocks.append(block)
        write_json_once(
            self.run_dir / "blocks" / f"{phase}-{contrast.contrast_id}-b{block_index:02d}.json",
            block,
        )
        if errors:
            raise ExperimentIncomplete(
                f"invalid complete block {phase}/{contrast.contrast_id}/{block_index}: {errors}"
            )

    def write_tables(self) -> None:
        attempts_path = self.run_dir / "attempts.csv"
        with attempts_path.open("x", newline="", encoding="utf-8") as stream:
            writer = csv.DictWriter(stream, fieldnames=list(Attempt.__dataclass_fields__))
            writer.writeheader()
            writer.writerows(asdict(attempt) for attempt in self.attempts)
        blocks_path = self.run_dir / "blocks.csv"
        fields = ("phase", "contrast_id", "block_index", "template", "status", "errors", "attempt_ids", "fixture_hash", "expected_hash")
        with blocks_path.open("x", newline="", encoding="utf-8") as stream:
            writer = csv.DictWriter(stream, fieldnames=fields)
            writer.writeheader()
            for block in self.blocks:
                row = dict(block)
                row["errors"] = json.dumps(row["errors"], separators=(",", ":"))
                row["attempt_ids"] = json.dumps(row["attempt_ids"], separators=(",", ":"))
                writer.writerow(row)


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
        raise ValueError("at least two independent complete-block contrasts are required")
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
        raise ValueError("distribution-free sensitivity is frozen for twelve main blocks")
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
    slope = sum(
        (index - x_mean) * (value - mean)
        for index, value in enumerate(values, start=1)
    ) / denominator
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


def attempts_for_block(executor: Executor, phase: str, contrast_id: str, block: int) -> list[Attempt]:
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


def component_value(attempt: Attempt, component: str) -> int | None:
    value = {
        "total": attempt.position_ns,
        "build": attempt.build_ns,
        "contains": attempt.contains_ns,
        "intersection": attempt.intersection_ns,
    }[component]
    if value is None or value <= 0:
        if component == PRIMARY_COMPONENT:
            raise ExperimentIncomplete(
                f"missing positive {component} value in {attempt.attempt_id}"
            )
        return None
    return value


def block_contrasts(
    executor: Executor, phase: str, contrast: Contrast, component: str
) -> list[float] | None:
    count = PILOT_BLOCKS if phase == "pilot" else (AA_BLOCKS if phase == "aa" else MAIN_BLOCKS)
    values = []
    for block_index in range(1, count + 1):
        attempts = attempts_for_block(executor, phase, contrast.contrast_id, block_index)
        if len(attempts) != 4 or any(attempt.status != "VALID" for attempt in attempts):
            raise ExperimentIncomplete(
                f"{phase}/{contrast.contrast_id}/{block_index} is not a complete valid block"
            )
        a_raw = [component_value(attempt, component) for attempt in attempts if attempt.label == "A"]
        b_raw = [component_value(attempt, component) for attempt in attempts if attempt.label == "B"]
        if len(a_raw) != 2 or len(b_raw) != 2:
            raise ExperimentIncomplete("complete block does not contain two A and two B positions")
        if any(value is None for value in a_raw + b_raw):
            return None
        a_values = [math.log(value) for value in a_raw if value is not None]
        b_values = [math.log(value) for value in b_raw if value is not None]
        values.append(statistics.fmean(b_values) - statistics.fmean(a_values))
    return values


def primary_block_contrasts(
    executor: Executor, phase: str, contrast: Contrast
) -> list[float]:
    values = block_contrasts(executor, phase, contrast, PRIMARY_COMPONENT)
    if values is None:
        raise ExperimentIncomplete(f"{phase}/{contrast.contrast_id} lost its primary contrasts")
    return values


def reliability(executor: Executor, phase: str) -> dict[str, Any]:
    attempts = [attempt for attempt in executor.attempts if attempt.phase == phase]
    return {
        "attempt_count": len(attempts),
        "valid_attempts": sum(attempt.status == "VALID" for attempt in attempts),
        "invalid_attempts": sum(attempt.status != "VALID" for attempt in attempts),
        "timeouts": sum(attempt.timed_out for attempt in attempts),
        "zero_total_samples": sum(attempt.zero_total_samples for attempt in attempts),
        "returncodes": sorted({attempt.returncode for attempt in attempts}, key=lambda value: -999 if value is None else value),
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
        components = {}
        for component in DESCRIPTIVE_COMPONENTS:
            values = block_contrasts(executor, "main", contrast, component)
            if values is None:
                components[component] = {
                    "status": "UNAVAILABLE",
                    "reason": (
                        "at least one position in a complete valid block reported a "
                        "nonpositive component median; a log ratio is undefined there"
                    ),
                    "decision_role": "descriptive decomposition; not a separate decision family",
                }
                continue
            components[component] = {
                "status": "AVAILABLE",
                "point_ratio": math.exp(statistics.fmean(values)),
                "block_log_contrasts": values,
                "decision_role": "descriptive decomposition; not a separate decision family",
            }
        row = asdict(contrast)
        row.update(
            {
                "estimand": "B/A geometric mean of complete-block composite time ratios",
                "block_log_contrasts": total,
                "simultaneous_interval": interval,
                "classification": classify_interval(interval),
                "components": components,
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
    if phase == "reference":
        cells = []
        for cell in CELLS:
            selected = [attempt for attempt in attempts if attempt.cell == cell.name]
            if len(selected) != REFERENCE_ATTEMPTS_PER_CELL:
                raise ExperimentIncomplete(f"reference schedule incomplete for {cell.name}")
            cells.append(
                {
                    "cell": cell.name,
                    "attempt_count": len(selected),
                    "median_composite_ns": median_int([attempt.position_ns for attempt in selected if attempt.position_ns is not None]),
                    "median_build_ns": median_int([attempt.build_ns for attempt in selected if attempt.build_ns is not None]),
                    "median_contains_ns": median_int([attempt.contains_ns for attempt in selected if attempt.contains_ns is not None]),
                    "median_intersection_ns": median_int([attempt.intersection_ns for attempt in selected if attempt.intersection_ns is not None]),
                    "payload_bytes": sorted({attempt.payload_bytes for attempt in selected}),
                    "decision_role": "descriptive reference cost; excluded from the optimized family",
                }
            )
        result["cells"] = cells
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
    if {attempt.candidate for attempt in attempts} != {"adaptive"}:
        mechanical_errors.append("labels did not execute the same adaptive candidate")
    templates = [
        block["template"] for block in executor.blocks if block["phase"] == "aa"
    ]
    if templates.count("ABBA") != 6 or templates.count("BAAB") != 6:
        mechanical_errors.append("A/A templates were not balanced six ABBA and six BAAB")
    for block_index in range(1, AA_BLOCKS + 1):
        block_attempts = attempts_for_block(executor, "aa", AA_CONTRAST.contrast_id, block_index)
        if len({attempt.fixture_hash for attempt in block_attempts}) != 1:
            mechanical_errors.append(f"block {block_index} fixture hashes differed")
        if len({attempt.expected_hash for attempt in block_attempts}) != 1:
            mechanical_errors.append(f"block {block_index} expected hashes differed")
        if len({attempt.payload_bytes for attempt in block_attempts}) != 1:
            mechanical_errors.append(f"block {block_index} payloads differed")
        work_records = [
            executor.records[attempt.attempt_id].get("work") for attempt in block_attempts
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
                    "geometric_mean_composite_ns": (
                        math.exp(statistics.fmean(math.log(value) for value in selected))
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
            "identical_candidate": "adaptive",
            "linked_image_sha256": executor.binary_hash,
            "parser_schema_version": SCHEMA_VERSION,
            "template_balance": {
                "ABBA": templates.count("ABBA"),
                "BAAB": templates.count("BAAB"),
            },
        },
        "null_diagnostic": {
            "interval": analyze_values(values, 0.05),
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
        raise ExperimentIncomplete(f"A/A mechanical integrity failed: {mechanical_errors}")
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
        for block_index, template in enumerate(balanced_templates(count, label), start=1):
            executor.run_block(phase, contrast, block_index, template)


def collect_disassembly(executor: Executor) -> dict[str, Any]:
    directory = executor.run_dir / "profile" / "linked-image"
    directory.mkdir(parents=True, exist_ok=False)
    before = sha256_file(executor.binary)
    commands: list[tuple[str, list[str]]]
    if platform.system() == "Darwin":
        commands = [
            ("symbols", ["/usr/bin/nm", "-nm", str(executor.binary)]),
            ("disassembly", ["/usr/bin/otool", "-tvV", str(executor.binary)]),
        ]
    else:
        nm = shutil.which("nm")
        objdump = shutil.which("objdump")
        if nm is None or objdump is None:
            raise ExperimentIncomplete("nm and objdump are required for linked-image evidence")
        commands = [
            ("symbols", [nm, "-an", str(executor.binary)]),
            ("disassembly", [objdump, "-Cd", str(executor.binary)]),
        ]
    records = {}
    capture_text: dict[str, str] = {}
    for name, argv in commands:
        result = subprocess.run(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
        write_bytes_once(directory / f"{name}.txt", result.stdout)
        write_bytes_once(directory / f"{name}.stderr.txt", result.stderr)
        records[name] = {
            "argv": argv,
            "returncode": result.returncode,
            "stdout_bytes": len(result.stdout),
            "stdout_sha256": sha256_bytes(result.stdout),
            "stderr_sha256": sha256_bytes(result.stderr),
        }
        capture_text[name] = result.stdout.decode("utf-8", "replace")
        if result.returncode != 0 or not result.stdout:
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
    stdout: bytes, candidate: str, cell: str, seed: int
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
        return None, [f"dynamic target emitted {len(records)} harness records, expected one"]
    position = Position(
        "profile",
        f"dynamic__{candidate}__{cell}",
        1,
        "D",
        1,
        "D",
        candidate,
        cell,
        seed,
        1,
        1,
    )
    record = records[0]
    return record, validate_record(position, record, PROFILE_REPETITIONS)


def dynamic_profile_attempt(
    executor: Executor, candidate: str, cell: str, ordinal: int
) -> dict[str, Any]:
    directory = (
        executor.run_dir
        / "profile"
        / "dynamic"
        / f"{ordinal:02d}-{candidate}-{cell}"
    )
    directory.mkdir(parents=True, exist_ok=False)
    seed = block_seed("profile", f"{candidate}:{cell}", 1)
    target_args = [
        str(executor.binary),
        "--candidate",
        candidate,
        "--workload",
        cell,
        "--seed",
        str(seed),
        "--warmups",
        "1",
        "--samples",
        "1",
        "--profile-repetitions",
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
    launch_error: str | None = None
    interrupted: BaseException | None = None
    if tool is None:
        timed_out = False
        returncode = None
        stdout = b""
        stderr = b"no supported dynamic sampling tool was available\n"
    else:
        try:
            result = subprocess.run(
                argv,
                cwd=REPO_ROOT,
                env=executor.env,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                timeout=executor.timeout,
                check=False,
            )
            timed_out = False
            returncode = result.returncode
            stdout = result.stdout
            stderr = result.stderr
        except subprocess.TimeoutExpired as error:
            timed_out = True
            returncode = None
            stdout = error.stdout or b""
            stderr = error.stderr or b""
        except OSError as error:
            launch_error = f"profiler launch failed: {error!r}"
            timed_out = False
            returncode = None
            stdout = b""
            stderr = launch_error.encode() + b"\n"
        except BaseException as error:
            # `subprocess.run` kills and reaps the profiler before re-raising,
            # so the attempt is finished; retain it, then re-raise.
            interrupted = error
            launch_error = f"profiler wait interrupted: {error!r}"
            timed_out = False
            returncode = None
            stdout = b""
            stderr = launch_error.encode() + b"\n"
    after = binary_digest(executor.binary)
    write_bytes_once(directory / "stdout.txt", stdout)
    write_bytes_once(directory / "stderr.txt", stderr)
    artifact_bytes = profile_artifact_bytes(artifact)
    artifact_exists = artifact is not None and artifact.exists()
    image_unchanged = before == executor.binary_hash == after
    target_record, target_record_errors = dynamic_target_record(
        stdout, candidate, cell, seed
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
    if interrupted is not None:
        raise interrupted
    return record


def run_profile(executor: Executor) -> dict[str, Any]:
    canary_attempts = []
    for ordinal, (candidate, cell) in enumerate(PROFILE_SPECS, start=1):
        canary = Position(
            "profile",
            f"profile_canary__{candidate}__{cell}",
            ordinal,
            "P",
            1,
            "P",
            candidate,
            cell,
            block_seed("profile", f"{candidate}:{cell}", 1),
            1,
            1,
        )
        attempt = executor.run_position(canary)
        canary_attempts.append(attempt.attempt_id)
        if attempt.status != "VALID":
            raise ExperimentIncomplete(f"profile canary failed: {attempt.invalid_reason}")
    linked_image = collect_disassembly(executor)
    dynamic = [
        dynamic_profile_attempt(executor, candidate, cell, ordinal)
        for ordinal, (candidate, cell) in enumerate(PROFILE_SPECS, start=1)
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


def execute_phase(executor: Executor, phase: str) -> dict[str, Any]:
    if phase in ("pilot", "main", "aa"):
        run_block_phase(executor, phase)
        if phase == "pilot":
            result = analyze_pilot(executor)
        elif phase == "main":
            result = analyze_main(executor)
        else:
            result = analyze_aa(executor)
    elif phase in ("quick", "reference"):
        run_direct_phase(executor, phase)
        result = analyze_direct(executor, phase)
    elif phase == "profile":
        result = run_profile(executor)
    else:
        raise ValueError(f"unknown phase {phase!r}")
    write_json_once(executor.run_dir / "analysis" / f"{phase}.json", result)
    return result


def checksum_manifest(run_dir: Path) -> tuple[str, int]:
    paths = sorted(
        path for path in run_dir.rglob("*") if path.is_file() and path.name != "manifest.sha256"
    )
    lines = []
    for path in paths:
        relative = path.relative_to(run_dir).as_posix()
        if "\n" in relative or "\r" in relative:
            raise ExperimentIncomplete("newline in evidence path cannot be represented safely")
        lines.append(f"{sha256_file(path)}  {relative}\n")
    payload = "".join(lines).encode("utf-8")
    write_bytes_once(run_dir / "manifest.sha256", payload)
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
    if len(CELLS) != 7 or len({cell.name for cell in CELLS}) != 7:
        errors.append("expected seven uniquely named workload cells")
    if len(CONTRASTS) != FAMILY_SIZE:
        errors.append("primary contrast family is not exactly 21")
    if set(PAIR_SPECS) != {("sorted", "dense"), ("sorted", "adaptive"), ("dense", "adaptive")}:
        errors.append("optimized candidate pairs changed")
    if PROFILE_SPECS != (
        ("sorted", "wide_sparse_low_overlap"),
        ("dense", "dense_local_high_overlap"),
        ("adaptive", "mixed_chunk_shapes"),
    ):
        errors.append("three representative dynamic profile specifications changed")
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
    all_schedule = planned_schedule(["pilot", "main", "reference", "aa"])
    expected_positions = (
        FAMILY_SIZE * PILOT_BLOCKS * 4
        + FAMILY_SIZE * MAIN_BLOCKS * 4
        + len(CELLS) * REFERENCE_ATTEMPTS_PER_CELL
        + AA_BLOCKS * 4
    )
    if len(all_schedule) != expected_positions:
        errors.append(f"planned {len(all_schedule)} positions, expected {expected_positions}")
    identifiers = []
    for row in all_schedule:
        identifiers.append(attempt_id(Position(**row)))
    if len(identifiers) != len(set(identifiers)):
        errors.append("planned attempt identifiers are not unique")
    quantile = student_t_quantile(0.975, 11)
    if not math.isclose(quantile, 2.200985, rel_tol=0.0, abs_tol=1.0e-5):
        errors.append(f"Student t implementation returned unexpected t(11,.975)={quantile}")
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
        choices=("plan", "self-check", "quick", "pilot", "main", "aa", "profile", "all"),
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        help="required for collection: absolute, outside Git, and initially nonexistent",
    )
    parser.add_argument("--timeout-seconds", type=float, default=DEFAULT_TIMEOUT_SECONDS)
    parser.add_argument("--rustflags", default="-C target-cpu=native")
    return parser.parse_args()


def run_experiment(args: argparse.Namespace) -> int:
    if not math.isfinite(args.timeout_seconds) or args.timeout_seconds <= 0:
        raise ValueError("--timeout-seconds must be finite and positive")
    if args.mode not in ("plan", "self-check") and args.timeout_seconds != DEFAULT_TIMEOUT_SECONDS:
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
        write_json_once(run_dir / "metadata" / "source-tree-before.json", initial_source)
        capture_source_snapshot(run_dir, initial_source)
        env = build_environment(args.rustflags)
        write_json_once(run_dir / "metadata" / "host.json", host_metadata())
        write_json_once(run_dir / "metadata" / "toolchain.json", toolchain_metadata(env))
        write_json_once(run_dir / "metadata" / "git.json", git_metadata(env))
        binary, build = build_benchmark(run_dir, args.rustflags)
        post_build_source = source_manifest()
        write_json_once(
            run_dir / "metadata" / "source-tree-after-build.json", post_build_source
        )
        if source_digest(post_build_source) != source_digest(initial_source):
            raise ExperimentIncomplete("source tree changed during the build")
        executor = Executor(run_dir, binary, args.timeout_seconds)
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
            status["completed_phases"].append(phase)
            status["last_progress_at"] = utc_now()
            replace_json(run_dir / "run-status.json", status)
        final_source = source_manifest()
        write_json_once(run_dir / "metadata" / "source-tree-after.json", final_source)
        if source_digest(final_source) != source_digest(initial_source):
            raise ExperimentIncomplete("source tree changed during collection")
        if sha256_file(binary) != build["binary_sha256"]:
            raise ExperimentIncomplete("retained linked image changed during collection")
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
                    for phase in sorted({attempt.phase for attempt in executor.attempts})
                },
            )
            try:
                executor.write_tables()
            except FileExistsError:
                pass
        write_text_once(run_dir / "failure.txt", traceback.format_exc())
    replace_json(run_dir / "run-status.json", status)
    manifest_hash, manifest_count = checksum_manifest(run_dir)
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
