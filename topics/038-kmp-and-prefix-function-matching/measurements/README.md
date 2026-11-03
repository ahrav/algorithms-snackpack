# Exact-source measurements

Frozen executable/runner source: `371e9175e11a98eea5dd2c454923eee0b461b1d3`. Both hosts passed unit tests,
exhaustive differential and streaming checks before timing. Each host ran 280
processes: 216 main measurements, 24 A/A and 40 warmups. Inputs and every
archive member hash were independently verified. Receipts link raw evidence.

Arm: aarch64, CPU implementer 0x41/part 0xd40, 64 available CPUs, rustc 1.98.1.
x86: Intel Xeon Platinum 8488C, x86_64, 192 available CPUs, rustc 1.99.0.
Both use Linux 6.12.113-138.209.amzn2023, CPU 0 affinity, `-O`,
`-C target-cpu=native`, `-C codegen-units=1`. Full target features and
environment are retained with binary hashes in the external archives.

Times below are median nanoseconds per query, followed by the observed
minimum and maximum across six processes. Select only an all-six-block >5%
advantage over both rivals. Unresolved cells retain their measured fastest
median in the table without claiming a resolved winner.

| Host | Workload | Boundary | Direct ns | KMP ns | DFA ns | Selection |
|---|---|---|---:|---:|---:|---|
| arm | tiny | total | 154.9 [152.6, 160.9] | 252.0 [249.5, 266.1] | 478.1 [472.6, 483.8] | direct |
| arm | tiny | prepared | 154.8 [151.6, 155.8] | 132.4 [131.4, 146.2] | 140.8 [140.0, 143.5] | unresolved |
| arm | random | total | 15496.4 [15391.8, 15914.8] | 11437.0 [11368.5, 11496.0] | 19940.6 [19906.7, 20710.7] | kmp |
| arm | random | prepared | 15630.0 [15423.3, 16048.2] | 11328.1 [11296.2, 31709.1] | 19233.7 [18947.7, 19972.2] | unresolved |
| arm | large | total | 254630.2 [250322.0, 255875.0] | 183627.8 [181963.5, 183924.0] | 307244.0 [307153.5, 481680.0] | kmp |
| arm | large | prepared | 251559.5 [247992.5, 257469.5] | 183484.8 [181745.5, 183562.5] | 302994.8 [302944.0, 303146.5] | kmp |
| arm | near | total | 373424.8 [372938.5, 385510.2] | 20379.1 [20292.1, 20854.3] | 23744.5 [23739.8, 24354.3] | kmp |
| arm | near | prepared | 373686.7 [372895.2, 383678.7] | 19980.8 [19736.0, 20434.3] | 19199.6 [19013.0, 19487.2] | unresolved |
| arm | dense | total | 375941.1 [374850.9, 383901.8] | 16127.7 [15998.9, 16138.1] | 25475.7 [25402.8, 26294.5] | kmp |
| arm | dense | prepared | 375566.4 [374911.0, 376353.9] | 15785.1 [15675.7, 16271.6] | 20738.8 [20456.5, 29715.1] | kmp |
| arm | binary | total | 33251.8 [32023.0, 34600.7] | 26541.7 [26124.2, 28496.0] | 19934.9 [19922.2, 20514.8] | dfa |
| arm | binary | prepared | 33755.2 [33142.1, 34005.4] | 27106.3 [25851.2, 27897.7] | 19088.2 [18925.9, 19235.0] | dfa |
| x86 | tiny | total | 115.4 [109.5, 121.1] | 204.0 [195.8, 215.9] | 329.5 [325.6, 332.5] | direct |
| x86 | tiny | prepared | 113.3 [110.0, 128.2] | 113.8 [110.6, 124.9] | 107.2 [105.0, 111.9] | unresolved |
| x86 | random | total | 5605.9 [5523.1, 6625.4] | 17917.7 [12226.2, 18663.2] | 18730.3 [18628.7, 20028.0] | direct |
| x86 | random | prepared | 5589.4 [5540.4, 7272.1] | 17281.7 [11015.1, 19058.7] | 17964.2 [17956.8, 21165.5] | direct |
| x86 | large | total | 134871.0 [134504.5, 136713.5] | 278381.0 [213706.5, 302159.0] | 292492.8 [292400.5, 292589.0] | direct |
| x86 | large | prepared | 135438.5 [133258.0, 203134.5] | 282167.0 [265217.0, 285537.5] | 289283.8 [286848.5, 364659.5] | direct |
| x86 | near | total | 254996.1 [254490.1, 348690.0] | 16715.1 [16680.2, 16755.7] | 23873.8 [23813.6, 26654.4] | kmp |
| x86 | near | prepared | 258059.2 [254475.8, 266163.1] | 16205.2 [16199.6, 16794.2] | 18110.6 [18057.0, 18310.1] | kmp |
| x86 | dense | total | 249953.5 [249851.0, 250116.6] | 18016.9 [16040.9, 18542.4] | 26058.2 [25908.1, 26173.2] | kmp |
| x86 | dense | prepared | 251310.5 [250242.5, 261024.9] | 17914.7 [17647.8, 18395.5] | 20251.0 [20137.8, 20812.5] | kmp |
| x86 | binary | total | 29999.8 [29054.8, 31868.4] | 35829.2 [34532.1, 38279.4] | 18615.4 [18545.6, 18720.3] | dfa |
| x86 | binary | prepared | 29905.2 [28417.5, 31880.3] | 35122.2 [34828.4, 38396.2] | 17959.5 [17956.0, 17966.4] | dfa |

The fixed near-miss workload performs 520,256 direct and 24,512 KMP byte
comparisons. These diagnostics run outside timing. The prefix loop sometimes
compares the same byte again after its while condition; the count follows
this exact implementation. Dense traversal performs 8192 table transitions.

Selection across 24 cells: {'direct': 6, 'unresolved': 4, 'kmp': 10, 'dfa': 4}. Direct wins
the tiny total cell on both hosts and all four x86 uniform cells. KMP wins
the Arm large and dense cells, Arm random total, Arm near total and all four
x86 near-miss and dense cells. DFA wins the binary workload on both hosts.
Arm random prepared stays unresolved on one KMP outlier (31709.1 ns max).
Preparation and host conditions can change the choice. The full table is the
selection guide.

A/A ratios (KMP/KMP-AA), observed ranges and medians:

arm: {'tiny': {'min_ratio': 0.981663538594453, 'median_ratio': 1.0022713230087565, 'max_ratio': 1.041176786179622}, 'near': {'min_ratio': 0.9779768364541377, 'median_ratio': 0.9985640261482494, 'max_ratio': 1.0059051094337907}}
x86: {'tiny': {'min_ratio': 0.8907697137774058, 'median_ratio': 0.9678802830718878, 'max_ratio': 1.0083370071453825}, 'near': {'min_ratio': 0.9683658005099494, 'median_ratio': 0.9977932639757228, 'max_ratio': 1.007748830995324}}

Earlier focus and initial committed campaigns remain separate external
evidence; the final campaign uses the later workspace-test-compatible runner.
Results differ across campaigns. No rerun was selected to manufacture a winner.

Inspected the exact linked direct, KMP scan and DFA traversal disassembly.
Distinct routines and state-dependent branches/transition indexing remain
visible. Source comparison counts support the aggregate-work explanation.
Branch, cache and instruction-cycle causes require isolated runtime profiling
and remain unmeasured. The oracle executes outside the timed interval.

Warm fixed inputs, one synthetic seed per workload, one core per host,
output-vector costs, shared host load and different toolchains bound these
results. Cold startup, library baselines, large patterns, transformed Unicode,
stream throughput and service-level latency remain unmeasured. Finite ranges
provide dispersion, without population confidence guarantees.
