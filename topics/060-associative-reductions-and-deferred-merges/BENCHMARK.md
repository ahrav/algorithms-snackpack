benchmark_required: true

Select the fastest build plus retained reads plus drop transaction among raw
replay, 32-row partial states, and one state per retained interval. The target
is this in-memory, deterministic, signed-integer workload on the named Arm and
x86-64 hosts. All policies return the same exact sum and count at every cut.

| Fixture | Values | Rows per cut | Reads | Repeats per process |
| --- | ---: | ---: | ---: | ---: |
| tiny | 5 | 2 | 2 | 20000 |
| large-once | 262144 | 65536 | 1 | 64 |
| large-read | 262144 | 65536 | 64 | 8 |
| dense-cuts | 16384 | 1 | 16 | 24 |
| medium-stripes | 32768 | 128 | 8 | 96 |
| short-stripes | 32768 | 8 | 1 | 96 |

Each even read selects latest. Odd reads cycle over retained cuts using stride
97. The fixture and independent oracle run before timing. Timed construction
includes cut validation, allocation and copying or reduction. Reads include
digest consumption. Drop is timed. Process startup, fixture generation, oracle,
output and storage I/O sit outside this boundary. Every process first validates
one transaction. Small histories are measured in fresh processes with warm
fixture and allocator state. Initial cold latency is outside the estimand.

A fresh process is the treatment unit. A block of three consecutive treatment
processes is the paired analysis unit. Inner transactions are subsamples. Run
one untimed process per fixture and policy, then 12 main blocks per fixture.
Every order occurs twice. Run 12 raw/raw control pairs per fixture, six in each
order, with distinct labels through the same binary and parser. Randomize the
block ordering and the main/control order within each block with seed 600032.
Use one eligible logical CPU per host, sequential fresh processes, and identical
native optimization flags. Shared host activity can still affect samples.

There is no pilot variance estimate. Use a fixed 12 blocks, no interim looks
and a 20-second process timeout. Selection requires every one of the 12 paired
ratios against each competitor to clear 1.05 in favor of the same policy. Under
independent null signs, the two-sided sign probability is at most 2/4096 per
pairwise contrast. The family has 36 contrasts across two hosts and six cases,
giving a Bonferroni upper bound of 0.018 conditional on that model. Treat this
as a conservative selection rule rather than general inference over hosts.
If the median absolute paired A/A discrepancy exceeds 5%, leave the case
unresolved. Report medians, median absolute deviations and paired ratio ranges.
Report all process failures. A missing, failed or identity-mismatched attempt
leaves the associated cell unresolved. Treat equal and practically inseparable
outcomes as unresolved.

Collect exact record visits and reserved vector buffers/bytes outside timing
and disassemble the linked binary. Use those as work and code observations.
Allocator overhead, hardware events and physical storage traffic are outside
this experiment. No claim relies on a particular cache or branch mechanism.
