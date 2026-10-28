# Topological order and incremental DAG evaluation

Compute each input before its consumers. Reuse cached values when edits affect a small region.

## Contract

`u -> v` means that v consumes u. `Graph::new` admits dense IDs `0..V`, isolated
vertices and empty graphs. It rejects invalid endpoints, repeated edges and cycles.
Each ordering returns a permutation with every edge forward. Minimum-ready-ID Kahn
also returns the lexicographically smallest order. FIFO and iterative DFS promise
any valid order.

Values obey `value[v] = base[v] + sum(value[parent]) mod 2^64`. Evaluation is pure.
Batched assignments use their last occurrence. Input errors preserve graph, bases
and values. Empty batches perform zero work. Allocation failure follows Rust's
allocator policy. Concurrent readers and effects require a separate contract.

## Running example

A=2, B=3, C=A+B=5, D=C+1=6, E=C+2=7. Edges are A->C, B->C, C->D, C->E.
The fixed order [A,B,C,D,E] is valid. After A's base changes to4, process A,C,D,E;
values become [4,3,7,8,9]. B stays3. This before/during/after example is also executable.

## Candidates

FIFO Kahn counts unfinished prerequisites and emits ready vertices. Minimum-heap Kahn
selects the smallest ready ID. Iterative DFS reverses finish order and detects active
back edges. Queue exhaustion before V vertices means a cycle; residual vertices can
include downstream vertices. An actual cycle witness needs a further search.

Full evaluation scans the stored order. Dirty scanning discovers all descendants,
then scans that order and recomputes marked vertices. Change-driven evaluation uses
a deduplicated minimum heap of topological positions, and schedules consumers only
when the complete output changes. Topological priority handles shared consumers and
multiple seeds before they consume updated inputs. Pure equality is the early-stop
contract. Effects and dynamic dependencies require additional state.

Forward edge insertions preserve order. Backward insertions validate a cloned graph
with FIFO Kahn, then publish its new order. Rejection preserves the original graph.
Accepted insertions refresh all values. This is a correctness-first fallback.
Local forward/backward-region repair, edge deletion APIs, dynamic dependencies,
demand-only evaluation, parallel completion, and cycle witnesses are revisit scope.

## Costs and selection

Let V/E count vertices/edges. FIFO Kahn and iterative DFS cost O(V+E). Minimum Kahn
costs O(E+V log V). All use O(V) auxiliary state beyond graph storage.
Full updates cost O(V+E). For A affected vertices, E_A discovery edges and P_A parent
reads, dirty scanning costs O(V+A+E_A+P_A). For Q evaluated vertices, P_Q parent
reads and S_Q propagation edges, change-driven work costs O(Q log V+P_Q+S_Q).
Persistent scheduling buffers avoid clearing V flags for each heap update. Graph,
values and scheduling capacity remain O(V+E). Initial construction checks repeated
edges with row scans and can exceed linear work; it stays outside update timings.

The example has V=5,E=4,A=4,E_A=3,P_A=4,Q=4,P_Q=4,S_Q=3. An unchanged A has
Q=1,P_Q=0,S_Q=0. Cost formulas describe work, not predicted nanoseconds.

Use FIFO for a simple order, minimum Kahn for canonical ready-ID order, and DFS
when traversal frames fit the caller. Compare full scans for broad propagation with
heap scheduling for localized or unchanged outputs. See measurements for tested
workload/host selections. Graph edits, output-vector copies and process startup have
separate costs. No universal crossover is claimed.

## Run

```bash
cargo test -p topic-032-topological-order-and-incremental-dags
cargo run --release -p topic-032-topological-order-and-incremental-dags --example demo
python3 topics/032-topological-order-and-incremental-dags/scripts/run.py --help
```

The demo prints order [0,1,2,3,4], before [2,3,5,6,7], after [4,3,7,8,9], and4
recomputed vertices. The standalone runner uses rustc and Python's standard library.
Pass a new absolute `--out` directory and the frozen `--source-commit`.

## Primary sources

- [Princeton FIFO Kahn](https://algs4.cs.princeton.edu/code/javadoc/edu/princeton/cs/algs4/TopologicalX.html): indegree and ready-queue ordering.
- [Princeton DFS ordering](https://algs4.cs.princeton.edu/code/javadoc/edu/princeton/cs/algs4/Topological.html): reverse finish order.
- [Pearce and Kelly](https://whileydave.com/publications/pk07_jea/): dynamic topological repair beyond this fallback.
- [Adapton](https://adapton.org/): broader demand-driven incremental computation.

The authoritative catalog at the base revision contains no coverage arrays and no
ALGORITHM_INDEX.md. This topic retains its published ID/slug and the explicit revisit
scope above. The separate catalog reconciliation remains pending.
