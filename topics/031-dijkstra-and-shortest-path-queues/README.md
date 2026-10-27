# Dijkstra and shortest-path queues

Find exact minimum costs from one source in an immutable directed graph. Choose
queue storage from the admitted weight domain and measured query costs.

## Running example

Edges are A→B:10, A→C:1, C→B:1, B→D:1 and C→D:20. E is isolated.

| State | Settled vertices | Tentative distances | Relevant queue records |
|---|---|---|---|
| After A | A:0 | B:10, C:1 | C:1, B:10 |
| After C | A:0, C:1 | B:2, D:21 | B:2, B:10, D:21 |
| Complete | A:0, C:1, B:2, D:3 | E:unreachable | B:10 and D:21 rejected as stale |

Relaxing an edge tests `candidate = distance[u] + weight(u,v)` and changes the
label only on a strict improvement. Processing C improves B to `1+1=2`.
Processing B improves D to `2+1=3`. Minimum extraction finalizes a distance
because nonnegative extensions preserve the ordering argument. First discovery
can precede a later improvement. Strict improvements terminate zero-cost cycles.

## Contract

`Graph::new` validates every endpoint and admits at most `u32::MAX` vertices.
Edges have `u32` costs; loops, parallel edges, zero cycles and isolated vertices
are admitted. `shortest_paths` returns all `u64` distances and work counters.
`INF = u64::MAX` marks unreachable. Invalid endpoints or sources return errors.
An empty graph has no valid source. Tie ordering can vary across queues; the
returned contract contains distances rather than a particular predecessor tree.

A simple shortest path uses at most `V-1` edges. One outgoing relaxation adds
at most `V` costs. Under admission, the largest such sum is bounded by
`(2^32-1)^2 = 18,446,744,065,119,617,025 < u64::MAX`. This establishes both
arithmetic safety and separation from the unreachable sentinel. Floating-point
costs, negative edges, changing graphs and path enumeration need another contract.

## Queue selection and costs

Let `V` count vertices, `E` directed edges including duplicates, `R` successful
relaxations, `W` maximum edge weight and `D` largest removed bucket key,
including stale entries. Storage below excludes the immutable graph and includes
the distance vector.

| Queue | Time bound for this implementation | Auxiliary storage | Admission and failure condition |
|---|---|---|---|
| Scan | `O(V²+E)` | `O(V)` | Dense or small baseline; repeated full scans cost more on large sparse graphs |
| Lazy binary | `O(V+E+(R+1) log(R+2))` | `O(V+R)` | General admitted domain; duplicate records add removal and memory cost |
| Indexed binary | `O(V+E+(V+R) log(V+1))` | `O(V)` | Position-map swaps and decrease-key must agree |
| Lazy radix | `O(V+E+65(R+1))` | `O(V+R+65)` | Monotone `u64` keys; redistribution retains duplicate-record costs |
| Circular Dial | `O(V+E+R+W+D)` | `O(V+R+W)` | Whole-graph `W≤64` check; gaps consume empty-bucket scans |
| 0/1 deque | `O(V+E)` | `O(V+E)` conservative lazy-record bound | Whole-graph weights in `{0,1}`; zero improvements enter front, unit improvements enter back |

For the example, `V=5,E=5,R=5,W=20,D=21`. Substituting into the displayed
work expressions gives scan `30`, lazy `10+6 log₂7≈26.8`, indexed
`10+10 log₂6≈35.8`, radix `10+65×6=400`, and Dial `56` bound units.
These are scaling expressions with hidden constants, not predicted timings.
The deque rejects this example's weight domain.

The scan repeatedly searches labels. The lazy heap records each improvement
and rejects stale extractions. The indexed heap updates the existing record
through its position map. A radix record is grouped by the highest differing
bit relative to the last extracted key; redistribution moves it to a smaller
bucket index. Full-width keys require 65 buckets including the equal-key bucket.
Dial uses `W+1` circular buckets and scans increasing distance values.

General lazy binary complexity can involve `log E` on an unrestricted
multigraph. The indexed radix theorem in Mehlhorn's notes uses handles and
constant-time decrease-key. Its `O(E+V log W)` result requires that specific
design; this duplicate-entry implementation uses the conservative bound above.

Start with lazy binary for general integer inputs. Compare indexed binary when
duplicate traffic or a vertex-sized queue bound matters. Admit deque or Dial
from the whole graph's weight domain. Compare scan on small and dense graphs.
Use measured workload-specific decisions in `measurements/README.md`.

Fibonacci/pairing heaps, bidirectional search, A*, parallel delta stepping,
predecessor selection and reusable query arenas are explicit revisit scope.
The authoritative catalog has no coverage array and `ALGORITHM_INDEX.md` is
absent at this base. This lesson records its implemented variants explicitly.

## Run

```bash
cargo test -p shortest-path-queues
cargo run --release -p shortest-path-queues --example running
cargo bench -p shortest-path-queues --bench queues -- tiny lazy
```

Expected example output:

```text
distances: [0, 2, 1, 3, unreachable]; stale pops: 2
```

The independent oracle uses synchronous Bellman-Ford rounds with `u128` optional
labels. It shares graph storage and reads with candidates, while its scheduling,
relaxation representation and arithmetic differ. Exhaustive and adversarial
checks are bounded evidence. See `TEST_STRATEGY.md` and `BENCHMARK.md`.

## Primary sources

- [Rust BinaryHeap](https://doc.rust-lang.org/std/collections/binary_heap/index.html): max-heap ordering and the lazy-entry example.
- [Boost.Graph Dijkstra](https://www.boost.org/doc/libs/latest/libs/graph/doc/html/graph/algorithms/shortest_paths/dijkstra_shortest_paths.html): nonnegative-weight contract and relaxation events.
- [Mehlhorn, Shortest Paths](https://www.mpi-inf.mpg.de/~mehlhorn/ftp/DataStructuresAndGraphAlgs.pdf): queue-operation cost derivation and handle-based radix construction.

Finalize on minimum extraction, schedule strict improvements, and measure the
physical records stored by the chosen queue.
