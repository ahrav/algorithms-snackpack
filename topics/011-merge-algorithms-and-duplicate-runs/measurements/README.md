# Exact-source results

Measured source commit `4ec5c71a1bad9462ef72bb63083a5f471a84495b`. Its four source/runner inputs match the final
published artifact by SHA-256 in EVIDENCE_RECEIPT.json. Six contracts and every workload-specific
oracle passed on both declared hosts. Each host completed672 processes across16 cells in12 paired
blocks. RESULTS.json retains geometric mean, median, range and simultaneous95% paired intervals.
An unresolved entry names the fastest point estimate and rivals it did not distinguish by the
predeclared5% gate. It does not establish equal performance.

| Workload | Arm | x86 |
|---|---|---|
| merge_cold32 | sort, 0.357 us; unresolved vs linear | sort, 0.444 us; unresolved vs linear,gallop |
| merge_warm32 | linear, 0.074 us; unresolved vs gallop | linear, 0.099 us; unresolved vs gallop,sort |
| merge_random4096 | linear, 13.441 us winner | gallop, 17.965 us; unresolved vs linear,sort |
| merge_interleaved1m | parallel4, 814.455 us winner | parallel4, 1281.636 us winner |
| merge_random1m | parallel4, 1599.907 us winner | parallel4, 2396.657 us winner |
| merge_disjoint1m | gallop, 546.731 us winner | parallel4, 1129.123 us; unresolved vs gallop |
| merge_equal1m | gallop, 569.181 us; unresolved vs parallel4 | parallel4, 1120.429 us; unresolved vs gallop |
| merge_skew1m | linear, 548.769 us; unresolved vs gallop | parallel4, 1115.479 us; unresolved vs linear,gallop |
| join_cold64 | replay, 0.574 us; unresolved vs probe | replay, 0.712 us; unresolved vs probe |
| join_sparse8192 | replay, 9.527 us winner | replay, 13.710 us winner |
| join_duplicates8192 | replay, 59.381 us winner | replay, 89.505 us winner |
| join_equal1024 | replay, 203.865 us winner | replay, 357.637 us; unresolved vs probe |
| cascade_tiny | balanced, 2.616 us; unresolved vs sort | balanced, 2.364 us; unresolved vs sort |
| cascade_4 | sort, 137.773 us winner | sort, 151.796 us winner |
| cascade_32 | balanced, 2368.451 us winner | balanced, 2311.104 us winner |
| cascade_skew | balanced, 68.859 us winner | balanced, 78.906 us winner |

Best tested selection depends on workload. Parallel-4 wins both large interleaved/random cells.
Galloping wins Arm disjoint million-row merge. Replay wins both sparse/duplicate join cells and
Arm all-equal join. Stable sort wins four-run random cascades on both hosts; adjacent balanced wins
32-run duplicate and skewed cascades. Tiny cases and several x86/ordered merges remain unresolved.
These are conditional results for the declared data and API, not universal dispatch thresholds.

Arm hostname dev-dsk-ahrav-2b-7dc7bd93.us-west-2.amazon.com, aarch64,64 available CPUs, Rust1.98.1,
LLVM22.1.8. xxl resolved to dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com, x86_64,192 available CPUs,
Xeon Platinum8488C, Rust1.99.0, LLVM23.1.1. Both kernels6.12.110-135.202.amzn2023. CPU feature and
model details plus exact build flags/cpuset are retained in raw environments. Host/compiler
versions differ, so these timings do not isolate an instruction-set effect.

Entry-to-output-destruction timing includes allocation/growth, output writes, initialization,
co-ranks, thread launch and join. Generation/input sorting/oracles excluded; process wall includes
those separately. No independent A/A calibration, CPU isolation/NUMA policy, cold-cache/storage
experiment or hardware counters. Linked assembly is retained with exact binary hash; no mechanism
claim is inferred from whole-process counters. Logical copy counts in README are code-derived.

External retention root:
/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-011/20261007

Initial and committed source archives/raw bundles are separately retained and hash verified.
Replay: extract committed-source.tar.gz in an empty directory on a declared Linux host, then run
`python3 campaign.py --out results`. Run it with plain `python3`: the runner's fail-closed checks
(source-manifest hashes, 4-CPU cpuset, subprocess exit codes, sample validity) are `assert`
statements, which `-O` and `PYTHONOPTIMIZE` remove. The runner is frozen at the receipt hash and
expects the archive layout (`source-manifest.json` beside `campaign.py`, sources under `topic/`) on a
Linux host (`sched_getaffinity`, `taskset`, `lscpu`). Runner/source hashes and safe archive
membership were checked on both transfers and raw retrievals. No raw process records or executable
binaries enter Git.

Replay regenerates environment files, contracts output, `binary.sha256`, linked assembly and
`attempts.jsonl`. The analysis step that turned each host's `attempts.jsonl` into RESULTS.json
(geometric means, paired log-ratio intervals, family-152 critical t, winner and unresolved-rival
fields) is not committed and its hash is not retained; BENCHMARK.md specifies the rule, the executed
analysis text is unattested. Per-host `binary.sha256` values live in the raw archives named by
`raw_archive_sha256`, not in the receipt. The runner aborts on the first nonzero candidate exit
before writing that attempt, so a failed process leaves `attempts.jsonl` short of 672 records with no
`CONTRACTS_AND_CAMPAIGN_OK` marker; both hosts completed all 672 scheduled processes.
