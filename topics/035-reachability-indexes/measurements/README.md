# Exact-source native results

Frozen executable commit: `86d74b91200b57e9e1e711a9127b29e4b2a392f7`.
Both source archives hash to `6076d9a3e7d547de11ff914ff9cc2378a2752c185ef7e6c294a78c7a206bdf0f`.
Each host completed 138 processes, all oracle checks, three contract groups and the example.
Warm batches have 1024 queries except few (8). Durations below are microseconds per batch.

| Host | Workload | DFS median [range] | Filter median [range] | Closure median [range] | Warm choice | Lifecycle choice |
|---|---|---|---|---|---|---|
| arm | tiny | 7.723 [7.341, 7.955] | 7.165 [6.780, 7.702] | 2.282 [2.253, 2.285] | closure | closure |
| arm | sparse | 101.025 [80.380, 105.712] | 44.105 [40.646, 46.645] | 3.265 [3.206, 4.052] | closure | closure |
| arm | dense | 132.799 [128.295, 136.912] | 105.569 [104.612, 107.043] | 3.014 [2.961, 3.053] | closure | dfs |
| arm | chain | 4412.301 [4392.424, 4544.965] | 5386.833 [5366.719, 6444.950] | 3.303 [3.256, 3.367] | closure | closure |
| arm | negative | 3561.671 [3559.207, 3596.518] | 29.680 [29.446, 29.773] | 2.894 [2.844, 2.967] | closure | filter |
| arm | few | 6.728 [6.043, 7.580] | 7.599 [7.320, 8.905] | 0.038 [0.037, 0.039] | closure | dfs |
| arm | large-sparse | 57.255 [54.355, 58.112] | 39.029 [38.289, 40.887] | 2.867 [2.851, 5.187] | closure | dfs |
| x86 | tiny | 5.921 [5.864, 10.082] | 5.869 [5.794, 5.924] | 2.026 [2.018, 2.236] | closure | closure |
| x86 | sparse | 56.449 [48.457, 75.650] | 24.324 [23.919, 27.170] | 2.030 [2.028, 2.037] | closure | closure |
| x86 | dense | 88.970 [88.295, 135.291] | 77.368 [74.649, 80.542] | 2.031 [2.026, 2.037] | closure | dfs |
| x86 | chain | 4168.120 [4071.988, 4171.746] | 4582.133 [4556.026, 4650.060] | 2.333 [2.054, 2.349] | closure | closure |
| x86 | negative | 3354.284 [3339.927, 3389.618] | 26.879 [26.524, 28.820] | 2.188 [2.184, 2.194] | closure | filter |
| x86 | few | 5.902 [5.882, 5.918] | 6.423 [6.376, 9.102] | 0.044 [0.043, 0.047] | closure | dfs |
| x86 | large-sparse | 40.629 [36.090, 43.172] | 29.150 [28.936, 30.140] | 2.332 [2.323, 2.337] | closure | dfs |

Lifecycle means build + one warm batch + teardown. Closure wins tiny/sparse/chain;
DFS wins dense/few/large-sparse; filtering wins forward cross-component negatives.
The six-block 5% criterion is descriptive. A/A control ranges: Arm 0.855..1.079;
x86 0.662..1.444. Host noise limits close comparisons and population extrapolation.
Toolchains differ: Arm rustc1.98.1; x86 rustc1.99.0, both opt-level3 generic target.
Arm reports 64 available CPUs; x86 reports 192. Runs use inherited affinity and no pinning.
CPU/OS/target cfg, binary/input hashes and full run records are retained externally.
Logical closure/label payload at n4096: 2097152/196608 bytes. DFS visit/stack
state, allocation slack and peak RSS are outside those index-payload figures.
Cold first-touch queries, update algorithms, compressed/chain/2-hop indexes and
production-size memory behavior remain unmeasured. Graph generation/validation,
oracle and startup are excluded. No cross-host hardware-cause claim.
Disassembly was retained and symbols inspected; whole-process counters would include
oracle work and are unsuitable for candidate-only causal attribution.

Reproduce from repository root:
```sh
python3 topics/035-reachability-indexes/scripts/run.py /tmp/topic035-fresh
```
See BENCHMARK.md for exact boundaries, generator and ordering.
