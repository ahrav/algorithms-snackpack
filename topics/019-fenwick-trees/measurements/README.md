# Measured selection

Exact source: `88b544a35a9535b0289dc3ba1bd97fdc6f10ff00`. The receipt hashes name `src/lib.rs`, `benches/portfolio.rs` and `runner.py` at that commit. Both hosts completed 256 fresh processes, four candidates and eight paired blocks per workload. Every query passed the eager-array oracle; the four unit-test groups passed inside the runner, and `tests/contracts.rs` passed through `cargo test` at the same commit.

| Cell | Arm winner, median µs [min,max] | x86 winner, median µs [min,max] |
|---|---|---|
| tiny | scan: 316.50 [313.65, 319.51] | unresolved (median-best scan): 363.59 [360.33, 371.74] |
| read_only | prefix: 9.92 [9.89, 9.98] | prefix: 7.47 [7.45, 7.59] |
| mixed | fenwick: 187.30 [185.93, 189.83] | fenwick: 187.59 [187.16, 188.53] |
| update_heavy | scan: 94.36 [94.07, 94.44] | scan: 82.01 [57.26, 85.78] |
| large | fenwick: 76.72 [74.56, 80.78] | fenwick: 61.63 [61.23, 62.40] |
| short | scan: 57.63 [56.26, 60.62] | scan: 27.79 [27.19, 36.64] |
| hot | scan: 24.33 [24.09, 24.69] | scan: 27.41 [26.91, 27.52] |
| lifecycle | scan: 4.02 [3.94, 4.05] | scan: 3.71 [2.98, 3.83] |

A winner beats every competitor by >5% in every paired block, with the lowest median. Full competitor medians/ranges and paired ratios are in [summary.json](summary.json). These ranges describe eight observed processes, not a population interval; the protocol declares no error bound, see [BENCHMARK.md](../BENCHMARK.md).

Initial scratch campaigns also completed 256 processes per host. Their selections agree except tiny x86 was unresolved. The first committed campaign also resolved tiny x86. The release campaign explicitly drops both validation objects before warmup, removing their incidental live allocations; its tiny x86 result is unresolved. Workloads, order, stopping and selection rule did not change. Both initial and committed raw campaigns are retained.

The committed campaign predates the identical-artifact A/A control, the linked-image hash, the `-O` cfg query and the integration-test step now in `runner.py`. Its winner labels are therefore provisional until a campaign with the control runs through this pipeline. The recorded `target_cfg` came from an unoptimized `rustc --print cfg`, so its `debug_assertions` line describes the query, not the `-O` artifact. The Arm `cpu_model` is empty because `lscpu` on that Graviton host prints no `Model name` line; the archived manifest holds the full `lscpu` output. The receipt records no linked-image hash, archive size or member count. In the committed campaign the per-process oracle was `Scan`, so the scan arm's own check was self-referential and its correctness there rests on the exhaustive unit tests; the bench now uses a leaf array local to the file. Source changes after the campaign are the bench argument guard, the leaf-array oracle and the post-timing checksum assertion in `benches/portfolio.rs`, a rustdoc section and comparator tests in `src/lib.rs`, and the runner evidence changes above; the timed loops are unchanged.

The [receipt](EVIDENCE_RECEIPT.json) records input hashes, archive hashes, host, CPU model, kernel, toolchain, target cfg and available affinity. Raw bundles, tests, every attempt and linked disassembly remain outside Git under the receipt root.

The default target is compiled with rustc --edition 2024 -O. Arm and x86 toolchains differ. CPU scheduling is unpinned; setup/oracle/warmup can touch memory. No cold-cache, hardware-counter, architecture-only, tail-latency or universal-optimality claim is made. Selection and range-add timing remain unmeasured. The short/hot cells change both length and locality, so their difference does not isolate skew.

Static linked disassembly contains the Fenwick constructor and instantiated candidates. Index arithmetic and bucket loops are visible, but no dynamic counter identifies their share of batch cost. Operation-count arguments are algorithmic derivations.

Selection: scan for narrow/update-heavy/brief lifecycle cells and tiny Arm; prefix for read-only; Fenwick for mixed and broad large ranges; tiny x86 unresolved. Segment is a correct comparator and can still be required by richer aggregate semantics.
