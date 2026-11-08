# Benchmark decision: true

Claim: choose the fastest correct tested educational candidate for each fixed
workload and host. This is an exploratory screen with six complete process
blocks. It has no powered population-level or simultaneous confidence claim.
The stopping rule is fixed; every attempt is retained and failures halt the run.

## Frozen design

Case IDs 0..10 are fixed in examples/experiment.rs:

| IDs | Input | Patterns | Shape |
|---|---:|---:|---|
| 0 | 64 | 2 | random 12-byte patterns and random bytes |
| 1 | 64 | 32 | 31 a bytes plus distinct final byte; all-a input |
| 2,3,4 | 32768 | 2,32,128 | random |
| 5 | 262144 | 128 | random |
| 6,7 | 32768 | 32,128 | shared-prefix absent near misses |
| 8,9 | 4096 | 8,32 | a, aa, ...; dense outputs |
| 10 | 32768 | 32 | dense outputs |

Seed 0x79ab1357, xorshift generation, deterministic on both hosts. Every process
checks the entire result multiset against an independent scalar oracle before
timing. Contract tests additionally cover duplicates, all bytes and every split.

Treatments are direct/sparse/dense. A process invocation receives one treatment.
Each endpoint/case has one discarded warmup block, followed by six blocks.
The six orders are abc,bca,cab,cba,acb,bac, giving each candidate every position
twice. No random assignment is claimed. Each complete block is the paired
analysis unit. Inner loop repetitions are subsamples. Each block shares host
cache, thermal and scheduler state; fresh processes do not establish independence.
Build and host populations each contain one artifact per required host.

The fixed selection screen requires the candidate to have the lowest median
and take at most 95% of every alternative's time in all six paired blocks.
Otherwise report unresolved. Report per-process normalized medians and min/max,
and paired ratios. No inference threshold is estimated from the observed data.
Six alternating AB/BA identical-binary direct/direct controls at case3 diagnose
label/order asymmetry. They provide descriptive ratios, not a calibrated bound.

Scan boundary: reused matcher, resident bytes, every output passed to a digest
callback, no result allocation. Includes direct empty-pattern validation. Build
boundary: clone/destroy owned direct patterns, construct/destroy sparse or dense
matcher from borrowed inputs. Direct's owned-generation boundary is deliberately
conservative and separate from its borrowed-scan API. These endpoints exclude
process startup, generation, oracle, sorting, printing and file I/O. All candidates
are correctness-warmed. No cold-cache claim. Scan repetitions are
clamp(2000000/(n*r),1,2000); build repetitions are
clamp(20000/(P+256*S),1,1000). One process aggregate divided by repetitions is one
sample. Build includes allocation and teardown; scan digest includes output work.

Linux rustc -O equivalent opt-level3, target-cpu=native, edition2024, taskset at
minimum allowed CPU. Retain compiler/features/kernel/CPU/affinity and input,
runner and linked binary SHA256. Hosts may use different compiler versions.
Dense executes one table transition per byte by source construction. Sparse
edge/failure work depends on input; no hardware-counter attribution is claimed.
The table size is exact payload arithmetic; allocation counts, total resident
memory and peak memory require additional instrumentation. Library comparison,
real logs, cold starts, Unicode and concurrent generations remain revisit scope.
