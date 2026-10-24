#!/usr/bin/env python3
"""Run the frozen order-balanced work-stealing campaign as fresh processes."""
import argparse
import json
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


def candidates_for(workload):
    placement = "locked_steal_balanced" if workload["name"].endswith("balanced") else "locked_steal_skewed"
    return ["global_fifo", placement]


def invoke(binary, workload, candidate):
    command = [
        str(binary), "--candidate", candidate, "--workers", str(workload["workers"]),
        "--tasks", str(workload["tasks"]), "--leaf-work", str(workload["leaf_work"]),
    ]
    process = subprocess.run(command, check=True, text=True, capture_output=True)
    fields = dict(item.split("=", 1) for item in process.stdout.strip().split())
    return {"candidate": candidate, "elapsed_ns": int(fields["elapsed_ns"]), "stdout": process.stdout.strip()}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--repetitions", type=int, default=8)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    if args.repetitions < 4 or args.repetitions % 2:
        raise SystemExit("--repetitions must be even and at least four")
    rows = []
    warmups = []
    for workload in WORKLOADS:
        candidates = candidates_for(workload)
        for candidate in candidates:
            row = invoke(args.binary, workload, candidate)
            row["workload"] = workload
            warmups.append(row)
        for repetition in range(args.repetitions):
            rotated = candidates[repetition % len(candidates):] + candidates[:repetition % len(candidates)]
            for candidate in rotated:
                row = invoke(args.binary, workload, candidate)
                row.update({"workload": workload, "repetition": repetition})
                rows.append(row)
    summaries = []
    for workload in WORKLOADS:
        observed = {}
        for candidate in candidates_for(workload):
            values = [r["elapsed_ns"] for r in rows if r["workload"]["name"] == workload["name"] and r["candidate"] == candidate]
            observed[candidate] = {"median_ns": statistics.median(values), "min_ns": min(values), "max_ns": max(values), "samples": len(values)}
        best = min(observed, key=lambda c: observed[c]["median_ns"])
        summaries.append({"workload": workload, "local_fastest_median": best, "candidates": observed})
    document = {
        "boundary": "scheduler lifecycle: queues, threads, task work, counters, joins and equality; sequential oracle before timer",
        "warmups": warmups,
        "selection_rule": "lowest median is descriptive; select only when every paired steal/global ratio passes the same 3 percent boundary; otherwise unresolved",
        "order": "candidate order rotates once per repetition",
        "repetitions": args.repetitions,
        "environment": {"platform": platform.platform(), "python": platform.python_version()},
        "created_unix_s": time.time(),
        "rows": rows,
        "summaries": summaries,
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")
    for summary in summaries:
        print(f'{summary["workload"]["name"]}: local median winner {summary["local_fastest_median"]}')


if __name__ == "__main__":
    main()
