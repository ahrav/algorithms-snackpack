"""Exact certificates and balanced benchmark for isotonic.rs."""

import argparse
import itertools
import json
import os
import random
import statistics
import subprocess
from fractions import Fraction
from pathlib import Path

ALGORITHMS = ("shift", "stack", "hull")
CELLS = (
    (16, "up", 8000),
    (16, "noise", 8000),
    (512, "up", 500),
    (512, "plateau", 500),
    (512, "fall", 500),
    (512, "noise", 500),
    (4096, "up", 64),
    (4096, "plateau", 64),
    (4096, "fall", 64),
    (4096, "noise", 64),
)


def execute(binary, *args, data=None):
    return subprocess.run(
        [binary, *map(str, args)], input=data, text=True, check=True, capture_output=True
    ).stdout


def partitions_oracle(values):
    """List every consecutive partition. Minimize exact squared error."""
    n = len(values)
    if not n:
        return []
    best_loss = None
    best_fits = set()
    for mask in range(1 << (n - 1)):
        cuts = [0] + [j for j in range(1, n) if mask & (1 << (j - 1))] + [n]
        fit = []
        for left, right in zip(cuts, cuts[1:]):
            avg = Fraction(sum(values[left:right]), right - left)
            fit.extend([avg] * (right - left))
        if any(a > b for a, b in zip(fit, fit[1:])):
            continue
        loss = sum((y - x) ** 2 for y, x in zip(values, fit))
        if best_loss is None or loss < best_loss:
            best_loss = loss
            best_fits = {tuple(fit)}
        elif loss == best_loss:
            best_fits.add(tuple(fit))
    assert len(best_fits) == 1, "distinct optima violate strict convexity"
    return list(best_fits.pop())


def certify(values, blocks):
    """Check monotonicity, total residual and all prefix constraints exactly."""
    cursor = 0
    expanded = []
    previous = None
    for total, count in blocks:
        assert 1 <= count <= len(values) - cursor
        level = Fraction(total, count)
        assert previous is None or previous < level
        partial = 0
        for length, value in enumerate(values[cursor : cursor + count], 1):
            partial += value
            assert count * partial >= length * total
        assert partial == total
        expanded.extend([level] * count)
        previous = level
        cursor += count
    assert cursor == len(values)
    return expanded


def fingerprint(blocks):
    mask = (1 << 64) - 1
    result = 0xCBF29CE484222325
    for total, count in blocks:
        result = ((result ^ (total & mask)) * 0x100000001B3) & mask
        result ^= count
    return result


def check(binary, workloads):
    edge = [
        [6, 2, 3, 8],
        [-(1 << 31), (1 << 31) - 1],
        [(1 << 31) - 1, -(1 << 31)],
        [(1 << 31) - 1] * 8,
        [-(1 << 31)] * 8,
    ]
    exhaustive = [
        list(row) for n in range(7) for row in itertools.product((-2, 0, 2), repeat=n)
    ]
    rng = random.Random(1121)
    random_rows = [[rng.randrange(-50, 51) for _ in range(8)] for _ in range(80)]
    small = edge + exhaustive + random_rows
    expected = [partitions_oracle(row) for row in small]
    limits = [[-(1 << 31)] * 32768, [(1 << 31) - 1] * 16384 + [-(1 << 31)] * 16384]
    cases = small + limits + workloads
    data = "".join(" ".join(map(str, row)) + "\n" for row in cases)
    encoded = {}
    for name in ALGORITHMS:
        lines = execute(binary, "fit", name, data=data).splitlines()
        assert len(lines) == len(cases)
        encoded[name] = []
        for i, (values, line) in enumerate(zip(cases, lines)):
            blocks = [tuple(map(int, item.split(":"))) for item in line.split()]
            fitted = certify(values, blocks)
            if i < len(small):
                assert fitted == expected[i], (name, values, blocks)
            encoded[name].append(blocks)
    assert encoded["shift"] == encoded["stack"] == encoded["hull"]
    return {
        "per_algorithm": len(cases),
        "exhaustive_partition_oracle_per_algorithm": len(small),
        "large_certificates_per_algorithm": len(limits) + len(workloads),
        "workloads": [
            {"blocks": len(row), "fingerprint": fingerprint(row)}
            for row in encoded["stack"][-len(workloads) :]
        ],
    }


def run_cell(binary, index, cell, expected_digest):
    n, shape, repetitions = cell

    def run(name):
        elapsed, digest = map(
            int, execute(binary, "bench", name, n, shape, repetitions).split()
        )
        assert digest == expected_digest, (cell, name, digest)
        return elapsed / repetitions

    warmup = {name: run(name) for name in ALGORITHMS}
    paired = []
    aa = []
    orders = list(itertools.permutations(ALGORITHMS))
    for block in range(6):
        order = orders[(block + index) % 6]
        pair = {}
        for name in order:
            pair[name] = run(name)
        paired.append({"order": list(order), "ns_per_fit": pair})
        control = [run("stack"), run("stack")]
        if block % 2:
            control.reverse()
        aa.append(control[0] / control[1])

    series = {name: [b["ns_per_fit"][name] for b in paired] for name in ALGORITHMS}
    summary = {
        name: {
            "median": statistics.median(values),
            "min": min(values),
            "max": max(values),
        }
        for name, values in series.items()
    }
    fastest = min(ALGORITHMS, key=lambda name: summary[name]["median"])
    worst_margin = min(
        rival / target
        for name in ALGORITHMS
        if name != fastest
        for rival, target in zip(series[name], series[fastest])
    )
    stable_control = all(1 / 1.05 <= ratio <= 1.05 for ratio in aa)
    winner = fastest if worst_margin >= 1.05 and stable_control else None
    return {
        "n": n,
        "shape": shape,
        "repetitions_per_process": repetitions,
        "warmup_discard_ns_per_fit": warmup,
        "paired": paired,
        "aa_stack_ratios": aa,
        "stable_control": stable_control,
        "ns_per_fit": summary,
        "lowest_median": fastest,
        "worst_pair_margin": worst_margin,
        "winner": winner,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=lambda path: str(Path(path).resolve()))
    parser.add_argument("--check-only", action="store_true")
    args = parser.parse_args()
    if not args.check_only and hasattr(os, "sched_setaffinity"):
        os.sched_setaffinity(0, {min(os.sched_getaffinity(0))})
    workloads = [list(map(int, execute(args.binary, "input", n, shape).split()))
                 for n, shape, _ in CELLS]
    checks = check(args.binary, workloads)
    report = {
        "correctness": checks,
        "affinity": sorted(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else None,
    }
    if not args.check_only:
        report["cells"] = [
            run_cell(args.binary, index, cell, verified["fingerprint"])
            for index, (cell, verified) in enumerate(zip(CELLS, checks["workloads"]))
        ]
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
