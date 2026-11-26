# Associative reductions, deferred merges, and compaction

A snapshot reader needs the state at its retained boundary. Store an exact sum
and count for each interval between cuts. Readers merge only visible intervals.
For observations `[10, 10, 10, 30, 80]` and cuts `[2, 4, 5]`, the three
retained answers are `(20, 2)`, `(60, 4)` and `(140, 5)`.

## Contract

History accepts signed 32-bit measurements up to `u32::MAX` rows. Their maximum
absolute sum is below the signed 64-bit limit for every subset. Cut offsets
increase strictly and include the end of the stream. Snapshot ID zero returns
the empty state. The other IDs are one-based cut ordinals. The engine returns
exact pairs. Divide once at the boundary that needs a mean. An empty mean is
undefined. The compact representation preserves precisely the declared cuts.

A sum is distributive. A mean is algebraic because its intermediate state
needs both sum and count. Combining finalized group means gives each group
equal weight even when row counts differ. Associativity permits regrouping.
Reordering also requires commutativity. Wrapping affine composition preserves
operand order and provides a separate numerical contract. Checked addition
can fail on one grouping and succeed on another. Floating-point grouping
changes rounding.

## Compare

- Raw replay copies the input and scans the visible prefix. Choose it when
  readers need raw rows or a measurement shows the read mix favors copying.
- Chunked reduction builds one summary per group of up to 32 rows with a cut
  between each snapshot interval. It preserves smaller intermediate intervals
  for future composition while reducing repeated prefix work.
- Compact reduction builds one state per retained interval. It gives the
  smallest state count of these candidates. With one cut per row it stores as
  many summaries as raw rows, each wider than the input.

All constructors process N values. A read at cut i visits the raw prefix
length, the number of visible chunks, or i compact states. For the example
and reads at cuts two and three, raw visits 4 + 5 = 9 inputs and each reduced
candidate visits 2 + 3 = 5 states. The width and build cost differ.

RocksDB distinguishes a partial result that remains an operand from a full
result based on ordered operands and an established base. Compaction also
respects surviving snapshots. Its file selection may exclude older state.
This crate models numerical aggregation across declared cuts in memory.

```sh
cargo test --test contracts
cargo run --example contracts
bash scripts/native.sh "$PWD" /tmp/reduction-results
```

Run from the topic directory. The native script selects a Linux CPU and writes
to a fresh output directory. The example prints both snapshot fractions and
the different results for reversed affine operations and reassociated floats.

[Data Cube](https://arxiv.org/abs/cs/0701155) ·
[The LSM-Tree](https://db.cs.berkeley.edu/cs286/papers/lsm-acta1996.pdf) ·
[RocksDB merge operator contract](https://github.com/facebook/rocksdb/blob/abeebd9630f11bd08c28b7bd43c7bdfc62050654/include/rocksdb/merge_operator.h).

See [benchmark design](BENCHMARK.md), [results](measurements/README.md), and
[test strategy](TEST_STRATEGY.md).
