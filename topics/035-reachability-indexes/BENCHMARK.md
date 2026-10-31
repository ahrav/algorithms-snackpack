benchmark_required: true

## Frozen protocol

Compare exact DFS, three-order min-post filtering plus DFS, and bitset closure.
One immutable snapshot; all candidates accept the same reflexive path contract.
Generator seed 35 uses a fixed 64-bit LCG, upper 32 bits for edge sampling.
Seven cells: tiny n32, sparse n512, dense n512, chain n4096, disconnected-chain
negative n4096, few-query chain n4096, and large-sparse n4096. Probabilities
are 1/64 for tiny/sparse, 1/4 dense, 1/1024 large-sparse.
Each batch has 1024 deterministic endpoint pairs, except few has 8.
Pairs are (37i mod n, 71i+3 mod n), so endpoints differ in parity and no batch asks
a reflexive query; tiny repeats 32 distinct negative pairs 32 times, sparse and
dense repeat 512 pairs twice, and few asks eight forward pairs within distance 241.
Negative has two chains and only forward cross-component queries.
Graph IDs follow topological order in this campaign; reversed IDs are tested.

Independent unit: fresh candidate process. Six blocks use all six permutations
of three candidates. Each candidate occupies every position twice; each pair
appears in either order three times. Fixed stopping: six blocks, no exclusions.
Within each process, one checked warmup batch precedes eight measured batches.
Those eight repetitions produce one mean and are subsamples, not independent runs.
An identical-code DFS control uses AB/BA order in alternating blocks on sparse.
Control records carry position -1 and candidate dfs or dfs-control; even blocks run
dfs first.
Total: 126 candidate processes plus 12 control processes per host.

Graph generation, validation, oracle construction, compilation and startup are
outside timing. Build includes candidate scratch/index allocation and population.
Warm query time includes visit clearing, stack operations, boolean accumulation,
black-box barriers and per-batch correctness assertion. Teardown times dropping
index and scratch separately. Shared graph/query teardown stays outside timing.
Lifecycle estimand is build + one warm batch + teardown. It models replacement
with warm query pages, not first-touch query latency or complete service startup.
No core pinning or host isolation. Record affinity, available CPUs and toolchains.

Predeclared criterion: select a candidate only if its paired duration is at least
5% lower than both alternatives in every one of six blocks. Otherwise unresolved.
Report median and min/max per cell, plus all six paired ratios for controls.
These are descriptive finite-run selections, with no confidence or population
error-rate claim. No pilot variance or sample-size power calculation is claimed.
Small build durations and shared-host interference limit close comparisons.
Memory figures are logical payload, excluding allocator slack and metadata.
Raw process records and binaries remain external. Compact tables and a verified
receipt identify their exact source and archives. No hardware-cause inference.
