# Measurements

Preliminary scratch contracts and 1,200-process campaigns completed on both declared Linux hosts.
Final exact-source campaigns each contain 1,200 valid processes. Source commit: `3fae2aced4bef391c2e99abee2f66b3242222517`. The external retention root is
`/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-010/20261006`.

Arm: dev-dsk-ahrav-2b-7dc7bd93.us-west-2.amazon.com, aarch64, Arm implementer 0x41/part 0xd40,
64 available CPUs, Rust1.98.1. xxl resolves to dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com,
x86_64, Xeon Platinum8488C,192 CPUs,Rust1.98.0. Both use Amazon Linux kernel6.12.110-135.202,
default target cfg and CPU0. Full strings/features and flags are retained in environment receipts.

Input generation, BTreeMap oracle and separately instrumented standard comparison counts are outside
sort timing. Input cloning has its own timing. Counts do not identify candidate-only dynamic
mechanisms. Raw disassembly and process wall times are external. No cache/branch causal attribution.

Runner attestation gap: the raw receipts name `campaign.py` as the executed runner. That wrapper
was staged beside the transferred source on each host and was not committed; no hash of it is
retained. `scripts/run_linux.py` is the committed replay of the same 12-block schedule, and the
receipt hash covers that committed script only. The exact-source selections below rest on the
archived raw samples and environment receipts; the executed runner text is unattested.


## Exact-source results

Times are geometric means in microseconds. A `unique` cell passes the simultaneous >5% rule
against every eligible rival. Other cells name unresolved rivals to the point-fastest candidate.
Stable candidates can also compete when equal order is unconstrained; unstable candidates are
excluded from the stable subset.

| Host | Workload | Unconstrained equal order | us | Rivals unresolved | Stable | us | Rivals unresolved |
|---|---|---|---:|---|---|---:|---|
| arm | cold16 | dispatch | 0.362 | unstable | dispatch | 0.362 | unique |
| arm | tiny16 | dispatch | 0.081 | unstable,stable,quick | dispatch | 0.081 | stable |
| arm | random4096 | unstable | 48.485 | unique | dispatch | 60.756 | stable |
| arm | random65536 | unstable | 1239.302 | radix | radix | 1309.606 | unique |
| arm | large1048576 | unstable | 23051.364 | radix | radix | 23526.519 | unique |
| arm | equal4096 | stable | 2.217 | unstable | stable | 2.217 | unique |
| arm | duplicates4096 | unstable | 14.426 | unique | stable | 18.994 | unique |
| arm | sorted4096 | unstable | 2.143 | stable | stable | 2.219 | unique |
| arm | reverse4096 | unstable | 3.796 | stable | stable | 3.86 | unique |
| arm | organ4096 | merge | 11.803 | unique | merge | 11.803 | unique |
| arm | runs4096 | merge | 29.208 | unique | merge | 29.208 | unique |
| arm | nearly4096 | stable | 39.208 | unstable,merge | stable | 39.208 | merge |
| arm | groups4096x16 | dispatch | 28.927 | groups | dispatch | 28.927 | groups |
| arm | skew4096 | unstable | 49.648 | unique | stable | 61.818 | dispatch,groups |
| xxl | cold16 | quick | 0.548 | unstable,stable,dispatch | dispatch | 0.556 | stable |
| xxl | tiny16 | dispatch | 0.074 | unstable,stable,quick | dispatch | 0.074 | stable |
| xxl | random4096 | unstable | 45.214 | unique | stable | 63.048 | dispatch |
| xxl | random65536 | unstable | 1423.743 | unique | stable | 1893.687 | radix,dispatch |
| xxl | large1048576 | unstable | 27096.434 | unique | stable | 41760.698 | dispatch |
| xxl | equal4096 | unstable | 1.354 | stable | stable | 1.393 | unique |
| xxl | duplicates4096 | unstable | 12.561 | unique | stable | 18.191 | unique |
| xxl | sorted4096 | unstable | 1.357 | stable | stable | 1.391 | unique |
| xxl | reverse4096 | stable | 2.955 | unstable | stable | 2.955 | unique |
| xxl | organ4096 | merge | 14.548 | unique | merge | 14.548 | unique |
| xxl | runs4096 | stable | 33.985 | merge | stable | 33.985 | merge |
| xxl | nearly4096 | unstable | 38.623 | stable | stable | 41.17 | dispatch |
| xxl | groups4096x16 | groups | 27.072 | dispatch | groups | 27.072 | dispatch |
| xxl | skew4096 | unstable | 44.355 | unique | dispatch | 61.271 | stable,groups |

The stable large-random Arm cells select radix; x86 stable std/dispatcher remains unresolved.
Unconstrained random x86 cells select standard unstable; Arm large random unstable/radix remains
unresolved. Natural merge wins organ-pipe on both and sixteen-run input on Arm. Group/dispatcher
remains unresolved for small presorted groups. The 75/25 skew restores the unconstrained standard
unstable winner; stable variants remain unresolved. Cold and tiny calls have limited timer resolution.
These choices apply only to the declared fixed-width records, keys, hosts, flags and timing boundary.
No production traffic, generic comparator, allocator peak or candidate-only hardware mechanism is measured.
