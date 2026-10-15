# Fixed portfolio protocol

benchmark_required: true

Question: which correct representation minimizes a complete declared batch?
Four statically dispatched candidates: eager scan, suffix-repaired prefix array,
linear-build Fenwick and iterative padded segment tree. All aggregate arithmetic
is modulo 2^64. Each process checks EVERY query against a separate eager array,
drops both validation objects, warms one full batch, then starts a fresh logical
state before timing. The timed batch checksum must equal the oracle batch sum.

Eight blocks use cyclic candidate permutations, then their reversals. The
process is the sample unit. There is one timed batch per process. Workload seed,
source, default target, optimization and schedule stay frozen. Timing excludes
oracle, workload generation, construction and teardown except lifecycle, which
includes index construction, sixteen queries and destruction. No allocator or
cache flush controls; processes remain unpinned. Capture available CPU affinity,
CPU model, kernel, rustc, target cfg queried with `-O`, flags, source hashes,
linked-image hash and linked objdump.

Cells: tiny n16/q32768/read50%; read_only n4096/q8192/read100%; mixed same
n/q/read50%; update_heavy same n/q/read5%; large n262144/q2048/read95%; short
n262145/q4096/read50%/width<=8; hot n262144/q4096/read50%/first16 starts and
width<=8; lifecycle n4096/q16/read100%. Read choices are deterministic
pseudo-random trials, so rates describe probabilities rather than exact counts.
Broad ranges choose a random start then an endpoint from that suffix, including
empty intervals. Inputs repeat i modulo17; additions alternate by seeded choice
between +1 and modular -1. The stream is identical for all candidates.

Control: after the candidate blocks, each cell runs eight further blocks of two
processes labeled `fenwick_a` and `fenwick_b`. Both labels run the one Fenwick
executable; their order alternates by block. The same parser, checksum check and
selection rule analyze them. Checksum equality is mechanical integrity; the label
ratio range is the null calibration. A control cell that reports a winner
invalidates every winner label on that host for the campaign.

Selection rule fixed before execution: choose lowest median only if each of its
eight block-paired ratios against every other candidate exceeds 1.05. Otherwise
report unresolved. Retain medians, min/max and paired ratio ranges; these are
descriptive dispersion, not confidence intervals or a universal threshold.
Stop after eight blocks. Do not rerun to manufacture a winner.

Inference: the effect boundary is a 5% paired ratio, one-sided (competitor
slower). When a competitor and the median-best candidate are exchangeable, all
eight paired ratios fall on one side of 1 with probability 2^-7, so a cell-host
winner claim has false-winner probability at most 2^-7 against any competitor it
does not truly beat. The family is sixteen cell-host winner claims with equal
Bonferroni allocation: the bound on any false winner is 16/128 = 1/8. No
interval estimate is formed. The rule predates execution; this characterization
of its error rate was written after the committed campaign.

Failures: each subprocess has a 600 s timeout. Every invocation, including a
nonzero exit or timeout, is appended to `raw/attempts.jsonl` before the campaign
aborts, and the log is truncated once at campaign start. A campaign with any
failed attempt is not published; timing uses complete campaigns only.

This measures batch throughput, not operation latency quantiles. Setup/validation
can pre-touch memory. Lifecycle is warmed whole-operation cost, not process
startup or a cold-cache claim. Direct selection, range additions, library-tuned
competitors, allocation byte peaks and hardware counters are outside timing.
No CPU-vendor/ISA causality follows from different hosts/toolchains.
