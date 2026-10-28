# Benchmark decision: true

Compare whole-source queries across scan, lazy binary, indexed binary, lazy
radix, circular Dial and the 0/1 deque. All candidates must agree with the
independent synchronous Bellman-Ford oracle before timing. The scan is the
simple selection baseline; the slower oracle is a separate correctness gate.

## Frozen protocol

Nine deterministic workloads: tiny five-vertex example; 128-vertex sparse;
4,096-vertex sparse, bounded (0..7) and binary (0..1); 512-vertex dense;
256-vertex repeated-decrease adversary; 4,096-vertex maximum-u32-weight chain;
and 4,096 vertices with only three reachable. Directed ring edges ensure
reachability in generated connected cases. Duplicate edges count separately.
Fixed graph generation and repetition counts live in `benches/queues.rs`.

One warmup process per workload/candidate, then six measured process blocks.
Alternate full forward/reverse order, giving every candidate pair 3:3 order.
Process means, rather than inner iterations, form the six samples. Stop after
six blocks, including unfavorable results. Each workload adds `lazy_aa`, the
same code as lazy, as a diagnostic control. It is excluded from candidate ranking.

Choose a unique winner only when it is over 5% faster than every other admitted
candidate in every paired block. Otherwise label selection unresolved and name
the lowest-median candidate descriptively. Report median and observed min/max.
Ranges describe these observations; they are not confidence intervals.

## Boundaries and controls

Build on each host with `--edition=2024 -C opt-level=3 -C target-cpu=native`.
No external crates. Record hostname, architecture, kernel, CPU, available CPU
set size, compiler and build flags, source and runner hashes, binary hash,
linked symbols and disassembly. The same source is transferred to both hosts.
No core isolation or frequency controls are imposed; host contention is a limit.

Timed repetitions include distance initialization, queue allocations, work
counters, edge processing, result construction and result/scratch destruction.
Each process first records a separate initial query, then validates it against
the oracle. Timed repetitions therefore use a previously traversed graph with
fresh query allocations. The initial query is a fresh-process observation;
hardware cache coldness is uncontrolled. Graph generation/admission, the oracle,
first query and process wall time are recorded outside steady-state timing.
Process wall time includes all validation, so it is not isolated startup latency.

The queue algorithm varies while layout and output contracts stay fixed. Graph
admission allocates adjacency lists and computes the maximum weight once.
Reported steady-state selections assume that graph is reused. One-shot users
must add admission and first-query costs. Queue peak records exclude allocator
slack, bucket headers and RSS. Static code and counters support source-level
work descriptions; they establish no cache or instruction-rate cause.

The pre-lesson scratch campaign is excluded from committed-source selections.
Reproduce on Linux with `python3 replay.py /absolute/new/run-directory` from
this topic. The runner compiles exact source, checks all contracts, validates
every measured process, freezes hashes and emits an external manifest.
