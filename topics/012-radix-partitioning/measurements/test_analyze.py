#!/usr/bin/env python3
"""Schedule-completeness checks for analyze.py; run: python3 measurements/test_analyze.py"""
import json
import sys

import analyze
import runner


def synthetic():
    for cell, n, bits, shape, width, reps, cold in runner.WORKLOADS:
        for block, order in enumerate(runner.ORDERS):
            for position, name in enumerate(order):
                yield {"candidate": name, "n": n, "bits": bits, "shape": shape, "width": width, "reps": reps,
                       "cold": cold, "elapsed_ns": 1000 * reps * (1 + runner.CANDIDATES.index(name)) + block,
                       "warmup_ns": 0, "checksum": 0, "oracle": "pass", "cell": cell, "block": block,
                       "position": position, "process_ns": 0}


def expect_rejection(samples, label):
    try:
        analyze.analyze(samples)
    except ValueError:
        return
    raise AssertionError(f"{label}: analyze accepted an incomplete schedule")


def main():
    full = list(synthetic())
    out = analyze.analyze(full)
    assert set(out) == {w[0] for w in runner.WORKLOADS}, sorted(out)
    assert all(c["contracts"]["stable"]["median_leader"] == "buckets" for c in out.values())
    expect_rejection(full[: 4 * len(runner.ORDERS)], "single cell")
    expect_rejection(full[:4], "single block")
    expect_rejection([s for s in full if not (s["cell"] == "wide" and s["block"] == 11)], "missing final block")
    expect_rejection([s for s in full if not (s["block"] == 3 and s["candidate"] == "cycles")], "missing candidate")
    swapped = [dict(s, position=(s["position"] + 1) % 4) for s in full]
    expect_rejection(swapped, "position mismatch")
    bad_reps = [dict(s, reps=s["reps"] * 2) if s["cell"] == "tiny" and s["block"] == 5 and s["candidate"] == "scatter" else s for s in full]
    expect_rejection(bad_reps, "reps mismatch")
    bad_shape = [dict(s, shape="uniform") if s["cell"] == "skew90" and s["block"] == 0 else s for s in full]
    expect_rejection(bad_shape, "shape mismatch")
    bad_cold = [dict(s, cold=False) if s["cell"] == "first_call" else s for s in full]
    expect_rejection(bad_cold, "cold mismatch")
    print(json.dumps({"cells": len(out), "blocks": len(runner.ORDERS), "status": "ok"}))


if __name__ == "__main__":
    sys.exit(main())
