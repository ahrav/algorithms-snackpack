# Declared experiment

benchmark_required: true

Compare hash baseline, merge, 16-way resident partition and row-capped chunk.
Eight cases: tiny 8/12 rows; uniform/disjoint/sorted 4096/8192 over 16384 keys;
duplicates 16 keys; heavy 1 key; large 65536/131072 over 262144 keys; small_build 32/131072.
Inputs are deterministic contiguous u64 arrays. HashMap random seeds vary per call.
Budget 1024 rows. Generation and independent BTreeMap oracle occur outside timing.
Two warmup joins precede fixed repetitions 4000/20/2/4 depending case.
Timing covers complete resident joins: copies/sorts/allocations/build/probe,
result assertion and destruction. It excludes startup/compilation/data generation.
No reusable-prebuilt-table claim. Spill is correctness/progress exercised separately.

Eight fresh process blocks, four rotations and reversals, all candidates per
block. Inner repetitions are not independent samples. A winner must be more
than 5% faster than every rival in every paired block. Otherwise report nondominated
contenders. Report median/min/max descriptive process means, not population intervals.
Stop after declared 8 blocks; never repeat to manufacture a winner.

Native target, optimization 3, Rust 2024, toolchain/host/affinity/features/source/binary
identity in raw manifest. Schedulers unpinned. No A/A, cold-cache eviction, PMU,
allocator-byte instrumentation, wider payloads or storage timing. Assembly retained
for code inspection only; no candidate mechanism claim from whole-process evidence.
