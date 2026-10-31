# Reachability indexes for large DAGs

Answer whether an immutable directed graph contains a path from source to target.
Zero-edge paths count. IDs need valid endpoints; their numeric order is arbitrary.
`Dag::new` rejects cycles and invalid edges. Duplicate edges are accepted.
Publish a replacement graph and its index together. Queries use one snapshot.

## Running example

Edges: 0->1, 0->2, 1->3, 2->3, 2->4, 3->5, 4->5.
Before construction, vertex 2 has successors 3 and 4.
During reverse-topological construction, completed rows are 3={3,5}, 4={4,5}.
After their union with {2}, row 2={2,3,4,5}.
Thus 0 reaches 5, 1 cannot reach 4, and 4 reaches itself.
The rendered, inspected figure and full lesson are retained outside Git.

## Candidates and costs

Let n be vertices, m edges, Q queries, W=ceil(n/64), and d=3 interval dimensions.
Graph validation and topological sorting cost O(n+m), common to all candidates.

- `search`: exact iterative DFS with a reused stack and cleared visited map.
  Topological rank rejects backward queries. A searched query costs O(n+m),
  including O(n) visited clearing. Payload is O(n) beyond the graph.
- `Closure`: process vertices in reverse topological order. Include self, then OR
  every successor row. Construction costs O((n+m)W); each query reads one bit.
  Logical row payload is 8nW bytes. A checked caller budget bounds this payload.
- `Intervals` plus `search`: build min-post labels for three deterministic DFS
  orders. Reject failed containment; resolve passing queries by exact DFS.
  Construction costs O(d(n+m)), label payload is 2dn machine words.
  Failed containment costs O(d) after query setup; this implementation clears its
  O(n) visit map before the root interval check. Worst-case fallback is O(n+m+dn).

The example has n=6, m=7, W=1: closure uses 48 payload bytes and interval labels
use 288 on 64-bit hosts. One million vertices require 125,000,000,000 closure
bytes before graph/allocator/metadata/replacement overhead.
The recurrence is exact because every nonempty path starts with one outgoing edge
and every successor row is complete before its parent consumes it.

Intervals include descendants through cross edges. Containment admits false
positives. The filter uses forward, reversed, and rotated deterministic orders;
its measurements apply to those orders and this fallback implementation.
The scalar Rust source exposes word unions; compiler vectorization and hardware
cache effects require separate evidence before causal claims.

## Selection and limitations

Choose traversal for low query counts or frequent replacement. Choose closure
when repeated queries amortize construction and its complete payload fits.
Choose interval-filtered traversal when negative queries prune enough to repay
label construction and checking. See measurements for declared cell selections.
Charge allocations, teardown and overlapping snapshots over their actual lifetime.
The benchmark reports logical index payload; allocator peak/RSS remains unmeasured.

A chain-cover index stores the earliest reachable position per directed chain.
Its straightforward O(nk) entries favor small chain count k; k can reach n.
Two-hop labels answer through intersection of outgoing and incoming hub sets;
the construction must cover every reachable pair. Chain/2-hop construction,
compressed closure, epoch visit maps and dynamic maintenance are revisit scope.
Their timing is unmeasured. General graphs first require SCC condensation.

## Reproduce

```sh
cargo test -p topic035 --all-targets
cargo run -p topic035 --example running
python3 topics/035-reachability-indexes/scripts/run.py /tmp/topic035-new-run
```

Use a fresh output directory. The runner compiles exact topic files with rustc,
checks the independent breadth-first oracle and runs six fresh-process blocks.
Raw evidence stays outside Git. See BENCHMARK.md and measurements/README.md.

## Catalog audit and sources

At base 5825c7016c336c5b6fbb45c57a379023287fe359, Topic 35 is
`reachability-indexes`. The catalog has no coverage arrays; ALGORITHM_INDEX.md
is absent. Explicit named scope above records that limitation.

- [GRAIL: randomized interval labeling and exact search](https://www.cs.rpi.edu/~zaki/PaperDir/VLDBJ12.pdf)
- [Cohen et al.: reachability and distance via 2-hop labels](https://epubs.siam.org/doi/10.1137/S0097539702403098)

The implementation, storage calculations and workload hypotheses are derived
here. Published algorithm descriptions and measured outcomes have separate roles.
