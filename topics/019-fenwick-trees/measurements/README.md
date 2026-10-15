# Measured selection

Exact source: `b306e91aa60d534257aa4606ea6bd31d8691e677`. The three frozen source/runner files match the publication tree. Both hosts completed 256 fresh processes, four candidates and eight paired blocks per workload. Every query passed the eager-array oracle; all four test groups passed.

| Cell | Arm winner, median µs [min,max] | x86 winner, median µs [min,max] |
|---|---|---|
| tiny | scan: 315.37 [312.45, 324.58] | scan: 354.32 [352.90, 361.14] |
| read_only | prefix: 9.93 [9.88, 10.15] | prefix: 7.53 [7.49, 7.64] |
| mixed | fenwick: 186.53 [186.16, 191.21] | fenwick: 187.59 [186.58, 200.17] |
| update_heavy | scan: 94.30 [94.19, 94.69] | scan: 81.44 [58.05, 82.04] |
| large | fenwick: 82.51 [80.93, 85.92] | fenwick: 63.27 [61.60, 65.38] |
| short | scan: 39.93 [38.40, 42.93] | scan: 44.77 [30.80, 46.80] |
| hot | scan: 24.44 [24.03, 24.67] | scan: 27.28 [26.96, 30.08] |
| lifecycle | scan: 4.24 [4.17, 4.33] | scan: 3.72 [3.65, 3.74] |

A winner beats every competitor by >5% in every paired block, with the lowest median. Full competitor medians/ranges and paired ratios are in [summary.json](summary.json). These ranges describe eight observed processes, not a population interval.

Initial scratch campaigns also completed 256 processes per host. Their selections agree except tiny x86 was unresolved. Source changes before the exact campaign add panic documentation and rename benchmark locals/literals; no workload or selection rule changed. Both initial and committed raw campaigns are retained.

The [receipt](EVIDENCE_RECEIPT.json) records input hashes, archive hashes, host, CPU model, kernel, toolchain, target cfg and available affinity. Raw bundles, tests, every attempt and linked disassembly remain outside Git under the receipt root.

The default target is compiled with rustc --edition 2024 -O. Arm and x86 toolchains differ. CPU scheduling is unpinned; setup/oracle/warmup can touch memory. No cold-cache, hardware-counter, architecture-only, tail-latency or universal-optimality claim is made. Selection and range-add timing remain unmeasured. The short/hot cells change both length and locality, so their difference does not isolate skew.

Static linked disassembly contains the Fenwick constructor and instantiated candidates. Index arithmetic and bucket loops are visible, but no dynamic counter identifies their share of batch cost. Operation-count arguments are algorithmic derivations.

Selection: scan for tiny/narrow/update-heavy/brief lifecycle cells; prefix for read-only; Fenwick for mixed and broad large ranges. Segment is a correct comparator and can still be required by richer aggregate semantics.
