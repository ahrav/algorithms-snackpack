# Topic 060 measurements

Each host completed 378 fresh processes, including 216 timed, 144 raw/raw
controls and 18 warmup processes. The frozen source passed all four
integration tests and the standalone example on both architectures.
The runner compared each measured query with an i128 prefix oracle before
timing. An external Python oracle verified every process digest and visit
count. Both schedules matched the frozen assignment, including A/A labels.

Linux kernel 6.12.115 on both hosts. Arm used a 64-CPU ARM host and Rust
1.98.1. x86 used a 192-CPU Xeon Platinum 8488C host and Rust 1.99.0.
Both used release optimization, `-C target-cpu=native`, and CPU 0.
See [the receipt](EVIDENCE_RECEIPT.json) for checked source, linked-image,
process manifest and raw archive identities. Run `bash scripts/native.sh
"$PWD" /tmp/reduction-results` from the topic root on Linux.

Times are median ± median absolute deviation in microseconds per transaction.

| Host | Fixture | Raw | Chunked | Compact | A/A error | Selection |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| Arm | tiny | 0.144 ± 0.001 | 0.157 ± 0.001 | 0.133 ± 0.000 | 0.71% | Compact |
| Arm | large-once | 53.943 ± 0.481 | 34.135 ± 0.093 | 27.143 ± 0.261 | 1.35% | Compact |
| Arm | large-read | 1586.989 ± 12.349 | 151.424 ± 0.253 | 27.482 ± 0.292 | 0.78% | Compact |
| Arm | dense-cuts | 45.748 ± 0.266 | 111.800 ± 0.108 | 76.126 ± 0.454 | 1.45% | Raw |
| Arm | medium-stripes | 22.538 ± 0.103 | 5.858 ± 0.029 | 3.287 ± 0.015 | 0.57% | Compact |
| Arm | short-stripes | 21.033 ± 0.119 | 24.503 ± 0.104 | 16.193 ± 0.077 | 1.00% | Compact |
| x86 | tiny | 0.151 ± 0.012 | 0.165 ± 0.006 | 0.144 ± 0.005 | 7.58% | Unresolved |
| x86 | large-once | 67.399 ± 3.223 | 31.081 ± 1.253 | 22.015 ± 0.483 | 4.78% | Compact |
| x86 | large-read | 1501.279 ± 140.942 | 188.694 ± 36.670 | 24.861 ± 1.290 | 14.37% | Unresolved |
| x86 | dense-cuts | 57.765 ± 7.431 | 192.890 ± 42.014 | 79.790 ± 6.989 | 5.18% | Unresolved |
| x86 | medium-stripes | 20.749 ± 0.679 | 6.138 ± 0.433 | 3.971 ± 0.434 | 5.002% | Unresolved |
| x86 | short-stripes | 22.303 ± 0.810 | 27.171 ± 2.055 | 18.152 ± 2.788 | 6.23% | Unresolved |

In large-read, raw, chunked and compact visited 14,680,064, 458,752 and 224
records per transaction. They reserved 1,048,608, 131,104 and 64 bytes of
vector capacity, respectively. In dense-cuts each policy visited 137,288
records. Raw reserved 196,608 bytes, chunked 393,216 bytes and compact 262,144
bytes. These counts cover record visits and vector capacity. Allocator
overhead and physical memory traffic require different instrumentation.

The [summary](summary.json) includes phase medians, vector capacities and
paired speedup ranges. Both linked disassemblies are retained. The benchmark
main includes vector integer additions on x86. It contains fixture work,
oracle and timed work together, so the instruction excerpt alone cannot
attribute a speed difference to a candidate. The result applies to a warm
in-memory, fixed-cut workload and excludes cold storage or engine I/O.

The initial source passed integration tests on both hosts, but the scheduler
stopped at its first parsed output because host Python 3.9 lacks strict zip.
Replay used the committed compatible scheduler. The initial attempt logs
are retained separately and supply no time samples.
