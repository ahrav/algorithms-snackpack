# Fixed performance selection protocol

benchmark_required: true

Claim: identify fastest correct tested API per workload/host and stable/unordered contract.
Source/runner/workloads/boundary frozen before initial measurements. Committed replay is
separate, not pooled with initial scratch runs. Exact hashes and raw evidence are retained.

Four candidates: growable bucket vectors, initialized histogram scatter, safe two-pass
scatter with intermediate allocations, immutable-input cycles including clone. Stable
contract excludes cycles. Prehashed key+identity rows are16bytes or64bytes with inline payload.

11 cases: n32,B16; n4096,B16; n262144,B16/1024/4096 uniform; n262144,B1024
90percent key5, all key5, zero low12bits, grouped,64byte payload, and first-call uniform.
All cases use immutable materialized input; grouping/hash generation excluded. Two warmup
calls except first-call zero. First-call means no candidate warmup, not guaranteed cold cache.

12 independent paired blocks, four separate candidate processes per block, every pair
six forward/six reverse. Orders are the first six lexicographic permutations and their
reversals, so buckets runs only at process positions 0 and 3 while the other candidates
occupy all four positions; endpoint-position effects are aliased with buckets, and the
forward/reverse pairing cancels only a linear position trend. Process pins to lowest allowed
CPU. Tiny1024 and small16 calls
within process reduce clock overhead; others1 call. Repetitions are not independent units.
Oracle and input prep outside timer; count/prefix/alloc/zero-fill/clone/scatter/output
observation/teardown inside. Process wall time and warmup recorded separately.

Selection: lowest median among contract-valid candidates. Unique only when all12 paired
rival/winner ratios exceed1.05. Otherwise retain unresolved contenders. Report observed
min/max times and ratio ranges, not confidence intervals. No A/A or universal noise-floor
claim. Stop after frozen528 processes per host, do not rerun for a desired winner.

Native Rust opt-level3,target-cpu=native; toolchain/CPU/kernel/allowed CPUs and affinity
recorded. Required Arm and runtime-resolved xxl independently verified. Assembly retained;
no PMU counters collected and no candidate-only cache/TLB/branch causal attribution.
No complete consumer benchmark, parallel kernel, buffered SIMD or NUMA tuning comparison.
