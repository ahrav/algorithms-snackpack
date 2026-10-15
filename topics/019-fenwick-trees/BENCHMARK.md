# Fixed portfolio protocol

benchmark_required: true

Question: which correct representation minimizes a complete declared batch?
Four statically dispatched candidates: eager scan, suffix-repaired prefix array,
linear-build Fenwick and iterative padded segment tree. All aggregate arithmetic
is modulo 2^64. Each process checks EVERY query against a separate eager array,
drops both validation objects, warms one full batch, then starts a fresh logical state before timing.

Eight blocks use cyclic candidate permutations, then their reversals. The
process is the sample unit. There is one timed batch per process. Workload seed,
source, default target, optimization and schedule stay frozen. Timing excludes
oracle, workload generation, construction and teardown except lifecycle, which
includes index construction, sixteen queries and destruction. No allocator or
cache flush controls; processes remain unpinned. Capture available CPU affinity,
CPU model, kernel, rustc, target cfg, flags, source hashes and linked objdump.

Cells: tiny n16/q32768/read50%; read_only n4096/q8192/read100%; mixed same
n/q/read50%; update_heavy same n/q/read5%; large n262144/q2048/read95%; short
n262145/q4096/read50%/width<=8; hot n262144/q4096/read50%/first16 starts and
width<=8; lifecycle n4096/q16/read100%. Read choices are deterministic
pseudo-random trials, so rates describe probabilities rather than exact counts.
Broad ranges choose a random start then an endpoint from that suffix, including
empty intervals. Inputs repeat i modulo17; additions alternate by seeded choice
between +1 and modular -1. The stream is identical for all candidates.

Selection rule fixed before execution: choose lowest median only if each of its
eight block-paired ratios against every other candidate exceeds 1.05. Otherwise
report unresolved. Retain medians, min/max and paired ratio ranges; these are
descriptive dispersion, not confidence intervals or a universal threshold.
Stop after eight blocks. Do not rerun to manufacture a winner.

This measures batch throughput, not operation latency quantiles. Setup/validation
can pre-touch memory. Lifecycle is warmed whole-operation cost, not process
startup or a cold-cache claim. Direct selection, range additions, library-tuned
competitors, allocation byte peaks and hardware counters are outside timing.
No CPU-vendor/ISA causality follows from different hosts/toolchains.
