# Exact-source results

Frozen executable source: `a278f9fec500c7ac3517644feeba37d5032d5d09`. Five executable/runner file hashes match on both hosts.
Later commits on this branch change only the benchmark's argument filter, the
example's assertions, the contract tests and the runner's JSON parsing and
integrity checks; `src/lib.rs`, every timed function, the timed loop and the
workload generator are the frozen commit's. The results below are that commit's.

Each host produced 306 measured processes and 51 warmups. Every process passed the independent oracle. Six blocks have 3:3 pair ordering. Seven test groups passed on each host, including direct radix and position-map tests.

A selected candidate was more than 5% faster than every other admitted candidate in every paired block. An unresolved cell names its lowest median descriptively. Ranges are observed process-mean ranges, not confidence intervals.

| Workload | V / E | Arm selection and lowest median, µs [range] | x86 selection and lowest median, µs [range] |
|---|---|---|---|
| tiny | 5 / 5 | lazy: 0.106 [0.105, 0.108] | lazy: 0.105 [0.103, 0.107] |
| sparse_small | 128 / 896 | indexed: 6.848 [6.769, 6.896] | indexed: 9.022 [8.962, 9.281] |
| sparse_large | 4096 / 28672 | radix: 501.316 [500.419, 504.162] | radix: 463.409 [460.124, 469.987] |
| bounded | 4096 / 28672 | unresolved; dial: 213.197 [211.475, 216.006] | unresolved; dial: 200.608 [198.673, 203.908] |
| zero_one | 4096 / 28672 | unresolved; zero_one: 180.692 [177.733, 186.044] | zero_one: 157.173 [155.656, 158.024] |
| dense | 512 / 262144 | indexed: 336.916 [330.556, 339.664] | indexed: 312.326 [304.330, 333.645] |
| decreases | 256 / 32640 | indexed: 181.202 [179.719, 182.362] | indexed: 188.402 [182.100, 201.656] |
| wide_chain | 4096 / 4095 | unresolved; lazy: 42.166 [40.659, 75.611] | lazy: 60.357 [60.258, 60.474] |
| disconnected | 4096 / 3 | unresolved; zero_one: 0.532 [0.530, 0.534] | unresolved; zero_one: 0.278 [0.273, 0.282] |

## Scope and interpretation

Arm: declared dev-dsk-ahrav-2b host, aarch64, 64 available CPUs, ARM implementer 0x41 / part 0xd40 / r1p1, rustc 1.98.1. x86: alias xxl resolved to dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com, x86_64, 192 available CPUs, Intel Xeon Platinum 8488C, rustc 1.99.0. Full kernel, flags, model identifiers, native target features and environments are retained externally.

Comparisons apply within each host and this compiler build. Cross-host ratios do not isolate architecture. No hardware-counter mechanism is claimed. Linked symbols/disassembly were retained; the work counters identify queue traffic rather than cache or instruction effects.

Identical-code lazy_aa/lazy ratios ranged from 0.542 to 1.134 on Arm and 0.366 to 1.058 on x86. Several short-duration cases showed large deviations; sparse_small x86 also showed a 0.838 minimum. These controls show host/process variation. The fixed selection rule describes this campaign, with limited predictive strength. No favorable rerun replaced it.

The decrease adversary produced 32,641 lazy removals, 32,385 stale records and 32,386 peak live records. Indexed binary removed 256 records, retained at most 255 and rejected zero stale records. Both returned the same distances. These are measured operation counts.

On the tiny graph, lazy first queries took 1.808 µs on Arm and 2.078 µs on x86, compared with 0.106/0.105 µs repeated-query medians. Graph generation/admission took 0.591/0.875 µs. This separates first-query behavior from the reused-graph timing. Cache coldness and isolated startup latency remain unmeasured.

All candidates allocate fresh query state and destroy it inside timing. Graph admission is outside. The indexed position map is initialized for every vertex, including disconnected vertices. Peak records omit allocator capacity, bucket headers and RSS. `summary.json` contains every admitted candidate, first-query/admission/oracle medians, queue counts and diagnostic control ranges.

Choose lazy for this tiny graph; indexed binary for the small sparse, dense and decrease-heavy graphs; radix for the large general sparse graph. Binary weights select the deque on x86 and remain unresolved on Arm. Bounded-weight Dial/radix and mostly disconnected cases remain unresolved. The wide chain selects lazy on x86 and remains unresolved on Arm. These are workload-specific results.

Replay: `python3 replay.py /absolute/new/run-directory` on Linux. See `../BENCHMARK.md` for boundaries and the fixed stopping/selection rule. Pre-lesson scratch records are separate and excluded.
