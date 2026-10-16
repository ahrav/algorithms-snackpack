"""Frozen two-pass process campaign: four candidates, then an identical-artifact A/A control."""

import hashlib
import json
import os
import pathlib
import platform
import shutil
import statistics
import subprocess

ROOT = pathlib.Path(__file__).resolve().parent
OUT = ROOT / "raw"
ATTEMPTS = OUT / "attempts.jsonl"
TIMEOUT = 600
FILES = ["src/lib.rs", "tests/contracts.rs", "benches/portfolio.rs", "runner.py"]
CANDIDATES = ["scan", "prefix", "eager", "lazy"]
CONTROL = ["lazy_a", "lazy_b"]
CELLS = ["tiny", "read_only", "mixed", "short", "update_heavy", "hot", "adversarial", "lifecycle"]
OPT = ["rustc", "--edition", "2024", "-O"]


def call(argv):
    """Run one subprocess, append its record to the attempt log, and return stdout on success."""
    try:
        r = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True, timeout=TIMEOUT, check=False)
        record = {"argv": argv, "exit": r.returncode, "stdout": r.stdout, "stderr": r.stderr}
    except subprocess.TimeoutExpired as e:
        record = {"argv": argv, "exit": "timeout", "stdout": str(e.stdout), "stderr": str(e.stderr)}
    except OSError as e:
        record = {"argv": argv, "exit": "launch", "stdout": "", "stderr": repr(e)}
    with ATTEMPTS.open("a") as f:
        f.write(json.dumps(record) + "\n")
    if record["exit"] != 0:
        raise RuntimeError(json.dumps(record))
    return record["stdout"]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build():
    manifest = {
        "inputs": {f: sha(ROOT / f) for f in FILES},
        "hostname": platform.node(),
        "arch": platform.machine(),
        "uname": platform.uname()._asdict(),
        "rustc": call(["rustc", "-Vv"]),
        "features": call(["rustc", "--print", "cfg", "-O"]),
        "cpus": os.cpu_count(),
        "affinity": sorted(os.sched_getaffinity(0)),
        "cpu": call(["lscpu"]),
        "flags": "--edition 2024 -O; default target; no target-cpu=native; unpinned",
        "timeout_s": TIMEOUT,
    }
    call(["rustc", "--edition", "2024", "--test", "src/lib.rs", "-o", str(OUT / "tests")])
    (OUT / "tests.log").write_text(call([str(OUT / "tests")]))
    rlib = OUT / "libsegment_trees.rlib"
    call(OPT + ["--crate-name", "segment_trees", "--crate-type", "rlib", "src/lib.rs", "-o", str(rlib)])
    ext = ["--extern", f"segment_trees={rlib}"]
    call(OPT + ["--test", "tests/contracts.rs"] + ext + ["-o", str(OUT / "contracts")])
    (OUT / "contracts.log").write_text(call([str(OUT / "contracts")]))
    call(OPT + ["benches/portfolio.rs"] + ext + ["-o", str(OUT / "portfolio")])
    manifest["linked_image_sha256"] = sha(OUT / "portfolio")
    (OUT / "manifest.json").write_text(json.dumps(manifest, indent=2))
    (OUT / "disassembly.txt").write_text(call(["objdump", "-d", str(OUT / "portfolio")]))


def sample(label, artifact, cell, block):
    stdout = call([str(OUT / "portfolio"), artifact, cell])
    try:
        row = json.loads(stdout)
    except json.JSONDecodeError as e:
        raise RuntimeError(f"{artifact} {cell} block {block}: unparsable stdout {stdout!r}") from e
    row.update(cell=cell, block=block, candidate=label)
    return row


def candidate_rows():
    rows = []
    for cell in CELLS:
        for block in range(8):
            order = CANDIDATES[block % 4 :] + CANDIDATES[: block % 4]
            if block >= 4:
                order = list(reversed(order))
            rows += [sample(c, c, cell, block) for c in order]
    return rows


def control_rows():
    rows = []
    for cell in CELLS:
        for block in range(8):
            order = CONTROL if block % 2 == 0 else list(reversed(CONTROL))
            rows += [sample(label, "lazy", cell, block) for label in order]
    return rows


def analyze(samples, labels):
    summary = {}
    for cell in CELLS:
        data = {c: [r["ns"] for r in samples if r["cell"] == cell and r["candidate"] == c] for c in labels}
        checksums = {r["checksum"] for r in samples if r["cell"] == cell}
        if len(checksums) != 1:
            raise RuntimeError(f"{cell}: checksums differ {sorted(checksums)}")
        med = {c: statistics.median(v) for c, v in data.items()}
        best = min(med.items(), key=lambda kv: kv[1])[0]
        ratios = {c: [data[c][i] / data[best][i] for i in range(8)] for c in labels if c != best}
        winner = best if all(min(v) > 1.05 for v in ratios.values()) else "unresolved"
        summary[cell] = {
            "winner": winner,
            "median_ns": med,
            "range_ns": {c: [min(v), max(v)] for c, v in data.items()},
            "paired_against_median_best": {
                c: {"median": statistics.median(v), "range": [min(v), max(v)]} for c, v in ratios.items()
            },
        }
    return summary


def select(candidates, control):
    """Apply the BENCHMARK.md rule: a control cell with a winner voids every candidate label."""
    failed = [c for c in CELLS if control[c]["winner"] != "unresolved"]
    summary = {
        cell: dict(s, winner="invalid" if failed else s["winner"], uncontrolled_winner=s["winner"])
        for cell, s in candidates.items()
    }
    summary["aa_control"] = dict(control, failed_cells=failed)
    return summary


def main():
    try:
        shutil.rmtree(OUT, ignore_errors=True)
        OUT.mkdir()
    except OSError as e:
        raise RuntimeError(f"cannot reset {OUT}: {e}") from e
    ATTEMPTS.write_text("")
    build()
    rows = candidate_rows()
    control = control_rows()
    summary = select(analyze(rows, CANDIDATES), analyze(control, CONTROL))
    (OUT / "rows.json").write_text(json.dumps(rows + control, indent=2))
    (OUT / "summary.json").write_text(json.dumps(summary, indent=2))
    report = {
        "processes": len(rows),
        "control_processes": len(control),
        "winners": {c: s["winner"] for c, s in summary.items() if c != "aa_control"},
        "aa_control_failed_cells": summary["aa_control"]["failed_cells"],
    }
    print(json.dumps(report))


if __name__ == "__main__":
    main()
