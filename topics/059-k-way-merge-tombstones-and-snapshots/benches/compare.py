#!/usr/bin/env python3
"""Run a fixed, position-balanced schedule of independent processes."""

import argparse
import json
from pathlib import Path
import statistics
import subprocess
import time

CASES = {
    "tiny": 3000,
    "low-fan-in": 32,
    "mixed-8": 32,
    "mixed-32": 32,
    "overlap-32": 32,
    "skew-64": 32,
    "disjoint-32": 32,
}
ARMS = ("replay", "flat", "heap", "loser")
ORDERS = ((0, 1, 3, 2), (1, 2, 0, 3), (2, 3, 1, 0), (3, 0, 2, 1))
BLOCKS = ORDERS + ORDERS[::-1]


def invoke(binary, *args):
    return subprocess.run([binary, *args], text=True, capture_output=True, timeout=45)


def run(binary, output):
    with output.open("x") as stream:
        def record(row):
            print(json.dumps(row, separators=(",", ":")), file=stream, flush=True)

        for case in CASES:
            result = invoke(binary, "check", case)
            record({"phase": "check", "case": case, "exit": result.returncode,
                    "stdout": result.stdout, "stderr": result.stderr})
            result.check_returncode()
            json.loads(result.stdout)

        def sample(phase, case, block, position, label, arm):
            start = time.monotonic_ns()
            result = invoke(binary, "bench", case, arm, str(CASES[case]))
            wall = time.monotonic_ns() - start
            record({"phase": phase, "case": case, "block": block, "position": position,
                    "label": label, "arm": arm, "repeats": CASES[case], "exit": result.returncode,
                    "stdout": result.stdout, "stderr": result.stderr, "wall_ns": wall})
            result.check_returncode()

        for case in CASES:
            for position, arm in enumerate(ARMS):
                sample("warmup", case, 0, position, arm, arm)
            for block, order in enumerate(BLOCKS):
                for position, index in enumerate(order):
                    arm = ARMS[index]
                    sample("timed", case, block, position, arm, arm)
                for position, label in enumerate(("A", "B")[::1 if block % 2 == 0 else -1]):
                    sample("control", case, block, position, label, "heap")


def spread(values):
    lower, _, upper = statistics.quantiles(values, n=4, method="inclusive")
    return [statistics.median(values), lower, upper]


def summarize(output):
    samples = {}
    check = {}
    with output.open() as stream:
        for line in stream:
            row = json.loads(line)
            if row["phase"] == "check":
                result = json.loads(row["stdout"])
                result["returned"] = len(result.pop("rows"))
                check[row["case"]] = result
            if row["phase"] in ("timed", "control"):
                key = row["phase"], row["case"], row["label"], row["block"]
                assert row["exit"] == 0 and key not in samples
                samples[key] = int(row["stdout"]) / row["repeats"]
    stats = {}
    for case in CASES:
        arms = {arm: [samples["timed", case, arm, i] for i in range(8)] for arm in ARMS}
        pairs = {}
        winners = []
        for arm in ARMS:
            rivals = [other for other in ARMS if other != arm]
            beat = []
            for other in rivals:
                ratios = [a / b for a, b in zip(arms[arm], arms[other])]
                pairs[f"{arm}/{other}"] = [*spread(ratios), sum(r < 1 for r in ratios)]
                beat.append(statistics.median(ratios) <= 0.95 and sum(r < 1 for r in ratios) >= 6)
            if all(beat):
                winners.append(arm)
        control = [samples["control", case, "A", i] / samples["control", case, "B", i]
                   for i in range(8)]
        stats[case] = {"counts": check[case], "winner": next(iter(winners), "unresolved"),
                       "ns_median_q1_q3": {arm: spread(values) for arm, values in arms.items()},
                       "ratios_median_q1_q3_and_wins": pairs, "aa_median_q1_q3": spread(control)}
    print(json.dumps(stats, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--summary-only", action="store_true")
    args = parser.parse_args()
    if not args.summary_only:
        run(str(args.binary.resolve()), args.output)
    summarize(args.output)


if __name__ == "__main__":
    main()
