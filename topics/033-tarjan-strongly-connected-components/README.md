# Tarjan strongly connected components

Find maximal mutual-reachability groups in an immutable directed graph. The Rust
implementation compares explicit-frame Tarjan, path-based Gabow and two-pass
Kosaraju. Prepared Kosaraju requires an exact transpose from the same snapshot.

## Contract and example

Vertices are `0..n`. Empty and disconnected graphs, loops and duplicate edge
entries are valid. Invalid destinations panic before traversal. Allocation failure
uses Rust's allocator behavior. The graph stays immutable throughout each call.
IDs are dense snapshot-local labels. Compare membership after canonicalization.
Tarjan and Gabow emit sink-first IDs; Kosaraju here emits source-first IDs.

Edges `0->1,1->2,2->0,2->3,3->4,4->3,4->5,5->6` produce canonical labels
`[0,0,0,3,3,5,6]`. Before traversal all vertices are undiscovered. During return
from4 to3, DFS frames contain `[0,1,2,3]`, unresolved vertices contain
`[0,1,2,3,4]`, and vertices5/6 have already been emitted. After completion the
condensation edges are `{0,1,2}->{3,4}->{5}->{6}`. The task delivery contains the
rendered and inspected three-state visual.

## Invariants and alternatives

Tarjan tracks discovery index, low value and unresolved membership separately.
After a tree child returns, propagate its low value. For an already discovered
unresolved target, use its discovery index. Emitted targets leave low unchanged.
At finish, `low[v]==index[v]` closes the component by popping through v. An explicit
`(vertex,next_edge)` continuation preserves return events and deep-graph safety.

Kosaraju records complete finish order, then visits reversed edges in reverse
finish order. Its one-shot call includes transpose construction/destruction.
Prepared mode retains the exact transpose externally. Gabow uses unresolved and
candidate-root stacks, pruning roots by discovery index on unresolved edges.

Let n/m/k count vertices/edge entries/components. Each candidate has O(n+m)
traversal work. Tarjan/Gabow need O(n) auxiliary state. One-shot Kosaraju adds an
O(n+m) transpose. Our validation scans and allocation costs are included in timing.
For the example n=7,m=8,k=4, so n+m=15 abstract vertex/edge units. A two-word frame
uses16 bytes of payload on64-bit hosts, or112 bytes for seven reserved frames.
For q=3 repeated prepared calls, total cost is one8-edge transpose build plus
three prepared traversals. These counts are derived, rather than measured cycles.

Repeated BFS reachability is the small oracle: O(n(n+m)) work/O(n squared) state.
The example uses seven searches and49 reachability entries. The large checker
independently requires every group to be strongly connected and the quotient
acyclic. These two properties certify the partition. It rejects split/merged groups.

## Graph edits and selection

A singleton component is cyclic only with a self-loop. Kahn's residual may also
contain downstream vertices. Adding6->1 merges this example into one SCC;
removing4->3 splits its second cyclic group. Recompute and invalidate IDs after
edits. For insertion A->B in a condensation graph, a B-to-A path means a merge;
merged components lie on B-to-A paths. Dynamic maintenance needs its own algorithm.
SCC membership alone supplies neither atomic commit nor concurrent ownership.

Use Kosaraju as a simple baseline. Consider Tarjan/Gabow when only outgoing
adjacency is retained. Compare prepared Kosaraju when reverse adjacency already
has a justified lifetime. See measurements/README.md for measured decisions.
Snapshot changes invalidate cached transposes. IDs and runtime choices require
explicit ordering and workload contracts.

## Run

```sh
cargo test -p topic033
cargo run -p topic033 --example components
cargo bench -p topic033 --bench scc -- groups tarjan
cargo bench -p topic033 --bench scc -- groups kosaraju
cargo bench -p topic033 --bench scc -- groups gabow
cargo bench -p topic033 --bench scc -- groups prepared
python3 topics/033-tarjan-strongly-connected-components/measurements/run.py /tmp/topic033-new-run
```

Each example line ends with `[0, 0, 0, 3, 3, 5, 6]`. The campaign output path
must be new. Source, environment and binary hashes bind each external run.

## Primary sources

- [Tarjan and Zwick, definitions, DFS and iterative variants](https://arxiv.org/html/2201.07197v3).
- [Princeton Tarjan](https://raw.githubusercontent.com/kevin-wayne/algs4/master/src/main/java/edu/princeton/cs/algs4/TarjanSCC.java).
- [Princeton Kosaraju](https://raw.githubusercontent.com/kevin-wayne/algs4/master/src/main/java/edu/princeton/cs/algs4/KosarajuSharirSCC.java).
- [Princeton Gabow](https://raw.githubusercontent.com/kevin-wayne/algs4/master/src/main/java/edu/princeton/cs/algs4/GabowSCC.java).

Revisit scope: dynamic insert/delete maintenance, transactional consumers and
CSR versus vector-of-vectors layouts. Today's static candidates cover Tarjan,
Kosaraju and Gabow explicitly. Current catalog lacks coverage arrays/index;
the retained catalog follow-up owns repository-wide reconciliation.
