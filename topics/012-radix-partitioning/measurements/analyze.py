#!/usr/bin/env python3
"""Derive one host's compact RESULTS cells from raw/samples.jsonl written by runner.py.

Applies the BENCHMARK.md rule: per-call time is elapsed_ns/reps; medians and observed
ranges span blocks; each pair entry is [min, median, max] over paired blocks of the
second-named candidate divided by the first (matching committed RESULTS.json keys);
the leader has the lowest median among contract-valid candidates and is unique only when
every paired rival/leader ratio exceeds 1.05. Usage: analyze.py raw/samples.jsonl
"""
import itertools
import json
import statistics
import sys

CANDIDATES = ["buckets", "scatter", "two_pass", "cycles"]
CONTRACTS = {"stable": ["buckets", "scatter", "two_pass"], "unordered": CANDIDATES}
THRESHOLD = 1.05
FIELDS = ["n", "bits", "shape", "width", "cold"]


def summary(xs):
    return [min(xs), statistics.median(xs), max(xs)]


def ratio(per, num, den):
    if len(per[num]) != len(per[den]):
        raise ValueError(f"unpaired blocks for {num}/{den}")
    return [x / y for x, y in zip(per[num], per[den])]  # noqa: B905 (python3.9 hosts)


def analyze(samples):
    cells = {}
    for s in samples:
        if s["oracle"] != "pass":
            raise ValueError(f"oracle failure in sample: {s}")
        cells.setdefault(s["cell"], {}).setdefault(s["block"], {})[s["candidate"]] = s
    out = {}
    for cell, blocks in cells.items():
        per = {c: [blocks[b][c]["elapsed_ns"] / blocks[b][c]["reps"] for b in sorted(blocks)] for c in CANDIDATES}
        contracts = {}
        for name, allowed in CONTRACTS.items():
            leader = min(allowed, key=lambda c: statistics.median(per[c]))
            rival = {r: summary(ratio(per, r, leader)) for r in allowed if r != leader}
            contenders = [leader] + [r for r in allowed if r != leader and rival[r][0] <= THRESHOLD]
            contracts[name] = {"median_leader": leader, "unique": len(contenders) == 1,
                               "contenders": contenders, "rival_over_leader": rival}
        first = next(iter(blocks.values()))[CANDIDATES[0]]
        out[cell] = {**{k: first[k] for k in FIELDS},
                     "median_ns": {c: statistics.median(per[c]) for c in CANDIDATES},
                     "observed_range_ns": {c: [min(per[c]), max(per[c])] for c in CANDIDATES},
                     "pairs": {f"{a}/{b}": summary(ratio(per, b, a)) for a, b in itertools.combinations(CANDIDATES, 2)},
                     "contracts": contracts}
    return out


def main(argv):
    if len(argv) != 2:
        sys.exit("usage: analyze.py raw/samples.jsonl")
    try:
        with open(argv[1]) as f:
            samples = [json.loads(line) for line in f if line.strip()]
        print(json.dumps(analyze(samples), indent=2))
    except (OSError, ValueError, KeyError) as e:
        sys.exit(f"analyze.py: {e}")


if __name__ == "__main__":
    main(sys.argv)
