benchmark_required: true

# Frozen benchmark design

Runtime, construction cost, payload crossover, and candidate selection are
central claims for this topic, so timing can answer part of the lesson's
selection question. Correctness tests alone cannot choose a representation.

## Status

This file freezes the experiment before collection. It contains no measured
result. Quick runs, pilots, reference observations, component timings, work
descriptions, and profiles cannot be promoted into the fixed primary family.

## Claim, outcome, and population

For each of seven immutable workload cells, compare the fixed total service
time of each pair of serious optimized representations. One timed sample
executes the cell's declared construction repetitions, membership probes, and
intersection-cardinality repetitions. Its primary outcome is:

```text
total_ns = build_component_ns
         + contains_component_ns
         + intersection_component_ns
```

`total_ns` is the sole primary outcome. The three component measurements are
retained descriptive diagnostics; they do not form extra hypotheses, change
the winner rule, or enlarge the 21-contrast family. The fixed mix is a
synthetic immutable workload, not latency for one API call or an observed
production arrival mix.

Each schedule position runs one untimed warmup of the complete fixed batch and
then exactly three timed complete-batch samples. The position outcome is the
median of those three `total_ns` values. For a pair named `A_vs_B`, the primary
ratio is `B / A`; a value below one favors `B`.

The target population is repeated post-one-warmup executions of the exact
linked `bitmap_sets` image on the recorded host, operating-system image,
toolchain, compiler flags, affinity policy, environment, and collection
window. One image on one host cannot estimate build-layout, host, architecture,
or fleet variation. No cell establishes a universal representation winner.

## Semantic equivalence and timing boundary

Before any timed sample, the native harness constructs `ReferenceSet` and the
selected optimized type from byte-identical inputs. It requires equal
universes, lengths, full ascending iterators, membership-probe answers, and
intersection cardinality. It also records fixture hashes, `payload_bytes`, and
adaptive container summaries. A failed canary invalidates the process
position.

One build repetition constructs both cell operands with `try_new` and consumes
their lengths and payload reports. The build timer includes allocation,
copying, sorting, duplicate normalization, dense zero-fill, chunking, run
detection, and container construction performed by the candidate. It excludes
fixture generation and semantic validation.

The contains component uses already-built immutable operands and the fixed
probe stream. Probe records name the operand and value; in-universe misses are
used so the out-of-universe fast path does not define the miss workload. The
intersection component repeatedly calls same-type `intersection_len(A, B)`.
Both component results are black-box consumed outside their timers. Query-set
construction, fixture generation, correctness canaries, hashing, output
formatting, and destruction are outside the query timers.

Each fresh process performs one warmup full batch before the three timed full
batches. This measures a narrowly defined post-one-warmup process regime.
Cold startup, first-call service time, mutation, serialization, memory mapping,
materialized results, and long-duration thermal steady state are out of scope.

## Fixed seven-cell protocol

The root protocol seed is `2026083103`. The runner derives seed-recorded
restricted schedules and deterministic per-block fixture seeds from that root.
The checked-in generator fixes construction order, duplicate positions, probe
operands, hit/miss positions, and exact members for each derived seed. Each run
records the generator version and per-block fixture-manifest hash. Percentages
below are protocol targets; each manifest records its exact integer counts,
including rounding for the 90% hit cell and the 5% and 10% duplicate inputs.

| Cell | Universe `U` | Semantic shape | Fixed operations per timed batch |
|---|---:|---|---|
| `tiny_sparse_shuffled` | 1,024 | `nA=nB=24`, overlap 8; every unique input appears twice, then is shuffled | 128 builds, 256 probes at 50% hits, 256 intersections |
| `wide_sparse_low_overlap` | `2^24` | `nA=nB=4,096`, overlap 128; membership spread uniformly across all 256 high-16 chunks; 10% duplicate inputs, shuffled | 4 builds, 4,096 probes at 25% hits, 64 intersections |
| `skewed_sparse` | `2^24` | `nA=64`, `nB=65,536`, overlap 32; members spread uniformly across the wide universe, shuffled | 1 build, 4,096 probes at 50% hits, 64 intersections |
| `dense_local_high_overlap` | 65,536 | `nA=nB=49,152`, overlap 40,960 in one chunk; 5% duplicate inputs, shuffled | 1 build, 4,096 probes at 90% hits, 64 intersections |
| `dense_wide_medium_overlap` | `2^18` | `nA=nB=131,072`, overlap 65,536; locally uniform across four high-16 chunks, shuffled | 1 build, 4,096 probes at 50% hits, 64 intersections |
| `long_runs` | `2^20` | 64 `A` runs of length 512 spaced 16,384 apart; `B` shifts each run by 256; `nA=nB=32,768`, overlap 16,384; construction input shuffled | 2 builds, 4,096 probes at 50% hits, 64 intersections |
| `mixed_chunk_shapes` | `2^20` | fixed 16-chunk generator manifest with sparse arrays, cardinalities 4,095/4,096/4,097, dense alternating lows, long and fragmented runs, keys missing from either side, and every ordered mixed container pair; `nA=nB=193,304` | 1 build, 4,096 probes at 50% hits, 64 intersections |

The runner rejects a fixture whose manifest differs from the frozen universe,
input-record counts, unique cardinalities, overlap, chunk distribution,
duplicate count, probe count, exact hit count, build/intersection repetitions,
or expected adaptive container-pair coverage for its derived seed. Within a
block, all four positions must have identical fixture and expected-result
hashes. In particular,
`mixed_chunk_shapes` must cover array/array, array/bitmap, array/run,
bitmap/array, bitmap/bitmap, bitmap/run, run/array, run/bitmap, run/run, and a
missing key on each side. Its exact generated cardinalities and member hashes,
not the approximate prose description, are the executable protocol.

The matrix varies universe span, global and local density, operand-size skew,
overlap, duplicate construction records, ordering, high-key occupancy,
cardinality thresholds, run length, run fragmentation, probe hit rate, and
operation mix. It does not cover every universe, cardinality, distribution,
mutation ratio, memory budget, or hardware.

## Candidate pairs and primary family

The optimized pairs are fixed, with the first candidate as arm `A` and the
second as arm `B`:

1. `sorted_vs_dense`: `SortedSet` versus `DenseBitSet`
2. `sorted_vs_adaptive`: `SortedSet` versus `AdaptiveBitmap`
3. `dense_vs_adaptive`: `DenseBitSet` versus `AdaptiveBitmap`

Crossing three pairs with seven cells creates exactly 21 primary contrasts.
`ReferenceSet` is the correctness oracle. It also receives four descriptive
attempts per cell under the same fixed operation mix, but it is not a primary
arm, receives no confirmatory interval, and cannot be promoted after results
are inspected.

## Work and representation facts recorded before timing

For each fixture and candidate, the harness records and independently checks:

```text
universe_exclusive
left_input_records, right_input_records
left_unique_cardinality, right_unique_cardinality
expected_intersection_cardinality
duplicate_records
probe_count, probe_hit_count, probe_miss_count
build_repetitions, intersection_repetitions
payload_bytes for both operands
dense logical word count = ceil(U / 64)
adaptive key, kind, cardinality, run_count, and payload bytes
fixture and expected-member hashes
```

These are semantic facts or elementary derivations, not hardware-counter
observations. `payload_bytes` excludes metadata, capacity, tree nodes, and
allocator overhead. Elapsed time alone cannot establish a branch, cache,
allocation, vectorization, or bandwidth mechanism.

## Unit roles and interference

- Treatment-application unit: one fresh benchmark-binary process at one
  schedule position, running one candidate and one cell.
- Randomization unit: one complete four-position block assigned `ABBA` or
  `BAAB`.
- Analysis unit: one complete block log contrast.
- Subsample unit: one of the three timed complete batches within a process
  position. Builds, probes, and intersections inside a batch are lower-level
  repeated operations, not independent observations.
- Sampling and generalization unit: the exact linked image, host, and run
  window. Repeated processes do not create host or build replication.
- Interference boundary: allocator, caches, thermal state, frequency control,
  scheduler activity, and memory-controller state can persist across fresh
  processes.

Every letter starts a fresh process, regenerates and verifies the immutable
fixture, constructs fresh sets, and performs no shared-file mutation. This
removes candidate object carryover but not host-state carryover. `ABBA` and
`BAAB` are justified only under the declared nuisance model of additive linear
position drift with negligible cross-position carryover. Nonlinear drift,
treatment-by-position interaction, thermal hysteresis, or serial dependence
can make a contrast inconclusive.

For positive position outcome `t`, the block contrast is:

```text
ABBA: d = ((log t_B2 + log t_B3) - (log t_A1 + log t_A4)) / 2
BAAB: d = ((log t_B1 + log t_B4) - (log t_A2 + log t_A3)) / 2
```

`exp(mean(d))` is the geometric mean paired `B/A` ratio. It is not a ratio of
arithmetic means.

## Assignment, pilot, and fixed stopping

For every primary contrast:

- run four pilot blocks, exactly two `ABBA` and two `BAAB`, in a restricted
  seed-shuffled order;
- exclude pilot observations from every primary estimate;
- report prospective 12-block interval-width sensitivity at `0.5x`, `1x`,
  `1.5x`, and `2x` the pilot block-log standard deviation;
- run exactly 12 main block IDs, six `ABBA` and six `BAAB`, in a separately
  seed-shuffled restricted order;
- inspect and stop only after complete blocks; never extend the fixed main
  count after inspecting timings or intervals.

The main phase is fixed at 12 blocks even when the pilot predicts inadequate
precision. Low power or a wide interval produces an inconclusive result, not
more blocks. Quick and self-check modes are harness diagnostics and never
enter pilot, main, A/A, or primary evidence.

## Effect boundary, intervals, and multiplicity

For each contrast, construct a two-sided paired Student `t` interval over the
12 complete block log contrasts, then exponentiate its endpoints. The model
assumes independent, identically distributed, approximately normal block
contrasts after the log transform. Retained block order and a
distribution-free order-statistic sensitivity analysis must expose strong
departures; neither repairs dependence or confounding.

The fixed family contains all 21 pair-by-cell total-time contrasts. Familywise
alpha is `0.05`. Bonferroni assigns `0.05 / 21` to each two-sided interval, or
`0.05 / 42` to each tail. There are no interim looks.

The practical boundary is the symmetric multiplicative time factor `1.05`:

- an upper simultaneous bound below `1 / 1.05` supports that arm `B` uses at
  most `1 / 1.05` of arm `A`'s time in that cell, which is about 4.76% less
  time relative to `A`;
- a lower simultaneous bound above `1.05` supports that arm `B` uses at least
  `1.05` times arm `A`'s time in that cell, which is at least 5% more time
  relative to `A`;
- otherwise report the estimate and interval without a practically meaningful
  winner.

Equality with either boundary is inconclusive. The rule is a scoped benchmark
interpretation, not a merge or repository policy. Component times, payloads,
profiles, reference attempts, and unplanned contrasts are descriptive and
cannot change this decision rule.

## Invalid attempts, failures, and retries

Every started attempt receives an immutable directory before process launch.
Retain its exact argv, controlled environment, cwd, timestamps, stdout, stderr,
exit status, timeout or signal, parser status, canary results, fixture hashes,
image hashes, and reset checks.

A timeout after 180 seconds, nonzero exit, panic, parse failure, missing sample,
nonpositive position median, semantic mismatch, count mismatch, fixture/hash
mismatch, artifact mutation, or reset failure marks the attempt `INVALID` and
the block incomplete. Component-dependent failures are retained as reliability
outcomes. The runner never substitutes a successful rerun for a candidate or
workload failure.

There are no automatic retries, including for external interruption. Any
invalid or missing required block makes the campaign `INCOMPLETE`; a later
campaign is a new run with its own identity and cannot silently replace the
failed evidence.
Timing inference uses only the fully complete predeclared phase; reliability
reports every started attempt by candidate, cell, and reason.

## Identical-artifact A/A

Run exactly 12 complete blocks on `mixed_chunk_shapes`. Both labels execute the
same `AdaptiveBitmap` artifact from the same linked image. Six templates are
`ABBA` and six are `BAAB`, seed-shuffled through the same fixture, process
launch, warmup, three-sample timing, parser, missing-data, stopping, and analysis
paths as primary contrasts.

Report two separate results:

1. Mechanical integrity requires identical source and linked-image hashes,
   candidate settings, fixture hashes, work counts, semantic canaries, complete
   schedules, parser versions, and retained output across labels. Label and
   schedule-position metadata are expected to differ.
2. Null diagnostics report all 12 A/A block log contrasts, mean ratio, an
   unadjusted two-sided 95% paired `t` interval, template balance, and
   label-by-position summaries.

Mechanical failure blocks every performance claim. One A/A campaign can expose
label or path asymmetry; it cannot prove absence of bias, adequate power,
production relevance, or a universal noise floor.

## Profile and linked code

The dynamic profile phase has exactly three representative targets, each after
a successful same-image canary:

1. `SortedSet` on `wide_sparse_low_overlap`;
2. `DenseBitSet` on `dense_local_high_overlap`;
3. `AdaptiveBitmap` on `mixed_chunk_shapes`.

Each target uses the exact optimized measured image and repeats its complete
fixed batch 64 times under the selected sampling profiler. The three targets
exercise wide sorted merge, one-chunk dense words, and every adaptive
container-pair kernel. They do not profile the other candidate-cell
combinations, so mechanisms outside those targets remain unobserved.

On Linux, retain `perf record` output when available. On macOS, retain an
`xctrace` Time Profiler trace when available. Record profiler and operating
system versions, exact argv, sampling configuration, duration, return status,
target output, and linked-image hash. Permission-denied, unsupported, or failed
profilers remain retained `UNAVAILABLE` attempts, not empty successful
profiles.

Separately retain whole-image symbols and optimized linked disassembly that
contains all four candidate paths, the timed loop, result consumer, and clock
boundary. Sampling profiles include process startup, fixture work, canaries,
warmup, repeated batches, and output; they do not yield per-call hardware
counter values. Linked assembly is observed code shape, and symbol retention
does not by itself prove call ordering. Cache, branch, allocation, instruction,
or vectorization explanations remain inferred unless an appropriately scoped
profile supports them.

## Artifact and evidence boundary

Before collection record the exact source commit and dirty-diff hash, source
tree and benchmark-source hashes, runner hash, `rustc -vV`, Cargo version,
target triple, compiler flags, profile, executable SHA-256, linked-image hash,
allocator, host, operating system, architecture, CPU model, affinity, frequency
policy, controlled environment, fixture manifests, assignment schedule, and
run window.

Collection commands require an absolute nonexistent output directory outside
the repository:

```bash
python3 topics/003-density-adaptive-bitmaps/scripts/run_experiment.py plan
python3 topics/003-density-adaptive-bitmaps/scripts/run_experiment.py self-check
python3 topics/003-density-adaptive-bitmaps/scripts/run_experiment.py quick \
  --output-dir /absolute/outside/repository/new-directory
python3 topics/003-density-adaptive-bitmaps/scripts/run_experiment.py pilot \
  --output-dir /absolute/outside/repository/new-directory
python3 topics/003-density-adaptive-bitmaps/scripts/run_experiment.py main \
  --output-dir /absolute/outside/repository/new-directory
python3 topics/003-density-adaptive-bitmaps/scripts/run_experiment.py aa \
  --output-dir /absolute/outside/repository/new-directory
python3 topics/003-density-adaptive-bitmaps/scripts/run_experiment.py profile \
  --output-dir /absolute/outside/repository/new-directory
python3 topics/003-density-adaptive-bitmaps/scripts/run_experiment.py all \
  --output-dir /absolute/outside/repository/new-directory
```

Retain every complete, partial, failed, timed-out, interrupted, and reset-failed
attempt outside Git. Package and verify the complete raw run directory in the
automation-owned external archive before removing its source. Git receives
only compact aggregates, review, and the verified evidence receipt. It never
receives per-attempt records, executables, stdout, stderr, profile dumps, raw
bundles, or the archive itself.

The eventual result must separate measured total/component timings and dynamic
counters, elementary work and payload derivations, observed linked code, and
inferred mechanisms. Until the fixed campaign and archive verification finish,
the only supported claims are the semantic tests and the derived cost equations.
