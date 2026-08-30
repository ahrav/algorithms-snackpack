# Difference arrays and deferred range updates

A batch of overlapping range additions can repeat the same write many times.
Boundary methods record where each addition starts and stops, then recover the
final values with one scan.

## Contract

The crate exposes four functions with the same signature:

```rust
fn apply_candidate(
    input: &[i64],
    updates: &[RangeUpdate],
) -> Result<Vec<i64>, RangeError>;
```

The public candidates are `apply_reference`, `apply_dense_sidecar`,
`apply_in_place_difference`, and `apply_sorted_events`.

`RangeUpdate { start, end, delta }` adds `delta` to every position in the
half-open range `[start, end)`. A range is invalid when `start > end` or
`end > input.len()`. Every function returns the first invalid update in input
order as:

```text
RangeError { update_index, start, end, len }
```

Every function validates the whole batch before constructing a result. Empty
ranges and zero deltas are valid no-ops. All arithmetic uses two's-complement
`i64` wrapping semantics. Success returns a new `Vec<i64>` with the same length
and order as `input`. The caller's input stays unchanged.

Semantic equality means exact equality of every output value in sequence. A
set or multiset comparison would erase an ordering defect.

## Running example

Start with:

```text
input   = [5, 1, 4, 0, 2, 7]
updates = [1, 5) += 3
          [3, 6) += -2
          [2, 2) += 99
```

The empty third range changes nothing. The active added value at each position
is `[0, 3, 3, 1, 1, -2]`. Adding that scan to the input gives:

```text
[5, 4, 7, 1, 3, 5]
```

## Why boundaries work

For one update `[l, r) += v`, record `+v` at `l` and `-v` at `r`. Let `e_j`
be the sum of records at boundary `j`. The active delta at position `i` is:

```text
s_i = e_0 + e_1 + ... + e_i
```

The result is `y_i = input_i + s_i`, with every addition performed modulo
`2^64`. For the running example, the nonzero boundary totals are:

```text
e_1 =  3
e_3 = -2
e_5 = -3
e_6 =  2
```

Their prefix scan is `[0, 3, 3, 1, 1, -2]` over positions `0..6`.

The in-place candidate first turns a copy of the input into adjacent
differences:

```text
d_0 = input_0
d_i = input_i - input_(i-1), for i > 0
```

It adds the same boundary records to `d`, then scans `d`. The scan reconstructs
the original values plus all active updates. Its core invariant is: before
writing output position `i`, the running sum equals the wrapped input value at
`i` plus every update whose range contains `i`.

## Candidate costs

Let `n` be the input length, `m` the update count, and
`W = sum(end - start)` over valid updates.

| Candidate | Work after validation | Extra storage beyond returned output | Main risk |
|---|---:|---:|---|
| Reference eager writes | `Theta(n + W)` | `Theta(1)` | Rewrites overlap one element at a time |
| Dense sidecar | `Theta(n + m)` | `Theta(n)` boundary slots | Allocates and clears a dense sidecar |
| In-place difference | `Theta(n + m)` | `Theta(1)` | Performs a backward difference pass and a forward scan |
| Sorted events | `Theta(n + m log m)` | `Theta(m)` events | Sorting can dominate small or dense batches |

All four candidates also spend `Theta(m)` validation checks and allocate the
returned `n`-element vector. Empty ranges may be skipped after validation.
The event candidate must preserve repeated events. Replacing equal-position
events with one value loses multiplicity.

Materializing an exact `n`-element result requires `Omega(n)` output writes.
Boundary methods remove repeated range writes. They do not remove the output
scan. The serious choice is therefore about overlap, event count, temporary
memory, sorting, and constant work on the target machine.

## Selection guide

- Use the eager reference for an oracle and for tiny batches whose total width
  `W` is small. Its direct loop is easy to inspect.
- Try the dense sidecar when `m` or overlap is high and another `n` scalar
  buffer fits the memory budget.
- Try the in-place difference candidate when one returned vector should carry
  both the temporary differences and the result.
- Try sorted events when `m` is small relative to `n` and the event storage is
  cheaper than a dense sidecar. Measure the sorting crossover.
- Use a Fenwick tree when updates and prefix queries must interleave online.
  Use a lazy segment tree when online range updates must interleave with richer
  range queries. This crate solves an offline batch that materializes all
  output values.

No candidate is universally fastest. The benchmark covers six declared cells
that vary `n`, `m`, range width, ordering, clustering, and repeated endpoints.
Its result applies only to the exact source, linked image, host, compiler, and
run window recorded with the evidence.

## Failure modes and limits

- Inclusive-end code changes the public range meaning and can write one element
  too far.
- Applying updates before full validation can do wasted work before returning
  the required error. The pure API exposes only the final error or result.
- Checked, saturating, or debug-only overflow behavior violates the wrapping
  arithmetic contract.
- Deduplicating equal events loses repeated updates. Sorting by position is
  allowed only when every event contribution remains in the wrapped sum.
- A dense sidecar can exceed the memory budget even when the returned output
  fits.
- Sorted events still scan all `n` output positions. Sparse updates do not make
  result materialization sublinear.
- These implementations are sequential. Parallel scans change the scheduling
  and memory-traffic model.

## Run

```bash
cargo test -p topic-002-difference-arrays-and-range-updates
cargo bench -p topic-002-difference-arrays-and-range-updates --no-run
```

The frozen experiment and evidence rules are in [BENCHMARK.md](BENCHMARK.md).
The correctness scope is in [TEST_STRATEGY.md](TEST_STRATEGY.md).

## Primary sources

- Guy E. Blelloch, [Prefix Sums and Their Applications](https://www.cs.cmu.edu/afs/cs.cmu.edu/project/scandal/public/papers/CMU-CS-90-190.html), CMU-CS-90-190, 1990. This report defines scan as a general primitive and gives its work model.
- ISO C++ working draft [N5054](https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2026/n5054.pdf), 2026. Section `[numeric.ops.scan]` specifies inclusive and exclusive scan semantics.
- Peter M. Fenwick, [A New Data Structure for Cumulative Frequency Tables](https://www.cs.auckland.ac.nz/~peter-f/FTPfiles/TechRep88.pdf), University of Auckland report 88, and the [1994 journal paper](https://doi.org/10.1002/spe.4380240306). These sources describe online cumulative-frequency updates and queries.
- AtCoder Library, [Fenwick Tree](https://atcoder.github.io/ac-library/production/document_en/fenwicktree.html) and [Lazy Segment Tree](https://atcoder.github.io/ac-library/production/document_en/lazysegtree.html). These are production-library contracts for online alternatives.
- Rust standard library, [`i64`](https://doc.rust-lang.org/std/primitive.i64.html). The `wrapping_add` and `wrapping_sub` methods define the arithmetic used here.
