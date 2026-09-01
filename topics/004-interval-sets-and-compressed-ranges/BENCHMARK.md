benchmark_required: true

# Frozen benchmark design

Runtime, construction cost, membership cost, and workload crossover are part of
this lesson's selection problem. Tests can establish semantic agreement. They
cannot choose the fastest representation for a declared workload. A benchmark
is required.

## Status

This design is frozen before decision-grade collection. A quick run is only a
harness check. Pilot results can size uncertainty but cannot enter the main
estimate. Profiles, work-count passes, and unplanned contrasts are descriptive.
They cannot change the primary family or stopping rule.

## Claim and target population

Arm A is the wide-endpoint flat sort-and-merge implementation, `flat`. Arm B is
one of:

1. `oracle`, the deliberately simple independent point reference;
2. `packed`, the packed endpoint representation;
3. `events`, the boundary-event sweep;
4. `btree`, the incrementally merged tree representation.

Each candidate is compared with `flat` in every workload cell and operation
phase. The ratio is `candidate / flat`. A ratio below one favors the candidate.

The target population is repeated post-one-warmup execution of the exact linked
`interval_sets` image on the recorded host, operating-system image, toolchain,
compiler flags, environment, affinity policy, and run window. This population
does not include another build, host, architecture, compiler, allocator,
workload distribution, mutation pattern, or memory budget. No result supports a
universal winner.

## Semantic equality and correctness canaries

All candidates implement the same half-open interval-set contract. Two results
are equal when they have the same canonical intervals, cardinality, and
membership answers for the fixed query stream.

Before timing, the native harness builds an independent `PointOracle`. It
requires the candidate to match the oracle's canonical intervals, cardinality,
and every membership answer. The harness also verifies that the packed form can
encode and decode the exclusive endpoint `2^32` without truncation. Every valid
record must contain:

```text
oracle_match      = true
membership_match  = true
domain_end        = 4294967296
packed_max_decode = 4294967296
```

The runner requires one JSON record, exact echo fields, exact schema fields,
valid hashes, a positive position median, stable image bytes, stable fixture
identity, equal outputs across candidates, and the fixed canaries. Exit status
zero alone does not validate an attempt.

## Frozen workload cells

The root fixture and assignment seed is `2026090104`. The fixture generator is
`topic004-fixture-v1`. Each block gets a deterministic derived seed. All four
positions in one block use byte-identical inputs and queries. The seed,
generator version, runner hash, and fixture hash together identify a fixture.

| Cell | Input records | Unique records | Exact duplicates | Input order | Canonical output runs | Membership queries |
|---|---:|---:|---:|---|---:|---:|
| `tiny_sparse_sorted_unique` | 8 | 8 | 0 | sorted | 8 | 64 |
| `cache_clustered_shuffled_duplicates` | 320 | 256 | 64 | seed-shuffled | 8 | 1,024 |
| `large_sparse_reverse_unique` | 4,096 | 4,096 | 0 | reverse | 4,096 | 4,096 |
| `large_adjacent_shuffled_duplicates` | 4,608 | 4,096 | 512 | seed-shuffled | 1 | 4,096 |

The matrix covers a tiny sorted case, clustered overlaps, exact duplicates,
reverse order, many surviving runs, and full coalescing into one run. Query
streams contain an equal count of deterministic hits and misses. The matrix
varies size, order, duplicate rate, overlap shape, and output run count. It does
not cover every span distribution, skew, hit rate, update ratio, universe,
cardinality, or resident-memory limit.

## Operation phases and timing boundary

Each schedule position launches a fresh process. The process builds and checks
the fixture and correctness canaries before timing. It then performs one
untimed warmup and exactly three timed samples. The position outcome is the
median elapsed nanoseconds across those samples.

`build` times one candidate construction, a cardinality read and checksum,
result consumption, and destruction. `build_membership` times the same work
plus every fixed membership query and its checksum. The default `inner` is one.
The profile path raises `inner` to 64 to create a sampleable target.

Fixture generation, the independent oracle canary, output hashing, untimed work
count passes, JSON formatting, process startup, and the dynamic loader are
outside the timer. Candidate allocation, normalization, sorting, merging,
membership work, and destruction are inside when the candidate performs them.

This is a local sequential microbenchmark. No request arrival process, queue,
retry, timeout, or service concurrency belongs to the estimand. The outcome is
not end-to-end latency, throughput, or capacity. Cold process startup and
long-duration thermal steady state are out of scope.

The one-warmup regime is part of the claim. Retained sample order must be
checked for warmup failure or drift. The harness rejects an individual zero-time
sample. A timeout or other invalid sample makes the block incomplete.

## Logical work recorded outside timing

Each record contains these nonnegative integer fields for one emitted sample.
Every count is scaled by `inner`. Warmups and the number of timed samples do not
multiply the count.

```text
input_intervals
unique_input_intervals
duplicate_intervals
sort_comparisons_count_pass
merge_comparisons
output_runs
membership_queries
canonical_binary_search_comparisons
result_scalar_slots
```

The runner independently checks input, unique, duplicate, output-run, and query
counts against the frozen cell. The sort and merge fields come from separate
untimed logical count passes. They do not prove the exact comparisons,
branches, moves, or instructions executed by the timed candidate.

`canonical_binary_search_comparisons` counts a separate binary search over the
candidate's canonical semantic projection. It is a representation-neutral
comparison proxy. It is not the actual lookup path for `BTreeIntervalSet` or
any other candidate. `result_scalar_slots` counts logical fields retained in
the result payload. It excludes vector capacity, tree nodes, transient events,
allocator metadata, and allocation calls. No field measures actual bytes
touched or hardware traffic.

## Primary family

Four candidate-to-flat contrasts cross four cells and two operation phases:

```text
4 candidates x 4 cells x 2 phases = 32 primary contrasts
```

The independent oracle stays in the family because construction cost is part
of the lesson. Its role as a correctness oracle does not exempt its measured
cost from the same schedule, parser, failure, and interval rules.

## Unit roles and interference

- Treatment-application unit: one fresh native process position running one
  candidate, cell, and operation phase.
- Randomization unit: one four-position block assigned `ABBA` or `BAAB`.
- Analysis unit: one complete four-position block log contrast.
- Subsample unit: one timed batch within a process position. The three timed
  batches are not independent units.
- Sampling and generalization unit: the exact linked image, host, and run
  window. Repeated processes do not create host or build replication.
- Interference boundary: allocator state, caches, thermals, frequency control,
  scheduler activity, kernel state, and memory-controller state can persist
  across fresh processes.

Each letter reconstructs an immutable fixture and candidate object in a fresh
process. That removes candidate object carryover. It does not reset shared host
state. `ABBA` and `BAAB` are justified only under the declared nuisance model:
position drift is additive and linear on the log-time scale, and cross-position
carryover is negligible. Nonlinear drift, treatment-by-position interaction,
serial dependence, or thermal hysteresis can make the affected comparison
inconclusive.

For positive position outcome `t`, the block contrast is:

```text
ABBA: d = ((log t_B2 + log t_B3) - (log t_A1 + log t_A4)) / 2
BAAB: d = ((log t_B1 + log t_B4) - (log t_A2 + log t_A3)) / 2
```

Both forms cancel an additive linear position term under the declared model.
`exp(mean(d))` is the geometric mean candidate-to-flat ratio. It is not a
ratio of arithmetic means.

## Assignment, pilot, and stopping

For every primary contrast:

- run four pilot blocks, exactly two `ABBA` and two `BAAB`, in a seed-shuffled
  restricted order;
- exclude every pilot block from the main estimate;
- report the prospective 12-block simultaneous interval half-width under
  `0.5x`, `1x`, `1.5x`, and `2x` the pilot standard deviation;
- run exactly 12 main block IDs, six `ABBA` and six `BAAB`, in a separately
  seed-shuffled restricted order;
- inspect and stop only after complete blocks;
- never add blocks after inspecting a timing, estimate, or interval.

The main horizon remains 12 blocks even when pilot variance predicts weak
precision for a 5% boundary. A wide interval is inconclusive. Inner iterations,
timed samples, and fresh processes do not replace missing complete blocks.

The schedule is derived with a recorded seed and a hash of this runner. The
seed alone is not enough to reproduce Python's shuffle across arbitrary Python
implementations or future versions.

## Effect boundary and simultaneous intervals

The symmetric practical factor is `1.05` on the candidate-to-flat time ratio:

- an upper simultaneous bound below `1 / 1.05` supports candidate time at most
  `1 / 1.05` of flat time in the declared cell and phase;
- a lower simultaneous bound above `1.05` supports candidate time at least
  `1.05` times flat time;
- every other interval supports no practically meaningful winner.

Equality with either boundary is inconclusive. This classification describes a
scoped benchmark result. It is not a merge or repository-policy verdict.

For each family member, construct a two-sided paired Student `t` interval over
the 12 block log contrasts, then exponentiate its endpoints. Familywise alpha
is `0.05`. Bonferroni assigns per-contrast alpha:

```text
0.05 / 32 = 0.0015625
```

Each tail receives `0.00078125`. Bonferroni does not require independence
between family members. The paired interval still assumes independent,
identically distributed, approximately normal block log contrasts after
blocking.

Retain block order and report a distribution-free diagnostic for the median
block contrast. With 12 independent continuous contrasts, the interval from
the third through tenth sorted values has coverage:

```text
1 - 2 * P(Binomial(12, 0.5) <= 2) = 0.96142578125
```

That diagnostic is coarse and not simultaneous across 32 contrasts. It cannot
repair dependence, confounding, nonlinear drift, or an invalid schedule.

## Invalid attempts and partial blocks

Before launch, every attempt gets an immutable directory. Retain the full argv,
cwd, complete controlled environment, timestamps, stdout, stderr, return code,
signal or timeout, parser result, canaries, fixture and output hashes, work
record, and pre/post linked-image hashes.

A signal, timeout after 180 seconds, nonzero exit, panic, launch failure, parse
failure, missing field, extra field, hash mismatch, canary failure, count
mismatch, nonpositive position median, source change, image change, or reset
failure marks the attempt invalid. The block is incomplete and cannot enter
analysis or stopping.

There are no automatic retries. A later campaign has a new run identity and
cannot silently replace the failed attempt. The signal relay forwards `SIGINT`,
`SIGTERM`, and `SIGHUP` to the active process group. Timeout handling kills the
whole process group so a profiler or harness descendant cannot survive. Every
partial and invalid record remains in raw evidence.

Before statistical reduction, the runner writes and verifies a checksum
manifest over each phase's raw attempt and block files. The final bundle also
gets a verified checksum manifest.

## Identical-artifact A/A

Run 12 complete blocks on
`cache_clustered_shuffled_duplicates` in the `build_membership` phase. Both
labels execute `flat` from the same linked image. Six blocks use `ABBA` and six
use `BAAB`, in seed-shuffled order.

Mechanical integrity requires one linked-image hash, one candidate path, equal
fixture and output hashes within every block, equal work records, valid
canaries, complete templates, and the same parser and missing-data rules. The
A/A diagnostic reports both the same Bonferroni interval construction used by
the primary path and an unadjusted 95% interval.

One A/A campaign can expose label, path, parser, schedule, or position
asymmetry. It does not calibrate the pipeline's false-positive rate, prove the
absence of bias, establish power, prove production relevance, or define a
noise floor. Null calibration across independent end-to-end campaigns remains
`NOT_CALIBRATED`.

## Static and dynamic profile evidence

Static evidence is mandatory. The runner retains optimized linked symbols and
disassembly for all five candidate workload functions, the timed loop, result
consumer, and clock boundary. It hashes the image before and after inspection.
Symbol retention and disassembly are observed code shape. They do not prove the
dynamic path or causal mechanism by themselves.

The dynamic profile phase uses these representative targets:

| Candidate | Cell | Phase | Reason |
|---|---|---|---|
| `oracle` | `tiny_sparse_sorted_unique` | `build` | bounded point-oracle construction |
| `flat` | `large_sparse_reverse_unique` | `build` | wide sort-and-merge baseline |
| `packed` | `large_sparse_reverse_unique` | `build_membership` | packed construction and lookup |
| `events` | `cache_clustered_shuffled_duplicates` | `build` | duplicated clustered boundary sweep |
| `btree` | `large_adjacent_shuffled_duplicates` | `build_membership` | incremental merge and tree lookup |

Each dynamic target uses `inner=64`. On Linux the runner attempts `perf record`.
On macOS it attempts an `xctrace` Time Profiler capture. A permission denial,
unsupported tool, missing artifact, failed target canary, or changed image is
retained as `ATTEMPTED_UNAVAILABLE`. It is not empty profile evidence.

The profiler wraps the whole harness. Samples may include process startup,
fixture generation, correctness canaries, warmup, timed work, untimed count
passes, and output formatting. Dynamic evidence can show where the recorded
process spent samples. It does not provide per-operation counters or isolate
the timed region. Elapsed time alone cannot establish cache, branch,
instruction, allocation, or bandwidth mechanisms.

## Exact artifact and evidence boundary

The runner builds with Cargo JSON into a new empty target directory. It accepts
exactly one compiler artifact that matches the package manifest, bench target,
bench kind, optimized profile fields, and absolute executable path. It rejects
zero or multiple matches, malformed Cargo JSON, an ambiguous target runner,
non-native target configuration, image changes, and source changes.

The retained artifact record includes Cargo and rustc versions, the Cargo
artifact message, source and runner hashes, Cargo configuration hashes,
compiler flags, target, host identity, environment, and linked-image SHA-256.
`--locked` pins dependency resolution to `Cargo.lock`; it does not pin the
compiler, configuration, flags, native libraries, environment, or machine.

Collection requires an absolute, nonexistent output directory outside the
repository:

```bash
python3 scripts/run_experiment.py plan
python3 scripts/run_experiment.py self-check
python3 scripts/run_experiment.py quick --output-dir /absolute/new/quick
python3 scripts/run_experiment.py pilot --output-dir /absolute/new/pilot
python3 scripts/run_experiment.py main --output-dir /absolute/new/main
python3 scripts/run_experiment.py aa --output-dir /absolute/new/aa
python3 scripts/run_experiment.py profile --output-dir /absolute/new/profile
python3 scripts/run_experiment.py all --output-dir /absolute/new/all
```

The runner rejects a relative path, an existing path, a missing parent, or a
path inside the repository. Raw output stays outside Git. Preserve every valid,
invalid, timed-out, signaled, partial, and profile record. Package and verify
the complete raw run under the automation evidence directory before removing
its source directory.

Git may contain only the compact aggregate, review, and evidence receipt. Do
not commit attempt directories, stdout or stderr, executables, disassembly,
profile data, run bundles, or the external archive.

Measured elapsed time applies only to the exact artifact, workloads, host,
compiler, schedule, and run window. Logical work fields are count-pass results
or elementary derivations. Assembly is observed static code shape. Dynamic
profiles are observed samples over their full process boundary. All other
mechanism and generalization claims remain inferred.
