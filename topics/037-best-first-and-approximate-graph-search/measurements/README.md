# Two-host measured selection

Times are median microseconds per query, normalized from warm repeated-query batches.
A result qualifies at full fill and mean recall>=0.95. Unique winners must pass
the frozen all-eight-pairs >5% rule against every qualified rival.

| Case | Host | Qualifying median leader | Median [min,max] us | Resolved winner |
|---|---|---|---|---|
| 0 | arm | flat | 0.343 [0.338,0.345] | flat |
| 1 | arm | flat | 0.738 [0.734,0.744] | flat |
| 2 | arm | flat | 3.897 [3.882,3.932] | flat |
| 3 | arm | 4 | 2.168 [2.137,2.257] | 4 |
| 4 | arm | flat | 5.430 [5.421,5.496] | flat |
| 5 | arm | flat | 5.440 [5.414,5.465] | flat |
| 6 | arm | flat | 6.251 [6.144,6.773] | unresolved |
| 7 | arm | flat | 4.315 [4.278,4.344] | flat |
| 0 | x86 | flat | 0.237 [0.232,0.275] | flat |
| 1 | x86 | flat | 0.597 [0.534,0.684] | flat |
| 2 | x86 | flat | 2.767 [2.578,4.158] | flat |
| 3 | x86 | 4 | 1.730 [1.561,2.069] | 4 |
| 4 | x86 | flat | 6.299 [6.207,7.805] | flat |
| 5 | x86 | flat | 6.241 [6.206,6.290] | flat |
| 6 | x86 | 16 | 6.541 [6.256,9.316] | unresolved |
| 7 | x86 | flat | 2.951 [2.903,4.931] | flat |

For case 3, flat medians are Arm 3.903 us and x86 2.748 us; width 4 medians
are Arm 2.168 us and x86 1.730 us at recall 127/128 = 0.9921875. Width 4 scores
79.594 vectors/query on average versus 1024 for flat. Its maximum frontier
occupancy is 58 despite retained width 4. Width 16/64 reach full recall there
but lose to flat. The 95% quality policy admits one missing neighbor among 128;
a per-query exactness policy would choose flat for this workload.

Case 2 width 64 reaches only 0.59375 recall; case 4 only 0.5. Case 5 width 16
has 0.9375 recall and fails the quality gate; width 64 qualifies at 0.984375
but loses to flat. Case 6 qualifies width 16 at 0.9609375 and width 64 at 1.0;
timing remains unresolved against flat on both hosts. Case 7 graph recall
stays 0 at all widths.

Both hosts run 288 processes: 32 warmup and 256 measured. Every result passes
independent ID/distance/order/checksum checks. Native tests pass 3 groups.
Exact executable inputs match committed af07996f913c4dd6d751f1c1d9dcf150cc647b46.
Compiler versions differ: Arm 1.98.1, x86 1.99.0; compare candidates within hosts.
The Arm CPU part is 0xd40; x86 reports Xeon Platinum 8488C. Environment JSON
retains full kernel, CPU, target features and available CPU affinity counts.

The observed preparation time includes corpus, graph and query generation.
Flat does not need graph preparation. For case 3 it is about 24.5 ms Arm and
25.9 ms x86. These are separate from query time and require query-lifetime
amortization. Raw process wall times include preparation and startup.

Limits: one fixed deterministic build/query set, warmed allocations/caches,
unpinned single-thread execution, small synthetic integer vectors, no service
load, cold storage, filters, churn, multiple build seeds or RSS campaign.
Timings describe tested candidates and controls. Work counters explain fewer
scores; cache/branch mechanisms require further isolated measurements. Graph
candidates parse the method string inside each timed query (measured 2.7 ns
against 0.9 ns for the flat string compare, under 0.4% of the smallest
median); the retained scan buffer and heap each grow once past k or width per
query. Both are shared by every measured block and are listed as revisit scope.

Replay: python3 measurements/run.py --out /tmp/topic037-run.
Raw source/evidence archives and archive-member hash manifests are retained
externally. See EVIDENCE_RECEIPT.json for hashes, binary/source identities and paths.

Review after measurement changed the committed sources, so their hashes differ
from the receipt inputs: `benches/search.rs` filters Cargo's `--bench` flag;
`src/lib.rs` validates `entry` before the k=0 return and documents it;
`tests/contracts.rs` compares work counters, entries 0 and N/2 and oversized
widths; `measurements/run.py` probes CPU and affinity portably and gives
`--focus` one measured block; `measurements/summarize.py` refuses incomplete or
misordered blocks. Scoring, traversal, selection and the timed loop are unchanged.
