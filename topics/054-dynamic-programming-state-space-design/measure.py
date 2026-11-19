#!/usr/bin/env python3
"""Run a balanced fresh-process schedule and validate full fixtures independently."""

import argparse
import itertools
import json
import statistics
import subprocess
import time
from pathlib import Path

CANDIDATES = ("exhaustive", "table", "rolling", "value", "frontier")
FIXTURES = {
    "tiny": 100_000,
    "tight": 3_000,
    "medium": 48,
    "small-value": 24,
    "wide-sparse": 24,
    "wide-dense": 12,
}


def enumerate_subsets(items, budget):
    # Enumerate raw subsets without DP, dominance pruning, or a row recurrence.
    totals = [(0, 0)]
    for w, v in items:
        totals += [(weight + w, value + v) for weight, value in totals]
    return max(value for weight, value in totals if weight <= budget)


def logical_work(items, budget):
    pairs = {0: 0}
    best = 0
    frontier_sizes = [1]
    merge_inputs = []
    for w, v in items:
        shifted = [(a + w, b + v) for a, b in pairs.items() if a + w <= budget]
        merge_inputs.append(len(pairs) + len(shifted))
        for weight, value in shifted:
            pairs[weight] = max(pairs.get(weight, 0), value)
        previous_best = -1
        kept = {}
        for weight, value in sorted(pairs.items()):
            if value > previous_best:
                kept[weight] = value
                previous_best = value
        pairs = kept
        frontier_sizes.append(len(pairs))
        best = previous_best
    n = len(items)
    total_value = sum(v for _, v in items)
    return {
        "frontier_score": best,
        "table_slots": (n + 1) * (budget + 1),
        "table_copy_slots": n * (budget + 1),
        "rolling_slots": budget + 1,
        "capacity_take_updates": sum(max(0, budget - w + 1) for w, _ in items),
        "value_slots": total_value + 1,
        "value_take_updates": sum(1 + prior for prior in itertools.accumulate(
            (v for _, v in items[:-1]), initial=0
        )),
        "frontier_sizes": frontier_sizes,
        "frontier_merge_inputs": sum(merge_inputs),
        "oracle_subsets": 2**n,
    }


def run(binary, candidate, fixture, repetitions):
    start = time.perf_counter_ns()
    result = subprocess.check_output([binary, candidate, fixture, str(repetitions)], text=True)
    process_ns = time.perf_counter_ns() - start
    elapsed, checksum, budget, pairs = result.strip().split("\t")
    return int(elapsed), int(checksum), int(budget), json.loads(pairs), process_ns


def describe(values):
    return {"median": statistics.median(values), "min": min(values), "max": max(values)}


def run_schedule(binary):
    result = {}
    for fixture, repetitions in FIXTURES.items():
        expected = None
        identity = None
        records = []

        def sample(candidate, phase, block, position):
            nonlocal expected, identity
            elapsed, checksum, budget, items, process_ns = run(
                binary, candidate, fixture, repetitions
            )
            if identity is None:
                identity = budget, items
                expected = enumerate_subsets(items, budget)
            assert identity == (budget, items), (fixture, budget, items)
            assert checksum == repetitions * expected, (fixture, candidate, checksum, expected)
            records.append({
                "phase": phase, "block": block, "position": position, "candidate": candidate,
                "solve_ns": elapsed / repetitions, "call_loop_ns": elapsed,
                "process_ns": process_ns, "checksum": checksum,
            })

        for position, candidate in enumerate(CANDIDATES):
            sample(candidate, "warmup", -1, position)
        for block in range(2 * len(CANDIDATES)):
            rotation = block % len(CANDIDATES)
            order = CANDIDATES[rotation:] + CANDIDATES[:rotation]
            if block >= len(CANDIDATES):
                order = order[::-1]
            for position, candidate in enumerate(order):
                sample(candidate, "measured", block, position)
            sample("rolling", "aa", block, len(order))

        measured = {name: [] for name in CANDIDATES}
        aa = []
        for record in records:
            if record["phase"] == "measured":
                measured[record["candidate"]].append(record["solve_ns"])
            if record["phase"] == "aa":
                aa.append(record["solve_ns"])
        assert len(aa) == 10 and all(len(samples) == 10 for samples in measured.values())
        timing = {name: describe(samples) for name, samples in measured.items()}
        leader = min(CANDIDATES, key=lambda name: timing[name]["median"])
        ratios = {
            name: describe([x / y for x, y in zip(measured[leader], measured[name])])
            for name in CANDIDATES if name != leader
        }
        clears_margin = all(summary["max"] <= 0.95 for summary in ratios.values())
        aa_ratio = describe([a / b for a, b in zip(aa, measured["rolling"])])
        budget, items = identity
        work = logical_work(items, budget)
        assert expected == work["frontier_score"]
        result[fixture] = {
            "capacity": budget, "items": items, "expected": expected, "repetitions": repetitions,
            "logical_work": work, "records": records, "timing_ns": timing, "median_leader": leader,
            "winner": leader if clears_margin else None, "leader_over_rivals": ratios,
            "rolling_aa": aa_ratio,
        }
        print(f"{fixture} score={expected} leader={leader} "
              f"winner={result[fixture]['winner']} AA={aa_ratio}", flush=True)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    args.output.write_text(json.dumps(run_schedule(args.binary.resolve()), indent=2) + "\n")


if __name__ == "__main__":
    main()
