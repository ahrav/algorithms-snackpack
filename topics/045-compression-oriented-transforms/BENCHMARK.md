benchmark_required: true

Claim: compare complete stored sizes and allocating scalar block encode/decode
latency among six correct candidates on two specified hosts. Population: fixed
synthetic u32 blocks, lengths16/1024/65536 crossed with ramp1000+i, constant1000,
LCG seed42 (1664525*x+1013904223 modulo2^32), alternating0x12340000/0x12340001.
Each operation returns and destroys a fresh output; include transform, packet
coding, framing, allocation and teardown. Exclude generation, compilation,
correctness checks, process launch and filesystem I/O. Decode uses pre-encoded bytes.

Process is the experimental unit. Inner repeats amortize clock calls; one warmup
process per candidate/cell, six measured processes, fixed stopping, all attempts
retained. Cases run in catalog order. Six candidate orders use shifts0,2,4 and
the reverse of each shifted order, so every candidate occupies each position once.
This deterministic balanced schedule screens position effects; it supplies no
randomized population inference. Carryover assumption: separate processes, same
warmed immutable input, no concurrent task introduced by this runner. Pin to the
first allowed CPU. No cold-cache reset or service contention model. Small blocks
exercise fixed costs while cache state remains warm. Six A/A pairs run raw encode
on random65536 through identical build, launch, parser and analysis paths.

Independent Python arithmetic/byte model checks all72 complete frames and values;
Rust checks each timed candidate's full roundtrip before starting its clock.
Incorrect or failed cells receive no selection. Retain failures/timeouts and
report missing cells. No adaptive reruns or candidate changes after the freeze.
A failed, malformed, mislabeled or timed-out attempt is written as a raw row, then
halts the run. A halted run is incomplete and supplies no selection. This rule was
recorded after the two-host measurements of source 842b5ac, which completed all
1020 attempts.

Speed winner must beat every competitor by at least5percent in all six paired
process means; otherwise unresolved. Size winner first minimizes full frame bytes,
then uses the speed screen among equal-sized candidates. Report medians/min/max
and paired ratio ranges. Six runs and A/A are exploratory screens with no nominal
confidence interval, pilot-calibrated power or familywise significance claim.
All24 host-operation cells form one descriptive family per host. No universal optimum.

Retain source/runner hashes, native flags, toolchain/CPU/kernel/affinity, linked
binary hashes, compiler assembly and archive hashes. Assembly supports only
observations of generated code. Hardware-counter causal attribution remains
unmeasured. Peak RSS and production codec/I/O comparisons remain revisit scope.
