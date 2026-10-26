# Measured traversal selection

Frozen executable source: `d2676bd5975d82856d4188982a3b75a3ab085c8f`.
Source/runner transfers were SHA-256 checked against that Git commit. Both hosts
passed all four contract-test groups. Every measured BFS vector matched the
independent heap oracle. DFS candidate orders and cycle flags matched, and
reached sets matched heap distances. Recursive order checking is bounded to
small graphs. All 434 rows/host are retained: 360 main measurements,12 A/A
measurements,60 main warmups,2 A/A warmups. Each main pair has3:3 order balance.

Arm: `dev-dsk-ahrav-2b-7dc7bd93.us-west-2.amazon.com`, aarch64,64 available CPUs,
kernel6.12.111-136.204.amzn2023.aarch64, rustc1.98.1. CPU implementer0x41,
part0xd40,r1p1; rustc identifies native CPU as neoverse-v1. x86: alias `xxl`
resolved to `dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com`,x86_64,192 available
CPUs,Intel Xeon Platinum8488C,kernel6.12.111-136.204.amzn2023.x86_64,rustc1.99.0.
Both builds use opt-level3,codegen-units1,target-cpu=native,edition2024.
Raw environment receipts retain enabled target features, topology and affinity.
The runner's cfg diagnostic used default optimization; the external supplement
records cfg under the exact build flags. Target-feature lists agree.

# Committed-source traversal results

All times are microseconds per full traversal. Brackets give the min/max of six fresh-process means. `unresolved` identifies a median leader that failed the frozen all-pairs 3% rule.

| Workload | Arm BFS | x86 BFS | Arm DFS | x86 DFS |
|---|---|---|---|---|
| tiny | deque 0.07 [0.07,0.07] selected | deque 0.05 [0.05,0.05] selected | frames 0.21 [0.21,0.21] selected | frames 0.13 [0.13,0.14] selected |
| path_small | deque 1.52 [1.50,1.55] unresolved | deque 1.43 [1.43,1.44] unresolved | events 3.49 [3.45,3.52] unresolved | events 2.52 [2.47,2.57] selected |
| star_small | vector 0.69 [0.68,0.71] selected | vector 0.56 [0.55,0.61] selected | events 2.92 [2.84,3.04] selected | events 1.90 [1.82,2.00] selected |
| path | deque 193.56 [192.79,194.79] unresolved | deque 185.75 [183.44,188.82] selected | frames 634.09 [629.13,645.50] selected | events 593.28 [587.96,602.43] unresolved |
| star | vector 67.66 [67.17,68.15] selected | deque 97.00 [96.38,97.66] selected | events 240.77 [223.43,257.28] selected | frames 179.22 [177.22,184.79] unresolved |
| grid | layers 68.63 [66.49,71.30] selected | layers 59.80 [57.78,60.46] selected | events 329.63 [320.77,336.22] unresolved | events 361.22 [354.09,399.47] unresolved |
| permuted_grid | layers 91.45 [86.42,93.26] selected | layers 69.46 [68.23,70.27] selected | events 328.12 [323.22,334.16] selected | events 359.54 [346.06,369.27] unresolved |
| sparse | layers 744.42 [724.08,778.46] unresolved | csr 583.42 [576.12,612.04] unresolved | events 640.85 [638.77,643.61] selected | events 768.87 [752.25,791.96] selected |
| dense | layers 52.43 [51.40,54.10] unresolved | vector 44.06 [43.66,45.53] unresolved | events 291.58 [289.63,293.75] selected | events 362.74 [359.75,369.32] unresolved |
| disconnected | deque 4.75 [4.72,4.82] unresolved | deque 6.30 [6.24,6.32] unresolved | events 3.76 [3.70,3.90] unresolved | events 2.93 [2.88,3.01] unresolved |

## Read the results

The large star selects vector on Arm and deque on x86. Tiny graphs select deque
on both. Small stars select vector on both. Both grids select level frontiers
on both. Small paths, random sparse/dense graphs and disconnected graphs have
unresolved BFS cells. The x86 large path selects deque; Arm large path remains
unresolved. Every DFS cell has its own event-contract comparison in the table.

CSR layout alone has no universal speed advantage here. For the original grid,
Arm deque median85.564us versus layers68.625us saves16.939us/traversal; median
CSR preparation223.814us gives a descriptive quotient13.2 traversals. x86 saves
25.140us/traversal with235.473us preparation, a quotient9.4. These arithmetic
estimates compare separately measured medians and exclude retained-layout
memory cost. They are illustrative, not calibrated crossover guarantees.

A/A uses the same binary and deque function with two treatment labels. Arm
B/A median1.005, range[0.977,1.006]; x86 median0.997,range[0.949,1.009]. Identity,
label parsing and3:3 order checks pass. Those ranges show measurable noise;
they do not establish confidence-interval coverage or a universal noise floor.

## Operations, storage and limits

Each BFS traversal initializes V distances, discovers R vertices and examines
E_R edges. DFS additionally emits R entry and R exit events. The workload
reported counts are derived operation controls, not hardware-instruction
measurements. All graphs in this campaign store edges only in reached regions,
so E_R=E. The disconnected case initializes32,768 states to reach only256
vertices and examine255 edges. This explains a cost term, not a measured
CPU-level bottleneck.

Timing includes output creation/destruction and fresh traversal state. Input
construction, CSR conversion, oracle and comparisons are outside the query
boundary. The first timed call follows correctness work and uses warmed data.
Process wall timings are retained separately and contain startup+setup+oracle+
repetitions. A cold-cache or startup-only comparison remains outside this run.

The experiment runs serial processes per host without affinity, exclusive CPU,
frequency control or cache flushing. Six processes provide descriptive ranges;
there is no confidence or multiplicity guarantee. Report unresolved cells
instead of declaring a global optimum. Heap/recursive oracles are correctness
controls, excluded from timing selection. Different host compilers prevent an
architecture-only causal comparison.

Linked binaries, symbols and complete disassembly are in the raw archives.
Both symbol tables contain a separately named dfs_frames function. The other
candidates have no separately named symbols in those listings. Full disassembly
is retained. These observations do not identify the runtime cause of a timing
difference. Cache, branch and allocation explanations remain hypotheses; no
candidate-only hardware-counter experiment was run.

## Replay and retention

From this topic, run `python3 run.py --out /tmp/traversal-fresh` with a new
output directory on either required Linux host. BENCHMARK.md fixes the full
workload, timing, warmup, assignment and selection protocol. SUMMARY.json is
the compact aggregate; EVIDENCE_RECEIPT.json identifies the externally retained
raw archives, exact source, environments, binaries and manifest verification.

The raw sample unit is a fresh process. Inner repetitions amortize the timer.
The final displayed medians/ranges are rounded; JSON retains full precision.
