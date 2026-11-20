# Native measurements

Both Linux hosts passed four optimized contract groups and the example. Each host ran
352 measured processes in complete blocks, 44 warmup processes and 32 same-candidate
control processes. Output vectors and untouched suffixes passed the oracle.

Medians are microseconds per call. Candidate order in each cell is cursor, ranks,
tiles, runs. The winner must beat every rival by at least 5% in all eight paired blocks.

| Records | Flag shape | Selected | Runs | Arm winner | Arm medians | x86 winner | x86 medians |
|---:|---|---:|---:|---|---|---|---|
| 16 | 50 | 11 | 4 | cursor | 0.012 / 0.019 / 0.018 / 0.022 | cursor | 0.019 / 0.031 / 0.032 / 0.045 |
| 4096 | 1 | 41 | 41 | runs | 2.159 / 4.183 / 2.283 / 1.141 | unresolved | 2.825 / 6.057 / 3.184 / 3.003 |
| 4096 | 50 | 2005 | 1033 | unresolved | 2.754 / 4.444 / 2.855 / 7.598 | unresolved | 5.148 / 8.063 / 5.025 / 10.893 |
| 4096 | 99 | 4053 | 44 | runs | 3.349 / 4.904 / 3.278 / 1.310 | runs | 4.937 / 6.937 / 5.493 / 3.219 |
| 4096 | alternate | 2048 | 2048 | unresolved | 2.688 / 4.406 / 2.910 / 9.319 | cursor | 4.253 / 7.922 / 5.110 / 16.376 |
| 4096 | long | 2048 | 8 | runs | 2.753 / 4.563 / 2.875 / 1.103 | runs | 3.803 / 6.502 / 4.327 / 2.741 |
| 1048576 | 1 | 10510 | 10397 | runs | 604.686 / 1129.776 / 672.565 / 421.161 | unresolved | 867.113 / 2198.292 / 980.646 / 911.741 |
| 1048576 | 50 | 524288 | 261851 | unresolved | 3577.366 / 4046.437 / 3631.829 / 4751.120 | unresolved | 5651.846 / 6797.699 / 5991.688 / 7612.373 |
| 1048576 | 99 | 1038087 | 10396 | runs | 910.801 / 1349.493 / 1035.683 / 448.533 | runs | 1232.978 / 1790.841 / 1419.040 / 942.396 |
| 1048576 | alternate | 524288 | 524288 | unresolved | 684.272 / 1145.330 / 751.471 / 2384.782 | cursor | 1028.324 / 1968.831 / 1238.710 / 4103.618 |
| 1048576 | long | 524288 | 2048 | runs | 698.452 / 1186.591 / 722.736 / 302.615 | runs | 929.125 / 1791.989 / 1042.553 / 816.751 |

Both hosts used one pinned logical CPU and locally optimized binaries. Arm used
rustc 1.98.1 on a c7g.16xlarge (CPU part 0xd40). x86 used rustc 1.99.0 on a
Xeon Platinum 8488C. Cursor-versus-cursor paired ratios ranged from
0.601 to 1.035 on Arm and 0.944 to 1.086 on x86. Raw data retain all per-process
samples, min/max ranges, build flags, kernel and CPU, and linked disassembly.
The timings cover warm out-of-place compaction with preallocated buffers.
Hardware causes, cold performance, parallel speedup, and allocation costs
remain unmeasured. The read and write counts in the guide are source-derived.

A first Arm build returned 18 instead of 41 for tile counts in the sparse
4,096-record fixture. A separate probe with 256 true flags returned 128 under
optimized native Arm for both filter-count and usize summation. The same
probe returned 256 at opt-level 0, at optimized generic, with optimized native
and SVE disabled, and for the bounded u32 reduction under every tested build.
The change uses u32 subtotals of at most 256 and accumulates in usize. The
fourth contract test checks a whole tile and tail under optimization. The
first campaign is excluded on both hosts. The linked Arm instructions in the
original image include scalable vector accumulation and a two-lane reduction.
The experiment isolates an optimization-dependent error on that build. A
compiler root-cause patch and other toolchains were outside this experiment.

Replay with `bash scripts/native.sh /tmp/compact-run` from the round directory.
Frozen source commit `785187bb74510b70b4d54b50bbe67c791b430a4f`. See [the receipt](EVIDENCE_RECEIPT.json)
for archive identities and the external raw-data location.
