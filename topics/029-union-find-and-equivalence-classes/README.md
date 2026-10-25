# Union-find and equivalence-class indexes

Maintain a partition while adding undirected equivalences. Dense IDs are in
`0..n`. `union(a,b)` returns whether two classes merged. `same(a,b)` tests
connectivity. Invalid IDs panic; a panic has no transactional state guarantee.
The structures require exclusive access during mutation.

Run the focused checks and experiment from the workspace root:

```bash
cargo test -p topic029-union-find
cargo run --release -p topic029-union-find --example bench -- full small
python3 topics/029-union-find-and-equivalence-classes/run.py /tmp/topic029-results
```

The last command compiles the exact topic sources with `rustc`, then runs the
fixed process campaign. Its output directory must be new. The benchmark JSON
contains `oracle: true`; the elapsed times vary by host.

## Running example

Merge `(0,1), (2,3), (4,5), (6,7), (0,2), (4,6), (0,4)`. Size balancing
makes all eight IDs equivalent. Before compression, the path from 7 is
`7 -> 6 -> 4 -> 0`. A full find first locates root 0, then makes 7, 6 and 4
point to 0. The equivalence class stays the same.

Render the labeled before/during/after diagram with:

```bash
dot -Tpng topics/029-union-find-and-equivalence-classes/compression.dot -o /tmp/compression.png
```

The source diagram was rendered and inspected for labels, alignment, contrast,
and clipping. It shows only the visited path.

## Candidates and costs

`QuickFind` uses eager labels. Queries compare two array entries; a successful
merge scans all labels. `Size` attaches the smaller root under the larger root.
`Full` uses the same size rule plus two-pass compression. `Half` redirects each
visited node toward its grandparent during traversal. Forest find functions
have `inline(never)` for generated-code inspection; selection applies to this
call policy and these implementations.

Let N be items, A merge attempts, U successful merges, Q queries, M=A+Q all
operations and w bytes per index. Eager labels cost O(N + U*N + M): every
attempt checks labels, while successful merges scan N labels. With N=8,A=7,
U=7,Q=3,M=10, a schematic count is 8+7*8+10=74. Duplicate attempts still cost
work even when U stays fixed. A balanced forest costs O(N + M*log N), with
maximum depth floor(log2 N): three for N=8. Each depth increase at least doubles
the class size. Counting two root searches per operation gives a schematic
bound 8+2*(7+3)*3=68, excluding bookkeeping. These counts describe growth,
rather than CPU instruction counts.

Balancing with compression gives O(N + M*alpha(N)) amortized work, where alpha
is the inverse Ackermann function. Here the expression is O(8+10*alpha(8)).
Individual searches still have variable cost. Rank is another balancing rule:
rank increases on equal-rank root merges and remains a height bound after
compression. The executable forest uses size.

Logical array storage is N*w for labels and 2*N*w for forests. For eight IDs
and 8-byte indices this is 64 and 128 bytes, excluding headers, allocation
metadata and capacity. Benchmark output allocation is inside the timed trace.

## Additional contracts

- A representative can change on union. A semantic minimum or external class
  name needs separate merge metadata. Requiring the minimum ID as the structural
  root can break balancing.
- `groups()` scans all IDs and allocates member vectors. Returning an eight-item
  class requires eight output records. Frequent single-class enumeration can
  use member lists, with smaller-to-larger movement bounded by O(N log N).
- `Rollback` leaves find paths unchanged, records successful merge writes and
  restores a history length in reverse order. Duplicate merges record nothing.
  Snapshot integers are valid only in their current history lineage. Discarded
  future snapshots become invalid when execution branches. They are not durable
  or cross-object version identifiers.
- Additive potentials store d[x]=g(x)-g(parent[x]). For g(b)-g(a)=w, attaching
  rb below ra requires the link w+da-db. For two singleton IDs and w=5 this is
  5+0-0=5; reversing attachment requires -5. Compression must compose link values.
  Parity uses exclusive-or. A same-root constraint must check consistency.
- Directed reachability, arbitrary edge deletion and witness paths require
  additional algorithms or retained graph data.

Implemented coverage: eager labels, size balancing, full compression, halving,
class sizes/enumeration and uncompressed rollback. Explicit revisit scope:
rank implementation, canonical-name metadata, weighted/parity implementations,
member lists, persistent snapshots, offline deletion, packed parent/size layout,
large working sets, hardware counters and concurrent variants.

## Evidence and selection

See [the fixed protocol](BENCHMARK.md) and
[obtained results](measurements/README.md). Choose a compressed balanced forest
for general incremental connectivity, uncompressed history for rollback, and
explicit metadata for each extra semantic requirement. Choose eager labels when
its measured query savings justify successful-merge scans. First and repeated
query timings exclude preparation of their partition.

The catalog at base 8a405f2 contains topic 029 with its stable slug and title.
It has no coverage array; `ALGORITHM_INDEX.md` is absent. The implemented and
revisit scopes above record the variants explicitly.

## Primary sources

- [Princeton connectivity derivation](https://algs4.cs.princeton.edu/15uf/)
- [Princeton rank and halving source](https://algs4.cs.princeton.edu/code/edu/princeton/cs/algs4/UF.java.html)
- [AtCoder DSU API](https://atcoder.github.io/ac-library/production/document_en/dsu.html)
- [AtCoder source](https://raw.githubusercontent.com/atcoder/ac-library/master/atcoder/dsu.hpp)
- [KACTL rollback source](https://github.com/kth-competitive-programming/kactl/blob/main/content/data-structures/UnionFindRollback.h)

Potential orientation and the numeric examples follow directly from the stated
invariants. Timing results are observations about the retained executable.
