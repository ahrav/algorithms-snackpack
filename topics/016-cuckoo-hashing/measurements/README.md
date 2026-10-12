# Exact committed-source measurements

Source commit `7282ff72c8c0370febb0a4cdac749ba67f925c72`. Six frozen paths match that commit; EVIDENCE_RECEIPT.json `post_measurement_changes` lists the later source edits. Both required Linux hosts completed396 candidate processes, all six library/two external groups and every prepared-map/batch oracle passing. Every selection is provisional until the identical-artifact A/A control required by METHODOLOGY.md runs through this pipeline.

| Workload | Arm selection | x86 selection |
|---|---|---|
| tiny_hit | linear | unresolved (linear, cuckoo1, cuckoo4) |
| small_hit | linear | linear |
| small_miss | linear | linear |
| hot | linear | linear |
| large_hit | linear | linear |
| large_miss | unresolved (linear, cuckoo1) | linear |
| high_hit | linear (sole admission) | linear (sole admission) |
| high_miss | linear (sole admission) | linear (sole admission) |
| build35 | linear | unresolved (linear, cuckoo1) |
| build80 | linear (sole admission) | linear (sole admission) |
| churn | linear | cuckoo1 |

At80percent requested occupancy, the measured commit's cuckoo1 rejects7,212/104,857 keys and cuckoo4 rejects25 for high queries; build80 rejects3,463/52,000 and13 respectively. Counts are identical on both hosts because layout, insertion order and budget are deterministic. Those timings are excluded. These failures demonstrate this cap/algorithm, not impossibility of another placement or policy. The current planner charges the budget only for recorded slots and keeps scanning the frontier after the cap; with it, cuckoo4 admits every high and build80 key while cuckoo1 counts are unchanged. The high_hit, high_miss and build80 "sole admission" rows describe the measured commit; a comparison that includes cuckoo4 there is unmeasured.

| Host | Compiler | CPUs available | CPU |
|---|---|---:|---|
| Declared Arm dev-dsk-ahrav-2b-7dc7bd93 | rustc1.98.1 |64| reported Arm CPU part0xd40 |
| xxl resolves dev-dsk-ahrav-2c-32182091 | rustc1.99.0 |192| Intel Xeon Platinum8488C |

Both kernels6.12.110-135.202.amzn2023 with architecture-specific suffix. Full uname/lscpu/cfg/flags/binary hash available in external manifests. Arm CPU part is the verified identifier; no unverified marketing CPU-name claim. Scheduling unpinned; versions/hardware/load differ.

Medians and all observed ranges are in results.json. For churn, units are delete/reinsert pairs; builds report complete cost amortized per inserted key. Exact x86 tiny linear range5.93–19.12ns and build35 range16.88–40.72ns prevent a winner under the frozen all-pairs rule, despite its lower median. No rerun to erase those observations.

Initial scratch comparisons remain separately in initial-summary.json and external initial-* raw. Their summary compatibility failure was repaired without changing the rows. Initial x86 tiny/build35 selected linear and large_hit unresolved; committed selections differ, all retained.

Replay from this topic:
```bash
python3 measurements/runner.py --out /tmp/topic016-fresh-run
```

Raw root: `/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-016/20261012`. Git carries compact summaries and EVIDENCE_RECEIPT.json; raw binaries/assembly/samples/logs remain external. Inspect generated assembly for code shape; no candidate-only PMU attribution. The exact benchmark protocol is ../BENCHMARK.md. No forced cold/startup, latency quantile, A/A, allocator-count, generic-payload, ISA or universal-optimal claim.
