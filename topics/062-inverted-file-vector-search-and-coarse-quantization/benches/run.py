"""Run all predeclared cells. Each command starts a fresh process."""
import csv
import json
import statistics as stat
import subprocess
import sys
from pathlib import Path

CELLS = ("tiny", "balanced", "boundary", "skew")
SLOTS = ("flat", "raw1", "raw4", "pq1", "pq4", "rr1", "rr4", "flat_b")
QUERIES = 96
TRUTH = QUERIES * 5

def process(binary, cell, mode):
    return subprocess.check_output([str(binary), cell, mode], text=True).strip()

def summary(samples, quality):
    valid = [m for m in SLOTS[:-1] if quality[m]["recall_hits"] * 100 >= TRUTH * 95]
    aa = [b / a for a, b in zip(samples["flat"], samples["flat_b"])]
    control = stat.median(aa)
    winners = []
    for candidate in valid:
        if any(
            stat.median(other / own for own, other in zip(samples[candidate], samples[competitor])) < 1.05
            or sum(own < other for own, other in zip(samples[candidate], samples[competitor])) < 13
            for competitor in valid if competitor != candidate
        ):
            continue
        winners.append(candidate)

    times = {}
    for mode in SLOTS:
        xs = samples[mode]
        q1, _, q3 = stat.quantiles(xs, n=4, method="inclusive")
        times[mode] = {"median_ns": round(stat.median(xs), 2),
                       "q1_ns": round(q1, 2), "q3_ns": round(q3, 2)}
    return {"a_a_ratio": control, "quality": quality, "eligible": valid,
            "winner": winners[0] if len(winners) == 1 and 0.95 <= control <= 1.05 else None,
            "time_per_query": times}

def main():
    binary, destination = map(Path, sys.argv[1:])
    directory = destination
    directory.mkdir(exist_ok=True)
    manifest = {}
    with (directory / "processes.csv").open("w", newline="") as out:
        writer = csv.writer(out)
        writer.writerow(("cell", "block", "position", "mode", "query_ns", "build_ns", "queries", "signature"))
        for cell in CELLS:
            qualitative = process(binary, cell, "quality")
            (directory / (cell + "-quality.csv")).write_text(qualitative + "\n")
            quality = {}
            for name, hits, p10, scanned, p95, reranked, signature in csv.reader(qualitative.splitlines()):
                quality[name] = dict(zip(
                    ("recall_hits", "p10_hits", "scanned", "scan_p95", "reranked", "signature"),
                    map(int, (hits, p10, scanned, p95, reranked, signature))))
            samples = {mode: [] for mode in SLOTS}
            for block in range(-1, 16):
                pivot = max(block, 0) % len(SLOTS)
                order = SLOTS[pivot:] + SLOTS[:pivot]
                if block >= len(SLOTS):
                    order = order[::-1]
                for position, mode in enumerate(order):
                    elapsed, build, queries, signature = map(int, process(binary, cell, mode).split(","))
                    assert signature == quality["flat" if mode == "flat_b" else mode]["signature"]
                    writer.writerow((cell, block, position, mode, elapsed, build, queries, signature))
                    out.flush()
                    if block >= 0:
                        samples[mode].append(elapsed / queries)
            manifest[cell] = summary(samples, quality)
            print(cell, manifest[cell]["winner"], flush=True)
    (directory / "summary.json").write_text(json.dumps(manifest, indent=2) + "\n")

if __name__ == "__main__":
    main()
