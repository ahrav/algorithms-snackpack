# Measurements

Frozen executable inputs: `cf90420d55899b4afadd726772262104f6cc95d5`. All five input hashes match this commit on both hosts.

Every host passed two native contract groups and the example, 612 main processes (544 measured, 68 warmup), plus 16 A/A processes. All 3969 short binary pairs passed the independent edit graph; all timed candidates matched independent full-matrix complete-ID results.

Medians below are microseconds per complete lookup. `?` marks an unresolved cell; the named candidate has the lowest median. Each clear selection beats every rival by >5% in every paired block.

| N | m | layout | k | Arm selection | x86 selection |
|---:|---:|---|---:|---|---|
| 32 | 8 | random | 1 | myers 1.063 [1.058,1.075] | myers 0.824 [0.822,1.024] |
| 32 | 8 | prefix | 1 | myers 1.088 [1.084,1.235] | myers 0.843 [0.838,0.850] |
| 32 | 8 | dense | 3 | myers 1.366 [1.360,1.381] | myers 1.005 [1.001,1.029] |
| 32 | 48 | random | 1 | band 2.141 [2.136,2.156] | band 1.502 [1.492,1.517] |
| 32 | 48 | prefix | 1 | myers 6.494 [6.405,6.719] | myers 5.520 [5.510,5.737] |
| 32 | 48 | dense | 3 | myers 6.600 [6.590,6.766] | myers 5.629 [5.613,5.638] |
| 4096 | 8 | random | 1 | myers 121.229 [121.172,121.410] | myers ? 95.467 [95.207,160.457] |
| 4096 | 8 | prefix | 1 | trie 46.408 [46.225,46.474] | trie 32.772 [32.527,34.210] |
| 4096 | 8 | dense | 3 | trie 51.881 [50.912,52.996] | trie 44.893 [44.619,45.572] |
| 4096 | 48 | random | 1 | band 193.285 [192.663,194.352] | band 155.413 [154.004,160.097] |
| 4096 | 48 | prefix | 1 | trie 297.803 [294.394,302.777] | trie 204.379 [201.943,306.649] |
| 4096 | 48 | dense | 3 | trie 279.909 [279.283,285.260] | trie 189.198 [187.527,195.786] |
| 4096 | 48 | random | 4 | myers ? 804.743 [803.714,806.386] | band 622.714 [610.370,638.905] |
| 4096 | 48 | prefix | 4 | myers 803.624 [803.498,807.134] | myers 694.301 [692.992,703.371] |
| 4096 | 48 | dense | 0 | trie 29.408 [29.036,30.609] | trie 19.356 [19.126,21.673] |
| 32 | 64 | random | 1 | band 2.397 [2.394,2.411] | band 1.664 [1.647,1.708] |
| 32 | 65 | random | 1 | band 2.410 [2.405,2.570] | band 1.658 [1.644,1.792] |

Ranges are the min/max of eight process samples, not confidence intervals.

| host | row clear | band clear | Myers clear | trie clear | unresolved | A/A ratio range |
|---|---:|---:|---:|---:|---:|---|
| arm | 0 | 4 | 7 | 5 | 1 | 0.984..1.018 |
| x86 | 0 | 5 | 6 | 5 | 1 | 0.996..1.016 |

## Build and reuse

For N=4096,m=48,prefix,k=1:
- arm: Myers 804.989 us, trie 297.803 us, build 3587.863 us. Crossover projection Q>7.074, or eight queries using medians. The trie has 39644 nodes and evaluates 864 edges per lookup.
- x86: Myers 705.958 us, trie 204.379 us, build 3961.785 us. Crossover projection Q>7.899, or eight queries using medians. The trie has 39644 nodes and evaluates 864 edges per lookup.

Construction and dictionary generation are outside lookup timing. Whole-process elapsed and build samples are retained externally. The crossover uses medians and assumes this unchanged dictionary/query distribution; it is an arithmetic projection rather than a separately timed batch experiment. Trie destruction is outside both lookup and build endpoints.

Dense workloads deliberately include many duplicate terms. This measures complete ID output with heavy prefix sharing; it is not a corpus of distinct natural-language words. At k=4, prefix traversal expands enough states for Myers to beat trie on both hosts. The 65-byte Myers label dispatches to full rows.

## Environment and replay

- arm: dev-dsk-ahrav-2b-7dc7bd93.us-west-2.amazon.com, aarch64; rustc 1.98.1 (48a229cea 2026-09-01); CPU 0 from 64 available CPUs. Full uname, lscpu, target-feature cfg, flags and binary digest are in the archive.
- x86: dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com, x86_64; rustc 1.99.0 (b940084d7 2026-09-28); CPU 0 from 192 available CPUs. Full uname, lscpu, target-feature cfg, flags and binary digest are in the archive.

Build flags: `--edition=2024 -O -C target-cpu=native`. Inputs, schedule, repetitions, correctness boundary and selection rule are frozen in `run.py` and BENCHMARK.md. Run the exact runner on Linux from this topic, keeping generated output outside Git.

At the frozen commit, `scan` built the Myers query table before every row and band scan and matched the method string once per term. The current source prepares each method's own state before the loop. A paired x86 check of the two builds (default `cargo bench` profile, CPU-pinned, 20000 lookups per process) measured that as about 0.1 us per lookup: 5% to 7% at N=32 and under 0.2% at N=4096. Row and band gain from it in the cells where they were already selected, and in the paired check Myers still led band by about 60% at N=32, m=8, so a replay with the current source is expected to keep the recorded selections and lower the N=32 row and band medians.

Linked binary disassembly is retained for inspection. Findings here use executed correctness checks, measured times and structural edge/node counts. No branch/cache/ISA cause is asserted. Rust versions differ between hosts; architecture alone cannot explain a timing difference. Cold-cache behavior, production Unicode corpora, compressed FSTs, dictionary updates, concurrency and service latency remain unmeasured.

Raw archives and their verified SHA-256 hashes are in EVIDENCE_RECEIPT.json. Preflight evidence, three executed mutant kills, all local gate logs, portable replay and the inspected visual are retained with the daily outputs. An SSH proxy emitted EPIPE after printing the x86 campaign summary; the complete 612 records, 16 A/A records, exit-zero runner output and independently retrieved verified archive establish the completed campaign.

Arm CPU identification from /proc/cpuinfo and MIDR is retained externally in arm-cpu-model.txt.
