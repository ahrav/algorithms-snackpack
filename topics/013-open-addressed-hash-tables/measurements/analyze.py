#!/usr/bin/env python3
"""Derive one host's results.json block from raw/samples.jsonl written by runner.py.

Applies the BENCHMARK.md rule over the frozen schedule: each sample's `ns` is the
per-rep average emitted by the probe; medians and observed ranges span the 12
blocks; `paired_ratio_ranges[rival]` is [min, max] over blocks of rival_ns divided
by the lowest-median candidate's ns in the same block; `selected` names that
candidate only when every rival ratio exceeds 1.05 in every block, else it is
"unresolved". Diagnostics are the deterministic per-process counters and must be
identical across blocks. The input must hold the complete frozen schedule in the
exact order runner.py executes it; a partial, duplicated, or reordered campaign is
rejected before any summary is computed.

The schedule constants duplicate the receipt-pinned runner.py, which performs its
campaign at import time and therefore cannot be imported.
Usage: analyze.py raw/samples.jsonl
"""
import itertools
import json
import statistics
import sys

WORKLOADS = ["tiny", "clean25", "clean75", "clean87", "churn_read", "mixed25", "mixed75", "cluster", "build", "first_lookup"]
CANDIDATES = ["lazy", "rebuild", "shift"]
ORDERS = list(itertools.permutations(CANDIDATES)) * 2
THRESHOLD = 1.05
DIAGNOSTICS = ["ops", "reps", "lookup_probe_sum", "start_deleted", "rebuilds", "reinserted", "shifted"]


def schedule():
    """Yield (block, workload, candidate) in the order runner.py runs processes."""
    for block, order in enumerate(ORDERS):
        for offset in range(len(WORKLOADS)):
            workload = WORKLOADS[(offset + block) % len(WORKLOADS)]
            for candidate in order:
                yield block, workload, candidate


def check_schedule(samples):
    expected = list(schedule())
    if len(samples) != len(expected):
        raise ValueError(f"incomplete campaign: {len(samples)} samples, expected {len(expected)}")
    for i, (s, (block, workload, candidate)) in enumerate(zip(samples, expected)):  # noqa: B905 (lengths checked above; python3.9 hosts)
        actual = (s["block"], s["workload"], s["candidate"])
        if actual != (block, workload, candidate):
            raise ValueError(f"schedule mismatch at sample {i}: {actual}, expected {(block, workload, candidate)}")
        if list(s["order"]) != list(ORDERS[block]):
            raise ValueError(f"order mismatch at sample {i}: {s['order']}, expected {list(ORDERS[block])}")
        if s["oracle"] is not True or not s["ns"] > 0:
            raise ValueError(f"oracle or timing failure at sample {i}: {s}")


def cell(workload, rows):
    per = {c: [r["ns"] for r in rows if r["candidate"] == c] for c in CANDIDATES}
    medians = {c: statistics.median(per[c]) for c in CANDIDATES}
    fastest = min(CANDIDATES, key=lambda c: medians[c])
    ratios = {r: [x / y for x, y in zip(per[r], per[fastest])] for r in CANDIDATES if r != fastest}  # noqa: B905
    diagnostics = {}
    for c in CANDIDATES:
        values = {tuple(r[k] for k in DIAGNOSTICS) for r in rows if r["candidate"] == c}
        if len(values) != 1:
            raise ValueError(f"diagnostics differ across blocks for {workload} {c}: {sorted(values)}")
        diagnostics[c] = dict(zip(DIAGNOSTICS, values.pop()))  # noqa: B905
    return {
        "workload": workload,
        "medians_ns": medians,
        "fastest_median": fastest,
        "selected": fastest if all(min(v) > THRESHOLD for v in ratios.values()) else "unresolved",
        "ranges_ns": {c: [min(per[c]), max(per[c])] for c in CANDIDATES},
        "paired_ratio_ranges": {r: [min(v), max(v)] for r, v in ratios.items()},
        "diagnostics": diagnostics,
    }


def analyze(samples):
    check_schedule(samples)
    return {
        "processes": len(samples),
        "results": [cell(w, [s for s in samples if s["workload"] == w]) for w in WORKLOADS],
    }


def main(argv):
    if len(argv) != 2:
        sys.exit("usage: analyze.py raw/samples.jsonl")
    try:
        with open(argv[1]) as f:
            samples = [json.loads(line) for line in f if line.strip()]
        print(json.dumps(analyze(samples), indent=2))
    except (OSError, ValueError, KeyError, TypeError) as e:
        sys.exit(f"analyze.py: {e}")


if __name__ == "__main__":
    main(sys.argv)
