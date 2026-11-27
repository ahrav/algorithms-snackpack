#!/usr/bin/env python3
"""Run six balanced process rounds and retain samples and control data."""

from bisect import bisect_left, bisect_right
import csv
import itertools
import json
import os
from pathlib import Path
import statistics
import subprocess
import sys

CASES = (
    "tiny", "ordered", "ordered-wide-block", "shuffled",
    "bimodal-gap", "bimodal-hit", "wide",
)
CANDIDATES = ("full", "minmax", "two")
FIELDS = ("build_ns", "query_ns", "repeats", "bytes", "matches", "summaries", "blocks", "rows")


def oracle(case):
    size = 64 if case == "tiny" else 1 << 18
    width = {"tiny": 16, "ordered-wide-block": 2048}.get(case, 256)
    rows = list(range(size))
    if case.startswith("bimodal"):
        rows = sorted((i % 64) + 4096 * ((i // 64) % 2) for i in rows)
    queries = {
        "tiny": [(5 * i, 5 * i + 2) for i in range(8)],
        "bimodal-gap": [(1000 + 37 * i, 1012 + 37 * i) for i in range(8)],
        "bimodal-hit": [(3 * i, 3 * i + 1) for i in range(8)],
        "wide": [(2048 * i, 230000 + 1024 * i) for i in range(8)],
    }.get(case, [(10123 + 25037 * i, 10135 + 25037 * i) for i in range(8)])
    matches = [bisect_right(rows, hi) - bisect_left(rows, lo) for lo, hi in queries]
    return size, width, matches


def attempt(binary, case, kind, round_id, name, expected):
    stdout = subprocess.check_output([binary, name, case], text=True).strip()
    exact, counters = stdout.splitlines()
    values = list(map(int, counters.split(",")))
    assert len(values) == len(FIELDS), stdout
    row = dict(zip(FIELDS, values))
    size, width, matches = expected
    assert list(map(int, exact.split(","))) == matches, (case, name, exact, matches)
    repeats = row["repeats"]
    matches = sum(matches)
    blocks = (size + width - 1) // width
    assert row["matches"] == matches * repeats, (case, name, row, matches)
    assert matches * repeats <= row["rows"] <= size * 8 * repeats
    if name == "full":
        assert row["rows"] == size * 8 * repeats and row["summaries"] == row["bytes"] == 0
    else:
        assert row["summaries"] == blocks * 8 * repeats
        assert row["blocks"] <= row["summaries"]
        assert row["bytes"] == blocks * (8 if name == "minmax" else 16)
    return dict(case=case, kind=kind, round=round_id, candidate=name, **row)


def summarize(rows):
    result = {}
    for case in CASES:
        cell = [row for row in rows if row["case"] == case]
        ordered = {
            name: [row for row in cell if row["kind"] == "timed" and row["candidate"] == name]
            for name in CANDIDATES
        }
        times = {
            name: [r["query_ns"] / (8 * r["repeats"]) for r in samples]
            for name, samples in ordered.items()
        }
        controls = [row["query_ns"] for row in cell if row["kind"] == "aa"]
        aa = max(abs(a - b) / max(a, b) for a, b in zip(controls[::2], controls[1::2]))
        limit = max(0.05, aa)
        medians = {name: statistics.median(t) for name, t in times.items()}
        best = min(medians, key=medians.get)
        comparisons = {}
        for name in CANDIDATES:
            if name != best:
                paired_savings = [1 - a / b for a, b in zip(times[best], times[name])]
                comparisons[name] = [statistics.median(paired_savings), sum(x > 0 for x in paired_savings)]
        winner = best if all(saving > limit and wins >= 5 for saving, wins in comparisons.values()) else "unresolved"
        result[case] = dict(
            winner=winner, best_median=best, aa_max=aa, comparisons=comparisons,
            candidates={
                name: dict(query_ns=[min(t), medians[name], max(t)],
                           build_ns=statistics.median(r["build_ns"] for r in ordered[name]),
                           rows_per_query=ordered[name][0]["rows"] // (8 * ordered[name][0]["repeats"]),
                           matches_per_batch=ordered[name][0]["matches"] // ordered[name][0]["repeats"],
                           summary_bytes=ordered[name][0]["bytes"])
                for name, t in times.items()
            })
    return result


def main():
    output = Path(sys.argv[1]).resolve()
    output.mkdir(parents=True, exist_ok=False)
    root = Path(__file__).resolve().parents[1]
    build = subprocess.run(
        [os.environ.get("CARGO", "cargo"), "build", "--release", "--bench", "scan",
         "--message-format=json", "--manifest-path", str(root / "Cargo.toml")],
        cwd=root, check=True, text=True, stdout=subprocess.PIPE,
    )
    (output / "cargo-build.jsonl").write_text(build.stdout)
    artifacts = [json.loads(line) for line in build.stdout.splitlines()]
    binary, = [a["executable"] for a in artifacts
               if a["reason"] == "compiler-artifact" and a["executable"]
               and a["target"]["kind"] == ["bench"]]
    (output / "binary-path").write_text(binary + "\n")
    permutations = list(itertools.permutations(CANDIDATES))
    rows = []
    for case_index, case in enumerate(CASES):
        expected = oracle(case)
        for name in CANDIDATES:
            rows.append(attempt(binary, case, "warm", -1, name, expected))
        for round_id in range(6):
            order = permutations[(round_id + case_index) % len(permutations)]
            assignment = [("timed", name) for name in order]
            control = [("aa", "minmax")] * 2
            if round_id % 2:
                assignment = control + assignment
            else:
                assignment += control
            for kind, name in assignment:
                rows.append(attempt(binary, case, kind, round_id, name, expected))
    with (output / "samples.csv").open("w", newline="") as file:
        writer = csv.DictWriter(file, fieldnames=rows[0].keys())
        writer.writeheader()
        writer.writerows(rows)
    (output / "summary.json").write_text(json.dumps(summarize(rows), indent=2) + "\n")


if __name__ == "__main__":
    main()
