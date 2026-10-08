# Frozen experiment

benchmark_required: true

Claim: fastest correct candidate depends on stability, length, key shape, run structure and
prefix-group skew. No universal optimal sorter or learned dispatch threshold is claimed.

Sample unit: one process per candidate/workload/block. Twelve paired blocks share a deterministic
seed across candidates. Within each pair of blocks, the same cyclic rotation runs forward then
backward. Thus every candidate pair has six observations in either order. Inner repetitions
amortize clocks and are not independent samples. Exactly twelve blocks; no winner-driven reruns.

Workloads: cold entry and warm 16; random 4,096, 65,536 and 1,048,576; equal, eight-key duplicates,
ascending, descending, organ-pipe, sixteen ascending runs, adjacent near-sorted swaps, 256 prefix
groups of 16 and a two-group 75/25 skew. Seven candidates per cell; `groups` additionally enters
only the two explicitly admissible group cells. This is 1,200 processes per host.

Boundary: sort entry to return, including algorithm scratch allocation/initialization/destruction.
Input cloning is measured separately. Generation, oracle, comparison instrumentation, output
checksum, input teardown and process startup are excluded; total process wall is retained separately.
Warmup: three calls, except first-entry cold cell. Input/allocator/OS caches are not claimed cold.
Cloning and validation warm input. Setup and repeated use are declared parts of this lab context.

Selection: geometric mean time. Pair log ratios by workload and block. Student-t intervals use
11 degrees of freedom and a Bonferroni family of 620 contrasts, conservatively covering all
616 two-host pairwise comparisons. A unique winner needs every eligible rival's simultaneous
lower time-ratio bound above 1.05. Otherwise report the point-fastest candidate and unresolved
rivals. Give a separate stable subset selection. Model-based intervals assume representative
process errors; shared-host drift and cold clock quantization remain limitations.

CPU: first available affinity CPU; default rustc target features, opt-level 3, no LTO or native
CPU flag. Record hostname, architecture, full kernel, CPU model/part, CPUs, toolchain and cfg.
Verify frozen transferred source/runner hashes, archive hashes and all raw-file hashes.

Independent ordered-tree oracle rejects incorrect output before warm timings, and validates
first-entry cold output afterward. All implemented candidates also pass exhaustive and shape
contracts. Comparisons are separately instrumented only for standard sorts. Structural scratch
and radix movement counts come from code, not allocator/PMU measurements. Linked disassembly
is retained; no candidate-only branch/cache causal claim comes from whole-process evidence.

Preliminary scratch campaign used an earlier comparison expression and an imperfect order rotation.
It is retained as diagnostic evidence. Exact committed source and corrected runner define selections.
