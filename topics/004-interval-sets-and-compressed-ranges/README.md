# Interval sets and compressed ranges

An interval set stores integer points as a small list of ranges. The design
depends on exact boundary rules and one canonical form. Compression helps when
covered points form long runs. It can lose when ranges fragment or when online
updates make flat-vector shifts expensive.

This crate builds immutable sets of `u32` points from arbitrary input ranges.
It compares an independent point oracle, wide and packed flat runs, a grouped
boundary sweep, and a range tree.

## Contract

Every input interval is half-open. `[start, end)` contains each integer `x` for
which `start <= x < end`. Public endpoints are `u64` values in `0..=2^32`.
The extra endpoint represents the valid final one-point range
`[u32::MAX, 2^32)`.

- `start == end` is valid and contributes no points.
- `start > end` returns `Reversed { index, start, end }`.
- An endpoint above `2^32` returns `OutOfDomain { index, endpoint }`.
- Domain validation runs before order validation for each input interval.
- Any error rejects the whole construction. No partial set escapes.
- Duplicate, overlapping, nested, and adjacent inputs have set semantics.
- Stored runs are nonempty, start-sorted, disjoint, and non-adjacent.
- Consecutive stored runs `p` and `q` satisfy `p.end < q.start`.
- `contains(x)` reports membership. `cardinality()` returns the number of
  points as `u64`.
- `union`, `intersection`, and `difference` return canonical sets.

Semantic equality means equal members. Canonicalization makes structural and
semantic equality agree. Canonicalization discards input order, duplicate
count, source identity, and original boundaries.

The crate exposes five representations:

```rust
PointOracle
FlatIntervalSet
PackedIntervalSet
BoundaryEventSet
BTreeIntervalSet
```

Each implements `IntervalSet`, whose semantic methods are `contains`,
`cardinality`, and `canonical_intervals`. Every representation builds through
`try_from_intervals(&[Interval])`. The free `union`, `intersection`, and
`difference` functions accept any two `IntervalSet` implementations and return
a canonical `FlatIntervalSet`.

## Running example

The input arrives in this order:

```text
A = [5, 9)
B = [1, 4)
C = [3, 7)
D = [9, 12)
E = [6, 6)    empty
F = [14, 16)
```

Read each numbered column as one integer point. `#` means covered. `.` means
not covered. The first table also shows the intersection of `A` and `C`.

# Visual 1 of 5: coverage, points 0 through 7

| Range or result | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| A `[5,9)` | . | . | . | . | . | # | # | # |
| B `[1,4)` | . | # | # | # | . | . | . | . |
| C `[3,7)` | . | . | . | # | # | # | # | . |
| A intersection C | . | . | . | . | . | # | # | . |
| Union so far | . | # | # | # | # | # | # | # |

Shared members of A and C: `{5, 6}`.

# Visual 2 of 5: coverage, points 8 through 15

| Range or result | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| A `[5,9)` | # | . | . | . | . | . | . | . |
| D `[9,12)` | . | # | # | # | . | . | . | . |
| F `[14,16)` | . | . | . | . | . | . | # | # |
| Full union | # | # | # | # | . | . | # | # |

`A` excludes point `9`, while `D` includes it. No integer lies between the two
ranges, so the canonical form joins them. Points `12` and `13` form a real gap.
The canonical result is `[1,12)` union `[14,16)`, with cardinality `13`.

## Candidates

The independent point oracle inserts each covered point into a
`BTreeSet<u32>`. It then groups consecutive points into runs. The oracle does
not call the optimized normalizer or its merge helper. The point expansion is
too expensive for long spans, but the simple representation gives tests an
independent membership oracle.

Wide flat runs discard empty intervals, sort by `(start, end)`, then scan once.
The last output interval represents the current connected component.

# Visual 3 of 5: sort-and-merge execution

| Step | Current input | Output before | Decision | Output after |
|---:|---|---|---|---|
| 1 | B `[1,4)` | empty | seed first run | `[1,4)` |
| 2 | C `[3,7)` | `[1,4)` | `3 < 4`, overlap | `[1,7)` |
| 3 | A `[5,9)` | `[1,7)` | `5 < 7`, overlap | `[1,9)` |
| 4 | D `[9,12)` | `[1,9)` | `9 == 9`, adjacent | `[1,12)` |
| 5 | F `[14,16)` | `[1,12)` | `14 > 12`, gap | `[1,12)`, `[14,16)` |

The merge condition is `current.start <= last.end`. Using `<` preserves the
same members but violates the canonical-form contract by retaining adjacency.

Packed flat runs store an eight-byte `(u32 start, u32 length_minus_one)` pair.
Every stored run is nonempty, so the pair can represent lengths from `1`
through `2^32`. End decoding must widen before addition:

```rust
u64::from(start) + u64::from(length_minus_one) + 1
```

The grouped-event path turns every nonempty `[a,b)` into `(a,+1)` and `(b,-1)`.
It sorts events and applies every delta at the same coordinate as one group.

# Visual 4 of 5: grouped boundary sweep

| Coordinate | Starts | Ends | Active before | Active after | Boundary emitted |
|---:|---:|---:|---:|---:|---|
| 1 | 1 | 0 | 0 | 1 | start `1` |
| 3 | 1 | 0 | 1 | 2 | none |
| 4 | 0 | 1 | 2 | 1 | none |
| 5 | 1 | 0 | 1 | 2 | none |
| 7 | 0 | 1 | 2 | 1 | none |
| 9 | 1 | 1 | 1 | 1 | none |
| 12 | 0 | 1 | 1 | 0 | end `12` |
| 14 | 1 | 0 | 0 | 1 | start `14` |
| 16 | 0 | 1 | 1 | 0 | end `16` |

Coordinate `9` is the critical case. Applying the end and start separately can
emit a false gap. The grouped net delta is zero, so coverage remains active.

The range-tree path stores canonical runs in a tree keyed by start. An insert
checks its predecessor, absorbs every connected successor, removes the old
runs, and inserts one replacement. The tree avoids long flat-vector shifts
during online updates. It pays for allocation, branches, pointer traversal, and
weaker locality.

# Visual 5 of 5: candidate comparison before measurement

| Path | Working state | Bulk construction | Membership after build | Main cost or risk |
|---|---|---|---|---|
| Point oracle | one tree node per point | `O(m_attempt log m)` | `O(log m)` | expands long spans |
| Wide flat runs | `16r` endpoint payload bytes | `O(n log n)` | `O(log r)` | vector sort and later tail shifts |
| Packed flat runs | `8r` encoded payload bytes | `O(n log n)` | `O(log r)` | widened end decoding |
| Event sweep | `2n` signed events | `O(n log n)` | `O(log r)` after materialization | twice the endpoints and grouped-tie rules |
| Range tree | tree node per run | `O(n log r)` workload-conditioned | `O(log r)` | allocations and pointer-heavy traversal |

The tree bound is workload-conditioned because one insertion can absorb many
runs and `r` changes during construction. The table states derived work and
payload costs. It does not report measured speed.

## Correctness invariants

After sort-and-merge consumes the first `k` nonempty sorted inputs:

1. The output equals the union of those `k` inputs.
2. Every output interval is nonempty.
3. Output intervals are sorted and separated by a real gap.
4. The last output interval is the complete connected component containing the
   `k`th input.

For the next `[a,b)`, `a <= last.end` means overlap or adjacency. Extending the
last end to `max(last.end,b)` preserves the union. When `a > last.end`, a real
gap exists and the scan appends the interval. These two cases preserve all four
invariants and yield the unique canonical union.

For grouped events, the counter after all events at coordinate `x` equals the
number of inputs that cover points immediately to the right of `x`. A
`0 -> positive` transition starts a run. A `positive -> 0` transition ends one.
The counter must never become negative and must return to zero.

Canonical operands make set algebra linear in run count. Two monotone cursors
compute union, intersection, and difference in `O(r + s)` work for operands
with `r` and `s` runs, plus output writes.

## Derived cost model

Use these separate quantities:

- `n`: nonempty input intervals.
- `r`: canonical output runs.
- `m`: distinct covered points.
- `m_attempt`: point insert attempts, including duplicates and overlap.
- `U`: bounded universe size for a dense bitmap alternative.

For the running example, `n = 5`, `r = 2`, and:

```text
m = (12 - 1) + (16 - 14) = 11 + 2 = 13
wide payload          = 16r = 32 bytes
packed payload        =  8r = 16 bytes
raw u32 point payload =  4m = 52 bytes
wide / raw points      = 4r / m = 8 / 13 = 0.615...
packed / raw points    = 2r / m = 4 / 13 = 0.307...
```

The `4m` value describes a contiguous array of raw `u32` point keys. It is not
the footprint of `PointOracle`, whose `BTreeSet` pays tree-node overhead. All
payload counts exclude headers, capacity, alignment, and allocator metadata. If
every point is isolated, `r = m`. Wide runs then use four times the raw
point-array payload and packed runs use twice the payload.

A dense bitmap needs `ceil(U/8)` payload bytes. The example window with
`U = 16` needs two bytes. The full `u32` universe needs `536,870,912` bytes.
The same representation can be useful or wasteful depending on the universe.

Comparison normalization of arbitrary unsorted, disjoint ranges has an
`Omega(n log n)` comparison lower bound. Encode each key as a separated
one-point range. Ordered canonical output reveals the sorted keys. This bound
does not cover fixed-width integer methods outside the comparison model. Every
method also needs `Omega(r)` output writes.

## Failure modes

- Mixing closed and half-open bounds changes membership at the upper endpoint.
- Storing the exclusive domain end in `u32` loses the valid endpoint `2^32`.
- Failing to merge adjacency breaks canonical structural equality.
- Processing equal-coordinate events separately can create a false gap.
- Testing a reversed out-of-domain input without checking validation precedence
  can pass through the wrong guard.
- Sharing a merge helper between the oracle and candidates makes the oracle
  circular.
- Canonicalization destroys data when multiplicity, provenance, or original
  boundaries are part of the contract.
- Trusting serialized runs can break binary search and cardinality arithmetic.
- Decoding packed ends in `u32` can overflow a valid terminal run.
- Calling one candidate universally optimal overstates one workload or host.

## Selection guide

- Start with wide flat sort-and-merge for offline batches that become
  read-mostly.
- Use packed runs when payload and tail-movement traffic matter enough to repay
  widened decode arithmetic.
- Use a verified sorted-input scan when an upstream contract supplies sorted
  ranges.
- Use grouped events when the input already uses deltas or the application
  needs counts or weights.
- Use a range tree when updates must remain online and vector shifts are
  material.
- Use a dense bitmap when `U` is small and bounded, fragmentation is high, and
  word-parallel set algebra dominates.
- Use a chunked hybrid when density and run shape vary across the universe.
- Use an interval tree when queries must return original overlapping records.
  Use a coverage-count structure when multiplicity matters.

Keep a portfolio when range shape, construction cost, mutation rate, memory
budget, or hardware can change the winner.

## Limits and evidence boundary

The crate models immutable Boolean set coverage. It does not preserve source
interval identity or overlap count. It does not define serialization or trust
unvalidated stored runs. The point oracle intentionally cannot expand huge
ranges. The first artifact contains no unsafe code or concurrent mutation.

Complexity, operation counts, payload equations, and lower-bound reductions are
derived. Production behavior is observed in the linked documentation and
source. Candidate timing and machine-mechanism claims require the frozen
benchmark. Any benchmark result applies only to its exact source, linked image,
workload, host, compiler, order, and run window.

## Run

```bash
cargo test -p topic-004-interval-sets-and-compressed-ranges
cargo bench -p topic-004-interval-sets-and-compressed-ranges --no-run
cargo bench --package topic-004-interval-sets-and-compressed-ranges \
  --bench interval_sets -- \
  --candidate flat \
  --workload tiny_sparse_sorted_unique \
  --phase build_membership \
  --seed 2026090104 \
  --warmup 0 \
  --samples 1 \
  --inner 1
```

The focused bench command checks canonical intervals, cardinality, membership,
and the terminal packed decode against the independent oracle before timing one
descriptive sample. Its JSON record must report `oracle_match: true`,
`membership_match: true`, `domain_end: 4294967296`, and
`packed_max_decode: 4294967296` in `canaries`.

The frozen experiment is in [BENCHMARK.md](BENCHMARK.md). Correctness scope and
the invariant-test audit are in [TEST_STRATEGY.md](TEST_STRATEGY.md).

## Primary sources

- [PostgreSQL 18 range and multirange documentation](https://www.postgresql.org/docs/current/rangetypes.html)
- [PostgreSQL `multirange_canonicalize` source](https://github.com/postgres/postgres/blob/master/src/backend/utils/adt/multirangetypes.c)
- [Boost ICL interval combining styles](https://www.boost.org/doc/libs/latest/libs/icl/doc/html/index.html)
- [Boost ICL `interval_set` reference](https://www.boost.org/doc/libs/latest/libs/icl/doc/html/boost/icl/interval_set.html)
- [Guava `TreeRangeSet` source](https://github.com/google/guava/blob/master/guava/src/com/google/common/collect/TreeRangeSet.java)
- [Linux generic interval-tree source](https://codebrowser.dev/linux/linux/include/linux/interval_tree_generic.h.html)
- [CRoaring API and run optimization](https://github.com/RoaringBitmap/CRoaring/blob/master/include/roaring/roaring.h)
- [Chambi, Lemire, Kaser, and Godin, *Better bitmap performance with Roaring bitmaps*](https://arxiv.org/abs/1402.6407)
- [Huacheng Yu, *Cell-probe Lower Bounds for Dynamic Problems via a New Communication Model*](https://arxiv.org/abs/1512.01293)
- [Fredman and Weide, *On the Complexity of Computing the Measure of a Union of Intervals*](https://doi.org/10.1145/359545.359553)
- [Rust slice `partition_point` documentation](https://doc.rust-lang.org/std/primitive.slice.html#method.partition_point)
