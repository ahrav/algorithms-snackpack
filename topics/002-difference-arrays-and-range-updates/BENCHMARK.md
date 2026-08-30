benchmark_required: true

# Frozen benchmark design

The lesson compares runtime, nominal allocation slots, selected logical work,
sorting work, and workload crossovers. Timing and code-shape evidence can answer
those claims, so a benchmark is required.

## Claim and population

For each declared workload cell, compare each serious candidate with
`apply_in_place_difference`. The outcome ratio is candidate divided by
in-place elapsed time. Smaller ratios favor the candidate.

The target population is repeated warm calls to the exact linked image on the
recorded host, CPU affinity, toolchain, environment, and collection window.
The experiment does not generalize to another machine, compiler, allocator,
input distribution, or online operation mix.

Every candidate has the same public contract. Before timing, the native harness
checks one candidate result against a bench-local eager canary that shares no
candidate helper. Timed results are black-box consumed after the clock stops.

## Candidate contrasts

Arm A is `apply_in_place_difference`. Arm B is one of:

1. `apply_reference`
2. `apply_dense_sidecar`
3. `apply_sorted_events`

Crossing three candidate pairs with six cells creates 18 primary contrasts.

| Cell | `n` | `m` | Shape | Timed calls per position |
|---|---:|---:|---|---:|
| `tiny_point_sorted` | 64 | 8 | `pattern=point`, `max_span=1`, `order=sorted` | 101 |
| `small_full_repeated` | 4,096 | 256 | `pattern=full`, `max_span=4096`, `order=reverse` | 31 |
| `cache_short_shuffled` | 65,536 | 2,048 | `pattern=uniform`, `max_span=32`, `order=shuffled` | 7 |
| `cache_clustered` | 65,536 | 2,048 | `pattern=clustered`, `max_span=1024`, `order=shuffled`; endpoint reuse in a hot 1,024-index region | 7 |
| `large_sparse_sorted` | 1,048,576 | 8 | `pattern=point`, `max_span=1`, `order=sorted` | 3 |
| `large_wide_shuffled` | 262,144 | 64 | `pattern=uniform`, `max_span=131072`, `order=shuffled` | 3 |

Every cell uses one warmup and `inner=1`. The fixture seed and assignment seed
are both `2026083002`. Fixture generation
uses deterministic mixed base values and positive and negative deltas. The
runner records the exact generator version and fixture hashes. The matrix varies
size, update count, range width, ordering, clustering, repeated endpoints, and
the ratio of event storage to dense storage. Duplicates and hit rates do not
apply to this contract. The fixed cells do not cover every skew, mutation ratio,
or memory budget.

## Work counted before timing

Each accepted native record reports these per-call counts:

```text
validation_checks
base_reads
base_writes
range_element_updates
boundary_updates
difference_steps
scan_steps
event_records
sort_comparisons_count_pass
vec_constructions
allocated_scalar_slots
```

Counts are elementary derivations from the executed fixture and candidate.
They are not hardware-counter observations. The runner independently
recomputes the fixed-size, range-width, boundary, event, construction, and
allocation-slot counts. `sort_comparisons_count_pass` comes from a separate
untimed Rust `sort_unstable_by` count pass. The runner checks its type and
stability for an identical fixture, but it does not reproduce Rust's exact sort
comparison sequence independently. The runner rejects every independently
checked mismatch and every count change for an identical candidate and fixture.
These fields do not count every logical byte touch, allocator metadata write,
or sort move. The experiment therefore makes no measured memory-traffic or
bandwidth-mechanism claim.

## Timing boundary and state

One timed sample wraps exactly one public candidate call. The sample includes:

- full-batch validation;
- result and temporary allocation;
- update or event processing;
- sorting when the candidate sorts;
- the full output scan and materialization.

Fixture construction, the correctness canary, canary-result hashing, and result
destruction stay outside the timer. Each process performs one untimed warmup,
then the fixed number of timed calls for its cell. The benchmark consumes every
timed result outside the timer so the compiler cannot remove the call. It
records the independently checked canary output hash rather than rehashing each
timed result.

The arrival model is not applicable. This is a local sequential operation
microbenchmark with no request queue. The primary period outcome is the median
elapsed nanoseconds across its fixed timed calls. The complete distribution of
call times remains in raw evidence. This outcome describes a typical warm call;
it is not a throughput estimate.

Cold construction and first-call costs are outside the primary estimand. The
single warmup defines the planned warm state. The raw time series must show
whether that warm state was credible. If it does not, the warm-state claim is
invalid and the result is inconclusive.

## Unit roles and interference

- Treatment-application unit: one fresh native process position running one
  arm for one workload cell.
- Randomization unit: one four-position block assigned `ABBA` or `BAAB`.
- Analysis unit: one complete four-position block contrast.
- Subsample unit: one timed candidate call within a process position.
- Sampling and generalization unit: the recorded linked image on one host and
  collection window. Repetition on that host does not estimate host or build
  variation.
- Interference boundary: allocator, caches, thermal state, kernel activity,
  frequency control, and background load can persist across positions.

Every position starts a fresh process and reconstructs the deterministic
fixture. The pure candidate receives immutable input and update slices, and its
result is destroyed after the timed call. These resets remove candidate state.
They do not reset shared host state.

`ABBA` and `BAAB` are valid here only under the declared nuisance model:
position drift is additive and linear within a block, and cross-position
carryover is negligible after the fresh-process reset. The time series and
profile must test those assumptions. Nonlinear drift, treatment-by-position
effects, or carryover makes the affected comparison inconclusive.

## Assignment, pilot, and fixed stopping

For each primary contrast:

- run four pilot blocks, with two `ABBA` and two `BAAB` templates shuffled by
  seed `2026083002`;
- run exactly 12 main block IDs, with six `ABBA` and six `BAAB` templates
  shuffled by the same recorded restricted-randomization procedure;
- inspect and stop only at a complete block boundary;
- do not use pilot blocks in the main estimate;
- do not add blocks after inspecting a point estimate or interval.

For period `p`, let `t_p` be the median call time. A block contrast is
the mean of `log(t_B)` positions minus the mean of `log(t_A)` positions. The
primary estimate is `exp(mean(block contrasts))`, a candidate-to-in-place ratio.

The four pilot contrasts estimate block-level variance. Before main results are
opened, report prospective 12-block interval half-widths under `0.5x`, `1x`,
`1.5x`, and `2x` the pilot standard deviation. Twelve main blocks remain fixed
even when the pilot predicts low power. A wide main interval yields an
inconclusive result.

The runner uses a 120-second timeout for each native attempt. It schedules
exactly the 12 declared main block IDs. A timeout, nonzero exit, parse failure,
hash mismatch, count mismatch, reset failure, or missing period makes the block
incomplete and stops that phase. The runner does not replace or silently omit
the block. Only an externally recorded `SIGINT`, `SIGTERM`, or `SIGHUP` permits
one retry of the entire same block ID with `retry_index = 1`. The original
partial attempt remains in raw evidence. Timing inference uses only a fully
complete fixed phase. Reliability counts include every attempt.

A raw `Instant` sample may equal 0 ns when the call falls below clock
resolution. The runner retains and counts those samples. It accepts the process
position only when the fixed sample array has the declared length, every value
is a nonnegative integer, and the position median is strictly positive. A zero
median cannot enter the log-ratio analysis and invalidates the block.

## Effect boundary, intervals, and multiplicity

The practical boundary is a multiplicative factor of `1.05` on the
candidate-to-in-place ratio:

- an upper simultaneous confidence bound below `1 / 1.05` supports that B is
  at least a factor of `1.05` faster in the declared cell;
- a lower simultaneous confidence bound above `1.05` supports that B is at
  least a factor of `1.05` slower in the declared cell;
- every other interval justifies no practically meaningful winner.

The primary family contains all 18 candidate-by-cell contrasts. Construct a
two-sided paired-`t` interval on block log contrasts for each member. Use family
wise error rate `0.05` and Bonferroni per-contrast alpha `0.05 / 18`. Exponentiate
the bounds. This construction assumes independent, approximately normal block
contrasts. Report the raw block contrasts, residual time order, and a
distribution-free sensitivity interval for the median block log contrast. With
12 independent blocks, the exact central order-statistic interval runs from the
third through the tenth sorted contrast. Under independent continuous draws,
its coverage is `1 - 2 * P(Binomial(12, 0.5) <= 2) = 0.96142578125`.
Exponentiate both endpoints to report candidate-to-in-place ratios. The
sensitivity result is diagnostic because 12 blocks give a coarse interval. No
exploratory metric enters the primary family or changes the stopping rule.

## Identical-artifact A/A

Run 12 complete blocks on `cache_short_shuffled`. Both arms execute the exact
`apply_in_place_difference` artifact, but they retain distinct A and B labels.
Use six `ABBA` and six `BAAB` templates through the same process launch, warmup,
timing, hashing, parsing, missing-data, stopping, and analysis paths.

Report mechanical integrity separately from null calibration:

- Mechanical integrity requires identical source, binary, linked-image hash,
  settings, fixture hash, work counts, complete schedules, and parse paths.
- Null calibration reports the predeclared A/A block log-ratio estimate and its
  Bonferroni-style interval as a diagnostic. One A/A campaign can expose label
  asymmetry. It cannot prove no bias or define a universal noise floor.

An A/A mechanical failure blocks every performance claim.

## Profile and linked code

Profile the optimized measured image for all four candidates on
`cache_short_shuffled`. Collect, when the host supports them:

```text
cycles
instructions
branches
branch-misses
cache-references
cache-misses
page-faults
context-switches
```

Record counter support, multiplexing, enabled time, running time, event scaling,
and process exit status where the selected tool exposes those fields. On macOS,
where the portable runner cannot request those hardware events, retain
`/usr/bin/time -l` resource totals and state that hardware counters are
unavailable. Retain optimized linked assembly for all four candidate functions
and confirm that the timed call and result consumer remain in the linked image.
The profile phase is incomplete if a dynamic collection fails or linked
disassembly fails. An explicitly unavailable dynamic tool is allowed only when
the linked disassembly succeeds, and it leaves the corresponding mechanism
inferred.

One narrow macOS failure is classified as `UNAVAILABLE`, not `FAILED`.
`/usr/bin/time -l` may execute the target successfully and then return 1 because
the host denies `sysctl kern.clockrate`. The classifier requires Darwin,
`time-l`, no timeout, an unchanged executable hash, exactly one schema-v1 target
record that matches the preceding valid canary in every candidate, position,
fixture, output, and work field, and stderr containing only one well-formed time
summary plus the exact `time: sysctl kern.clockrate: Operation not permitted`
line. The raw return code and output remain retained. Any extra text, target
mismatch, binary change, other platform, timeout, or different denial remains
`FAILED` and blocks the phase.

Elapsed time is measured evidence. Work counts are elementary derivations.
Assembly is observed code shape. Any explanation involving cache traffic,
branch prediction, allocator behavior, or instruction cost remains inferred
unless the corresponding counter or profile supports it.

## Artifact and raw-evidence rules

Run the orchestrator with an absolute, nonexistent output directory outside the
repository:

```bash
python3 scripts/run_experiment.py quick --output-dir /absolute/outside/repo/new-dir
python3 scripts/run_experiment.py pilot --output-dir /absolute/outside/repo/new-dir
python3 scripts/run_experiment.py main --output-dir /absolute/outside/repo/new-dir
python3 scripts/run_experiment.py aa --output-dir /absolute/outside/repo/new-dir
python3 scripts/run_experiment.py profile --output-dir /absolute/outside/repo/new-dir
python3 scripts/run_experiment.py all --output-dir /absolute/outside/repo/new-dir
```

Before collection, record the exact source commit, `rustc -Vv`, Cargo version,
compiler flags, profile, binary SHA-256, linked-image hash, host, architecture,
CPU model, affinity, governor or frequency policy, kernel, allocator,
environment, run window, and fixture hashes.

On Darwin, the runner obtains the exact CPU model from
`/usr/sbin/system_profiler SPHardwareDataType -json`. It parses the output in
memory and retains only `chip_type`, `machine_model`, `machine_name`,
`model_number`, `number_processors`, and `physical_memory`, plus the command,
status, return code, and a sanitized failure reason. It never retains raw
`system_profiler` output, serial numbers, UUIDs, or device identifiers. A
missing or generic chip name makes host identity incomplete.

Retain every complete, partial, failed, timed-out, interrupted, and reset-failed
attempt outside Git. Package the raw run directory into the required external
topic archive and verify that archive before deleting its source. Git receives
only the compact aggregate, review, and evidence receipt. It never receives
per-attempt output, run bundles, executables, profiler dumps, or the raw archive.

The eventual result applies only to the recorded artifact, workload, host,
compiler, assignment, and run window. It cannot establish universal optimality.
