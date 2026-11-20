#!/usr/bin/env python3
"""Run fixed fresh-process blocks and summarize paired wall-clock measurements."""
import csv
import sys
from pathlib import Path
from statistics import median
from subprocess import check_output

CANDIDATES = ("cursor", "ranks", "tiles", "runs")
CELLS = [(16, "50")] + [
    (size, shape)
    for size in (4096, 1048576)
    for shape in ("1", "50", "99", "alternate", "long")
]
ORDERS = [
    group[i:] + group[:i]
    for group in (CANDIDATES, CANDIDATES[::-1])
    for i in range(4)
]


def sample(binary, candidate, n, shape, repeats):
    result = check_output([binary, candidate, str(n), shape, str(repeats)], text=True)
    count, runs, nsecs = map(int, result.strip().split(","))
    assert 0 <= runs <= count <= n and nsecs > 0
    return count, runs, nsecs / repeats


def main():
    binary, result_path = sys.argv[1:]
    result = Path(result_path)
    result.mkdir(parents=True, exist_ok=True)
    with (result / "samples.csv").open("x") as file, (result / "summary.tsv").open("x") as summary:
        writer = csv.writer(file)
        writer.writerow(["size", "shape", "phase", "block", "position", "candidate",
                         "iterations", "selected", "runs", "ns_per_call"])
        print("size", "shape", "selected", "runs", "winner", *CANDIDATES, "max_win_ratio",
              sep="\t", file=summary)
        for size, shape in CELLS:
            repeats = max(4, min(32768, 4194304 // size))
            samples = {candidate: [] for candidate in CANDIDATES}
            identities = set()
            for block, order in enumerate([CANDIDATES] + ORDERS, start=-1):
                for pos, candidate in enumerate(order):
                    count, runs, elapsed = sample(binary, candidate, size, shape, repeats)
                    phase = "warmup" if block < 0 else "measured"
                    writer.writerow([size, shape, phase, block, pos, candidate, repeats,
                                     count, runs, elapsed])
                    identities.add((count, runs))
                    if block >= 0:
                        samples[candidate].append(elapsed)
                file.flush()
            (count, runs), = identities
            assert all(len(values) == 8 for values in samples.values())
            winner = min(samples, key=lambda candidate: median(samples[candidate]))
            ratios = [
                first / second for rival in CANDIDATES if rival != winner
                for first, second in zip(samples[winner], samples[rival])
            ]
            print(size, shape, count, runs, winner if max(ratios) <= 0.95 else "unresolved",
                  *(f"{median(samples[c]):.1f}[{min(samples[c]):.1f},{max(samples[c]):.1f}]"
                    for c in CANDIDATES), f"{max(ratios):.4f}", sep="\t", file=summary)
            summary.flush()
        with (result / "aa.tsv").open("x") as aa:
            print("size", "block", "first_ns", "second_ns", "ratio_A_over_B", sep="\t", file=aa)
            for size in (16, 1048576):
                repeats = max(4, min(32768, 4194304 // size))
                for block in range(8):
                    _, _, first = sample(binary, "cursor", size, "50", repeats)
                    _, _, second = sample(binary, "cursor", size, "50", repeats)
                    ratio = first / second if block % 2 == 0 else second / first
                    print(size, block, first, second, f"{ratio:.4f}", sep="\t", file=aa)


if __name__ == "__main__":
    main()
