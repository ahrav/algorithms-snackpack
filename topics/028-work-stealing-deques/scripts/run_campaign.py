#!/usr/bin/env python3
"""Run the frozen order-balanced work-stealing campaign as fresh processes.

Every invocation is retained as a row with a status of ok, failed or timeout.
The output document is rewritten after each invocation so an interrupted
campaign keeps its completed observations. Each repetition also runs the
global FIFO artifact twice under the labels A and B as an identical-artifact
control that flows through the same scheduling, parsing and analysis paths.
"""
import argparse
import hashlib
import json
import os
import platform
import statistics
import subprocess
import time
from pathlib import Path

WORKLOADS = [
    {"name": "tiny_balanced", "workers": 2, "tasks": 256, "leaf_work": 0},
    {"name": "tiny_skewed", "workers": 2, "tasks": 256, "leaf_work": 0},
    {"name": "medium_balanced", "workers": 4, "tasks": 16384, "leaf_work": 8},
    {"name": "medium_skewed", "workers": 4, "tasks": 16384, "leaf_work": 8},
    {"name": "coarse_skewed", "workers": 4, "tasks": 4096, "leaf_work": 512},
]
BASELINE = "global_fifo"
CONTROL_LABELS = ["A", "B"]
BOUNDARY = 1.03
TOPIC = Path(__file__).resolve().parent.parent
REPOSITORY = TOPIC.parent.parent
SOURCE_FILES = [
    "Cargo.lock",
    "Cargo.toml",
    "topics/028-work-stealing-deques/Cargo.toml",
    "topics/028-work-stealing-deques/examples/compare.rs",
    "topics/028-work-stealing-deques/scripts/run_campaign.py",
    "topics/028-work-stealing-deques/src/lib.rs",
    "topics/028-work-stealing-deques/tests/contracts.rs",
]


def candidates_for(workload):
    placement = "locked_steal_balanced" if workload["name"].endswith("balanced") else "locked_steal_skewed"
    return [BASELINE, placement]


def captured_text(output):
    """TimeoutExpired carries bytes even under text=True, so decode before the row is serialised."""
    if output is None:
        return ""
    if isinstance(output, bytes):
        output = output.decode("utf-8", errors="replace")
    return output.strip()


def invoke(binary, workload, candidate, timeout):
    command = [
        str(binary), "--candidate", candidate, "--workers", str(workload["workers"]),
        "--tasks", str(workload["tasks"]), "--leaf-work", str(workload["leaf_work"]),
    ]
    row = {"candidate": candidate, "command": command, "started_unix_s": time.time()}
    try:
        process = subprocess.run(command, text=True, capture_output=True, timeout=timeout)
    except subprocess.TimeoutExpired as expired:
        row.update({"status": "timeout", "timeout_s": timeout, "stdout": captured_text(expired.stdout), "stderr": captured_text(expired.stderr)})
        return row
    row.update({"stdout": process.stdout.strip(), "stderr": process.stderr.strip(), "returncode": process.returncode})
    fields = dict(item.split("=", 1) for item in process.stdout.split() if "=" in item)
    expected = {"candidate": candidate, "workers": str(workload["workers"]), "tasks": str(workload["tasks"]), "leaf_work": str(workload["leaf_work"])}
    if process.returncode != 0:
        row.update({"status": "failed", "reason": f"exit code {process.returncode}"})
    elif any(fields.get(key) != value for key, value in expected.items()):
        row.update({"status": "failed", "reason": f"echoed arguments {fields} differ from {expected}"})
    elif not fields.get("elapsed_ns", "").isdigit() or int(fields["elapsed_ns"]) <= 0:
        row.update({"status": "failed", "reason": f"elapsed_ns {fields.get('elapsed_ns')!r} is not a positive integer"})
    else:
        row.update({"status": "ok", "elapsed_ns": int(fields["elapsed_ns"])})
    return row


def probe(command, cwd=None):
    try:
        return subprocess.run(command, cwd=cwd, text=True, capture_output=True, check=True, timeout=60).stdout.strip()
    except (OSError, subprocess.SubprocessError):
        return None


def sha256_of(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest() if Path(path).is_file() else None


def cpu_model():
    cpuinfo = Path("/proc/cpuinfo")
    if cpuinfo.is_file():
        for line in cpuinfo.read_text(errors="replace").splitlines():
            if line.lower().startswith(("model name", "cpu implementer", "hardware")):
                return line.split(":", 1)[1].strip()
    return platform.processor() or None


def identity(binary):
    dirty = probe(["git", "status", "--porcelain", "--untracked-files=no"], cwd=REPOSITORY)
    return {
        "hostname": platform.node(),
        "platform": platform.platform(),
        "uname": list(platform.uname()),
        "python": platform.python_version(),
        "cpu_model": cpu_model(),
        "available_cpus": len(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else os.cpu_count(),
        "source_commit": probe(["git", "rev-parse", "HEAD"], cwd=REPOSITORY),
        "worktree_dirty": None if dirty is None else bool(dirty),
        "source_sha256": {name: sha256_of(REPOSITORY / name) for name in SOURCE_FILES},
        "binary": str(binary),
        "binary_sha256": sha256_of(binary),
        "rustc": probe(["rustc", "-Vv"]),
        "cargo": probe(["cargo", "-V"]),
        "rustflags": os.environ.get("RUSTFLAGS", ""),
    }


def paired_ratios(rows, numerator, denominator, repetitions):
    """Return numerator/denominator per repetition and the count of repetitions missing an ok pair."""
    by_label = {}
    for row in rows:
        if row["status"] == "ok":
            by_label.setdefault(row["label"], {})[row["repetition"]] = row["elapsed_ns"]
    ratios, incomplete = [], 0
    for repetition in range(repetitions):
        top = by_label.get(numerator, {}).get(repetition)
        bottom = by_label.get(denominator, {}).get(repetition)
        if top is None or bottom is None:
            incomplete += 1
        else:
            ratios.append(top / bottom)
    return ratios, incomplete


def summarize(rows, warmups, repetitions):
    summaries = []
    for workload in WORKLOADS:
        cell = [r for r in rows if r["workload"]["name"] == workload["name"]]
        warmup_failures = len([w for w in warmups if w["workload"]["name"] == workload["name"] and w["status"] != "ok"])
        baseline, challenger = candidates_for(workload)
        observed = {}
        for label in [baseline, challenger] + CONTROL_LABELS:
            values = [r["elapsed_ns"] for r in cell if r["label"] == label and r["status"] == "ok"]
            observed[label] = {"median_ns": statistics.median(values) if values else None, "min_ns": min(values, default=None), "max_ns": max(values, default=None), "samples": len(values)}
        ratios, incomplete = paired_ratios(cell, challenger, baseline, repetitions)
        aa_ratios, aa_incomplete = paired_ratios(cell, CONTROL_LABELS[0], CONTROL_LABELS[1], repetitions)
        if incomplete or warmup_failures:
            selected = "unresolved"
        elif all(r < 1 / BOUNDARY for r in ratios):
            selected = challenger
        elif all(r > BOUNDARY for r in ratios):
            selected = baseline
        else:
            selected = "unresolved"
        ranked = [label for label in (baseline, challenger) if observed[label]["median_ns"] is not None]
        summaries.append({
            "workload": workload,
            "candidates": observed,
            "lowest_median": min(ranked, key=lambda label: observed[label]["median_ns"]) if ranked else None,
            "paired_ratios": ratios,
            "ratio_range": [min(ratios), max(ratios)] if ratios else None,
            "incomplete_repetitions": incomplete,
            "selected": selected,
            "aa_ratios": aa_ratios,
            "aa_incomplete_repetitions": aa_incomplete,
            "failures": len([r for r in cell if r["status"] != "ok"]),
            "warmup_failures": warmup_failures,
        })
    return summaries


def write_document(out, document):
    out.parent.mkdir(parents=True, exist_ok=True)
    scratch = out.with_name(out.name + ".partial")
    scratch.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")
    os.replace(scratch, out)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--repetitions", type=int, default=8)
    parser.add_argument("--timeout", type=float, default=60.0, help="seconds allowed per process")
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    if args.repetitions < 4 or args.repetitions % 2:
        raise SystemExit("--repetitions must be even and at least four")
    document = {
        "status": "running",
        "boundary": "scheduler lifecycle: queues, threads, task work, counters, joins and equality; sequential oracle before timer",
        "selection_rule": "lowest median is descriptive; select only when every paired steal/global ratio passes the same 3 percent boundary; otherwise unresolved",
        "control": "identical global_fifo artifact labelled A and B once each per repetition with alternating order",
        "order": "treatment order rotates once per repetition; control order alternates once per repetition",
        "repetitions": args.repetitions,
        "timeout_s": args.timeout,
        "environment": identity(args.binary),
        "created_unix_s": time.time(),
        "warmups": [],
        "rows": [],
        "summaries": [],
    }
    for workload in WORKLOADS:
        candidates = candidates_for(workload)
        for candidate in candidates:
            row = invoke(args.binary, workload, candidate, args.timeout)
            row.update({"workload": workload, "label": candidate, "arm": "warmup"})
            document["warmups"].append(row)
            write_document(args.out, document)
        for repetition in range(args.repetitions):
            rotated = candidates[repetition % 2:] + candidates[:repetition % 2]
            treatments = [(candidate, candidate, "treatment") for candidate in rotated]
            controls = [(BASELINE, label, "control") for label in (CONTROL_LABELS if repetition % 2 == 0 else CONTROL_LABELS[::-1])]
            for candidate, label, arm in treatments + controls:
                row = invoke(args.binary, workload, candidate, args.timeout)
                row.update({"workload": workload, "label": label, "arm": arm, "repetition": repetition})
                document["rows"].append(row)
                write_document(args.out, document)
    document["summaries"] = summarize(document["rows"], document["warmups"], args.repetitions)
    document["status"] = "complete"
    write_document(args.out, document)
    for summary in document["summaries"]:
        print(f'{summary["workload"]["name"]}: lowest median {summary["lowest_median"]}, selected {summary["selected"]}, failures {summary["failures"]}, warmup failures {summary["warmup_failures"]}')


if __name__ == "__main__":
    main()
