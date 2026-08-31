# Bitsets and density-adaptive bitmaps

A set of integer identifiers can be represented by the identifiers themselves,
by one bit per possible identifier, or by a portfolio that chooses a local
representation from the data. The right choice depends on universe span,
cardinality, clustering, and the operation being performed. Density alone is
not a universal selection rule.

## Contract

The crate exposes four immutable set types:

```rust
ReferenceSet
SortedSet
DenseBitSet
AdaptiveBitmap
```

Each type has the same semantic API:

```rust
try_new(universe_exclusive: u64, values: &[u32]) -> Result<Self, BitmapError>
universe_exclusive(&self) -> u64
len(&self) -> usize
is_empty(&self) -> bool
contains(&self, value: u32) -> bool
iter(&self) -> impl Iterator<Item = u32> + '_
intersection_len(&self, other: &Self) -> Result<usize, BitmapError>
payload_bytes(&self) -> u64
```

The universe is the half-open range `[0, universe_exclusive)`. It may be empty
and may be at most `2^32`. Construction accepts values in any order and removes
duplicates. It rejects the first input value outside the universe. Iteration
returns owned `u32` values in strictly increasing order. `contains` returns
`false`, rather than an error, for a value outside the universe.

`intersection_len` accepts only operands with equal universes and returns the
exact number of shared members. It does not allocate the intersection. The
types are intentionally immutable: construction cost is explicit, and no
insert, remove, union, serialization, or concurrent-mutation contract is
claimed.

`BitmapError` distinguishes these failures exactly:

```text
UniverseTooLarge { universe_exclusive }
ValueOutOfUniverse { input_index, value, universe_exclusive }
UniverseMismatch { left_universe_exclusive, right_universe_exclusive }
DenseAllocation { required_bytes }
```

`DenseAllocation` preserves the requested logical byte count when the fixed
word array cannot be allocated. `try_new(0, &[])` is valid for every type; any
value at universe zero is `ValueOutOfUniverse`.

For a representation `R`, define:

```text
decode(R) = { x in [0, U) | R.contains(x) }
```

Two representations are semantically equal when their universes match and
their decoded sets match. Capacity, trailing zero words, tree shape, container
kind, and serialized bytes are not semantic. This matters because the Roaring
portable format permits more than one encoding of the same logical bitmap.

## Running example

Let the universe be `[0, 16)` and construct:

```text
A = {1, 2, 3, 4, 10, 12, 14, 15}
B = {2, 3, 5, 10, 11, 14}
```

The same values have three useful views:

```text
[Dense-bit view: index labels are decimal]

index  0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15
A      0 1 1 1 1 0 0 0 0 0  1  0  1  0  1  1
B      0 0 1 1 0 1 0 0 0 0  1  1  0  0  1  0
A AND B
       0 0 1 1 0 0 0 0 0 0  1  0  0  0  1  0

[Sorted-merge view]

A cursor: 1, 2, 3, 4,    10,     12, 14, 15
B cursor:    2, 3,    5, 10, 11,     14
matches:     2, 3,       10,         14

[Result]

A intersection B = {2, 3, 10, 14}
intersection_len = 4
```

All values share adaptive high key zero. `A` has eight values in four maximal
runs, so its 16-byte array baseline beats an 18-byte run payload. `B` has six
values in four runs, so its 12-byte array also wins. The example establishes
semantic equality and operation behavior. The later threshold equations
explain when actual 65,536-value chunks instead become bitmaps or runs.

## Candidates and invariants

`ReferenceSet` is the deliberately simple semantic model. It favors clarity
over contiguous layout and supplies the differential oracle.

`SortedSet` stores one sorted `u32` per member. Binary search answers
membership. Intersection advances two ordered cursors and can stop when either
side is exhausted.

`DenseBitSet` stores `ceil(U / 64)` words. Membership selects one word and one
bit. Intersection cardinality applies `AND` and population count to every word,
including words that represent no members.

`AdaptiveBitmap` partitions by the high 16 bits. Each occupied key has one
low-value container. Its baseline rule is exact:

```text
cardinality <= 4096  -> Array
cardinality >  4096  -> Bitmap
```

It then chooses `Run` only when the run payload is strictly smaller than that
baseline. A tie keeps the baseline. Empty chunks have no container.

`AdaptiveBitmap::container_summaries()` yields `ContainerSummary` values in key
order. The public fields are `key`, `kind`, `cardinality`, `run_count`, and
`payload_bytes`; `kind` is one of the `ContainerKind` variants `Array`,
`Bitmap`, or `Run`. These summaries expose the selected representation for
tests and evidence without exposing mutable internals.

## Derived costs and thresholds

Let `U` be universe size, `W = ceil(U / 64)`, `c` cardinality, `a` and `b`
operand cardinalities, and `r` run count. `payload_bytes` reports the logical
representation payload only. It excludes struct fields, allocator metadata,
spare capacity, and reference-model node overhead.

| Candidate | Logical payload | Membership | Intersection cardinality |
|---|---:|---:|---:|
| `ReferenceSet` | `4c` reported bytes | logarithmic model lookup | model traversal |
| `SortedSet` | `4c` bytes | `O(log c)` | `O(a + b)` merge upper bound |
| `DenseBitSet` | `8W` bytes | `O(1)` | `Theta(W)` word operations |
| `AdaptiveBitmap` | sum of container payloads | key lookup plus local lookup | key merge plus container-pair work |

A fixed dense bitmap and a global sorted `u32` payload are equal when:

```text
U / 8 = 4c  =>  c = U / 32
```

That is 3.125% density when `U` is divisible by 64. It is a payload crossover,
not a latency threshold.

Within one 65,536-value adaptive chunk, the portable payload formulas are:

```text
Array:   2c bytes
Bitmap:  1024 * 8 = 8192 bytes
Run:     2 + 4r bytes
```

Therefore `2c = 8192` at `c = 4096`, which is 6.25% local density. Run wins
over an array only when `2 + 4r < 2c`; it wins over a bitmap when
`2 + 4r < 8192`, equivalently `r <= 2047`. The implementation's strict rule
keeps an array for a three-value single run because both payloads are six
bytes, but chooses a run for four consecutive values because six bytes is then
smaller than eight.

Adaptive intersection first merges occupied high keys. Matched container
pairs then have different work:

- array/array merges sorted lows, with work bounded by both cardinalities;
- bitmap/bitmap examines 1,024 word pairs;
- array/bitmap scans the array and probes the bitmap;
- run/run advances ordered intervals;
- mixed run pairs advance runs against values or covered bitmap words.

The 4,096 boundary and the payload equations predict representation size. They
do not predict a universal runtime winner. Branches, cache residency,
population-count throughput, allocation, and operand skew remain machine- and
workload-dependent.

## Selection guide

- Use `ReferenceSet` as an oracle, not as a presumed performance winner.
- Try `SortedSet` for globally sparse immutable sets, compact iteration, and
  intersections that can terminate early. Strong operand skew can favor a
  galloping search not implemented in this first artifact.
- Try `DenseBitSet` when `U` is bounded, bitmap operations dominate, and paying
  for every word is acceptable. Its predictable loop can beat compressed
  representations even above or below a payload crossover.
- Try `AdaptiveBitmap` when density and clustering differ by high-16 chunk, or
  when both random-looking dense chunks and long local ranges occur.
- Keep a measured portfolio when shape changes. Do not turn 3.125%, 6.25%, or
  the original Roaring paper's historical heuristics into universal speed
  rules.

WAH and EWAH are serious alternatives only for read-mostly workloads with long
global clean-word runs and much more Boolean algebra than random access. They
are not implemented here. Roaring run containers cover the important local
range case while retaining high-key indexing.

## Limits

- The primary `total_ns` includes construction: each timed sample builds both
  operands from the raw, duplicate-containing inputs, so allocation, sorting,
  duplicate normalization, dense zero-fill, chunking, and run detection are
  inside the measured cost and inside the representation rankings.
- Insertion, removal, materialized union/intersection, serialization, memory
  mapping, and iteration throughput are outside the primary benchmark.
- `payload_bytes` is not resident memory. In-memory headers, tree nodes,
  capacities, alignment, and allocator fragmentation are excluded.
- Dense allocation scales with universe span and may fail even for an empty
  logical set.
- Sparse values scattered one per high-key chunk can make adaptive metadata
  dominate its payload. Dense random chunks with no runs can favor a plain
  bitmap. Random updates can fragment runs, but this immutable artifact does
  not measure that lifecycle.
- The sorted intersection is a linear merge; it does not implement the
  size-ratio galloping heuristic studied by Roaring implementations.
- No unsafe code, SIMD-specific kernel, 64-bit identifier wrapper, portable
  serializer, or concurrent API is included.
- Benchmark results apply only to the exact source, linked image, host,
  compiler, seven cells, and run window recorded with the evidence.

## Run

```bash
cargo test -p topic-003-density-adaptive-bitmaps
cargo bench -p topic-003-density-adaptive-bitmaps --bench bitmap_sets --no-run
```

The frozen experiment is in [BENCHMARK.md](BENCHMARK.md). Correctness scope and
the invariant-test audit are in [TEST_STRATEGY.md](TEST_STRATEGY.md).

## Primary sources

- Chambi, Lemire, Kaser, and Godin, [Better bitmap performance with Roaring bitmaps](https://arxiv.org/abs/1402.6407), 2014. This is the original array/bitmap design and operation analysis.
- Lemire, Ssi-Yan-Kai, and Kaser, [Consistently faster and smaller compressed bitmaps with Roaring](https://arxiv.org/abs/1603.06549), revised 2018. This adds run containers and analyzes their selection.
- Lemire et al., [Roaring Bitmaps: Implementation of an Optimized Software Library](https://arxiv.org/abs/1709.07821), revised 2022. This documents optimized container kernels and skew-aware array intersection.
- RoaringBitmap project, [Roaring bitmap portable format specification](https://github.com/RoaringBitmap/RoaringFormatSpec/). This is the current wire-layout authority for array, bitmap, and run payloads.
- RoaringBitmap project, [`roaring-rs` 0.11.5 API](https://docs.rs/roaring/0.11.5/roaring/bitmap/struct.RoaringBitmap.html) and [0.11.5 serialization source](https://docs.rs/roaring/0.11.5/src/roaring/bitmap/serialization.rs.html). These are current production API and validation references; this crate does not depend on them.
- Oracle, [Java SE 24 `BitSet`](https://docs.oracle.com/en/java/javase/24/docs/api/java.base/java/util/BitSet.html). This is a production dense-bit-vector contract for indexed membership and Boolean operations.
- Wu, Otoo, and Shoshani, [Compressing bitmap indexes for faster search operations](https://sdm.lbl.gov/~kewu/ps/LBNL-49626-tods.pdf). This is the original WAH design.
- Lemire, Kaser, and Aouiche, [Sorting improves word-aligned bitmap indexes](https://arxiv.org/abs/0901.3751), with the official [JavaEWAH implementation](https://github.com/lemire/javaewah). These delimit the read-mostly compressed-word alternative not implemented here.
