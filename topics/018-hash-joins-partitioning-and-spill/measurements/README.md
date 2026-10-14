# Exact committed measurements

Measured source commit: `b2f96008cad45e9dbc632aa318ef1fba4addbdc7`. RESULTS.json `inputs` hashes
match both remote manifests. Review edits after both measured commits (`b2f9600`, `eddb79c`)
touch only the untimed spill path:
`src/lib.rs` gains an empty-probe-child guard in `Spill::join`, and `tests/contracts.rs` gains
exact spill counter and rejection assertions. The timed resident candidates are unchanged.
Raw evidence: `/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-018/20261014`.

| Case | Arm choice; median [min,max] microseconds | x86 choice; median [min,max] microseconds |
|---|---|---|
| disjoint | merge: 146.534 [144.339,148.727] | merge: 126.866 [125.204,129.563] |
| duplicates | merge: 61.215 [60.881,61.519] | merge: 49.156 [48.855,50.060] |
| heavy | merge: 11.867 [11.844,12.289] | merge: 10.058 [10.005,10.483] |
| large | merge: 3446.495 [3437.140,3478.966] | merge: 3267.829 [3252.858,3274.421] |
| small_build | unresolved; chunk: 1208.936 [1175.508,1279.208]; hash: 1218.140 [1187.484,1237.586] | unresolved; chunk: 1218.635 [1209.016,1242.011]; hash: 1210.850 [1207.060,1239.718] |
| sorted | merge: 25.326 [25.294,25.333] | merge: 21.632 [21.570,22.138] |
| tiny | merge: 0.081 [0.081,0.084] | merge: 0.075 [0.069,0.090] |
| uniform | merge: 163.804 [162.770,165.926] | merge: 141.256 [138.936,145.847] |

Both exact 256-process campaigns pass every independent oracle. Two library groups,
three external file contracts and example pass per host. Initial 256-process campaigns
also pass, with x86 uniform unresolved between hash/merge; retained independently.
Final data is the exact committed campaign, with no repeat to seek a winner.

Arm verified aarch64, 64 CPUs, Linux 6.12.110; model field CPU part retained in raw.
x86 verified x86_64, 192 CPUs, Intel Xeon Platinum 8488C, Linux 6.12.110.
Arm rustc 1.98.1 LLVM 22.1.8; x86 rustc 1.99.0 LLVM 23.1.1; both optimization 3/native features.
Full target cfg and uname are retained per manifest. Schedulers unpinned. See BENCHMARK.md for
sample controls.

Merge selected tiny/uniform/disjoint/duplicates/heavy/large/sorted on both hosts.
Small_build leaves hash/chunk unresolved. Every selection is provisional until the
identical-artifact A/A control required by METHODOLOGY.md runs through this pipeline.
Frequency compression and optimized
standard sorting make this a count-only, fixed-key workload claim. Resident partition
has extra arrays and no byte bound. Chunk has a row cap and can rescan 64 times.

File spill uses actual temporary files. Tests assert exact logical read/write counts for a
leaf-only join, the heavy-key fallback and a disjoint-bucket split. OS cache,
physical media, durable completion, byte admission, full payload output and cancellation
are unmeasured. Assembly is retained; no hardware mechanism claim is inferred.

## Tiny quadratic reference supplement

Exact committed supplement source: `eddb79c0746ec0b68442301256bbc8009e8b7b01`. TINY_REFERENCE.json
`inputs` hashes pin the measured benchmark, runner and library. A later review edit to
`measurements/tiny_reference.py` records timed-out attempts in `attempts.jsonl` before
re-raising; the measured runs completed without a timeout. Both Linux hosts run 16 processes in eight
paired AB/BA blocks, matching the primary tiny 8/12-key data. Independent oracle
checks 5pairs. Each process uses two warmups and 100,000 timed complete joins.
The primary four-candidate campaign is retained separately.

Arm nested 77.535ns [77.361,77.883], merge 81.229ns [80.895,81.446].
x86 nested 74.580ns [73.212,80.450], merge 72.223ns [70.721,73.617].
Both cells remain unresolved under >5% in every paired block. Initial ephemeral
supplements also remain unresolved on both hosts. No winner was sought by repeats.
With the reference included, choose between nested and merge for tiny inputs.
Merge remains selected for uniform/disjoint/duplicates/heavy/large/sorted among
the four main candidates; small_build remains hash/chunk unresolved.

Nested is unmeasured for larger cells. The large cell needs 8,589,934,592 comparisons
per join, so the bounded reference supplement tests its relevant small regime.
This limitation prevents a universal/all-five winner claim on larger inputs.
Cold startup/cache/storage, allocator byte footprint and payload output remain
unmeasured. See TINY_REFERENCE.json for exact medians/ranges/receipts.
