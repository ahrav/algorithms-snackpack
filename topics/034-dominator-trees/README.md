# Dominator trees and incremental maintenance

A vertex dominates a reachable target if every path from a designated entry to
that target contains it. Root is its own parent internally; unreachable vertices
have no parent. Dense IDs and valid endpoints are preconditions. Loops and
parallel edges are allowed. Multiple entries require an explicit synthetic root.

Example: `0->1,1->2,1->3,2->4,3->4,4->5,6->4`, entry0.
Vertex6 is unreachable. Parents are `[0,0,1,1,1,4,None]`. Insert `0->4`:
parents become `[0,0,1,1,0,4,None]`. Vertex1 ceases to dominate4 and5.

Full sets use `D(root)={root}` and
`D(v)={v} union intersection(D(p) for reachable predecessors p)`.
Initialize reachable nonroots to the reachable universe, then descend to a fixed
point. Unreachable predecessors contribute no entry paths. CHK (Cooper, Harvey,
Kennedy) instead intersects immediate-parent chains in reverse DFS postorder.
Parents encode dominance as tree ancestry. Depth-first search (DFS) uses heap
continuations here, so graph depth does not consume the call stack.

With n vertices, m edges, W=ceil(n/64) words and P passes, set solving costs
O(P(n+m)W), followed by O(nW+n²) parent extraction. It stores O(nW+n+m).
CHK has O(n+m) storage; observed work is graph preparation, P predecessor scans
and H parent-chain steps: O(n+m+P(n+m)+H). These are implementation cost
expressions, not a claimed tight general worst-case theorem. For n7,m7,W1,
one set pass scans at most7 vertices and7 predecessor entries; matrix payload56
bytes excludes headers. Join4 intersects `{0,1,2}` with `{0,1,3}` then adds4.

Insertion between reachable endpoints only shrinks sets. `insert` recomputes the
destination and queues successors only on change. Its graph preparation costs
O(n+m) per edit; A worklist pops and E_A predecessor visits add O((A+E_A)W).
Inserting0->4 changes4 then5, with4 incoming-edge word intersections. New
reachability triggers a rebuild. Deletion requires fresh initialization: cyclic
predecessors can otherwise preserve stale fixed points. Caller owns coherent
publication of the graph and its analysis, plus root/cache identity.

Choose full sets for explicit-set consumers when memory fits; CHK for compact
immediate parents; compare cached propagation with rebuilding for update streams.
Each update benchmark retains full sets between edits for all candidates. CHK
rebuild therefore pays parent-chain expansion. This is a state contract, and its
winner need not match the immediate-parent-only build benchmark.

Lengauer-Tarjan uses DFS semidominators and link/eval ancestor-path minima.
Its simple variant has an O(m log n) bound; more elaborate linking improves that
bound. For n7,m7,7log2(7) is about19.7, with hidden constants and different
operations. It is explained and deferred, with no implementation/timing claim.
Revisit: Lengauer-Tarjan, dominance frontiers, fast ancestor intervals,
postdominators/exit semantics, low-high certification, persistent predecessor
storage and specialized dynamic trees. The catalog has no coverage arrays or
ALGORITHM_INDEX.md at this base; this explicit scope preserves the named variants.

## Run

```sh
cargo test -p topic034
cargo run -p topic034 --example demo
python3 topics/034-dominator-trees/measurements/run.py /tmp/topic034-results
```

The portable manifest under measurements/portable runs the exact topic source
without other workspace crates. `demo` prints the two parent arrays above.
The removal/reachability oracle is independent of predecessor intersection.
See TEST_STRATEGY.md, BENCHMARK.md and measurements/README.md for validation,
measurement boundaries and results. Whole-process oracle work supports correctness;
its events cannot explain candidate-only cache or branch behavior.

## Primary sources

- [Cooper, Harvey, Kennedy: set iteration and CHK](https://hipersoft.cs.rice.edu/grads/publications/dom14.pdf).
- [Lengauer and Tarjan: static algorithm](https://www.cs.princeton.edu/courses/archive/spring03/cs423/download/dominators.pdf).
- [Georgiadis et al.: dynamic algorithms](https://arxiv.org/html/1604.02711).
- [Dynamic dominators and low-high orders in DAGs](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ESA.2019.50).
- [LLVM: graph/update coherence and flush contracts](https://llvm.org/docs/doxygen/classllvm_1_1GenericDomTreeUpdater.html).
