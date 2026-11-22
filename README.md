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
- Every topic records a benchmark decision. Performance-sensitive algorithms
  include runnable benchmarks and profiles; a topic may omit them only with a
  concrete explanation of why timing cannot answer its central question.
- Performance work records the exact source, binary, machine, workload, sample
  unit, ordering, and external raw-evidence receipt. Pull requests keep compact
  results instead of per-attempt files.
- Sampled tests and benchmarks are reported as bounded evidence, not proof.
- The complete lesson is delivered in the daily task; the repository retains a
  compact, runnable learning guide rather than a transcript.

See [the methodology](METHODOLOGY.md) and [documentation rules](DOCUMENTATION.md).

## Table of contents

The catalog was deduplicated from all 31 historical installments in the source
conversation, then split where one heading contained materially different
contracts or cost models. `curriculum.toml` is the machine-readable authority. See the
[algorithm index](ALGORITHM_INDEX.md) for named coverage and sources.

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
| 55 | Concurrent graph marking, write barriers, and work-list termination | Which barrier and termination conditions keep reachable objects marked during mutation? |
| 56 | Cycle detection, topological constraints, and deadlock resolution | Which wait cycles are actual deadlocks under the declared resource model? |
| 57 | Deadline scheduling: EDF, CBS, and admission control | When can a workload meet every deadline under a stated processor and task model? |
| 58 | Deficit round robin, fair queueing, and active queue management | How do packet costs and queue policies affect service share and queue delay? |
| 59 | K-way merge, sweep-line tombstones, and snapshot semantics | Which version survives when sorted runs and overlapping tombstones meet at one snapshot? |
| 60 | Associative reductions, deferred merges, and compaction | Which reducer state can be regrouped and merged without changing the answer? |
| 61 | Block range indexes and lossy summaries | How much scanning can a compact file summary safely rule out? |
| 62 | Inverted-file vector search and coarse quantization | How do coarse partitioning and compressed distances spend a recall budget? |
| 63 | TinyLFU and frequency-based cache admission | When should a hot cache entry survive a miss for an unproven key? |
| 64 | Sharded free lists and cross-thread reclamation | How can another thread return an object without violating the owning thread's reuse invariant? |
| 65 | Spatial indexes, R-trees, and STR packing | How does rectangle overlap change the number of spatial candidates that must be checked? |
| 66 | Incremental multiset computation and differential dataflow | How do weighted updates propagate through joins and iteration? |
| 67 | Cumulative resource scheduling, timetables, and edge finding | Which possible task starts can a shared-capacity constraint rule out? |
| 68 | Static range minima, Cartesian trees, and RMQ | How can preprocessing answer immutable range minima with small query cost? |
| 69 | Wavelet trees and matrices | How can a sequence index answer order statistics without sorting each subrange? |
| 70 | Succinct bitvectors and rank/select | How much support data does a static bitmap need to answer positional queries? |
| 71 | Elias-Fano and compressed monotone indexes | When can sorted integer addresses stay compressed during search? |
| 72 | Minimal perfect hashing for static keys | How do static keys map to a dense range while handling unknown keys? |
| 73 | Learned sorted indexes and bounded search | What bound turns a rank prediction into an exact lower-bound query? |
| 74 | Suffix arrays, induced sorting, and longest common prefixes | How can an indexed text answer substring queries with reusable suffix order? |
| 75 | FM-index and backward search | How can a compressed text index count and locate arbitrary substrings? |
| 76 | Palindromic substrings, Manacher radii, and eertrees | Which representation fits longest-palindrome queries or online distinct counts? |
| 77 | Suffix automata and substring occurrence | How does an online automaton represent all substrings and share their right contexts? |
| 78 | Regular expression automata and bounded matching | How does a regular-expression engine preserve matching semantics while bounding active state? |
| 79 | Erasure coding and reconstruction | Which missing chunks can be recovered from a redundancy budget? |
| 80 | Lempel-Ziv parsing and dictionary compression | When is a shorter local match cheaper over the whole encoded block? |
| 81 | Entropy coding and normalized symbol models | How does a symbol model become a reversible stream with bounded decode state? |
| 82 | Fractional cascading over sorted catalogs | How does a search result in one list narrow the search in a neighboring list? |
| 83 | Robust geometric predicates and convex hulls | Which arithmetic guarantee keeps a geometric decision correct near degeneracy? |
| 84 | Sweep-line algorithms for geometric intersections | How do ordered events expose all segment intersections without testing all pairs? |
| 85 | Delaunay triangulations and Voronoi diagrams | How do local geometric updates maintain a global triangulation invariant? |
| 86 | Exact spatial nearest-neighbor search | Which metric bound safely prunes a subtree in an exact nearest-neighbor query? |
| 87 | Minimum spanning forests and cut certificates | How can local minimum edges certify an optimal spanning forest? |
| 88 | Fully dynamic graph connectivity | How do online deletions replace a removed tree edge or prove that connectivity changed? |
| 89 | Heavy-light decomposition and tree path queries | How can a fixed tree path become a small set of contiguous ranges? |
| 90 | Link-cut trees and dynamic path aggregates | How can path queries survive online changes to the forest itself? |
| 91 | Offline range queries with Mo ordering | When does reordering queries save more endpoint movement than it costs? |
| 92 | Maximum flow and minimum cut | How does reverse residual capacity repair an earlier flow choice? |
| 93 | Bipartite cardinality matching and vertex covers | When do batches of shortest augmenting paths reduce the work to find a maximum matching? |
| 94 | Minimum-cost flow and weighted assignment | How can a solver improve assignment cost without breaking capacity and balance? |
| 95 | General graph matching and blossom contraction | Why do odd cycles require contraction before a general-graph augmenting search can continue? |
| 96 | Contraction hierarchies for route queries | When does one-time route preprocessing justify its shortcut storage and query savings? |
| 97 | K shortest paths, loopless routes, and walks | How do path and walk semantics change the enumeration algorithm and output order? |
| 98 | Timetable routing with transfers | What state must a route search retain when a later departure can arrive earlier? |
| 99 | Random walks and personalized PageRank | How do teleportation and residual error define what a graph ranking converges to? |
| 100 | Random spanning trees and loop-erased walks | How can a randomized traversal sample trees with the intended distribution? |
| 101 | Polynomial convolution with FFT and NTT | When can transforms replace nested coefficient products while preserving the output contract? |
| 102 | Multiprecision integer multiplication | Where does subquadratic splitting repay allocations, carries, and asymmetry? |
| 103 | Modular arithmetic and Montgomery reduction | How many repeated operations justify conversion into a cheaper modular representation? |
| 104 | Sparse direct solves, pivoting, and fill-in | Why do elimination order and pivot choices change both work and numerical error? |
| 105 | Sparse matrix products and accumulators | Which accumulator fits a sparse product as output rows get wider? |
| 106 | Randomized low-rank approximation | How does a matrix spectrum trade extra passes for a tighter low-rank approximation? |
| 107 | Conjugate gradients and preconditioning | When does better conditioning justify the cost of a preconditioner? |
| 108 | SAT solving with conflict-driven clause learning | How does a conflict teach a solver to avoid repeating the same failed choices? |
| 109 | Exact cover and dancing links | How can exact-cover search undo candidate removal without rebuilding the remaining matrix? |
| 110 | Submodular greedy optimization | Which diminishing-returns property justifies reusing a previous marginal score as an upper bound? |
| 111 | Online bipartite matching and Ranking | What can irrevocable matching guarantee when one side arrives over time? |
| 112 | Optimal transport and Sinkhorn scaling | How does an entropy penalty trade transport accuracy for faster matrix scaling? |
| 113 | Rectangle packing with skyline and free rectangles | Which free-space representation keeps packing efficiency under a required insertion order? |
| 114 | Multilevel graph partitioning and separators | When does grouping vertices expose a better balanced partition than local moves alone? |
| 115 | Isotonic regression and adjacent pooling | How can an ordered fit minimize squared error while preserving ties, exact comparisons, and the original predictor order? |

Source audit details are retained in [CATALOG_PROVENANCE.md](CATALOG_PROVENANCE.md).
