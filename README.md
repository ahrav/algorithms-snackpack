# Algorithms Snackpack

Executable learning guides for algorithms and data structures as they are used
in production systems.

Each topic starts from a precise contract, builds a simple reference model,
derives the relevant cost model, and then develops the fastest implementation
we can justify with evidence. Correctness comes first. Performance claims need
operation counts, allocation counts, generated-code inspection, and a controlled
benchmark. A smaller time is not an explanation by itself.

## Run the workspace

```bash
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo bench --workspace --no-run
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
```

## Repository rules

- Topic numbers and titles are stable once published.
- A first visit creates `topics/NNN-slug`; a revisit adds a round under that
  topic rather than replacing prior evidence.
- Every implementation has an explicit contract and an independent oracle.
- Every claimed invariant names a plausible violation that its test rejects.
- Performance work records the exact source, binary, machine, workload, sample
  unit, ordering, and raw observations.
- Sampled tests and benchmarks are reported as bounded evidence, not proof.
- The complete lesson is delivered in the daily task; the repository retains a
  compact, runnable learning guide rather than a transcript.

See [the methodology](METHODOLOGY.md) and [documentation rules](DOCUMENTATION.md).

## Table of contents

The catalog was deduplicated from all 31 historical installments in the source
conversation, then split where one heading contained materially different
contracts or cost models. `curriculum.toml` is the machine-readable authority.

| # | Topic | Central question |
|---:|---|---|
| 1 | Prefix sums and scans | How do we assign output positions at memory bandwidth? |
| 2 | Difference arrays and deferred range updates | When should boundary records replace eager mutation? |
| 3 | Bitsets and density-adaptive bitmaps | How should representation change as density changes? |
| 4 | Interval sets and compressed ranges | How do we preserve range semantics without per-element state? |
| 5 | Two pointers and sliding windows | Which monotonic movement makes a scan linear? |
| 6 | Monotonic stacks and deques | How do we remove dominated candidates with bounded latency? |
| 7 | Binary, galloping, and branch-aware search | When is fewer comparisons not the fastest search? |
| 8 | Selection, Quickselect, and Top-K | How do we avoid fully sorting data we will discard? |
| 9 | Reservoir and weighted sampling | How do we sample an unknown-length stream reproducibly? |
| 10 | Sorting algorithm portfolios | How should data shape choose the sorting strategy? |
| 11 | Merge algorithms and duplicate-run joins | How do ordering and multiplicity shape a merge? |
| 12 | Radix partitioning | How do digits become cache, ownership, and parallelism boundaries? |
| 13 | Open-addressed hash tables | What probe invariant gives predictable lookup and deletion? |
| 14 | SwissTable-style metadata filtering | How can control bytes reject candidates before key access? |
| 15 | Robin Hood hashing | When does equalizing probe distance improve tail behavior? |
| 16 | Cuckoo hashing | How do bounded candidate locations trade lookup for insertion risk? |
| 17 | Bloom, cuckoo, and quotient filters | How should an error budget buy memory and lookup speed? |
| 18 | Hash joins, recursive partitioning, and spill | When does a hash lookup become an external-memory algorithm? |
| 19 | Fenwick trees | What algebra makes compact prefix aggregation possible? |
| 20 | Segment trees and lazy propagation | How do we combine range queries with deferred updates? |
| 21 | Heaps and priority queues | Which heap shape fits the update and memory-access pattern? |
| 22 | Skip lists and hybrid ordered indexes | When is randomized order useful beside direct lookup? |
| 23 | B-trees and posting-list deduplication | How do page layout and duplicates reshape a search tree? |
| 24 | Tries, radix trees, ART, and sparse radix arrays | How should prefix structure adapt to fanout and memory layout? |
| 25 | Finite-state transducers and compact dictionaries | When can sorted keys become a minimal automaton? |
| 26 | Dynamic arrays and growth policies | What growth factor minimizes copies, slack, and allocator cost? |
| 27 | Ring buffers and bounded queues | How do wraparound and ownership define queue correctness? |
| 28 | Work-stealing deques | Which scheduler protocol permits one owner and many thieves? |
| 29 | Union-find and equivalence-class indexes | Which extra semantics survive path compression and union by rank? |
| 30 | Breadth-first and depth-first traversal | How do frontiers become schedulers and execution plans? |
| 31 | Dijkstra and shortest-path queues | When do weights, queue choice, and distribution change the algorithm? |
| 32 | Topological order and incremental DAG evaluation | How do updates preserve dependency order without a full rebuild? |
| 33 | Tarjan strongly connected components | How can components become safe graph-mutation boundaries? |
| 34 | Dominator trees and incremental maintenance | How do we maintain mandatory-path structure after graph edits? |
| 35 | Reachability indexes for large DAGs | When should traversal results become a persistent index? |
| 36 | Merkle trees and incremental rebuild caches | How do hashes localize change, verification, and repair? |
| 37 | Best-first and approximate graph search | How should a search budget trade recall for latency? |
| 38 | KMP and prefix-function matching | How does failure-state reuse guarantee linear matching? |
| 39 | Boyer-Moore, Two-Way, and adaptive string search | Which text and pattern properties justify skipping work? |
| 40 | Rolling hashes and content-defined chunking | How do layered verification and adversaries shape hash use? |
| 41 | Edit distance and automaton-guided lookup | How can a dynamic program prune a whole dictionary search? |
| 42 | Longest common subsequence and diff | Which equality and memory tradeoffs produce useful edits? |
| 43 | Multi-pattern matching and automata | How do many patterns share traversal without losing match semantics? |
| 44 | Varints, codecs, and bounded decoding | How do canonical encodings remain safe under hostile input? |
| 45 | Compression-oriented transforms | Which reversible transform exposes redundancy cheaply? |
| 46 | Streaming sketches | How should Count-Min, HyperLogLog, and quantile sketches spend error? |
| 47 | Consistent hashing and bounded-load placement | How do we limit movement, imbalance, and assignment churn? |
| 48 | LRU, CLOCK, and workload-aware eviction | Which approximation preserves value without hot-path contention? |
| 49 | Timing wheels, heaps, and timer trees | Which deadline structure fits the timer distribution and clock model? |
| 50 | Token buckets and rate accounting | How do burst policy, byte accounting, and fairness interact? |
| 51 | Buddy allocation and hierarchical free-space tracking | When does coalescing beat fragmentation and hot-path overhead? |
| 52 | Red-black trees and intrusive representations | How does representation affect balanced-tree cost? |
| 53 | Load balancing and stable assignment | How do power-of-two choices, rendezvous hashing, and locality compete? |
| 54 | Dynamic programming as state-space design | How do we reduce states, memory, and recomputation without changing the recurrence? |

Source audit details are retained in [CATALOG_PROVENANCE.md](CATALOG_PROVENANCE.md).
