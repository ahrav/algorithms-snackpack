#!/usr/bin/env python3
"""Schedule, selection and schema checks for analyze.py; run: python3 measurements/test_analyze.py"""
import json
import pathlib
import sys

import analyze

HERE = pathlib.Path(__file__).resolve().parent
RESULT_KEYS = ["workload", "medians_ns", "fastest_median", "selected", "ranges_ns", "paired_ratio_ranges", "diagnostics"]


def synthetic(ns):
    """One full campaign; ns(workload, candidate, block) supplies the timing."""
    for block, workload, candidate in analyze.schedule():
        yield {"workload": workload, "candidate": candidate, "ns": ns(workload, candidate, block), "reps": 1,
               "ops": 1, "setup_oracle_ns": 0, "checksum": 0, "lookup_probe_sum": 0, "start_deleted": 0,
               "rebuilds": 0, "reinserted": 0, "shifted": 0, "oracle": True, "block": block,
               "order": list(analyze.ORDERS[block]), "process_elapsed_ns": 0}


def expect_rejection(samples, label):
    try:
        analyze.analyze(samples)
    except ValueError:
        return
    raise AssertionError(f"{label}: analyze accepted the campaign")


def main():
    runner = (HERE / "runner.py").read_text()
    assert f"workloads={str(analyze.WORKLOADS).replace(' ', '')}" in runner, "WORKLOADS drifted from runner.py"
    assert "orders=list(itertools.permutations(['lazy','rebuild','shift']))*2" in runner, "ORDERS drifted from runner.py"
    assert len(list(analyze.schedule())) == 360

    # A minimum paired lazy/shift ratio of exactly 1.05 leaves `selected` unresolved although shift has the fastest median.
    rank = {"lazy": 1.05, "rebuild": 1.5, "shift": 1.0}
    full = list(synthetic(lambda w, c, b: 1000 * rank[c] + (10 if c == "lazy" and b == 7 else 0)))
    out = analyze.analyze(full)
    assert out["processes"] == 360 and [r["workload"] for r in out["results"]] == analyze.WORKLOADS
    assert all(list(r) == RESULT_KEYS for r in out["results"]), out["results"][0].keys()
    tiny = out["results"][0]
    assert tiny["fastest_median"] == "shift" and tiny["selected"] == "unresolved", tiny
    assert tiny["paired_ratio_ranges"]["lazy"][0] == 1.05 and "shift" not in tiny["paired_ratio_ranges"], tiny
    assert tiny["ranges_ns"]["lazy"] == [1050, 1060] and tiny["medians_ns"]["lazy"] == 1050, tiny
    committed = json.loads((HERE / "results.json").read_text())["arm"]["results"][0]
    assert list(committed) == RESULT_KEYS and list(committed["diagnostics"]["lazy"]) == analyze.DIAGNOSTICS

    selected = analyze.analyze(list(synthetic(lambda w, c, b: 1000 * rank[c] + (1 if c != "shift" else 0))))
    assert all(r["selected"] == "shift" for r in selected["results"]), selected["results"][0]

    expect_rejection(full[:359], "missing final process")
    expect_rejection(full[:30], "single block")
    expect_rejection(full + [full[100]], "extra duplicate process")
    expect_rejection(full[:100] + [full[101], full[100]] + full[102:], "swapped candidates")
    expect_rejection([dict(s, order=list(reversed(s["order"]))) for s in full], "order mismatch")
    expect_rejection([dict(s, oracle=False) if s["block"] == 11 and s["workload"] == "build" else s for s in full], "oracle failure")
    expect_rejection([dict(s, ns=0) if s["block"] == 0 else s for s in full], "zero timing")
    expect_rejection([dict(s, rebuilds=11) if s["workload"] == "build" and s["block"] == 3 else s for s in full], "diagnostics differ")
    print(json.dumps({"processes": out["processes"], "cells": len(out["results"]), "status": "ok"}))


if __name__ == "__main__":
    sys.exit(main())
