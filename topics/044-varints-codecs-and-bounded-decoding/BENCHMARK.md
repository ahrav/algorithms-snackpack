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

## Design declarations

Recorded after the two-host measurements of source 7c35370. They describe the schedule
that run.py executed and leave the reported results as they are.

This is an exploratory screen. Its unit count of six blocks per cell is fixed, with no
pilot variance estimate, power or sensitivity analysis, confidence level, error budget
or familywise control. The 16 host/cell selections are reported separately as
descriptive outcomes of the 5% rule.

Units: one process invocation receives one candidate and is the treatment unit. Each
complete block of three processes is the paired analysis unit. Internal decode
repetitions (100,000 for N=32, 50 for N=65536) are subsamples. Each host contributes one
build artifact, so each result generalizes to that host and binary.

Assignment: deterministic, with a fixed order and no schedule seed. The six measured
orders over scalar/unrolled/fixed are abc, bca, cab, cba, acb, bac, so each candidate
holds every position twice. The warmup block runs cab. Pseudorandom workload values use
a fixed LCG state of 42; the threshold and alternating shapes are fixed sequences.

Interference: processes in one block share host cache, thermal and scheduler state, and
a fresh process resets process state only. The analysis therefore pairs candidates
within a block.

Baseline and oracle: scalar is the timed baseline. The `reference` oracle runs before
timing in every process, and its decode time is unmeasured.

Semantic equivalence: before timing, every process checks that the oracle and all three
candidates decode the workload to the same values. Fixed32 reads a different byte
representation of those values.

Missing data: selections use complete 180-row schedules. Both recorded hosts completed
180 rows.
