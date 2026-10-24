# Linux measurements

Both required hosts ran exact committed source `200b157100cb9aa25c4622a545daaab9c329a627`.
All 160 measured and 20 discarded warmup processes passed their oracle checks.
Each cell has eight paired observations, with candidate order split 4:4.
The initial source also passed tests and three focused invocations on both hosts.

The timer covers scheduler queues, threads, work, shared counters, joins and
equality checking. The sequential oracle runs before timing. Read
[the frozen protocol](../BENCHMARK.md) for controls and selection.

Times below are median [minimum, maximum] in microseconds. Ratios divide the
stealing duration by the global duration within each repetition. A selection
requires all eight ratios beyond the same 3 percent boundary. Lowest median
rankings remain descriptive when selection is unresolved.

| Host | Workload | Global FIFO us | Locked stealing us | Paired ratio range | Lowest median | Selection |
| --- | --- | ---: | ---: | ---: | --- | --- |
| arm | tiny_balanced | 134.821 [123.773, 155.339] | 140.530 [129.422, 157.371] | 0.861..1.183 | Global | Unresolved |
| arm | tiny_skewed | 132.625 [120.706, 141.649] | 133.795 [129.967, 142.281] | 0.932..1.126 | Global | Unresolved |
| arm | medium_balanced | 2575.575 [2145.165, 2692.646] | 1685.665 [1627.761, 1765.101] | 0.613..0.776 | Steal | Steal |
| arm | medium_skewed | 2608.695 [2462.817, 2769.190] | 3284.731 [2752.066, 3457.002] | 1.117..1.382 | Global | Global |
| arm | coarse_skewed | 1028.186 [981.060, 1081.002] | 1226.745 [1201.693, 1359.304] | 1.114..1.320 | Global | Global |
| x86 | tiny_balanced | 143.404 [126.548, 151.554] | 146.865 [116.322, 193.435] | 0.833..1.359 | Global | Unresolved |
| x86 | tiny_skewed | 119.846 [109.003, 184.268] | 129.773 [108.919, 199.130] | 0.601..1.450 | Global | Unresolved |
| x86 | medium_balanced | 5070.350 [4022.070, 6078.882] | 2716.511 [1769.228, 3486.822] | 0.378..0.727 | Steal | Steal |
| x86 | medium_skewed | 5080.444 [3725.726, 5867.035] | 5357.240 [4596.361, 6841.520] | 0.884..1.610 | Global | Unresolved |
| x86 | coarse_skewed | 2368.871 [1840.257, 2639.322] | 1687.072 [1566.359, 1864.734] | 0.601..1.013 | Steal | Unresolved |

## Environment and limits

Arm: `dev-dsk-ahrav-2b-7dc7bd93.us-west-2.amazon.com`, aarch64, 64 available
CPUs, Rust 1.98.1. Linux 6.12.111-136.204.amzn2023.aarch64. The exposed CPU
identity is implementer `0x41`, part `0xd40`, variant `0x1`, revision 1;
the host supplies no model-name label.

x86: alias `xxl` resolved to `dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com`,
x86_64, Intel Xeon Platinum 8488C, 192 available CPUs, Rust 1.99.0.
Linux 6.12.111-136.204.amzn2023.x86_64.

Both builds use Cargo release defaults, generic targets and empty RUSTFLAGS.
Full uname, CPU identification, target configuration, binary SHA-256, symbol
list, replay workspace and test logs are retained in the external archives.
All seven executable/workspace input hashes match the measured commit.

Timing includes setup and shared counters. Different hosts and compilers,
unpinned scheduling and uncontrolled caches limit interpretation. These
measurements select among the locked teaching candidates for the listed
workloads. Lock-free throughput, production tails, allocator bytes and causal
hardware mechanisms require separate evidence.

Raw archives and identities are in [EVIDENCE_RECEIPT.json](EVIDENCE_RECEIPT.json).
The original 70-process macOS campaign remains separate in LOCAL_SMOKE.md.
