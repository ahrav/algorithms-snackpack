benchmark_required: true

## Frozen exploratory design

Compare direct, Horspool, full Boyer-Moore, Two-Way, and adaptive exact first
match on each named Linux host. The target population is the fixed synthetic
fixtures on these two host/toolchain combinations. Generalization to fleets,
cold caches, service latency, Unicode, and other libraries requires new work.

Cross n={64,32768}, m={4,32}, five shapes, and two endpoints: 40 cells per host.
Shapes are xorshift seed39 broad-byte late match, four-byte-background late match with a terminal255 sentinel,
repeated-prefix miss a^(m-1)b, repeated-suffix miss ba^(m-1), and immediate match.
An independent oracle asserts actual result positions before every timing.

Prepared endpoint times search and checksum consumption. One-shot endpoint
also includes preparation, allocation, and drop. Input generation, correctness
oracle, diagnostics, warmup, and process startup are outside the timers.
Each process performs four untimed searches and then one timed batch. Early
matches use 100000 queries; others use max(16,floor(500000/n)). Inner queries
are subsamples. Warm cache is expected; CPU caches are shared between processes.

A fresh process receives one candidate. A complete five-candidate block is the
analysis unit. Eight blocks alternate forward/reverse order, balancing every
candidate's positions. Assignment is deterministic with no randomization seed;
fixture seed is39. The schedule is fixed before collection. Processes run
serially per host, pinned to the smallest permitted CPU; hosts run independently.
Shared thermals, kernel activity and non-linear time drift remain uncontrolled.

Use one warmup process per candidate/cell, excluded by its predeclared phase.
Report per-candidate median and observed min/max across eight processes. A
candidate is a screened winner only if it is at least5% faster than every rival
in every paired block. Otherwise report unresolved plus the smallest observed
median. These descriptive ranges are not confidence intervals. No significance,
familywise error, power, or production-optimum claim is made. All40 cells/host
form the fixed report family. There is no measured pilot variance; eight blocks
are an exploratory resource allocation. Fixed stopping; retain failures and
partial blocks. Failed correctness removes the candidate from speed selection
and is reported explicitly. Complete-case selection requires all eight blocks.

Eight A/A pairs use the same direct candidate/binary through labels A and B,
with alternating order. Report mechanical identity/completeness separately from
observed paired ratios. One A/A experiment cannot calibrate a general null law.

The runner retains all attempts, schedule, SHA256 input and linked binary
identities, host/kernel/CPU/toolchain, native target flags, affinity, contracts,
example work counts, disassembly, and complete marker. Diagnostic counters are
separate from primary timed loops through constant specialization. Tables use
fixed byte arrays; full Boyer-Moore also builds two pattern-length arrays, one
temporary. Memory costs are derived from source, not allocator measurements.

No hardware causal claim is planned. Disassembly records available code shape;
path-specific cache/branch counter attribution requires a separate experiment.
