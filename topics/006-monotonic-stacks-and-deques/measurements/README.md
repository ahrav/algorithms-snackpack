# Measured results

Source commit: `a13f3bf3c89abf7e51980e85e5d6ae956ab2ad35`. Archive and runner identities are in [the receipt](EVIDENCE_RECEIPT.json). The library, input generator, runner, and protocol remain identical at the publication head. Later review commits in the same pull request change only the driver's argument filtering (`--bench` is dropped before the existing argument checks), add extreme-value test assertions, and edit documentation; the timed `--measure` path and its inputs are unchanged, and the archived binary identities refer to the measured commit.

Arm host: `dev-dsk-ahrav-2b-7dc7bd93.us-west-2.amazon.com`, aarch64, ARM Model 1 stepping r1p1, 64 available CPUs, Linux `6.12.103-129.197.amzn2023.aarch64`, Rust 1.98.1.
x86 host: `xxl` resolved to `dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com`, x86_64, Intel Xeon Platinum 8488C, 192 available CPUs, Linux `6.12.103-127.188.amzn2023.x86_64`, Rust 1.98.0.
Both compilers use LLVM 22.1.8, default target CPU/features, optimization 3 and debug level 1. Process affinity was CPU 0. Full host and compiler records are sealed externally.

## Results

Each row uses 12 order-balanced independent process pairs, with 16 timed calls per process. There are 408 complete process observations per host, including A/A. Times are median per full call in microseconds; ranges give the min/max process observations. Ratios are exponentiated mean paired log ratios, not ratios of medians. Intervals are nominal familywise 95% Bonferroni Student-t intervals over 32 contrasts, conditional on the declared model.

| Host | Shape | n / w | Candidate | Scan median [range], µs | Candidate median [range], µs | Ratio [interval] | 5% result |
|---|---|---|---|---|---|---|---|
| Arm | random | 4096 / 3 | blocks | 22.73 [22.26, 23.12] | 32.03 [30.04, 33.43] | 1.4074 [1.3528, 1.4642] | scan faster |
| Arm | random | 4096 / 3 | deque | 22.69 [21.99, 23.50] | 26.94 [25.84, 27.95] | 1.1846 [1.1421, 1.2288] | scan faster |
| Arm | random | 32768 / 3 | blocks | 207.21 [204.67, 219.67] | 566.67 [561.20, 579.93] | 2.7159 [2.6195, 2.8159] | scan faster |
| Arm | random | 32768 / 3 | deque | 207.04 [205.59, 229.61] | 282.76 [280.47, 284.42] | 1.3512 [1.3025, 1.4018] | scan faster |
| Arm | random | 4096 / 256 | blocks | 957.00 [940.58, 1029.33] | 25.12 [24.95, 25.75] | 0.0263 [0.0255, 0.0271] | candidate faster |
| Arm | random | 4096 / 256 | deque | 939.68 [936.32, 969.11] | 34.33 [33.54, 34.67] | 0.0363 [0.0357, 0.0368] | candidate faster |
| Arm | random | 32768 / 256 | blocks | 8557.60 [8203.30, 9198.09] | 395.31 [391.80, 409.94] | 0.0459 [0.0440, 0.0479] | candidate faster |
| Arm | random | 32768 / 256 | deque | 8687.79 [8258.32, 9089.88] | 375.26 [372.84, 376.33] | 0.0430 [0.0415, 0.0445] | candidate faster |
| Arm | increasing | 32768 / 256 | blocks | 8060.86 [8052.19, 8098.24] | 386.74 [384.32, 394.49] | 0.0480 [0.0475, 0.0484] | candidate faster |
| Arm | increasing | 32768 / 256 | deque | 8063.71 [8052.81, 8081.47] | 118.75 [117.69, 126.09] | 0.0149 [0.0145, 0.0154] | candidate faster |
| Arm | decreasing | 32768 / 256 | blocks | 7410.45 [7189.96, 7427.24] | 387.78 [386.00, 390.35] | 0.0527 [0.0519, 0.0535] | candidate faster |
| Arm | decreasing | 32768 / 256 | deque | 7420.12 [7187.09, 7429.41] | 159.18 [156.36, 164.86] | 0.0216 [0.0212, 0.0221] | candidate faster |
| Arm | equal | 32768 / 256 | blocks | 8063.60 [8049.71, 8142.64] | 405.28 [403.89, 408.04] | 0.0502 [0.0499, 0.0506] | candidate faster |
| Arm | equal | 32768 / 256 | deque | 8058.55 [8048.89, 8145.02] | 119.36 [117.16, 126.17] | 0.0149 [0.0144, 0.0154] | candidate faster |
| Arm | burst | 32768 / 256 | blocks | 7417.08 [7389.49, 7479.45] | 390.67 [388.78, 392.65] | 0.0527 [0.0524, 0.0530] | candidate faster |
| Arm | burst | 32768 / 256 | deque | 7420.97 [7394.34, 7437.18] | 161.22 [160.02, 162.47] | 0.0217 [0.0216, 0.0219] | candidate faster |
| x86 | random | 4096 / 3 | blocks | 26.43 [25.60, 29.70] | 43.71 [42.44, 46.14] | 1.6412 [1.5468, 1.7413] | scan faster |
| x86 | random | 4096 / 3 | deque | 26.43 [25.83, 27.32] | 27.55 [26.45, 29.41] | 1.0474 [1.0015, 1.0955] | unresolved |
| x86 | random | 32768 / 3 | blocks | 250.41 [246.29, 252.86] | 597.10 [593.35, 604.34] | 2.3921 [2.3594, 2.4252] | scan faster |
| x86 | random | 32768 / 3 | deque | 250.28 [249.20, 256.14] | 313.98 [309.52, 347.25] | 1.2646 [1.2158, 1.3152] | scan faster |
| x86 | random | 4096 / 256 | blocks | 822.61 [820.41, 848.66] | 22.93 [22.88, 24.16] | 0.0280 [0.0273, 0.0287] | candidate faster |
| x86 | random | 4096 / 256 | deque | 824.67 [820.72, 828.68] | 38.93 [37.50, 39.20] | 0.0470 [0.0462, 0.0478] | candidate faster |
| x86 | random | 32768 / 256 | blocks | 6989.96 [6980.94, 7009.47] | 342.97 [341.08, 346.07] | 0.0491 [0.0489, 0.0493] | candidate faster |
| x86 | random | 32768 / 256 | deque | 6989.25 [6981.99, 7007.47] | 422.40 [414.94, 429.14] | 0.0603 [0.0596, 0.0611] | candidate faster |
| x86 | increasing | 32768 / 256 | blocks | 6808.27 [6799.31, 6815.79] | 333.43 [331.72, 335.02] | 0.0490 [0.0488, 0.0491] | candidate faster |
| x86 | increasing | 32768 / 256 | deque | 6812.71 [6806.00, 6821.66] | 113.76 [112.97, 115.00] | 0.0167 [0.0166, 0.0168] | candidate faster |
| x86 | decreasing | 32768 / 256 | blocks | 5566.45 [5551.49, 5593.69] | 335.64 [334.30, 521.03] | 0.0625 [0.0537, 0.0729] | candidate faster |
| x86 | decreasing | 32768 / 256 | deque | 5567.62 [5556.15, 5585.43] | 159.46 [152.43, 162.19] | 0.0286 [0.0280, 0.0291] | candidate faster |
| x86 | equal | 32768 / 256 | blocks | 6816.32 [6806.43, 6825.50] | 358.55 [356.11, 360.22] | 0.0526 [0.0523, 0.0528] | candidate faster |
| x86 | equal | 32768 / 256 | deque | 6814.10 [6796.08, 6838.55] | 113.72 [113.37, 126.77] | 0.0170 [0.0162, 0.0178] | candidate faster |
| x86 | burst | 32768 / 256 | blocks | 5836.02 [5829.80, 5876.31] | 337.25 [335.06, 339.18] | 0.0577 [0.0575, 0.0580] | candidate faster |
| x86 | burst | 32768 / 256 | deque | 5837.34 [5828.97, 5853.45] | 137.80 [137.35, 147.18] | 0.0240 [0.0232, 0.0248] | candidate faster |

Scan has a 5% separation in seven of eight width-three contrasts. The x86 4096-element deque contrast is unresolved. Both optimized candidates have a separation from scan in all 24 width-256 contrasts. There is no declared deque-versus-block test, so their median differences do not establish a winner.

A/A mechanical integrity passed: identical linked binary, distinct arm labels, all 12 complete pairs, positive times. Diagnostic unadjusted 95% intervals:
- Arm: ratio 0.9985, interval [0.9949, 1.0021].
- x86: ratio 0.9977, interval [0.9874, 1.0081].

Startup is outside candidate timing. The recorded in-process setup ranges include generation, direct-oracle verification and three warmup calls, so they do not estimate OS process-launch latency. Inner calls are subsamples. Shared-host interference, compiler differences, and the short campaign limit generalization.

## Correctness, counts, and generated code

Both hosts passed six native integration tests and one doctest, including 3,280 exhaustive arrays and 27,884 array-width cases. The focused example reports deque 10 comparisons and stack 12. The suffix oracle uses 14. The width-64 descending-then-large regression observes 63 deque removals in one arrival, demonstrating why linear total work does not imply constant update work.

| Shape at n=32768, w=256 | Scan comparisons | Deque comparisons | Block comparisons | Deque peak | Maximum removal burst |
|---|---:|---:|---:|---:|---:|
| random | 8290815 | 65263 | 97793 | 18 | 13 |
| increasing | 8290815 | 32767 | 97793 | 1 | 1 |
| decreasing | 8290815 | 32767 | 97793 | 256 | 0 |
| equal | 8290815 | 32767 | 97793 | 1 | 1 |
| burst | 8290815 | 65025 | 97793 | 256 | 255 |

Library assembly and linked symbols were inspected on both hosts. Scan, deque, block and counted entry points exist separately. Allocation calls remain visible in the deque entry point. These observations do not establish a bottleneck. `perf stat` succeeded on both hosts, but its whole-process counters include direct-oracle setup and counted implementations. They cannot identify a candidate-only branch or cache mechanism.

The source reserves one output buffer for scan, candidate and output buffers for deque, and prefix, suffix and output buffers for blocks. Those are code-level buffer counts, not measured allocator-call counts. Timing includes allocation and destruction.

## Retention and validation

External raw archive directory: `/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-006/20260930/`. Both receipt archives passed SHA-256, unique/safe member, complete manifest, exact source/runner, sample-count, pair-completeness and binary-identity checks. Replay archives and the exact runner are retained.

Local checks passed: whitespace, formatting, all workspace lib/examples and all-targets tests (60 tests), workspace doctests, Clippy with warnings denied, benchmark compilation, and rustdoc with warnings denied. Python syntax and Student-t numerical/convergence checks passed. The running-example tables were rendered and inspected.

Measured facts: complete process times, operation instrumentation, and test outcomes. Derived facts: asymptotic bounds and source-level storage budgets. Inferred claims: any causal explanation of timing. Unmeasured domains include other machines, allocation-free reuse, cold starts, strict per-arrival wall time, floating-point ordering and mutable historical data.
