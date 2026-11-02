# Benchmark decision: true

Compare flat and graph widths 4/16/64 at k=4 on these fixed cases:

| Case | N | d | maximum degree | geometry |
|---|---:|---:|---:|---|
| 0 | 32 | 2 | 4 | uniform |
| 1 | 64 | 16 | 16 | uniform |
| 2 | 1024 | 2 | 4 | uniform |
| 3 | 1024 | 2 | 16 | uniform |
| 4 | 1024 | 16 | 4 | uniform |
| 5 | 1024 | 16 | 16 | uniform |
| 6 | 1024 | 16 | 16 | two clusters, second-cluster queries |
| 7 | 1024 | 2 | 16 | disconnected clusters, first-cluster entry |

Build ranks permitted neighbors, keeps M-2 nearest, adds allowed ring neighbors.
Seed 37 controls corpus and 32 subsequent queries. Entry 0 and exact scorer/tie
order remain fixed. Every process checks distances, IDs, order, checksum,
flat-oracle agreement and recall using independent Python exhaustive scoring.

Query timing includes validation, allocation, scoring, traversal, selection,
result consumption and teardown. Build and process wall time are separate.
Each process first runs one diagnostic query batch, then max(2,4096/N) repeated
batches. One warmup process per case/candidate is excluded. Eight measured
process blocks use four position/precedence-balanced orders repeated twice.
The process is the sample unit. Stop at eight blocks. Report medians, ranges
and paired ratios. A candidate qualifies at full fill and mean ID recall>=0.95.
A unique winner must beat every qualified rival by >5% in every paired block;
otherwise report unresolved plus the descriptive minimum median.

The regime is warm repeated-query batches, one fixed build, single-threaded,
unpinned. Small inputs control working-set size. Cold caches, wider k, multiple
build seeds, allocated capacity/RSS, service tails and update costs remain
revisit scope. Source/binary hashes, native compiler features, CPU identities,
affinity availability, assembly and work counts accompany results. Timings
make no hardware-causal or universal claim. Quality rejection differs from
implementation failure. Build amortization requires a separate lifetime model.
