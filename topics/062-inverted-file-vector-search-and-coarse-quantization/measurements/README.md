# Topic 062 native measurements

Frozen executable source: `57d8357c1ce996cf9c28eff660ff275c9d9a9202`.
Replay from a scratch copy of the topic directory at that commit with
`bash benches/native.sh`. The [evidence receipt](EVIDENCE_RECEIPT.json)
names both verified archives, their linked image hashes, and the
environment and workload manifests. Later commits change `benches/run.py`
and `benches/native.sh` and add `benches/test_run.py`. The Rust sources
are unchanged. Both scripts now refuse an existing output directory, and
a replay of the later scripts writes the runner output to `evidence/run/`
and records the allowed CPU list in `host.txt`.

On Arm the host exposed 64 CPUs and CPU implementer/part 0x41/0xd40.
The x86-64 host exposed 192 CPUs and reported Intel Xeon Platinum 8488C.
Both ran the declared Linux 6.12.115 kernel family and Rust 1.93.0
with edition 2024, opt-level 3 and target-cpu=native. Exact host and
target feature data are in the retained archives. Both runs were unpinned.
Their host records hold the `nproc` count and no allowed CPU list, so their
CPU placement cannot be reconstructed. Three integration tests
and the demo passed on each host. Each host ran 512 timed processes
(including 64 duplicate flat runs), 32 warmup processes and four untimed
quality processes. Quality counts and timed output digests matched.

The combined index build medians in ms, in tiny, balanced, boundary and
skew order, were 0.040, 1.995, 1.965, 1.909 on Arm and 0.048, 2.526, 2.522,
1.779 on x86-64. The query timer starts after setup and one untimed sweep.

Both hosts selected raw1 on tiny, pq1 on balanced and raw4 on boundary.
On skew, raw1 gave the lowest eligible point estimate on both hosts and
fell short of the preset paired advantage rule. The skew raw1 p95 scan
count reached 14,834 of 16,384 rows. The boundary rows cover 92 queries on
an adjacent-centroid bisector and the four seeded from group 15, which keep
their jittered coordinate. Boundary raw1 recovered 247/480 true
IDs and raw4 recovered 480/480. With four probes PQ returned 196/480 and
exact reranking from 16 compressed candidates returned 353/480. Scanning
all codes also returned 196/480 in that workload. Expanding probes alone
did not fix the fixed-codebook distortion. The balanced grid was deliberately
representable exactly and does not predict accuracy on trained embeddings.

Each `processes.csv` row holds one process. `query_ns` is the elapsed
nanoseconds of the whole timed query batch, `queries` is the number of
searches in that batch and `build_ns` is index construction. Time per query
is `query_ns / queries`.

Each time below is the process median [first, third quartile] in microseconds
per query. Recall counts are shared by both hosts. Mean scans and p95 scans
count original rows or compressed entries as appropriate. Rerank work is
listed separately from scanned entries.

| Workload | Path | Recall IDs / 480 | p10 recall / 5 | Mean scanned | p95 scanned | Mean reranked | Arm | x86-64 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tiny | flat | 480 | 5 | 256.0 | 256 | 0.0 | 1.430 [1.421, 1.436] | 1.040 [1.029, 1.067] |
| tiny | raw1 | 480 | 5 | 16.4 | 27 | 0.0 | 0.304 [0.299, 0.305] | 0.225 [0.222, 0.259] |
| tiny | raw4 | 480 | 5 | 65.1 | 77 | 0.0 | 0.548 [0.541, 0.550] | 0.489 [0.483, 0.503] |
| tiny | pq1 | 382 | 3 | 16.4 | 27 | 0.0 | 0.287 [0.285, 0.292] | 0.216 [0.212, 0.223] |
| tiny | pq4 | 382 | 3 | 65.1 | 77 | 0.0 | 0.519 [0.515, 0.526] | 0.447 [0.439, 0.463] |
| tiny | rr1 | 479 | 5 | 16.4 | 27 | 14.6 | 0.514 [0.510, 0.520] | 0.374 [0.364, 0.386] |
| tiny | rr4 | 479 | 5 | 65.1 | 77 | 16.0 | 0.815 [0.811, 0.829] | 0.674 [0.654, 0.691] |
| tiny | raw_all | 480 | 5 | 256.0 | 256 | 0.0 | Untimed | Untimed |
| tiny | pq_all | 382 | 3 | 256.0 | 256 | 0.0 | Untimed | Untimed |
| balanced | flat | 480 | 5 | 16384.0 | 16384 | 0.0 | 63.382 [63.193, 63.454] | 45.627 [45.544, 46.164] |
| balanced | raw1 | 480 | 5 | 1029.6 | 1102 | 0.0 | 5.875 [5.868, 5.898] | 4.977 [4.924, 5.041] |
| balanced | raw4 | 480 | 5 | 4097.7 | 4174 | 0.0 | 21.503 [21.464, 21.545] | 18.296 [18.211, 18.598] |
| balanced | pq1 | 480 | 5 | 1029.6 | 1102 | 0.0 | 3.299 [3.289, 3.302] | 2.862 [2.817, 2.880] |
| balanced | pq4 | 480 | 5 | 4097.7 | 4174 | 0.0 | 11.293 [11.270, 11.310] | 9.871 [9.740, 9.933] |
| balanced | rr1 | 480 | 5 | 1029.6 | 1102 | 16.0 | 5.212 [5.199, 5.229] | 4.249 [4.163, 4.357] |
| balanced | rr4 | 480 | 5 | 4097.7 | 4174 | 16.0 | 13.212 [13.188, 13.227] | 11.348 [11.246, 11.498] |
| balanced | raw_all | 480 | 5 | 16384.0 | 16384 | 0.0 | Untimed | Untimed |
| balanced | pq_all | 480 | 5 | 16384.0 | 16384 | 0.0 | Untimed | Untimed |
| boundary | flat | 480 | 5 | 16384.0 | 16384 | 0.0 | 63.435 [63.116, 63.634] | 46.092 [45.939, 48.348] |
| boundary | raw1 | 247 | 1 | 1028.8 | 1086 | 0.0 | 5.965 [5.962, 5.993] | 5.165 [5.144, 5.192] |
| boundary | raw4 | 480 | 5 | 4095.1 | 4154 | 0.0 | 21.698 [21.670, 21.720] | 18.618 [18.494, 19.119] |
| boundary | pq1 | 142 | 0 | 1028.8 | 1086 | 0.0 | 3.383 [3.380, 3.399] | 3.021 [2.967, 3.217] |
| boundary | pq4 | 196 | 1 | 4095.1 | 4154 | 0.0 | 11.489 [11.468, 11.508] | 10.134 [10.006, 10.509] |
| boundary | rr1 | 222 | 1 | 1028.8 | 1086 | 16.0 | 5.454 [5.439, 5.466] | 4.761 [4.673, 4.848] |
| boundary | rr4 | 353 | 2 | 4095.1 | 4154 | 16.0 | 13.907 [13.877, 13.922] | 12.058 [12.026, 12.266] |
| boundary | raw_all | 480 | 5 | 16384.0 | 16384 | 0.0 | Untimed | Untimed |
| boundary | pq_all | 196 | 1 | 16384.0 | 16384 | 0.0 | Untimed | Untimed |
| skew | flat | 480 | 5 | 16384.0 | 16384 | 0.0 | 63.451 [63.336, 63.573] | 45.946 [43.961, 46.315] |
| skew | raw1 | 480 | 5 | 11764.9 | 14834 | 0.0 | 43.886 [43.828, 43.924] | 43.593 [43.238, 44.115] |
| skew | raw4 | 480 | 5 | 12088.2 | 15161 | 0.0 | 45.716 [45.644, 45.805] | 45.152 [44.881, 45.425] |
| skew | pq1 | 144 | 0 | 11764.9 | 14834 | 0.0 | 30.952 [30.899, 30.960] | 26.714 [26.690, 26.928] |
| skew | pq4 | 144 | 0 | 12088.2 | 15161 | 0.0 | 31.898 [31.866, 31.934] | 27.606 [27.555, 27.826] |
| skew | rr1 | 262 | 1 | 11764.9 | 14834 | 16.0 | 33.785 [33.731, 33.832] | 29.009 [28.977, 29.208] |
| skew | rr4 | 262 | 1 | 12088.2 | 15161 | 16.0 | 34.724 [34.703, 34.803] | 29.895 [29.811, 30.297] |
| skew | raw_all | 480 | 5 | 16384.0 | 16384 | 0.0 | Untimed | Untimed |
| skew | pq_all | 144 | 0 | 16384.0 | 16384 | 0.0 | Untimed | Untimed |

Duplicate flat control median ratios (tiny, balanced, boundary, skew):
- Arm: 0.995, 1.000, 1.001, 1.002.
- x86-64: 0.995, 1.008, 0.980, 0.999.

Generated library assembly includes widened multiply-accumulate on Arm and
packed multiply in the x86 search function. Those instructions do not isolate
the source of process latency differences. Training, real embeddings, cold I/O,
concurrency, floating-point behavior and production engines are outside this run.
