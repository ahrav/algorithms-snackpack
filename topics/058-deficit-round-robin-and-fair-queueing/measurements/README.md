# Exact Linux replay

Executable inputs were frozen at `781a2eb1de25b3332d141c378883145a40647d76`.
Run `bash measurements/run.sh` from this topic directory on Linux.
Both native hosts passed four contract groups covering 384 scheduling batches
and 512 arrival traces. Both agreed on all 132 process work counts and checksums.
Each host ran 72 timed, 48 control, and 12 warmup processes.
See [the receipt](EVIDENCE_RECEIPT.json) for host, source, raw archive, linked
image, and verification identities. CPU 0 was selected on both hosts.

At tick 5,800 the fluid model gives both backlogged flows 2,900 bytes.
Packet round robin gives A 1,300 and B 4,500. DRR and WFQ give A 2,800
and B 3,000. In the fixed burst, both queue policies start at 0, 400,
800, and 1,200 ticks. The persistence controller marks the last packet.

Median nanoseconds per packet and observed ranges across six fresh processes
per cell and policy. A/A shows the range of paired DRR control ratios.

| Host | Workload | Packet RR | DRR | WFQ | A/A | Fastest under the rule |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| Arm | tiny | 5.41 (5.39–5.47) | 5.96 (5.93–6.47) | 13.90 (13.47–14.01) | 0.994–1.000 | Packet RR |
| Arm | equal | 2.79 (2.78–2.81) | 2.03 (2.03–2.11) | 29.86 (29.61–29.98) | 0.996–1.020 | DRR |
| Arm | skew | 2.79 (2.78–2.79) | 2.98 (2.98–2.98) | 27.69 (27.49–27.74) | 0.997–1.002 | Packet RR |
| Arm | weighted-wide | 2.80 (2.79–2.81) | 2.65 (2.60–2.84) | 80.67 (80.34–80.75) | 0.957–1.013 | RR vs. DRR unresolved |
| x86-64 | tiny | 4.51 (4.29–5.50) | 4.47 (4.32–5.55) | 12.26 (12.20–12.92) | 0.955–1.029 | RR vs. DRR unresolved |
| x86-64 | equal | 2.78 (2.73–3.98) | 1.63 (1.56–1.89) | 30.45 (30.30–36.13) | 0.958–1.067 | DRR |
| x86-64 | skew | 3.35 (2.74–3.80) | 3.15 (2.48–3.71) | 32.35 (27.18–33.99) | 0.683–1.104 | RR vs. DRR unresolved |
| x86-64 | weighted-wide | 2.76 (2.75–2.91) | 2.81 (2.74–3.07) | 67.87 (67.60–68.66) | 0.951–1.047 | RR vs. DRR unresolved |

Both ring schedulers beat heap WFQ in all eight cells. DRR used
0.750, 0.342, 0.634, and 0.278 visits per packet in workload order.
Packet RR made one visit per packet. Heap WFQ made one selection per
packet, with potentially multiple comparisons per selection. Compiled
assembly on both architectures shows allocator calls inside the timed
batch loop and wide integer multiplication for finish comparisons.
A glibc interposer ran the archived binaries and counted one allocation and
one free per batch for every policy on both hosts. Reallocation calls per
batch were 0, 2, 2, and 6 in workload order. Every accepted process had two
monotonic clock reads and matched the original work count and output digest.
The CPU comparison uses the original uninstrumented runs.
The measurements cover total policy cost within that loop; the experiment
does not attribute the timing differences to individual instructions,
allocations, or memory effects.

Packet RR's CPU performance does not imply byte fairness. Treat the CPU
results as a choice among policies that already meet the required service
contract. These runs measure a preloaded batch, not Linux packet processing
or closed-loop transport response.
