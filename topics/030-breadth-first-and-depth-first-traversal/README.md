# Breadth-first and depth-first traversal

Choose the result contract first: minimum unweighted distances, or ordered
DFS entry/finish events and directed-cycle detection. Queue and graph layout
choices are measured only inside their respective result family.

## Contract and running example

Vertices are `0..V`; graph construction validates every endpoint. Graphs stay
immutable during traversal. Preserve neighbor order, duplicates and self loops.
Invalid source IDs are ignored. Duplicate sources count once. Empty source
lists leave all vertices unreachable. Distances use `usize::MAX` as a sentinel.
A supplied root list defines the searched region; all vertex IDs produce a
complete DFS forest. BFS assumes unit edge cost.

```text
0: [1, 2]
1: [3]
2: [3]
3: [1, 4]
4: []
5: []
```

BFS claims a vertex before enqueueing. Distances become `[0,1,1,2,3,unreachable]`.
DFS frames retain `(vertex,next_neighbor)` while following a child.
Preorder is `[0,1,3,4,2]`; finish order is `[4,3,1,2,0]`. Edge `3->1` reaches
an active ancestor and establishes a directed cycle. Edge `2->3` later reaches
a finished vertex. Vertex5 stays unreachable from source0.

The source adjacency lists above define the graph directly. A rendered,
inspected before/during/after diagram is retained in the daily task outputs.

## Candidates and costs

- `bfs_deque`: adjacency lists, front removal/back insertion. A simple baseline.
- `bfs_vec`: adjacency lists, append IDs and advance a head; retains every reached ID.
- `bfs_csr`: the same vector queue over compressed sparse rows. Conversion costs
  time and extra storage; contiguous adjacency is a performance hypothesis.
- `bfs_layers`: two CSR frontier vectors, explicit distance-level boundaries.
  Deep narrow inputs incur a boundary per level. Execution stays sequential.
- `dfs_frames`: one saved continuation per active vertex; preserves recursive events.
- `dfs_events`: reverse-pushed entry events and explicit exits. Discovery occurs
  when entry executes; stale entries are checked. Pending events can scale with edges.

`distance_oracle` uses a binary heap and repeated minimum-key extraction. The
bounded test DFS oracle uses recursion. Neither is a timed candidate. A plain
vertex stack suffices for reachability; preserving recursive finish events
requires continuation state or correctly ordered entry/exit events.

Let V be allocated vertices, S supplied roots/sources, R reached vertices,
E_R scanned outgoing edges, H maximum active DFS depth and w bytes/usize.
All implemented candidates cost O(V+S+R+E_R), including initialization and
root-list scanning. The example substitutes V6,S1,R5,E_R6. Vector BFS retains
R IDs; deque/two-frontier live lengths follow frontier width, with capacity
slack. DFS frames use O(H) frames plus O(V) colors and O(R) returned orders;
events can retain O(R+E_R) events. CSR payload uses w(V+1+E) bytes. The example
has six edges: 8(6+1+6)=104 logical bytes on a 64-bit target, excluding headers,
spare capacity and allocator overhead. Building CSR while retaining adjacency
lists raises peak storage.

For CSR preparation B and per-query times A (adjacency), C (CSR), k queries
favor CSR when B+kC<kA. If A>C, the break-even condition is k>B/(A-C).
Use measured per-host values; tiny or changing graphs can favor direct lists.

BFS gives minimum edge counts under unit weights. DFS requires an active/
finished distinction for directed-cycle detection. Dependency execution needs
predecessor completion accounting beyond BFS levels. Weighted shortest paths,
topological evaluation, strongly connected components, and parallel traversal
have separate contracts.

## Run and evidence

```bash
cargo run -p topic030-traversal --example traversal
cargo test -p topic030-traversal
python3 topics/030-breadth-first-and-depth-first-traversal/run.py --out /tmp/traversal-new
```

Standalone, from this directory:

```bash
rustc --edition=2024 --test tests/contracts.rs -o /tmp/traversal-contracts
/tmp/traversal-contracts
python3 run.py --focused --out /tmp/traversal-focused-new
```

See BENCHMARK.md for the frozen controls, TEST_STRATEGY.md for independent
oracles and measurements/README.md for results, dispersion and limits.

Catalog boundary: current origin/main has no coverage arrays or
ALGORITHM_INDEX.md. This lesson explicitly covers single/multiple-source BFS,
deque/vector/level frontiers, adjacency/CSR layouts, frame/event DFS,
root forests, and directed-cycle state. Revisit scope: bidirectional traversal,
direction-optimizing/parallel traversal, epoch reuse and mutable-graph protocols.
The missing authoritative coverage/index remains separate pending metadata work.

Primary sources:

- [Boost BFS](https://www.boost.org/latest/libs/graph/doc/html/graph/algorithms/traversal/breadth_first_search.html): FIFO and traversal events.
- [Boost DFS](https://www.boost.org/latest/libs/graph/doc/html/graph/algorithms/traversal/depth_first_search.html): recursive discovery/finish and colors.
- [Cornell graph traversal](https://courses.cis.cornell.edu/courses/cs2110/2026sp/lectures/lec22/): unweighted traversal semantics.
- [Rust VecDeque](https://doc.rust-lang.org/std/collections/struct.VecDeque.html): growable ring representation and queue operations.

These primary sources support the algorithm/library facts. Byte counts and
bounds above are derivations from this artifact. Performance results are
measurements under the recorded experiment; cache/branch causes remain inference.
