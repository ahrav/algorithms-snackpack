# Frozen experiment

benchmark_required: true

Three deletion policies; each candidate/process checks operation outputs and
final map against independent BTreeMap before timing. No incorrect candidates
are admitted. Deterministic identical workload generation, fixed mixer or
explicit identity-hash cluster, native optimized builds.

Ten workloads: tiny 8 live/32 slots; clean 4096/16384,12288/16384,14336/16384;
read after two full churn rounds at75%; mixed25%/75% delete/insert/hit/miss;
identity-hash256-key cluster/512 slots;8192-key build from 8 slots; first lookup
without warmup. Read batches have65536 queries with half hits/misses (tiny 512).
Mixed batches have 16384 cycles (cluster 4096), four operations each.

Twelve process blocks use all six candidate orders twice. Workload order
rotates per block. Pin the first allowed logical CPU. Two untimed warmup calls
except first_lookup. Read batches repeat four times, tiny128; these repetitions
are not independent samples. Keep all raw samples. Stop after the frozen12
blocks, never rerun for a winner.

Lookup and mixed timing excludes input generation, oracle, prepared table,
process startup and final table destruction. It includes operation result
checksum and mutation maintenance. Build includes allocation, growth, checksum
and drop. First lookup is not a startup or controlled cold-cache measurement.
Oracle/setup and whole-process elapsed are recorded separately. Churn_read
excludes its preparation maintenance, so it cannot decide total churn cost.

Selection: lowest median is descriptive. Name a unique selected option only
if every paired rival/selected ratio exceeds1.05 in all 12 blocks. Otherwise
unresolved, with medians and full observed ranges. No confidence interpretation
is assigned to those ranges; no A/A calibration or tail statistic is claimed.

Rebuild/reinsert/shift counters include candidate maintenance and are explicit
instrumentation overhead. Post-operation lookup probe diagnostics are outside
timing; mixed diagnostics examine final state, not the chronological stream.
Assembly is retained; no candidate-specific PMU/cache/branch attribution.

Run standalone in a fresh path-limited copy:

```bash
python3 measurements/runner.py
```

The runner records hostname, architecture, kernel,CPU,compiler,native cfg/flags,
affinity, frozen source hashes and binary hash. Raw data stays outside Git.
Initial scratch campaign and exact committed replay remain separately retained.
