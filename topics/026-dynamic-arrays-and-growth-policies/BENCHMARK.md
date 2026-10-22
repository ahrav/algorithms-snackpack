benchmark_required: true

# Frozen Topic026 experiment

Before collection: seven workloads; five grow policies on tiny17/medium8193/large262145 8-byte records, wide65537 256-byte records, and sixteen clear/refill reuse cycles. Three shrink policies on4096 alternating pop/push cycles at1025, and four8193-record full drain/refill cycles. Known peak hint is an explicit information advantage for reserved, not an online universal winner.

Each process validates all pop results and final records against independent BTreeMap, releases oracle storage, performs one warmup, then times allocation, operations, full output checksum and teardown for a fixed repeat count. Input generation/validation/process startup/printing are excluded. Each process aggregate is one sample; inner repeats are subsamples. Ten rotating mirrored blocks, each candidate once/cell/block, A/A std labels per block; fixed450 processes/host/campaign. No same-host campaign overlap, no peeking or retuning. Main five-policy family every position twice; shrink three-policy position counts differ by at most one. Machine, compiler and binary/source/runner SHA256 retained.

Selection: lowest median is reported; select a sole winner only if every rival is >3% slower in ALL ten paired block ratios. Otherwise unresolved with close rivals. Full sample and ratio ranges are descriptive, not confidence intervals or error-controlled population claims. Fixed sample budget is exploratory; no prospective power or calibrated significance claim. A/A checks identity/output mechanically and reports ratio ranges as diagnostics, never as a noise floor. No universal/ISA-only, tail-latency, cold allocator, production contention, exact copied-byte/RSS claim. No retuning to manufacture a winner.

Counters are explicit growth/shrink API requests, actual reported capacities, observed growth-address changes, and sum of live lengths at growth requests. They do not observe allocator call internals, physical movement, peak resident memory, or shrink address changes. Allocation may grow in place or remap; live-at-growth is a hypothetical full-copy work model, not measured bytes.
