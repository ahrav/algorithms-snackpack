# 061 Block range indexes and lossy summaries

A block summary can rule out an entire physical block. For an inclusive
predicate `[lo, hi]`, skip a block if every summary interval is disjoint.
A matching row implies at least one overlap. Check the original rows in
admitted blocks.

For `[50, 51]`, the example's middle block has values `18, 20, 91, 93`.
Its minimum and maximum admit the block. Splitting a sorted copy after 20
yields `[18, 20] ∪ [91, 93]`, which skips the block. The same query still
reads the final block and counts its one matching row.

| Layout | Build | Query work | Use |
|---|---|---|---|
| Full scan | None | N row predicates | Small data or most blocks admitted |
| Min/max | One pass | B summaries and R row predicates | Ordered blocks with selective filters |
| Two ranges | Sort a scratch copy of each block | B larger summaries and at most the min/max R | Separated values within blocks and frequent gap queries |

N is the number of rows, B is the number of blocks, and R is the number of
rows in admitted blocks. Each index trades B summary checks for N − R skipped
row checks. The example checks 12, 8, and 4 rows respectively. The two-range
layout stores 16 payload bytes per block versus 8 for min/max. For block
width w ≥ 2 it sorts in O(N log w) comparisons with O(min(N,w)) scratch.

The index borrows immutable rows. Signed bounds include both endpoints.
Reversed predicates are empty. The last block may be short. Duplicate matches
count separately. The builder uses widened gap arithmetic and breaks gap ties
at the first occurrence. An all-equal block repeats the singleton. A mutable
implementation must update or invalidate coverage before exposing a new
out-of-summary value. Read a block whose metadata is unknown.

```sh
cargo run --release -p block-range-indexes-and-lossy-summaries --example demo
cargo test -p block-range-indexes-and-lossy-summaries
python3 topics/061-block-range-indexes-and-lossy-summaries/benches/run.py /tmp/topic061-results
```

See [measurements](measurements/README.md) for native selection and scope.

- [PostgreSQL 17 BRIN](https://www.postgresql.org/docs/17/brin.html)
- [PostgreSQL minmax-multi implementation](https://github.com/postgres/postgres/blob/REL_17_STABLE/src/backend/access/brin/brin_minmax_multi.c)
- [Apache Parquet page index](https://parquet.apache.org/docs/file-format/pageindex/)
