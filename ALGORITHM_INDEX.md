# Algorithm index

Use this index to find the owner for each algorithm and plan revisits. The
catalog stores stable IDs, titles and named coverage.

## Topics 001–054

| ID | Topic | Named coverage |
|---:|---|---|
| 001 | Prefix sums and scans | inclusive scans, exclusive scans, serial scans, blocked and parallel scans, stable compaction |
| 002 | Difference arrays and deferred range updates | difference arrays, boundary events, offline range updates, prefix materialization |
| 003 | Bitsets and density-adaptive bitmaps | dense bitsets, sorted sparse sets, Roaring-style adaptive bitmaps, bitmap intersections |
| 004 | Interval sets and compressed ranges | canonical disjoint intervals, interval union, interval intersections, compressed ranges |
| 005 | Two pointers and sliding windows | two-pointer walks, monotone window endpoints, bounded windows, nonnegative sums |
| 006 | Monotonic stacks and deques | monotone stacks, monotone deques, sliding extrema, block extrema |
| 007 | Binary, galloping, and branch-aware search | binary search, galloping search, branch-aware search, lower-bound semantics |
| 008 | Selection, Quickselect, and Top-K | Quickselect, partition strategies, bounded heaps, sorted top-k |
| 009 | Reservoir and weighted sampling | reservoir sampling, skip-based sampling, weighted sampling, reproducible random trials |
| 010 | Sorting algorithm portfolios | comparison sorting, radix sorting, adaptive sort portfolios, key and payload movement |
| 011 | Merge algorithms and duplicate-run joins | sorted merging, galloping merge, duplicate runs, join multiplicity |
| 012 | Radix partitioning | radix partitioning, histograms, prefix offsets, stable scatter, skew |
| 013 | Open-addressed hash tables | linear probing, probe invariants, deletion markers, resizing and load factor |
| 014 | SwissTable-style metadata filtering | SwissTable metadata, control bytes, group probing, fingerprint filters |
| 015 | Robin Hood hashing | Robin Hood insertion, probe distances, early exit, backward-shift deletion |
| 016 | Cuckoo hashing | cuckoo hashing, choice locations, kick-out chains, stash and rehash |
| 017 | Bloom, cuckoo, and quotient filters | Bloom filters, cuckoo filters, quotient filters, false-positive budgets, deletions |
| 018 | Hash joins, recursive partitioning, and spill | hash joins, recursive partitioning, skew handling, spill |
| 019 | Fenwick trees | Fenwick trees, point updates, prefix queries, range reductions |
| 020 | Segment trees and lazy propagation | segment trees, lazy propagation, tag composition, range queries |
| 021 | Heaps and priority queues | binary heaps, d-ary heaps, indexed priority queues, update and pop costs |
| 022 | Skip lists and hybrid ordered indexes | skip lists, randomized height, ordered iteration, hybrid point lookup |
| 023 | B-trees and posting-list deduplication | B-trees, page layouts, posting lists, duplicate-key compression |
| 024 | Tries, radix trees, ART, and sparse radix arrays | tries, radix trees, adaptive radix trees, sparse radix arrays |
| 025 | Finite-state transducers and compact dictionaries | finite-state transducers, minimal automata, sorted dictionaries, prefix lookup |
| 026 | Dynamic arrays and growth policies | geometric growth, dynamic arrays, capacity and slack, amortized copies |
| 027 | Ring buffers and bounded queues | ring buffers, bounded queues, wraparound, producer and consumer ownership |
| 028 | Work-stealing deques | Chase-Lev deques, work stealing, owner and thief coordination, last-item races |
| 029 | Union-find and equivalence-class indexes | union-find, path compression, union by rank, class enumeration, offline rollback |
| 030 | Breadth-first and depth-first traversal | breadth-first search, depth-first search, frontier order, visited-state representation |
| 031 | Dijkstra and shortest-path queues | Dijkstra, shortest-path heaps, stale queue entries, integer-weight queues |
| 032 | Topological order and incremental DAG evaluation | topological sorting, DAG evaluation, incremental order repair, affected descendants |
| 033 | Tarjan strongly connected components | Tarjan SCCs, lowlinks, explicit stacks, condensation graphs |
| 034 | Dominator trees and incremental maintenance | dominators, immediate dominators, Cooper-Harvey-Kennedy, Lengauer-Tarjan, graph edits |
| 035 | Reachability indexes for large DAGs | DAG reachability, bitset transitive closure, negative reachability labels, incremental updates |
| 036 | Merkle trees and incremental rebuild caches | Merkle trees, content hashes, incremental rebuild, proof verification |
| 037 | Best-first and approximate graph search | best-first exploration, approximate graph search, beam budgets, recall and work |
| 038 | KMP and prefix-function matching | Knuth-Morris-Pratt, prefix function, failure links, streaming matches |
| 039 | Boyer-Moore, Two-Way, and adaptive string search | Boyer-Moore, Horspool, Two-Way, adaptive substring search |
| 040 | Rolling hashes and content-defined chunking | rolling hashes, content-defined chunking, buzhash and gear-style hashing, candidate verification |
| 041 | Edit distance and automaton-guided lookup | Levenshtein distance, banded dynamic programming, Myers bit vectors, trie pruning |
| 042 | Longest common subsequence and diff | longest common subsequence, Myers diff, Hirschberg, edit-script reconstruction |
| 043 | Multi-pattern matching and automata | Aho-Corasick, literal-set matching, dense and sparse automata, output ordering |
| 044 | Varints, codecs, and bounded decoding | varints, bounded decoding, canonical encodings, integer codec trade-offs |
| 045 | Compression-oriented transforms | delta coding, XOR transforms, byte shuffles, run-length and transform selection |
| 046 | Streaming sketches | Count-Min, HyperLogLog, quantile sketches, error budgets and merge |
| 047 | Consistent hashing and bounded-load placement | consistent hashing, rendezvous hashing, bounded-load placement, membership churn |
| 048 | LRU, CLOCK, and workload-aware eviction | LRU, CLOCK, eviction metadata, workload-dependent retention |
| 049 | Timing wheels, heaps, and timer trees | timing wheels, deadline heaps, timer trees, expiry and cancellation |
| 050 | Token buckets and rate accounting | token buckets, integer refill, burst policies, byte-rate accounting |
| 051 | Buddy allocation and hierarchical free-space tracking | buddy allocation, order bitmaps, split and coalesce, fragmentation |
| 052 | Red-black trees and intrusive representations | red-black trees, left-leaning red-black trees, intrusive and arena layouts, identity stability |
| 053 | Load balancing and stable assignment | round robin, power of two choices, rendezvous assignment, locality and skew |
| 054 | Dynamic programming as state-space design | state-space design, knapsack, rolling state, value-axis dynamic programming, sparse frontiers |

## Planned follow-through

### 055 Concurrent graph marking, write barriers, and work-list termination

Which barrier and termination conditions keep reachable objects marked during mutation?

**Coverage.** tricolor marking, insertion barriers, snapshot-at-the-beginning barriers, work-list termination.

**Experiment.** Enumerate collector and mutator steps and check live objects against snapshot or graph traversal semantics. [Primary source](https://www.cs.utexas.edu/~EWD/transcriptions/EWD06xx/EWD630.html)

### 056 Cycle detection, topological constraints, and deadlock resolution

Which wait cycles are actual deadlocks under the declared resource model?

**Coverage.** wait-for graphs, single-instance cycle detection, multi-instance resource reduction, victim selection.

**Experiment.** Compare graph cycles and resource-reduction results against exhaustive completion sequences. [Primary source](https://doi.org/10.1145/356586.356588)

### 057 Deadline scheduling: EDF, CBS, and admission control

When can a workload meet every deadline under a stated processor and task model?

**Coverage.** earliest deadline first, constant bandwidth server, periodic task demand, admission control.

**Experiment.** Compare earliest-deadline dispatch with fixed priorities against exhaustive small schedules and test utilization assumptions. [Primary source](https://www.cis.upenn.edu/~lee/01cis642/papers/AB98.pdf)

### 058 Deficit round robin, fair queueing, and active queue management

How do packet costs and queue policies affect service share and queue delay?

**Coverage.** deficit round robin, weighted fair queueing, variable packet sizes, active queue management.

**Experiment.** Replay fixed arrivals and compare byte fairness with a fluid-service oracle while varying packet sizes. [Primary source](https://repository.library.washu.edu/cse_research/339/)

### 059 K-way merge, sweep-line tombstones, and snapshot semantics

Which version survives when sorted runs and overlapping tombstones meet at one snapshot?

**Coverage.** heap merge, loser trees, range tombstone sweep, snapshot visibility.

**Experiment.** Compare heap and loser-tree merges with a materialized version-history oracle over overlapping deletes. [Primary source](https://github.com/facebook/rocksdb/wiki/DeleteRange)

### 060 Associative reductions, deferred merges, and compaction

Which reducer state can be regrouped and merged without changing the answer?

**Coverage.** associative merge, distributive aggregates, algebraic aggregates, order-sensitive merge contracts.

**Experiment.** Compare raw recomputation with partial-state merges over multiple parenthesizations and numeric contracts. [Primary source](https://arxiv.org/abs/cs/0701155)

### 061 Block range indexes and lossy summaries

How much scanning can a compact file summary safely rule out?

**Coverage.** block range indexes, min/max summaries, multi-range summaries, false-positive rechecks.

**Experiment.** Compare a full scan with per-block summaries on clustered and shuffled data using exact rechecks. [Primary source](https://www.postgresql.org/docs/17/brin.html)

### 062 Inverted-file vector search and coarse quantization

How do coarse partitioning and compressed distances spend a recall budget?

**Coverage.** inverted file lists, coarse centroid search, probe budgets, product quantization, exact reranking.

**Experiment.** Compare exhaustive search with inverted-file probes and exact reranking over varied density and skew. [Primary source](https://doi.org/10.1109/TPAMI.2010.57)

### 063 TinyLFU and frequency-based cache admission

When should a hot cache entry survive a miss for an unproven key?

**Coverage.** TinyLFU, Window TinyLFU, frequency sketch aging, admission versus eviction.

**Experiment.** Hold the eviction policy fixed while comparing always-admit and TinyLFU on scans and shifting hot sets. [Primary source](https://arxiv.org/abs/1512.00727)

### 064 Sharded free lists and cross-thread reclamation

How can another thread return an object without violating the owning thread's reuse invariant?

**Coverage.** page-local free lists, remote free queues, batch reclamation, ownership transfer.

**Experiment.** Model ownership states and compare direct and batched remote returns under same-thread and cross-thread workloads. [Primary source](https://www.microsoft.com/en-us/research/publication/mimalloc-free-list-sharding-in-action/)


## Indexes and sequence algorithms

### 065 Spatial indexes, R-trees, and STR packing

How does rectangle overlap change the number of spatial candidates that must be checked?

**Coverage.** R-trees, Sort-Tile-Recursive packing, bounding-box candidate queries, exact predicate refinement, spatial joins, index construction versus query reuse.

**Experiment.** Compare a full scan, incremental insertion and bulk packing on clustered and elongated rectangles. [Primary source](https://citeseerx.ist.psu.edu/document?doi=e680839472e7f5cab0f12fcc7c4aba6834a2e096&repid=rep1&type=pdf)

### 066 Incremental multiset computation and differential dataflow

How do weighted updates propagate through joins and iteration?

**Coverage.** signed multiplicities, timestamped updates, incremental joins and aggregates, arrangements, update consolidation, frontiers and history compaction.

**Experiment.** Replay insertions and deletions and compare each consolidated output with full multiset recomputation. [Primary source](https://www.cidrdb.org/cidr2013/Papers/CIDR13_Paper111.pdf)

### 067 Cumulative resource scheduling, timetables, and edge finding

Which possible task starts can a shared-capacity constraint rule out?

**Coverage.** boundary-event resource profiles, compulsory parts, timetable propagation, overload checking, timetable edge finding, optional interval presence, propagation versus complete feasibility.

**Experiment.** Compare timetable and edge-finding bounds against exhaustive feasible starts on small schedules. [Primary source](https://vilim.eu/petr/cpaior2011.pdf)

### 068 Static range minima, Cartesian trees, and RMQ

How can preprocessing answer immutable range minima with small query cost?

**Coverage.** range minimum queries, sparse tables, Cartesian trees, Fischer-Heun indexing, deterministic ties.

**Experiment.** Compare scans, sparse tables and blocked RMQ across query reuse and repeated minima. [Primary source](https://epubs.siam.org/doi/10.1137/090779759)

### 069 Wavelet trees and matrices

How can a sequence index answer order statistics without sorting each subrange?

**Coverage.** wavelet trees, wavelet matrices, range quantile, alphabet rank, range frequency.

**Experiment.** Compare subrange sorting and wavelet indexes over skewed alphabets using an independent sorted-slice oracle. [Primary source](https://repositorio.uchile.cl/bitstream/handle/2250/133661/The-wavelet-matrix.pdf)

### 070 Succinct bitvectors and rank/select

How much support data does a static bitmap need to answer positional queries?

**Coverage.** rank directories, sampled select, word popcount, boundary indexes.

**Experiment.** Compare direct scans, dense prefix counts and sampled directories at different bit densities. [Primary source](https://gwern.net/doc/cs/algorithm/information/compression/1989-jacobson.pdf)

### 071 Elias-Fano and compressed monotone indexes

When can sorted integer addresses stay compressed during search?

**Coverage.** Elias-Fano, partitioned Elias-Fano, monotone predecessor, compressed random access.

**Experiment.** Compare a plain sorted array, delta blocks and partitioned Elias-Fano with duplicate and gap-heavy sequences. [Primary source](https://doi.org/10.1145/2600428.2609615)

### 072 Minimal perfect hashing for static keys

How do static keys map to a dense range while handling unknown keys?

**Coverage.** static key bijections, hypergraph peeling, RecSplit, absent-key verification.

**Experiment.** Measure build, lookup and verification against a sorted-key baseline with hits and misses. [Primary source](https://arxiv.org/abs/1910.06416)

### 073 Learned sorted indexes and bounded search

What bound turns a rank prediction into an exact lower-bound query?

**Coverage.** PGM-index, piecewise linear rank models, error-bounded correction, duplicate-key lower bounds.

**Experiment.** Compare binary search and a bounded model on clustered, gapped and shifted key distributions. [Primary source](https://arxiv.org/abs/1910.06169)

### 074 Suffix arrays, induced sorting, and longest common prefixes

How can an indexed text answer substring queries with reusable suffix order?

**Coverage.** suffix arrays, prefix doubling, induced sorting, longest common prefix arrays.

**Experiment.** Compare explicit suffix sorting and induced sorting against brute-force match positions on repeated and varied text. [Primary source](https://www.cin.ufpe.br/~paguso/courses/if767/bib/Nong_2011.pdf)

### 075 FM-index and backward search

How can a compressed text index count and locate arbitrary substrings?

**Coverage.** Burrows-Wheeler transform, backward search, occurrence rank, sampled suffix locations.

**Experiment.** Compare suffix-array search with backward search and vary location sampling density. [Primary source](https://people.unipmn.it/manzini/fmindex/)

### 076 Palindromic substrings, Manacher radii, and eertrees

Which representation fits longest-palindrome queries or online distinct counts?

**Coverage.** Manacher radii, palindromic trees, suffix links, online distinct palindrome queries.

**Experiment.** Compare center expansion, radii and a palindromic tree against enumeration on repeated and alternating text. [Primary source](https://arxiv.org/abs/1506.04862)

### 077 Suffix automata and substring occurrence

How does an online automaton represent all substrings and share their right contexts?

**Coverage.** Suffix automata, end-position equivalence, cloned states, occurrence propagation.

**Experiment.** Check membership and counts against enumeration and compare state growth across alphabets. [Primary source](https://doi.org/10.1016/0304-3975(85)90157-4)

### 078 Regular expression automata and bounded matching

How does a regular-expression engine preserve matching semantics while bounding active state?

**Coverage.** Thompson NFA, epsilon closure, DFA caching, submatch priority and output contracts.

**Experiment.** Compare a Thompson simulator with a bounded deterministic cache against exhaustive small input match sets. [Primary source](https://people.csail.mit.edu/virgi/6.s078/papers/regexp2.pdf)

### 079 Erasure coding and reconstruction

Which missing chunks can be recovered from a redundancy budget?

**Coverage.** XOR parity, Reed-Solomon erasure coding, finite-field matrix inversion, systematic codes.

**Experiment.** Exhaust small erasure masks and compare XOR parity and systematic Reed-Solomon encoding costs. [Primary source](https://sites.math.rutgers.edu/~zeilberg/akherim/reed.pdf)

### 080 Lempel-Ziv parsing and dictionary compression

When is a shorter local match cheaper over the whole encoded block?

**Coverage.** LZ77, match-finder search, greedy parsing, lazy parsing, optimal parse cost.

**Experiment.** Compare greedy, lazy and exhaustive small parses under one fixed token-cost and decoder contract. [Primary source](https://courses.cs.duke.edu/spring03/cps296.5/papers/ziv_lempel_1977_universal_algorithm.pdf)

### 081 Entropy coding and normalized symbol models

How does a symbol model become a reversible stream with bounded decode state?

**Coverage.** canonical Huffman, range coding, asymmetric numeral systems, frequency normalization.

**Experiment.** Compare coded size, model cost and round trips for fixed and shifting symbol frequencies. [Primary source](https://arxiv.org/abs/0902.0271)

### 082 Fractional cascading over sorted catalogs

How does a search result in one list narrow the search in a neighboring list?

**Coverage.** linked catalog search, sampled bridge entries, lower-bound queries, duplicate-key policy.

**Experiment.** Compare independent binary searches and fractional cascading across list lengths and query paths. [Primary source](https://www.cs.princeton.edu/~chazelle/pubs/FractionalCascading1.pdf)


## Computational geometry

### 083 Robust geometric predicates and convex hulls

Which arithmetic guarantee keeps a geometric decision correct near degeneracy?

**Coverage.** orientation predicates, integer widening, adaptive exact predicates, monotone-chain hulls.

**Experiment.** Compare integer and filtered predicates with an exact arithmetic oracle on nearly collinear inputs. [Primary source](https://people.eecs.berkeley.edu/~jrs/jrspapers.html)

### 084 Sweep-line algorithms for geometric intersections

How do ordered events expose all segment intersections without testing all pairs?

**Coverage.** Bentley-Ottmann, event ordering, sweep status, coincident intersection policy.

**Experiment.** Compare event sweep and brute-force pairs under clustered intersections and tied coordinates. [Primary source](https://doi.org/10.1109/TC.1979.1675432)

### 085 Delaunay triangulations and Voronoi diagrams

How do local geometric updates maintain a global triangulation invariant?

**Coverage.** empty-circle predicates, Delaunay triangulation, Voronoi duality, edge flips, degeneracies.

**Experiment.** Validate small triangulations using exact orientation and empty-circle tests, then compare insertion orders. [Primary source](https://doi.org/10.1145/800061.808751)

### 086 Exact spatial nearest-neighbor search

Which metric bound safely prunes a subtree in an exact nearest-neighbor query?

**Coverage.** k-d trees, ball trees, vantage-point trees, distance lower bounds, exact top-k.

**Experiment.** Compare tree pruning with flat exhaustive search across dimensionality and clustered queries. [Primary source](https://doi.org/10.1145/355744.355745)


## Graph algorithms and routing

### 087 Minimum spanning forests and cut certificates

How can local minimum edges certify an optimal spanning forest?

**Coverage.** Kruskal, Prim, Boruvka, cut property, equal-weight ties.

**Experiment.** Compare three edge-selection strategies against exhaustive forest cost on small graphs. [Primary source](https://www.cs.yale.edu/homes/lans/readings/routing/kruskal-1956.pdf)

### 088 Fully dynamic graph connectivity

How do online deletions replace a removed tree edge or prove that connectivity changed?

**Coverage.** Euler-tour trees, spanning forest witnesses, replacement edge search, level promotion.

**Experiment.** Replay insertions and deletions against fresh traversal with varied bridge frequency and component sizes. [Primary source](https://u.cs.biu.ac.il/~rodittl/p723-holm.pdf)

### 089 Heavy-light decomposition and tree path queries

How can a fixed tree path become a small set of contiguous ranges?

**Coverage.** heavy-light decomposition, path aggregates, linearized heavy chains, noncommutative query order.

**Experiment.** Compare an explicit path walk and heavy-light ranges under chains, stars and mixed query/update loads. [Primary source](https://www.cs.cmu.edu/~sleator/papers/dynamic-trees.pdf)

### 090 Link-cut trees and dynamic path aggregates

How can path queries survive online changes to the forest itself?

**Coverage.** link-cut trees, preferred paths, splay access, link and cut, path reversal.

**Experiment.** Replay links, cuts and path queries against a forest walk and test orientation-sensitive aggregates. [Primary source](https://www.cs.cmu.edu/~sleator/papers/dynamic-trees.pdf)

### 091 Offline range queries with Mo ordering

When does reordering queries save more endpoint movement than it costs?

**Coverage.** Mo ordering, reversible window state, block-size selection, frequency queries.

**Experiment.** Compare naive scans and offline orders on identical distinct-count queries, including original answer order. [Primary source](https://codeforces.com/blog/entry/7383)

### 092 Maximum flow and minimum cut

How does reverse residual capacity repair an earlier flow choice?

**Coverage.** residual networks, augmenting paths, blocking flows, max-flow min-cut certificates.

**Experiment.** Compare Edmonds-Karp and Dinic against exhaustive cuts on small integral networks. [Primary source](https://www.mathnet.ru/eng/dan35701)

### 093 Bipartite cardinality matching and vertex covers

When do batches of shortest augmenting paths reduce the work to find a maximum matching?

**Coverage.** augmenting paths, Hopcroft-Karp, alternating layers, minimum vertex cover certificate.

**Experiment.** Compare one-path augmentation with layered batches and check both matching and cover sizes. [Primary source](https://epubs.siam.org/doi/abs/10.1137/0202019)

### 094 Minimum-cost flow and weighted assignment

How can a solver improve assignment cost without breaking capacity and balance?

**Coverage.** minimum-cost flow, reduced costs, successive shortest paths, Hungarian assignment, optimality witnesses.

**Experiment.** Compare weighted assignment and generic min-cost flow with enumeration and optimality certificates. [Primary source](https://doi.org/10.1002/nav.3800020109)

### 095 General graph matching and blossom contraction

Why do odd cycles require contraction before a general-graph augmenting search can continue?

**Coverage.** blossom contraction, odd alternating cycles, augmenting path lifting, maximum cardinality.

**Experiment.** Compare a blossom implementation and exhaustive matchings on small odd-cycle and nested fixtures. [Primary source](https://doi.org/10.4153/CJM-1965-045-4)

### 096 Contraction hierarchies for route queries

When does one-time route preprocessing justify its shortcut storage and query savings?

**Coverage.** contraction order, shortcut witnesses, upward bidirectional queries, path unpacking.

**Experiment.** Compare baseline Dijkstra and shortcut queries with exact distances and reconstructed paths. [Primary source](https://publikationen.bibliothek.kit.edu/1000015090)

### 097 K shortest paths, loopless routes, and walks

How do path and walk semantics change the enumeration algorithm and output order?

**Coverage.** Yen spur paths, Eppstein sidetracks, loopless path contract, duplicate suppression.

**Experiment.** Compare a small exhaustive path oracle with spur-path enumeration and include cycles and tied costs. [Primary source](https://pubsonline.informs.org/doi/10.1287/mnsc.17.11.712)

### 098 Timetable routing with transfers

What state must a route search retain when a later departure can arrive earlier?

**Coverage.** RAPTOR, connection scanning, earliest arrival, transfer counts, Pareto route labels.

**Experiment.** Compare chronological scans and route rounds against a time-expanded graph with transfer limits. [Primary source](https://renatowerneck.files.wordpress.com/2016/06/dpw14-raptor.pdf)

### 099 Random walks and personalized PageRank

How do teleportation and residual error define what a graph ranking converges to?

**Coverage.** transition matrices, power iteration, teleportation, dangling vertices, personalized residual pushes.

**Experiment.** Compare dense iteration and sparse pushes against a small linear-solve oracle with dangling nodes. [Primary source](https://bytes.usc.edu/files/cs103/pagerank/pagerank.pdf)

### 100 Random spanning trees and loop-erased walks

How can a randomized traversal sample trees with the intended distribution?

**Coverage.** Wilson's algorithm, loop erasure, uniform spanning trees, weighted walks.

**Experiment.** Enumerate small spanning trees and compare empirical frequencies for loop-erased and cover-time walks. [Primary source](https://www.cs.cmu.edu/~15859n/RelatedWork/RandomTrees-Wilson.pdf)


## Arithmetic and numerical algorithms

### 101 Polynomial convolution with FFT and NTT

When can transforms replace nested coefficient products while preserving the output contract?

**Coverage.** naive convolution, fast Fourier transform, number-theoretic transform, padding, coefficient reconstruction.

**Experiment.** Compare schoolbook, floating and modular methods using exact coefficients and an overflow bound. [Primary source](https://web.stanford.edu/class/cme324/classics/cooley-tukey.pdf)

### 102 Multiprecision integer multiplication

Where does subquadratic splitting repay allocations, carries, and asymmetry?

**Coverage.** schoolbook limb multiplication, Karatsuba, carry propagation, unbalanced operands.

**Experiment.** Compare limb multiplication and Karatsuba against exact host integers and vary operand aspect ratios. [Primary source](https://www.mathnet.ru/eng/dan26729)

### 103 Modular arithmetic and Montgomery reduction

How many repeated operations justify conversion into a cheaper modular representation?

**Coverage.** modular multiplication, Montgomery representation, Barrett reduction, conversion costs.

**Experiment.** Compare exact wide-integer remainder, Barrett and Montgomery across repeated and one-off products. [Primary source](https://doi.org/10.1090/S0025-5718-1985-0777282-X)

### 104 Sparse direct solves, pivoting, and fill-in

Why do elimination order and pivot choices change both work and numerical error?

**Coverage.** sparse LU, elimination order, pivoting, symbolic versus numeric factorization, fill-in.

**Experiment.** Compare orderings on sparse systems with known solutions and report fill and residual separately. [Primary source](https://epubs.siam.org/doi/10.1137/1.9780898718881)

### 105 Sparse matrix products and accumulators

Which accumulator fits a sparse product as output rows get wider?

**Coverage.** sparse matrix multiplication, Gustavson row accumulation, heap merge, symbolic sizing, semiring operations.

**Experiment.** Compare dense scratch, hash and merge accumulators with a dense oracle at varied skew and fill. [Primary source](https://research.ibm.com/publications/two-fast-algorithms-for-sparse-matrices-multiplication-and-permuted-transposition)

### 106 Randomized low-rank approximation

How does a matrix spectrum trade extra passes for a tighter low-rank approximation?

**Coverage.** randomized range finding, oversampling, power iteration, singular-value error, matrix pass budget.

**Experiment.** Compare randomized and exact decompositions on fixed spectra with measured residual and pass counts. [Primary source](https://arxiv.org/abs/0909.4061)

### 107 Conjugate gradients and preconditioning

When does better conditioning justify the cost of a preconditioner?

**Coverage.** conjugate gradients, symmetric positive-definite systems, Jacobi preconditioning, residual stopping.

**Experiment.** Compare unpreconditioned and preconditioned solves with direct solutions and independently recomputed residuals. [Primary source](https://nvlpubs.nist.gov/nistpubs/jres/049/jresv49n6p409_a1b.pdf)


## Search, optimization and partitioning

### 108 SAT solving with conflict-driven clause learning

How does a conflict teach a solver to avoid repeating the same failed choices?

**Coverage.** unit propagation, watched literals, implication graphs, clause learning, backjumping.

**Experiment.** Compare backtracking and learning on small formulas against exhaustive truth tables. [Primary source](https://eprints.soton.ac.uk/262031/1/jpms-iccad96.pdf)

### 109 Exact cover and dancing links

How can exact-cover search undo candidate removal without rebuilding the remaining matrix?

**Coverage.** Algorithm X, dancing links, minimum-column branching, reversible deletions.

**Experiment.** Enumerate solutions using both backtracking representations and test exact restoration after each undo. [Primary source](https://arxiv.org/abs/cs/0011047)

### 110 Submodular greedy optimization

Which diminishing-returns property justifies reusing a previous marginal score as an upper bound?

**Coverage.** monotone submodularity, maximum coverage, greedy selection, lazy marginal evaluation.

**Experiment.** Compare eager and lazy greedy with exact small optima and count marginal evaluations. [Primary source](https://doi.org/10.1007/BF01588971)

### 111 Online bipartite matching and Ranking

What can irrevocable matching guarantee when one side arrives over time?

**Coverage.** irrevocable online matching, randomized Ranking, adversarial arrival order, offline optimum.

**Experiment.** Compare greedy and Ranking against offline optimum over fixed seeds and adversarial arrival traces. [Primary source](https://doi.org/10.1145/100216.100262)

### 112 Optimal transport and Sinkhorn scaling

How does an entropy penalty trade transport accuracy for faster matrix scaling?

**Coverage.** transport plans, earth mover distance, entropic regularization, Sinkhorn iteration, log-domain stabilization.

**Experiment.** Compare exact transport and regularized plans with marginal residual and transport cost reported separately. [Primary source](https://arxiv.org/abs/1306.0895)

### 113 Rectangle packing with skyline and free rectangles

Which free-space representation keeps packing efficiency under a required insertion order?

**Coverage.** skyline packing, guillotine splits, MaxRects, rotation and placement constraints, fragmentation.

**Experiment.** Compare three placements against exhaustive small grids while checking bounds and pairwise overlap. [Primary source](https://github.com/juj/RectangleBinPack)

### 114 Multilevel graph partitioning and separators

When does grouping vertices expose a better balanced partition than local moves alone?

**Coverage.** graph coarsening, balanced edge cuts, partition refinement, separator quality.

**Experiment.** Compare a greedy partition and multilevel refinement against exhaustive small balanced cuts. [Primary source](https://aiichironakano.github.io/cs653/Karypis-METIS-SIAMJSC98.pdf)
