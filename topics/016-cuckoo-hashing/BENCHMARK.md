# Frozen benchmark decision

benchmark_required: true

Compare linear, cuckoo1 and cuckoo4 exact fixed-capacity maps with the same slot
storage and declared requested keys. Reject a candidate from a workload if any
requested insertion fails. Retain those failure counts, without timing an
incomplete map as a completed service. All valid timings require an independent
BTreeMap oracle and batch checksum agreement.

11 workloads: tiny hit (32 slots/10 keys), small hit/miss and hot16-key queries
(4,096/1,400), large hit/miss (1,048,576/360,000), high hit/miss
(131,072/104,857), complete build35 (65,536/22,000), build80
(65,536/52,000), and delete/reinsert churn (8,192/2,800).
The adversarial same-pair and feasible-small-budget cases are correctness/admission
witnesses, not competitive timings. Query accesses use a fixed modular permutation;
hot queries cycle16 keys. There is no random/adaptive request-distribution claim.

Two warmup batches; fixed inner rounds128 for small/hot/churn,32 for large/high,
12 for complete builds. Exactly12 process blocks, three candidate processes per
block and workload:396 processes per host. Rotate all three candidate positions
and reverse each order, then repeat once. Inner rounds are repeated work within
one sample. No optional stopping, outlier deletion, A/A calibration or rerun to
manufacture a winner. Initial and exact committed campaigns remain separate.

Query timing includes integer hashing, exact equality, black_box and checksum;
excludes allocation/build, oracle, query preparation costs already resident outside
cycle, post-run diagnostics and final destruction. Query-key arithmetic remains
inside the timed cycle. Churn includes remove+reinsertion as one reported pair.
Build timing includes allocation, initialization, all insertions and destruction.
The deterministic seed, u64 payload and copy movement are fixed. No stash or
resizing cost is hidden in a query result. Assertions validating batch checksums
are inside the boundary; full independent oracle is outside.

Selection: among candidates admitting all keys in all12 blocks, a unique selection
must be more than5percent faster than every rival in every paired block. Otherwise
report unresolved and undominated contenders. A sole admissible candidate is
selected on admission, without a comparative throughput claim. Medians and
observed min/max describe dispersion, not confidence intervals.

Compile: rustc edition2024, opt-level3, target-cpu=native. Record exact frozen path
hashes, binary hash, compiler, hostname, arch, uname/kernel, CPU details, cfg target
features and available CPUs. Source transfers must pass SHA256SUMS before running.
Scheduling is unpinned for Topic016, with no frequency-control claim. Arm/x86
compiler differences and uncontrolled host load prevent isolated ISA attribution.

Retain generated assembly for source/code-shape inspection. Slot-examination
diagnostics run outside timing. We do not infer candidate-only cache/branch
mechanisms from whole-process counters or oracle-containing profiles. No PMU,
insertion latency quantile, forced cold cache, variable payload, memory-allocation
count or production library speedup claim is made.
