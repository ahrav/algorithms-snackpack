#!/usr/bin/env python3
"""Paired log ratios and Bonferroni Student-t intervals using standard Python."""
import json
import math
from pathlib import Path
import statistics
import sys

def student_cdf(value, df, steps=4096):
    factor = math.exp(math.lgamma((df + 1) / 2) - math.lgamma(df / 2)) / math.sqrt(df * math.pi)
    def density(x):
        return factor * (1 + x * x / df) ** (-(df + 1) / 2)
    step = value / steps
    total = density(0) + density(value)
    total += 4 * sum(density(step * i) for i in range(1, steps, 2))
    total += 2 * sum(density(step * i) for i in range(2, steps, 2))
    return .5 + total * step / 3

def quantile(probability, steps=4096):
    low, high = 0., 32.
    for _ in range(45):
        mid = (low + high) / 2
        if student_cdf(mid, 11, steps) < probability:
            low = mid
        else:
            high = mid
    return (low + high) / 2

def main():
    # Published Student-t 0.975/11 value is a numerical self-check, not evidence.
    assert abs(quantile(.975) - 2.200985160082949) < 1e-8
    critical = quantile(1 - .05 / (2 * 32))
    assert abs(critical - quantile(1 - .05 / (2 * 32), 8192)) < 1e-8
    results = []
    for path in sys.argv[1:]:
        root = Path(path)
        manifest = json.loads((root / "manifest.json").read_text())
        import hashlib
        for name, expected in manifest.items():
            assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, name
        records = [json.loads(line) for line in (root / "samples.jsonl").read_text().splitlines()]
        assert len(records) == 408
        groups = {}
        for record in records:
            key = (record["cell"], record["contrast"])
            pairs = groups.setdefault(key, {})
            pair = pairs.setdefault(record["pair"], {})
            assert record["arm"] not in pair
            pair[record["arm"]] = record
        for (cell, contrast), pairs in sorted(groups.items()):
            assert len(pairs) == 12 and all(set(pair) == {"a", "b"} for pair in pairs.values())
            logs = [math.log(pair["b"]["elapsed_ns"] / pair["a"]["elapsed_ns"]) for pair in pairs.values()]
            mean = statistics.mean(logs)
            error = statistics.stdev(logs) / math.sqrt(12)
            multiplier = quantile(.975) if contrast == "aa" else critical
            interval = [math.exp(mean - multiplier * error), math.exp(mean + multiplier * error)]
            a_times = [pair["a"]["elapsed_ns"] / 16 for pair in pairs.values()]
            b_times = [pair["b"]["elapsed_ns"] / 16 for pair in pairs.values()]
            first = pairs[0]["a"]
            status = "diagnostic" if contrast == "aa" else "candidate_faster" if interval[1] < 1/1.05 else "scan_faster" if interval[0] > 1.05 else "unresolved"
            results.append({"host": json.loads((root / "host.json").read_text())["hostname"], "cell": cell, "contrast": contrast, "shape": first["shape"], "n": first["n"], "w": first["w"], "pairs": 12, "ratio": math.exp(mean), "interval": interval, "status": status, "scan_or_a_ns": {"median": statistics.median(a_times), "min": min(a_times), "max": max(a_times)}, "candidate_or_b_ns": {"median": statistics.median(b_times), "min": min(b_times), "max": max(b_times)}, "setup_ns_range": [min(record["setup_ns"] for pair in pairs.values() for record in pair.values()), max(record["setup_ns"] for pair in pairs.values() for record in pair.values())], "counts": {key: first[key] for key in ["scan_comparisons", "deque_comparisons", "block_comparisons", "peak", "burst"]}})
    print(json.dumps({"family": 32, "confidence": .95, "critical_t": critical, "results": results}, indent=2))

if __name__ == "__main__":
    main()
