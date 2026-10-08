# Binary, galloping, and branch-aware search

Find the first index whose `u64` value is at least the query. Input must be
nondecreasing and immutable during lookup. Return `len` for an above-range
query and zero for an empty slice. Equality preserves the first duplicate.
Every implementation is safe Rust and allocates nothing during lookup.

For `[2, 4, 4, 4, 9, 13, 18, 21]`, queries `0, 4, 10, 22` return `0, 1, 5, 8`.
Binary search for `4` probes indices `4, 2, 1, 0`; equality moves `hi` left.
Before: `[lo, hi) = [0, 8)`. During: `[0, 4)`, `[0, 2)`, `[0, 1)`.
After: `[1, 1)`, return 1. Positions below `lo` are too small; positions at
or above `hi` already qualify. The rendered diagram is retained externally
with the receipt; no image binary is committed.

## Candidates and costs

- Linear stops at the first qualifying value. It favors short or early-rank
  lookups; late answers require scanning the prefix.
- Binary halves an unresolved interval. It bounds comparisons without a
  new representation; accesses remain dependent and nonconsecutive.
- Gallop doubles probes from zero, then binary-searches a proven bracket.
  Early ranks can benefit; late ranks pay expansion and refinement.
- Select uses a query-independent loop length and `select_unpredictable`.
  This is a source-level candidate, not a promise of branch-free assembly.
- Standard uses `partition_point(|&v| v < key)` with identical semantics.
  `binary_search` is not a first-duplicate substitute.

Let `n` be length and `r` the answer. Linear makes `min(r+1,n)` comparisons:
our `n=8,r=1` example makes two. Binary's nonempty worst-case bound is
`floor(log2(n))+1`, four here. Select makes `ceil(log2(n))+1`, also four.
Gallop has `O(log(r+2))` comparisons; this implementation makes three here.
These are derived counts, not timing measurements. The independent oracle
counts every element below the key and does not share bracket logic.

A forward-only hint must satisfy `hint == 0 || a[hint-1] < key`, with
`hint <= n`. An arbitrary index within the equal run can skip the first
duplicate. Our candidate avoids this obligation by starting at zero.
Saturating probe growth and clipping bound its arithmetic and indexing.

Value storage is `8n` bytes, 64 bytes here. A maintained index costs
`B + qT + uU`, where `B` is build cost, `q` queries cost `T` each, and `u`
updates cost `U` each. For this eight-value index with 1000 queries and ten
updates: `B_8 + 1000T_8 + 10U_8`. Lookup timing covers only `T`.

## Run

```bash
cargo run -p lower-bound-portfolio --example search
cargo test -p lower-bound-portfolio
python3 topics/007-binary-galloping-branch-aware-search/scripts/run.py --out /tmp/topic007-results
```

See `BENCHMARK.md`, `TEST_STRATEGY.md`, and `measurements/README.md` for the
frozen protocol, actual selection, source identities, and limitations.
Use the standard API as a maintainable default; test scan for tiny/early
answers, gallop for small displacement from a proven starting bound, and
conditional selection for the actual query distribution and emitted code.
Eytzinger or other layouts add build/update/mapping costs and are untested.

## Primary sources

- [Rust slice contracts](https://doc.rust-lang.org/std/primitive.slice.html#method.partition_point).
- [Rust select_unpredictable](https://doc.rust-lang.org/std/hint/fn.select_unpredictable.html), stable since 1.88, no guaranteed instruction lowering.
- [Rust 1.98.0 slice source](https://github.com/rust-lang/rust/blob/1.98.0/library/core/src/slice/mod.rs), a versioned implementation snapshot.
- [CPython 3.14 galloping discussion](https://github.com/python/cpython/blob/v3.14.0/Objects/listsort.txt), merge-specific adaptive behavior, no transferable threshold.
- [Khuong and Morin, Array Layouts for Comparison-Based Searching](https://arxiv.org/abs/1509.05053), hardware/workload-specific evidence for layout and prefetch alternatives.
