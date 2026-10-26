benchmark_required: true

# Frozen traversal experiment

Compare four equivalent minimum-unweighted-distance implementations (`deque`,
`vector`, `csr`, `layers`) and, separately, two equivalent ordered DFS event
implementations (`frames`, `events`). Graphs are immutable directed adjacency
lists, vertex IDs are `usize`, neighbor order is preserved, source is zero.
All four BFS candidates must equal a separately implemented minimum-key heap
oracle. DFS must match recursive entry/exit/cycle events in bounded tests;
large workload DFS candidates must agree and match oracle reachability.
The heap and recursive oracle are correctness controls, excluded from selection.

Workloads: six-vertex running example; 256- and 32,768-vertex paths and stars;
128x128 directed grids with original and bijectively permuted IDs; 32,768
vertices with four outgoing edges; 1,024 vertices with 64 outgoing edges;
32,768 allocated vertices with only a 256-vertex path reachable. Generator
seed is 123 and every process reports input counts and an ordered fingerprint.
Paths test depth, stars width, grids layout, random cyclic graphs density,
and disconnected graphs full-initialization cost. Mutation/read mix is
immutable construction followed by repeated reads. Weighted, parallel,
external-memory and incrementally updated graphs are outside this campaign.

Timing includes per-traversal state initialization, queues/stacks, traversal,
materialized output, and destruction. Construction, validation, CSR conversion,
oracle and result comparison are excluded. Construction and CSR conversion
are recorded separately in every process. Both layouts coexist; this campaign
buys steady-state query timing with extra preparation/storage. First timed
calls follow correctness work, so `first_ns` is a warmed diagnostic, not cold
cache evidence. Process wall time includes startup, setup, oracle and all
repetitions; it cannot isolate process startup from those other components.

One discarded warmup process/candidate/cell; six measured processes/candidate/
cell. Fixed complete forward/reverse orders alternate. Every pair has exactly
three precedences in either direction. No random assignment or host sampling
is claimed. Inner repetition counts are fixed in the generator and amortize
the timer. The analysis unit is one fresh process; its mean time per complete
traversal is a single observation. Hosts run independently; within-host
processes are serial. No affinity, exclusive host, fixed clock or cache flush
is imposed. State/output allocation resets each traversal. Allocator/cache
history within a process is shared and limits generalization.

Predeclared selection: the lowest median within each result family is the
measured leader. Select it decisively only if every paired competitor/leader
ratio exceeds 1.03 across all six blocks. Report full process ranges and every
unresolved cell. This is a descriptive criterion without a nominal confidence
level, calibrated error probability or multiplicity guarantee. Six processes
are a bounded instructional budget, fixed before the committed campaign;
the initial two-process tiny example establishes runnable correctness, not
precision for the full matrix. No statistical power claim is made. Stop after
six complete blocks. Retain failed/partial runs; a missing process makes its
cell unresolved. Do not repeat to obtain a winner.

A/A: `deque_a` and `deque_b` dispatch to the same deque function in the same
binary on the 32,768-vertex path. One discarded warmup per label, six measured
processes per label, three runs in each order. Use the same subprocess/parser/
mean/paired-ratio path. Report mechanical identity/order checks separately
from the descriptive A/A ratio range. One A/A campaign cannot establish
coverage of a statistical confidence procedure.

Exact flags: rustc --edition=2024 -C opt-level=3 -C codegen-units=1
-C target-cpu=native. The runner records verbose compiler version, enabled
cfg target features, uname, CPU model/topology and available CPU affinity.
Source and runner SHA-256, linked binary SHA-256, symbols and disassembly are
retained. Static generated code can establish symbol/code presence. Timing
alone cannot attribute a cache, branch or allocation mechanism. Hardware
counters are optional diagnostics and do not contain selection evidence.

Replay from this topic: `python3 run.py --out /tmp/traversal-new-run`.
Use a new destination. The focused mode limits measurements to the tiny graph.
