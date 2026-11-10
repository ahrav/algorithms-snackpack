benchmark_required: true

# Benchmark decision

Claim: decode time and bytes vary with integer lengths and representation.
Candidates: scalar and unrolled canonical u32 varints; fixed32 little-endian.
Eight cells: N=32 or 65536 crossed with small(0..127), threshold cycle,
deterministic full-width pseudorandom, alternating 1/u32::MAX distributions.

Unit: one fresh process returns one ns/decode sample after repeated internal decodes.
Internal iterations estimate that process mean; they are not independent samples.
One discarded warmup block and six measured blocks per cell. Rotate three candidates,
then reverse the last three block orders. No data-dependent reruns or stopping.
Selection: beat both alternatives by at least 5% in every paired block; otherwise unresolved.
Report median and min/max, per host/cell, including A/A control range.
The A/A control uses scalar case 6, six balanced AB/BA blocks of the identical binary.

Boundary: complete-buffer decoding, checked input, output stores and vector clear/push,
with reused preallocated output and black_box consumption. Input generation, encoding,
allocation, independent correctness, startup and teardown occur outside timing.
No startup, cold-cache, encode-throughput, storage-I/O or whole-service claim.
Tiny cells test short-buffer latency in a warm repeated workload.

Pin each timing process to the lowest permitted CPU. Record affinity, CPU model,
kernel, toolchain, effective features, flags, exact three input hashes and linked binary.
Native optimized Rust; compiler versions may differ across hosts. Counts include
encoded input bytes and decoded values. Mechanism claims require separate evidence;
no branch/cache/SIMD causal inference follows from these elapsed times.
Correctness oracles execute before timing. Rejected candidates would be excluded.
Retain every raw row externally; Git holds compact results and an archive receipt.
A failed, malformed, mislabeled or timed-out attempt is written as a raw row, then halts the run.
