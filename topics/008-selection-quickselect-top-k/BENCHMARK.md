# Frozen Top-K experiment

benchmark_required: true

Claim: choose the fastest correct candidate among sort, standard, three_way,
heap, and buffer for exactly sorted smallest-K output over immutable resident
integer input. No universal dispatch threshold or dynamic mechanism claim.

The fixed `scripts/run.py` WORKLOADS declare all 11 cells: N=32/K=4;
N=4096/K=8; N=65536/K=8 random, ascending, descending, 8-value duplicates,
all equal, organ pipe; N=4096/K=2048 and K=4096; and first use at N=65536/K=8.
All seeds and call counts are frozen in runner/driver.

Boundary: start immediately before candidate calls; stop after the returned
vector drops. Include allocation, N-copy where applicable, preprocessing,
selection, updates, sorted output, black-box result exposure, endpoint
checksum consumption, and teardown. Exclude process launch, compilation,
input generation, oracle construction, correctness checks, and diagnostic
operation counts. Warm cases use four untimed calls, then a fixed batch.
First use uses one timed call, no candidate warmup; it is not cold-cache or
end-to-end startup latency because input/oracle preparation occurred first.

20 process blocks. Rotate five candidate orders forward then reverse in pairs;
all five positions balance over 10 blocks. Each cell uses one independent
process per candidate per block. Inner calls reduce clock noise; they are NOT
independent samples. A/A runs identical standard artifacts in alternating order.
There are exactly 1140 timed processes per complete host campaign.

Pin to the first permitted CPU on Linux. Compiler: rustc edition 2024,
opt-level=3, debuginfo=1, compiler-default target features; record rustc -Vv,
rustc --print cfg, hostname, uname, lscpu, available CPUs, binary SHA256,
and path-limited source SHA256. No frequency governor manipulation.

Criterion: geometric mean per-call time. The point-fastest candidate is
selected only if every competing other/fastest paired log-ratio interval has
lower bound >1.05. Otherwise report the inseparable contender set. Bonferroni
95% family intervals over 111 contrasts (11 cells * 10 pairs + A/A),
Student-t log-ratio model with 19 degrees of freedom. The model assumes
independent approximately normal block log ratios and is not a universal
coverage guarantee. Retain min/max dispersion and all attempted observations.
No optional stopping, winner-seeking reruns, or post-hoc candidate removal.

Correctness: each candidate process checks the complete sorted output against
an independent BTreeMap ordered-count oracle outside the timer. Reject a
failure before selection. Tests inspect rank, partition ordering, multiset,
integer extremes, invalid requests, equal-band progress, and budget fallback.

Counters: teaching selector classifications, explicit swaps, partitions, and
fallback are measured after timing. They are source-level events, not CPU
instructions. Linked disassembly is retained. Whole-process counters, if
collected, include oracle/setup and cannot identify candidate-only mechanisms.

Initial source campaigns happen before repository mutation. Exact committed
code/runner campaigns run on BOTH declared Linux hosts. Their receipts are
separate; final claims use the committed campaigns. Raw stdout/stderr,
ledgers, environments, and disassembly remain external to Git.
