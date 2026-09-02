benchmark_required: true

# Frozen benchmark design

Runtime and workload crossover are part of this lesson's selection problem.
Tests can establish exact semantic agreement. They cannot choose the fastest
candidate for a declared workload. A benchmark is required.

## Status

This design is frozen before decision-grade collection. A quick run is only a
harness check. Pilot results size uncertainty but never enter the main
estimate. Profiles, work-count passes, and unplanned contrasts are descriptive.
They cannot change the primary family, effect boundary, or stopping rule.

## Claim and target population

Arm A is `direct`, the direct sliding-window implementation. Arm B is one of
`reference`, `quadratic`, `reset`, or `prefix` in one of the 12 frozen
candidate-workload contrasts below. The ratio is `B / direct`. A ratio below
one favors B.

The target population is repeated post-one-warmup execution of the exact linked
`bounded_subarrays` image on the recorded host, operating-system image,
toolchain, compiler flags, environment, affinity policy, and run window. This
population does not include another build, host, architecture, compiler,
allocator, workload distribution, input type, signed-value extension, or
streaming system. No result supports a universal winner.

## Semantic equality and correctness canaries

All candidates implement the exact `(&[u64], u128) -> u128` contract from the
lesson. Equality is exact `u128` equality.

Before timing, the native harness constructs the named fixture and checks the
candidate against its frozen expected count. Every valid record contains this
exact canary shape:

```text
exact_match      = true
expected_count   = the frozen workload value
observed_count   = expected_count
mathematical_u128 = true
```

The runner requires one JSON record, exact candidate, workload, phase, seed,
warmup, sample, and inner echoes, exact schema fields, valid canaries, a stable
fixture hash, a stable linked-image hash, a positive position median, and the
frozen work record. Exit status zero alone does not validate an attempt.

## Frozen fixture catalog

The fixture generator is `topic005-fixture-v1`. The assignment seed is
`0x0000_0005_5A17_D0C5`. The runner records a derived seed for every scheduled
position, but these ten fixed cells do not vary their values or budget by seed.

| Workload | Exact values | Budget | Exact expected count | Intended stress |
|---|---|---:|---:|---|
| `n8_zero_heavy_budget0` | `[0, 2, 0, 0, 5, 0, 1, 0]` | 0 | 6 | tiny zero runs and duplicate prefixes |
| `n8_all_fit` | `[2, 0, 3, 2, 1, 4, 0, 1]` | 13 | 36 | tiny triangular all-fit case |
| `n64_immediate_reject` | 64 copies of `1` | 0 | 0 | quadratic early break at every start |
| `n4096_immediate_reject` | 4,096 copies of `1` | 0 | 0 | larger early-break case |
| `n4096_all_fit` | 4,096 copies of `1` | 4,096 | 8,390,656 | quadratic worst-case traversal |
| `n65536_all_fit` | 65,536 copies of `1` | 65,536 | 2,147,516,416 | long direct scan and full-window count |
| `n65536_oversized_every64` | value `64` when `index % 64 == 0`, otherwise `1` | 63 | 2,064,384 | 1,024 oversized separators and 63-value runs |
| `n65536_half_oversized_alternating_zero` | value `1` at even indices, `0` at odd indices | 0 | 32,768 | reset every other value and isolated zeros |
| `n64_zero_heavy_budget0` | `[0, 0, 0, 1]` repeated 16 times | 0 | 96 | prefix duplicates and zero-run lower bounds |
| `n4096_uniform_moderate` | 4,096 copies of `1` | 32 | 130,576 | prefix search with a stable maximum width of 32 |

The expected counts are frozen correctness canaries. For example, the
oversized-every-64 fixture contains 1,024 independent runs of 63 ones, so its
count is `1,024 * 63 * 64 / 2 = 2,064,384`. The uniform-moderate fixture counts
all suffixes of length at most 32:

```text
1 + 2 + ... + 32 + (4096 - 32) * 32 = 130,576
```

These equations establish the canaries. They do not predict elapsed time.

## Frozen primary family

The family contains exactly these 12 candidate-to-direct contrasts:

| Family ID | B candidate | Workload | Reason fixed before collection |
|---:|---|---|---|
| 1 | `reference` | `n8_zero_heavy_budget0` | bounded independent recomputation with zeros |
| 2 | `reference` | `n8_all_fit` | bounded reference all-fit work |
| 3 | `quadratic` | `n8_all_fit` | tiny all-fit crossover |
| 4 | `quadratic` | `n64_immediate_reject` | small best-shaped early exit |
| 5 | `quadratic` | `n4096_immediate_reject` | large best-shaped early exit |
| 6 | `quadratic` | `n4096_all_fit` | declared quadratic worst case |
| 7 | `reset` | `n65536_all_fit` | extra reset branch with no oversized values |
| 8 | `reset` | `n65536_oversized_every64` | periodic oversized separators |
| 9 | `reset` | `n65536_half_oversized_alternating_zero` | maximum reset frequency in the family |
| 10 | `prefix` | `n64_zero_heavy_budget0` | duplicate prefixes on a small input |
| 11 | `prefix` | `n4096_uniform_moderate` | moderate valid width and repeated searches |
| 12 | `prefix` | `n65536_all_fit` | large prefix allocation and all-fit searches |

Direct has the same fixture, budget, process boundary, warmup, timed batches,
and `inner` as B within a contrast. The family does not include every candidate
on every workload. That omission is part of the frozen scope, not evidence that
an omitted pairing is unimportant.

## Operation and timing boundary

The only operation phase is `count`. The native CLI still requires
`--phase count` so the schema and schedule identify the estimand explicitly.

Each schedule position launches a fresh process. The process generates and
hashes the fixture, verifies the literal expected count, checks the candidate,
and performs one untimed warmup outside the timer. It then records exactly three
timed batches. The position outcome is the median of `sample_ns`. The default
scheduled `inner` is 64, so each timed batch contains 64 candidate calls. The
native harness still permits an explicit `inner = 1` for a manual canary run.

One timed batch calls only the selected counting function `inner` times,
consumes each returned `u128` so the optimizer cannot discard the call, and
reads the clock boundary. Fixture construction, input cloning, canary
verification, warmup, work accounting, fixture hashing, JSON formatting,
process startup, and the dynamic loader are outside the timer. The immutable
input allocation remains live but is not created or destroyed inside the
timer.

This is a local sequential microbenchmark. No request arrival process, queue,
retry, timeout, or service concurrency belongs to the estimand. The outcome is
not end-to-end latency, throughput, or capacity. Process startup is excluded.
Long-duration thermal steady state is outside the one-warmup target population.

A timeout or other invalid position makes its block incomplete. Inner
iterations and the three timed batches are subsamples, not independent
replicates. They cannot replace a missing complete block.

## Logical work recorded outside timing

Each record contains these exact nonnegative integer fields:

```text
input_values
mathematical_subarrays
zero_values
oversized_values
candidate_calls
candidate_value_visits
candidate_prefix_slots
result_scalar_slots
```

Every work field scales with `inner`. For one candidate call,
`input_values = n`, `mathematical_subarrays = n(n + 1) / 2`, `zero_values`
counts literal zeros, and `oversized_values` counts values greater than the
workload budget. `candidate_calls = 1` and `result_scalar_slots = 1` per call.
The prefix candidate records `candidate_prefix_slots = n + 1`; every other
candidate records zero prefix slots.

`candidate_value_visits` uses a frozen candidate-specific definition:

- reference records `n(n + 1)(n + 2) / 6`, the sum of all candidate subarray
  lengths;
- quadratic records one visit per examined value, including the first value
  that makes a start's running sum exceed the budget;
- direct records one visit per incoming value plus one per left-edge value
  removed during repair;
- reset uses the direct definition, except that an individually oversized
  incoming value resets without a left-edge removal visit;
- prefix records `n` prefix-build reads plus the exact lower-bound comparisons
  over `prefix[0..end]` for every exclusive end.

Counts come from separate formulas or untimed count passes. They do not prove
the exact loads, comparisons, branches, allocations, instructions, cache
misses, or bytes moved by optimized code. Prefix slots do not include a vector
header, spare capacity, alignment, or allocator metadata.

## Unit roles and interference

- Treatment-application unit: one fresh native process position running one
  candidate on one workload in phase `count`.
- Randomization unit: one four-position block assigned `ABBA` or `BAAB`.
- Analysis unit: one complete four-position block log contrast.
- Subsample unit: one timed batch within a process position. The three batches
  are not independent units.
- Sampling and generalization unit: the exact linked image, host, and run
  window. Repeated processes do not create host or build replication.
- Interference boundary: allocator state, caches, thermals, frequency control,
  scheduler activity, kernel state, and memory-controller state can persist
  across fresh processes.

Each letter reconstructs the same immutable fixture in a fresh process. That
removes candidate object carryover. It does not reset shared host state.
`ABBA` and `BAAB` are justified only under the declared nuisance model:
position drift is additive and linear on the log-time scale, and
cross-position carryover is negligible. Nonlinear drift,
treatment-by-position interaction, serial dependence, or thermal hysteresis
can make the comparison inconclusive.

For positive position outcome `t`, the block contrast is:

```text
ABBA: d = ((log t_B2 + log t_B3) - (log t_A1 + log t_A4)) / 2
BAAB: d = ((log t_B1 + log t_B4) - (log t_A2 + log t_A3)) / 2
```

Both forms cancel an additive linear position term under the declared model.
`exp(mean(d))` is the geometric mean B-to-direct ratio. It is not a ratio of
arithmetic means.

## Assignment, pilot, and stopping

The root assignment seed is `0x0000_0005_5A17_D0C5`. The runner records the
derived schedule, assignment template, block ID, position, runner hash, and
fixture hash. The seed alone is not a complete reproduction asset because
shuffle behavior can depend on the runner implementation and language version.

For every primary contrast:

- run four pilot blocks, exactly two `ABBA` and two `BAAB`, in a seed-shuffled
  restricted order;
- exclude every pilot observation from the main estimate;
- report the prospective 12-block simultaneous interval half-width under
  `0.5x`, `1x`, `1.5x`, and `2x` the pilot standard deviation;
- run exactly 12 main block IDs, six `ABBA` and six `BAAB`, in a separately
  seed-shuffled restricted order;
- inspect and stop only after complete blocks;
- never add blocks after inspecting a timing, estimate, interval, or winner.

The main horizon remains 12 blocks even when pilot variance predicts weak
precision at the 5% boundary. A wide interval is inconclusive. The protocol
does not retry until significance, stop when a desired candidate wins, or pool
pilot and main data.

## Effect boundary and simultaneous intervals

The symmetric practical factor is `1.05` on the B-to-direct time ratio:

- an upper simultaneous bound below `1 / 1.05` supports B time at most
  `1 / 1.05` of direct time in the declared workload;
- a lower simultaneous bound above `1.05` supports B time at least `1.05`
  times direct time;
- every other interval supports no practically meaningful winner.

Equality with either boundary is inconclusive. This classification describes a
scoped timing result. It is not a merge or repository-policy verdict.

For each family member, construct a two-sided paired Student `t` interval over
the 12 main block log contrasts, then exponentiate its endpoints. Familywise
alpha is `0.05`. Bonferroni assigns per-contrast alpha:

```text
0.05 / 12 = 0.004166666666666667
```

Each tail receives `0.0020833333333333333`. Bonferroni does not require
independence between family members. The paired interval still assumes
independent, identically distributed, approximately normal block log contrasts
after blocking.

Retain block order and report a distribution-free diagnostic for the median
block contrast. With 12 independent continuous contrasts, the interval from
the third through tenth sorted values has coverage:

```text
1 - 2 * P(Binomial(12, 0.5) <= 2) = 0.96142578125
```

That diagnostic is coarse and not simultaneous across 12 contrasts. It cannot
repair dependence, confounding, nonlinear drift, an invalid schedule, or weak
power at the practical boundary.

## Invalid attempts and partial blocks

Before launch, every attempt gets an immutable directory. Retain its full argv,
cwd, controlled environment, timestamps, stdout, stderr, return code, signal or
timeout, parser result, canaries, fixture and output hashes, work record, and
pre/post linked-image hashes.

A signal, timeout after 180 seconds, nonzero exit, panic, launch
failure, parse failure, missing field, extra field, echo mismatch, hash
mismatch, canary failure, work-count mismatch, nonpositive position median,
source change, image change, or reset failure marks the attempt invalid. Its
block is incomplete and cannot enter analysis or stopping.

There are no automatic retries. A later campaign has a new run identity and
cannot silently replace the failed attempt. Signal handling forwards `SIGINT`,
`SIGTERM`, and `SIGHUP` to the active process group. Timeout handling kills the
whole process group so a profiler or harness descendant cannot survive. Every
partial and invalid record remains in raw evidence.

Before statistical reduction, the runner writes and verifies a checksum
manifest over each phase's raw attempt and block files. The final evidence
bundle also gets a verified checksum manifest.

## Identical-artifact A/A

Run 12 complete blocks on `n4096_uniform_moderate` in phase `count`. Both labels
execute `direct` from the same linked image. Six blocks use `ABBA` and six use
`BAAB`, in seed-shuffled order.

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
consumer, and clock boundary. It hashes the linked image before and after
inspection. Symbol retention and disassembly are observed code shape. They do
not prove the dynamic path or a causal mechanism by themselves.

The dynamic profile phase uses these representative targets:

| Candidate | Workload | Reason |
|---|---|---|
| `reference` | `n8_zero_heavy_budget0` | keep cubic recomputation bounded |
| `direct` | `n65536_all_fit` | long default linear scan |
| `quadratic` | `n4096_all_fit` | declared quadratic worst case |
| `reset` | `n65536_oversized_every64` | periodic separator fast path |
| `prefix` | `n4096_uniform_moderate` | allocation plus repeated lower bounds |

The profile path also uses `inner = 64`. On Linux the runner attempts
`perf record`. On macOS it attempts an `xctrace` Time Profiler capture. A
permission denial, unsupported tool,
missing artifact, failed target canary, or changed image is retained as
`ATTEMPTED_UNAVAILABLE`. It is not empty profile evidence.

The profiler wraps the whole harness. Samples may include process startup,
fixture generation, correctness canaries, warmup, timed work, untimed work
accounting, and output formatting. Dynamic evidence can show where the recorded
process spent samples. It does not provide per-operation counters or isolate
the timed region. Elapsed time alone cannot establish cache, branch,
instruction, allocation, or bandwidth mechanisms.

## Exact artifact and evidence boundary

The runner builds with Cargo JSON into a new empty target directory. It accepts
exactly one compiler artifact that matches the package manifest, bench target
`bounded_subarrays`, bench kind, optimized profile fields, and absolute
executable path. It rejects zero or multiple matches, malformed Cargo JSON, an
ambiguous target runner, non-native target configuration, image changes, and
source changes.

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
