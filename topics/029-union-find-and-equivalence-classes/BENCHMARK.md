# Fixed performance experiment

Benchmark decision: **true**. Choose among four correct connectivity
representations for declared operation mixes. Rollback and group enumeration
have separate contracts and are checked for correctness outside this contest.

Candidates: labels (simple baseline), size, full compression and halving.
Workloads are deterministic, using the seed in `src/bench.rs`:

- tiny: N=8, 64 operations, 32 merge attempts and 32 queries;
- small: N=256, 2048 operations, 1024 merge attempts and 1024 queries;
- mixed: N=4096, 8192 operations, 4096 merge attempts and 4096 queries;
- reads: N=4096, 8192 operations, 410 merge attempts and 7782 queries;
- unions: N=4096, 8192 operations, 7782 merge attempts and 410 queries;
- local: N=4096 with endpoints in 0..64, 4096 merge attempts and 4096 queries;
- duplicates: N=4096, 8192 merge attempts among eight IDs; seven merge classes;
- first: N=4096, 4095 balanced prefix merges, then 8192 connectivity queries;
- repeat: same prefix and queries; execute the query sequence once before timing.

Each timed trace allocates its response vector and applies all listed operations.
The constructor, prefix and repeated-query preparation are `setup_ns`.
Deallocation of candidate state and response vector is `teardown_ns`.
Oracle graph construction/traversal, trace generation, process startup,
compilation, printing and correctness comparison are outside the timed trace.
The output is consumed and compared after timing. Setup and teardown remain
reported costs. Selection minimizes trace time; a prepared-query selection
therefore requires amortizing its separately reported build cost.

One discarded warmup per cell/candidate. Eight independent measured processes
per cell/candidate. Four rotations followed by the reversal of those rotations
balance every pair 4:4. Each process starts with fresh candidate state. No
inner iteration is counted as an independent observation. An additional sixteen
processes compare halving with an identical alias, alternating order 4:4.

Predeclared criterion: identify the lowest median. Select it only if all eight
paired rival/candidate ratios exceed 1.03 for every rival. Otherwise mark the
cell unresolved. Report medians, sample ranges, and eight samples. This is a
descriptive rule, without confidence-interval or population-significance claims.
The fixed stopping rule is eight blocks. Keep all observed rows.

Control scope: single-threaded, generic CPU target, optimized Rust, 64-bit
indices, same source hashes within each phase, independent graph-search oracle
outside timing, and no concurrent campaign on the same declared host. Forest
find functions are deliberately uninlined. CPU affinity, frequency, host load,
NUMA placement and cache state remain uncontrolled. First means first traversal
of a prepared forest, rather than an evicted-cache measurement. Toolchains
differ by host. Results apply to these small working sets and response-vector
boundary. Full end-to-end builds, member output and rollback are separate costs.

The initial scratch campaign used rotations without their reversals. Its pair
ordering was imbalanced. Retain it as exploratory correctness/timing evidence;
use only the corrected committed-source campaign for selection. Changing order
repairs the protocol, and does not change candidate algorithms or workloads.

Generated code: retain whole executable disassembly and named find symbols.
Inspect candidate functions directly. Whole-process counters would include the
oracle and cannot establish candidate-only cache/branch effects. No hardware
mechanism or allocator/RSS measurement is claimed.
