# Stable compaction by exclusive scan

Pack selected u32 values into the beginning of a separate output buffer.
Preserve input order, duplicates, and the suffix after the returned filled
length. Use one bool flag per input value and an output slice at least as long
as the input slice. The rank and tile methods use caller-provided scratch.

For flags `[0, 1, 1, 0, 1, 0, 1, 1]`, the exclusive prefix sums are
`[0, 0, 1, 2, 2, 3, 3, 4]`. A selected record goes into the slot equal to
its rank. With two four-record tiles, the counts are `[2, 3]` and the tile
offsets are `[0, 2]`. Each tile's local rank plus its offset gives the
global slot. The ordered prefix assigns disjoint ranges even if workers
complete out of order. The implementations here run on one thread.

| Method | Source-level cost for n flags, k selected, q tiles, r runs |
|---|---|
| Cursor | One flag pass, k output writes, one running rank. |
| Rank array | Two flag passes, n rank writes, k output writes, n usize entries. |
| Tile offsets | Two flag passes, q offset writes, k output writes, q usize entries. |
| Run copying | One search over flags with at most two repeated checks per run, r copies of k total values, constant scratch. |

For eight flags split into two tiles, all candidates write five records.
A 64-bit rank array uses 64 bytes. Tile offsets use 16 bytes. The three
selected runs contain two, one, and two values.

Choose a cursor for simple sequential output. Compare run copying for long
selected spans or stretches that can be skipped. Use tile offsets to assign
disjoint ranges before launching workers. Keep the full rank array when
downstream code needs each rank. See [measured results](measurements/README.md)
for workload and host-specific choices. Buffer allocation, in-place overlap,
parallel speedup, segmented resets and GPU execution have separate contracts.

```bash
cargo test -p stable-flag-compaction
cargo run --quiet -p stable-flag-compaction --example compact
cargo bench -p stable-flag-compaction --bench compact -- cursor 4096 long 1024
```

Sources include [Blelloch's prefix-sum report](https://www.cs.cmu.edu/afs/cs.cmu.edu/project/scandal/public/papers/CMU-CS-90-190.html),
[Harris, Sengupta, and Owens on scan and scatter](https://developer.nvidia.com/gpugems/gpugems3/part-vi-gpu-computing/chapter-39-parallel-prefix-sum-scan-cuda)
and [Merrill and Garland on look-back and in-place compaction](https://research.nvidia.com/sites/default/files/pubs/2016-03_Single-pass-Parallel-Prefix/nvr-2016-002.pdf).
