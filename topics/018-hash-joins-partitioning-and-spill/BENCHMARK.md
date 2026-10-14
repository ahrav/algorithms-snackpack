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

## Bounded quadratic reference supplement

The quadratic oracle is also timed against merge for the primary tiny 8/12-key
cell, where its allocation-free setup can decide selection. Larger oracle cells
are unmeasured: the large cell alone needs 8,589,934,592 equality checks per join;
this is a documented measurement limit, not an elapsed result or all-five winner.
The four serious resident candidates remain the primary portfolio.

Supplement predeclared before execution: same tiny keys, two warmups, 100,000 timed
joins, eight fresh paired process blocks in AB/BA order. Same complete-call
boundary including assertion, allocation/sort/drop. Independent BTreeMap oracle
runs outside timing and verifies 5pairs. Same >5% every block selection rule and
observed mean ranges. This separate experiment adds the reference to tiny
selection; it does not create a simultaneous five-candidate paired campaign.
Both required Linux hosts run ephemeral and exact committed copies.
