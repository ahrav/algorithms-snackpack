"""Eight paired, order-balanced process blocks; all raw observations retained."""
import hashlib
import json
import pathlib
import platform
import random
import statistics
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT = pathlib.Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else ROOT / "runs"
OUT.mkdir(parents=True, exist_ok=True)
TIMEOUT = 600
KINDS = ["flat", "tree", "skip"]
CELLS = ["tiny", "lookup", "rank", "update", "ties", "range", "build_ascending", "build_descending"]

def cmd(args):
    return subprocess.check_output(args, text=True, stderr=subprocess.STDOUT)

metadata = {"hostname": platform.node(), "architecture": platform.machine(), "uname": list(platform.uname()), "rustc": cmd(["rustc", "-Vv"]), "cpu": cmd(["lscpu"]), "available": cmd(["nproc"]), "affinity": cmd(["taskset", "-pc", str(__import__("os").getpid())]), "flags": "--edition=2024 -C opt-level=3 -C target-cpu=native", "target_features": cmd(["rustc", "--print", "cfg", "-C", "opt-level=3", "-C", "target-cpu=native"]), "files": {p: hashlib.sha256((ROOT / p).read_bytes()).hexdigest() for p in ["src/lib.rs", "tests/public_contract.rs", "benches/ordered.rs", "scripts/run.py"]}, "seed": 22018, "timeout_s": TIMEOUT, "blocks": 8, "warmups": 1, "subsamples": 3, "boundary": "steady: setup/drop excluded; build: construction+first20-range+fold+drop included; all: output folding included", "selection": "lowest median, resolved only if all 8 paired competitor/winner ratios exceed 1.03; descriptive ranges, no confidence or generalization claim"}
(OUT / "metadata.json").write_text(json.dumps(metadata, indent=2) + "\n")
subprocess.run(["rustc", "--edition=2024", "-C", "opt-level=3", "-C", "target-cpu=native", str(ROOT / "benches/ordered.rs"), "-o", str(OUT / "ordered-bench")], check=True)
subprocess.run(["rustc", "--edition=2024", "--test", str(ROOT / "src/lib.rs"), "-o", str(OUT / "unit-tests")], check=True)
subprocess.run(["rustc", "--edition=2024", "--crate-type", "lib", str(ROOT / "src/lib.rs"), "-o", str(OUT / "libtopic022.rlib")], check=True)
subprocess.run(["rustc", "--edition=2024", "--test", str(ROOT / "tests/public_contract.rs"), "--extern", "topic022_skip_lists_and_hybrid_indexes=" + str(OUT / "libtopic022.rlib"), "-o", str(OUT / "contract-tests")], check=True)
tests = [subprocess.run([str(OUT / name)], text=True, capture_output=True) for name in ["unit-tests", "contract-tests"]]
(OUT / "tests.txt").write_text("".join(t.stdout + t.stderr for t in tests))
for t in tests:
    t.check_returncode()
metadata["binary_sha256"] = hashlib.sha256((OUT / "ordered-bench").read_bytes()).hexdigest()
(OUT / "metadata.json").write_text(json.dumps(metadata, indent=2) + "\n")
rows = []
schedule = []
rng = random.Random(22018)
for block in range(8):
    cells = CELLS[:]
    rng.shuffle(cells)
    for cell in cells:
        offset = (block // 2) % len(KINDS)
        order = KINDS[offset:] + KINDS[:offset]
        if block % 2: order.reverse()
        for label in order + (["aa_left", "aa_right"] if block % 2 == 0 else ["aa_right", "aa_left"]):
            schedule.append({"block": block, "cell": cell, "label": label, "candidate": "tree" if label.startswith("aa_") else label})
(OUT / "schedule.json").write_text(json.dumps(schedule, indent=2) + "\n")
with (OUT / "raw.jsonl").open("w") as raw, (OUT / "attempts.jsonl").open("w") as attempts:
    for assignment in schedule:
        started = time.monotonic()
        try:
            proc = subprocess.run([str(OUT / "ordered-bench"), assignment["candidate"], assignment["cell"]], text=True, capture_output=True, timeout=TIMEOUT)
        except subprocess.TimeoutExpired as e:
            attempt = dict(assignment, stdout=str(e.stdout), stderr=str(e.stderr), returncode="timeout", wall_seconds=time.monotonic() - started)
            attempts.write(json.dumps(attempt) + "\n"); attempts.flush()
            raise RuntimeError(attempt) from e
        attempt = dict(assignment, stdout=proc.stdout, stderr=proc.stderr, returncode=proc.returncode, wall_seconds=time.monotonic() - started)
        attempts.write(json.dumps(attempt) + "\n"); attempts.flush()
        if proc.returncode: raise RuntimeError(attempt)
        row = dict(json.loads(proc.stdout), **assignment)
        rows.append(row); raw.write(json.dumps(row) + "\n"); raw.flush()
summary = {}
for cell in CELLS:
    group = [r for r in rows if r["cell"] == cell]
    if len({r["checksum"] for r in group}) != 1:
        raise RuntimeError({"cell": cell, "checksums": sorted({r["checksum"] for r in group})})
    stats = {}
    for label in KINDS + ["aa_left", "aa_right"]:
        samples = [r["ns"] for r in group if r["label"] == label]
        stats[label] = {"median_ns": statistics.median(samples), "min_ns": min(samples), "max_ns": max(samples)}
    winner = min(KINDS, key=lambda k: stats[k]["median_ns"])
    ratios = {}
    for rival in KINDS:
        if rival == winner: continue
        paired = [next(r["ns"] for r in group if r["block"] == b and r["label"] == rival) / next(r["ns"] for r in group if r["block"] == b and r["label"] == winner) for b in range(8)]
        ratios[rival] = {"median": statistics.median(paired), "min": min(paired), "max": max(paired)}
    aa = [next(r["ns"] for r in group if r["block"] == b and r["label"] == "aa_right") / next(r["ns"] for r in group if r["block"] == b and r["label"] == "aa_left") for b in range(8)]
    summary[cell] = {"lowest_median": winner, "resolved": all(v["min"] > 1.03 for v in ratios.values()), "unresolved_rivals": [k for k, v in ratios.items() if v["min"] <= 1.03], "stats": stats, "paired_rival_over_best": ratios, "aa_right_over_left": {"median": statistics.median(aa), "min": min(aa), "max": max(aa)}}
(OUT / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
print(json.dumps({"hostname": platform.node(), "processes": len(rows), "oracle": "all pass", "cells": {c: {k: v[k] for k in ["lowest_median", "resolved", "unresolved_rivals"]} for c,v in summary.items()}}))
