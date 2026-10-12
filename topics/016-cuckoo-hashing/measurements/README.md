# Measurements and evidence boundary

Initial frozen scratch campaigns completed396 candidate processes on each required
Linux host, all six library groups and all per-process oracles/checksums passing.
The summary step initially failed because host Python3.9 lacks zip(strict=True).
Every raw sample is retained; compatible analysis preserves the predeclared
criterion without changing the source timings. Initial input and repair identities
are distinct. External archive root:
/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-016/20261012

See initial-summary.json for exploratory precommit medians/ranges and admission.
Exact committed-source campaigns are collected separately after the source commit;
publication will include their verified receipt and results. Initial timings do
not constitute exact committed-source verification.

Slot size24 bytes is an observed target layout, not a stable Rust ABI. Query and
build sizes/mix, balances, warmup, timing boundaries, selection, controls and
limitations are frozen in ../BENCHMARK.md. Churn units are remove/reinsert pairs.
A sole admissible baseline at80percent is an admission result, not a measured
speed advantage over a successfully built cuckoo table.

Replay from this topic with a fresh output path:
python3 measurements/runner.py --out /tmp/topic016-new-run

The runner requires rustc and Python3.7+ and writes all raw samples/identities to
its explicit external output directory. Source and generated assembly are kept
with raw evidence. Correctness is independent of timings. No A/A, PMU, cold,
latency-tail, allocator-count or universal-optimal claim.
