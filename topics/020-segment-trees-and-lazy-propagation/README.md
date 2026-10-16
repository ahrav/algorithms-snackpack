# Segment trees and lazy propagation

Keep range summaries current while postponing leaf work. The stored node sum
already includes its pending tag; descendants may still owe that update.

## Contract and example

Fixed zero-based positions, half-open `[l,r)`, wrapping u64 sums and affine
updates `x -> a*x+b` modulo 2^64. Add d uses `(1,d)`; assign v uses `(0,v)`.
Invalid bounds panic before mutation. Empty ranges return zero/do nothing.
Allocation failure follows Vec policy. This is single-threaded educational code.

```text
Indices                 0  1  2  3  4  5  6  7
Before: stored leaves   2  1  3  4  0  5  2  1   root sum = 18

During: stored leaves   2  1  3  4  0  5  2  1   root sum = 30
Pending intervals             [2,4) [4,6)
Current interval sums           13     11
Pending tags                    +3     +3

After query pushes tags 2  1  6  7  3  8  2  1
Query [3,5)                      7 +3          = 10
```

The middle row is physical storage, not the logical array. Add three to `[2,6)`:
`[2,4)` changes from 7 to 13, `[4,6)` from 5 to 11. Query `[3,5)` pushes tags
before reading 7+3=10. Full-cover operations update a node directly; partial
operations push before descent; partial updates pull child sums afterward.

## Algebra and failure cases

Associative combine plus identity supports range folding. Inverse and
commutativity are unnecessary. For an ordered iterative fold, append left
fragments and prepend right fragments. The separate OrderedTree test checks all
ranges of thirteen distinct strings; sum-only tests cannot detect reversal.
String summaries are a correctness demonstration, not constant-cost benchmark states.

Lazy maps additionally compose and distribute over combine. A sum needs length:
`mapping((a,b),(s,k))=(a*s+b*k,k)`. Here s is sum and k is interval length;
our implementation gets k from node bounds. Example: 7+3*2=13. New `(a,b)` after
old `(c,d)` is `(a*c,a*d+b)`. Assign four then add three gives assign seven;
add three then assign four gives assign four. Reversing composition passes
add-only tests. Rebuilding a node from stale children loses pending updates.
Finite-width exact or floating-point semantics require another admission contract.

## Cost and selection

Let n be length, k range width, h=ceil(log2(n)), C the uniform combine/mapping
cost. For the example n=8, k=4, h=3. Scanning touches four updated values;
lazy decomposition covers two nodes. Scan costs O(k), materialized prefix
queries O(1) with O(n) rebuild after updates, eager-tree queries O(log n) with
O(k+log n) leaf updates, and lazy queries/updates O(log n) for fixed-size tags.
Build is O(n). A full-cover lazy operation can take O(1). With nonconstant
oracles, multiply by their cost, or sum node-dependent costs along the traversal.

This deliberately simple recursive layout reserves 4n sums; Lazy also reserves
4n two-u64 tags. Payload is 96n bytes for Lazy (768 at n=8), 32n for Eager
(256), 8n for Scan (64), and 16n+8 for Prefix (136), excluding allocator
metadata/capacity slack. Empty trees reserve a small sentinel allocation.

Start with scan for little total work, prefixes for immutable queries, and
lazy state for broad interleaved updates. Two Fenwick trees specialize additive
range updates/sums; ordinary two-tree construction does not supply assignment.
Benchmark alternatives under the actual update/read/range-width mix.

## Catalog coverage and revisit

Range aggregation, lazy range updates and ordered combine have runnable code.
Vectorized window aggregation is covered by DuckDB's primary report: batching
leaf inputs and tree states through aggregate update/combine APIs is distinct
from deferred mutation tags. Its 2048-row batch is a historical implementation
fact, not an ISA SIMD guarantee or a threshold for this Rust code.

Large-state and holistic limits are part of the selection. Child medians plus
lengths cannot determine the union median. Exact holistic state can be merged
when sufficient data are retained; merge/allocation costs can grow. Copying full
sequences at every level gives O(n log n) stored elements in that particular
representation. Specialized overlapping-window state can avoid repeated work.
Revisit production vectorized aggregate APIs, SQL FILTER/EXCLUDE/order rules,
exact distinct/quantile state, large-state allocation and specialized moving
windows. Persistent trees, segment-tree Beats and concurrent snapshots are also
outside this first implementation, not hidden coverage claims.

## Reproduce

```bash
cargo test -p segment-trees
cargo run -p segment-trees --example states
cargo bench -p segment-trees --bench portfolio -- lazy mixed
cd topics/020-segment-trees-and-lazy-propagation
python3 runner.py
```

The runner is Linux-only for its environment capture. Every candidate process
checks each query and final state against an independent eager-array oracle,
then releases validation allocations before warmup. See BENCHMARK.md and
measurements/README.md for exact timing, identity, results and limits.

## Primary sources

- [AC Library contract and complexity qualification](https://github.com/atcoder/ac-library/blob/864245a00b00dd008d1abfdc239618fdb7d139da/document_en/lazysegtree.md)
- [Affine sum/length example](https://github.com/atcoder/ac-library/blob/864245a00b00dd008d1abfdc239618fdb7d139da/test/example/lazyseg_practice1.cpp)
- [Ordered fold and new-after-old implementation](https://github.com/atcoder/ac-library/blob/864245a00b00dd008d1abfdc239618fdb7d139da/atcoder/lazysegtree.hpp)
- [DuckDB vectorized windows, 2025-02-14](https://duckdb.org/2025/02/14/window-flying)
- [DuckDB holistic-window limits, 2021-11-12](https://duckdb.org/2021/11/12/moving-holistic)
