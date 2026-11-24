# Benchmark decision

Benchmark: true for the CPU cost of choosing packets on preloaded queues.
For AQM, compare service starts and marks in simulated ticks. Timing a
fixed-arrival mark decision cannot measure sender feedback.

## Workloads

| Cell | Active flows | Packets per flow | Lengths | Weights |
| --- | ---: | ---: | --- | --- |
| Tiny batch | 2 | 4 | 400 / 1500 bytes | equal |
| Equal lengths | 16 | 512 | 512 bytes | equal |
| Skewed lengths | 16 | 512 | 400 / 1500 bytes | equal |
| Weighted wide | 256 | 64 | seeded 64 to 1500 bytes | cycling 1, 2, 3, 4 |

The scheduler processes one simultaneous arrival batch per iteration. Packet
round robin, weighted DRR (base quantum 1500), and heap WFQ all emit packet IDs
into the same checksum. Count ring visits and heap selections separately. The
tiny case includes complete short batches under a warmed process. Cold startup,
dynamic arrivals, network I/O, packet payloads, and kernel dispatch are outside
the timed boundary.

## Assignment and decision

The timer includes scheduler metadata allocation, initialization, full replay,
checksum, and teardown. Workload construction, validation, and reporting are
outside. Use native release builds, one pinned CPU and serial runs. Use one
discarded process per cell and policy before measurement. Run six fresh-process
blocks per cell with all six policy orders. Each block also runs a pair of
identical DRR controls. Alternate control pairs before and after the policy
block. Each process produces one timing observation, even when it loops over
the batch many times. Stop after the declared blocks.

Report median nanoseconds per packet and each policy's observed range.
For A to beat B, all six paired ratios B/A must exceed both 1.05 and the
largest observed forward or reverse A/A ratio for that cell. A unique fastest
policy must beat both alternatives. Retain the pairwise results where a leader
remains unresolved. Inspect the linked compiler output for the benchmark
boundary and code shape. Keep hardware attribution outside the claim.
