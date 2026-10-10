benchmark_required: true

# Frozen metadata-filter experiment

Compare unfiltered, scalar-tag and exact-word masks on one owned fixed layout. This is
lookup filtering selection, not a standard-map, hasher or production SwissTable comparison.

Sample unit: one candidate/workload process. Twelve blocks run all six candidate orders twice;
each candidate pair has six forward and six reverse positions. Workload order rotates by block.
Stop after360 valid process observations. Reject any failed oracle before selecting timings.
Each process's repeated batches improve clock resolution, not sample count. Warmup: two batches.

Selection: lowest median only if every paired rival/fastest time ratio exceeds1.05 in all12
blocks. Otherwise unresolved, even if medians differ. Report observed process min/median/max,
not a confidence interval. No A/A calibration is available; no reruns to manufacture winners.

| Workload | Capacity/live | Hits | Comparison and boundary |
|---|---:|---:|---|
| tiny | 16/8 | 50% | last word,32 batches |
| small_hit | 8192/4096 | 100% | last word,32 batches |
| small_miss | 8192/4096 | 0% | last word,32 batches |
| wide_miss | 8192/6144 | 0% |64-byte keys,32 batches |
| large87 | 131072/114688 | 0% |64-byte keys,32 batches |
| bad_tag | 8192/6144 | 0% | every tag zero,32 batches |
| cluster | 512/256 | 0% | all home groups zero,1024 queries,32 batches |
| deleted | 8192/4096 | 0% | insert6144 then delete every third,32 batches |
| build_probe | 8192/6144 | 50% | allocation/build/probe/drop,4 batches |
| first_batch | 8192/6144 | 50% | one batch after the oracle and diagnostics passes; no warmup batches |

Other cells use8192 prehashed queries. Approximate requested50% follows the fixed i%100 schedule.
All keys retain64-byte storage; narrow/wide changes equality width, not object size. Wide keys
have a common56-byte prefix. Trusted deterministic synthetic hashing has no security guarantee.

Steady lookup excludes prehash, queries, table preparation, independent BTreeMap oracle,
startup and final teardown; includes checksum. Build includes allocation, initialization,
insertion, probing and drop. First batch is neither cold cache nor startup: the oracle pass and the
untimed diagnostics pass each ran every query through the selected filter before its timer started.
The groups/equalities counters are incremented inside every `get`, timed or not; the diagnostics
pass reads them untimed. The unfiltered arm builds a lane mask through the same two-phase structure
as the filters; a direct one-pass loop is faster (see measurements/README.md). The runner retains library
assembly; no hardware counters identify cache causes or whole-process candidate-only effects.

Flags: rustc --edition2024 -C opt-level=3 -C target-cpu=native. The runner records rustc-vV,
CPU/kernel/features, logical and available CPUs, binary/source hashes and process wall time.
Linux pins first available CPU; macOS has no supported affinity control. Compiler and host
changes prevent ISA attribution. Required Linux hosts currently reject expired Midway auth;
retain exact-source replay assets and publish this gap rather than inventing measurements.
