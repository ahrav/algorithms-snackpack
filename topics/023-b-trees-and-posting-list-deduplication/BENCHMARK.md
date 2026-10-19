# Frozen experiment

benchmark_required: true

Five candidates: flat, tree, posting, packed8, packed32. Nine cells:tiny32pairs/
8keys; distinct16384/16384; duplicates32768/128; hot32768/1; range32768/128 with
width8; broad16384/16384 width4096; shuffled build_distinct/build_duplicates;
and small_lifecycle32/8. Keys are twice their ordinal. Input is a fixed
Fisher-Yates permutation, query seed23019,64fixed queries. Half-open width1
mixes hits/misses except hot which always returns every pair. No update timing
or production database timing claim. Mutable posting correctness is separate.

Eight process blocks, mirrored order within each pair, rotated starts, shuffled
cell order. One whole-operation warmup, three subsamples reduced to a median
per process. Five candidates plus two identical pair-tree controls per block:
504fresh processes per host. Inner repeats/subsamples are not independent runs.

Steady: index setup/final destruction excluded; range allocation, complete
materialization, fold and output destruction included. Build: construction,
64queries, fold and destruction included. Frozen repeats depend on cell, see
bench. Oracle/checking occurs before warmup; validation index is released for
lifecycle cells. Logical-byte calculations occur outside timing. No explicit
cache flush, pinning, frequency control, PMU attribution or allocation counter.

Selection: lowest process median. Resolve only if every paired rival/best
ratio exceeds1.03 in all8blocks. Retain raw process ranges and A/A ranges.
This is descriptive, not statistical confidence; scheduling/toolchains differ.
Stop after fixed schedule. No rerun to seek a winner. Replays exist to verify
committed source and preserve initial observations independently.

Every process checks each returned query against a scan/hashset/sort oracle.
Reject any mismatch before warmup. Verify transfer/source/binary identity,
architecture, hostname, CPU/kernel/toolchain/features/available CPUs. Raw
records, schedules, failures and metadata remain outside Git. Compact receipts
and aggregate medians/ranges are committed. See measurements/README.md.
