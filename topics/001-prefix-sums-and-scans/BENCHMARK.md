benchmark_required: true

# Prefix-scan benchmark protocol

## Status

This file freezes the experiment before collection. No result in this file is
measured evidence. The runner writes all evidence under `measurements/runs/`.

## Claim and outcome

The primary question is narrow: for an inclusive prefix scan of an immutable
`[i64]` into a newly allocated `Vec<i64>`, which implementation has the lower
post-warmup service time on the declared host, artifact, compiler, workload,
and run window?

All additions use `i64::wrapping_add`. Each candidate must return the same
ordered prefix at every index. The input remains unchanged. The candidates are:

- `reference`: the independent simple model;
- `linear`: one serial pass and the baseline for every primary contrast;
- `blocked`: serial local scans, a scan of block totals, and offset repair;
- `parallel`: scoped worker scans, a scan of worker totals, and offset repair.

The primary outcome for one schedule position is the median elapsed
nanoseconds across the position's timed subsamples. The timer starts immediately
before the candidate call and stops immediately after it returns. Output
allocation is inside that boundary. Process startup, input construction,
candidate warmup, semantic validation, and output hashing are outside it. The
ratio is `B / linear`. A ratio below `1` favors `B`; a ratio above `1` favors
`linear`.

This is a local operation microbenchmark. It has no arrival process, queue, or
request concurrency model. It does not measure caller allocation reuse,
end-to-end service latency, cold startup, or an in-place API.

## Target population and evidence boundary

The target population is repeated executions of the exact linked benchmark
image on the recorded host, operating-system image, Rust 1.93.0 toolchain,
compiler flags, affinity policy, and run window. The experiment uses one build
and one host. It cannot estimate build-layout, host, architecture, or fleet
variation. Results do not establish a universal winner.

The selection rule may name the fastest justified candidate only for a listed
cell. The lesson keeps a portfolio because input size, block size, worker count,
thread startup cost, memory bandwidth, allocator behavior, and hardware can
change the winner.

## Fixed primary workload

All primary cells use `pattern=mixed` and input seed `2026082901`. The generator
is part of `benches/prefix_sums.rs`. It is deterministic and combines zeros,
small positive and negative values, index-derived runs, and large positive and
negative values. Those values force frequent wrapping without changing the
contract. Every treatment in a cell receives byte-identical input.

The 12 primary contrasts are fixed:

| Contrast ID | A | B | Length | Block size | Workers |
|---|---|---|---:|---:|---:|
| `ref_linear_n64` | `linear` | `reference` | 64 | n/a | n/a |
| `ref_linear_n512` | `linear` | `reference` | 512 | n/a | n/a |
| `ref_linear_n4096` | `linear` | `reference` | 4,096 | n/a | n/a |
| `blocked_linear_n4096_b16384` | `linear` | `blocked` | 4,096 | 16,384 | n/a |
| `blocked_linear_n262144_b16384` | `linear` | `blocked` | 262,144 | 16,384 | n/a |
| `blocked_linear_n4194304_b16384` | `linear` | `blocked` | 4,194,304 | 16,384 | n/a |
| `parallel2_linear_n262144` | `linear` | `parallel` | 262,144 | n/a | 2 |
| `parallel4_linear_n262144` | `linear` | `parallel` | 262,144 | n/a | 4 |
| `parallel8_linear_n262144` | `linear` | `parallel` | 262,144 | n/a | 8 |
| `parallel2_linear_n4194304` | `linear` | `parallel` | 4,194,304 | n/a | 2 |
| `parallel4_linear_n4194304` | `linear` | `parallel` | 4,194,304 | n/a | 4 |
| `parallel8_linear_n4194304` | `linear` | `parallel` | 4,194,304 | n/a | 8 |

The runner also performs a descriptive distribution audit at length 262,144.
It runs `linear`, `blocked` with block size 16,384, and `parallel` with four
workers on `zero`, `constant`, `ascending`, `alternating`, and `mixed` inputs.
These observations are outside the primary family. They can reveal a workload
interaction. They cannot be promoted into a new confirmatory claim after the
results are visible.

## Work counted before timing

The benchmark emits the precomputed work description for every position. The
count formulas follow the implementation phases and are checked against the
requested parameters before the timer starts. They include prefix additions,
offset additions, output elements written, input elements read, output bytes,
input bytes, result allocations, and requested worker threads. Setup work stays
separate from timed scan work.

For `parallel` with `k > 1`, the auxiliary read and write counts each include
`k` chunk totals and `k-1` offsets. The thread counts each include `k` local-scan
threads and `k-1` repair threads. The explicit `Vec` count is `k+6`. Rust thread
runtime and operating-system allocations remain uncounted.

The counts are elementary derivations, not hardware-counter observations. The
runner never uses elapsed time alone to claim a cache, branch, vectorization, or
memory-bandwidth mechanism.

## Unit roles

- Treatment-application unit: one fresh benchmark-binary process at one
  schedule position.
- Randomization unit: one complete four-position block receives either the
  `ABBA` or `BAAB` template.
- Analysis unit: one complete block log contrast. It is the mean of the two
  `log(B)` position outcomes minus the mean of the two `log(A)` outcomes.
- Subsample unit: one timed candidate call inside a schedule-position process.
- Sampling and generalization unit: the one recorded linked image, host, and
  run window. Process repetition does not create host or build replication.
- Interference boundary: host-wide cache, allocator, thermal, scheduler, and
  memory-controller state can cross process boundaries.

Each schedule letter launches a fresh process. The process constructs a fresh
input and output state. Candidates hold no durable state and write no shared
file. That reset makes algorithm-level carryover negligible. It does not reset
host state. `ABBA` and `BAAB` are justified only for additive linear position
drift. Counterbalancing cannot repair nonlinear drift, thermal hysteresis, or a
treatment-by-position interaction. The retained position order is inspected
before interpreting a contrast.

## Assignment, pilots, and stopping

The assignment seed is `2026082901`. Each contrast has four pilot blocks, with
two `ABBA` and two `BAAB` templates in a seeded shuffled order. Pilot outcomes
are excluded from every primary estimate and confidence interval. The runner
reports pilot block variance and interval-width sensitivity at 0.5, 1.0, 1.5,
and 2.0 times the observed pilot standard deviation. The pilot cannot change
the fixed main count.

Each contrast then has exactly 12 main blocks, with six `ABBA` and six `BAAB`
templates in a new seeded shuffled order. The run stops after those 12 complete
analysis units. There is no early stopping, peeking rule, sample-size extension,
or result-dependent exclusion.

Each main position uses one warmup call followed by three timed subsamples.
Warmup and subsample counts are fixed before collection. A quick phase may use
smaller counts for harness checks, but quick output is exploratory and can
never enter the pilot, main, A/A, or repository decision evidence.

One warmup call defines the measured post-warmup regime. It does not prove
thermal stability or a general steady state. Cold process startup, first-call
latency, and long-duration thermal behavior remain outside the primary claim.

## Effect boundary, intervals, and multiplicity

For each main block `k`, the paired log contrast is:

```text
d_k = mean(log(B position median nanoseconds))
      - mean(log(A position median nanoseconds))
```

The point estimate is `exp(mean(d_k))`. The runner constructs a two-sided
paired Student t interval on the log scale and exponentiates both endpoints.
The model assumes the 12 complete block contrasts are independent, identically
distributed, and approximately normal after the log transform. Host-state
dependence or strong nonnormality weakens the interval.

The primary family contains exactly 12 contrasts. Familywise alpha is `0.05`.
Bonferroni assigns `0.05 / 12` to each two-sided interval, or `0.05 / 24` to
each tail. The family has no interim looks.

The practical multiplicative boundary is `1.05`. A simultaneous interval
wholly below `1 / 1.05` supports a practically meaningful `B` speed advantage
in that cell. An interval wholly above `1.05` supports a practically meaningful
`linear` speed advantage. Any interval that crosses either boundary is reported
without a winner claim. This rule is a performance interpretation, not a merge
or repository policy verdict.

## Invalid attempts, failures, and retries

Every started attempt gets an immutable directory before process launch. The
runner retains the exact argv, environment, cwd, start and end timestamps,
stdout, stderr, exit status, timeout or signal, parser status, semantic canary,
checksums, and pre-execution and post-execution hashes.

A crash, timeout, parse failure, missing output, semantic mismatch, checksum
mismatch, artifact mutation, or reset failure marks the attempt `INVALID` and
the containing block incomplete. The runner never replaces those outcomes
silently. A treatment or workload failure receives no retry. One whole-block
retry is allowed only when the recorded cause is an external `SIGINT`,
`SIGTERM`, or `SIGHUP` interruption. The partial block and all of its attempts
remain in the raw bundle. A second external interruption makes the experiment
`INCOMPLETE`.

Timing analysis uses only the predeclared complete-block estimand. Reliability
is reported separately as counts by treatment, workload, and failure reason.
If any required valid block is absent after the retry rule, the experiment is
`INCOMPLETE` and no primary interval is treated as decision-grade evidence.

## Identical-artifact A/A

The A/A control uses 12 complete blocks at `n=262144`, `pattern=mixed`, and the
same seed. Both labels execute `linear` from the same linked image. Six blocks
use `ABBA`; six use `BAAB`. The labels pass through the same schedule,
fresh-process launch, environment, parser, checksum, missing-data, stopping,
and analysis paths as a primary contrast.

Mechanical integrity is reported separately. It requires identical executable
hashes, candidate parameters, input checksums, semantic checks, complete
position counts, parser versions, and successful bundle verification across
the two labels. Label and schedule metadata are expected to differ.

Null diagnostics report the 12 A/A log contrasts, their mean ratio, an
unadjusted two-sided 95% paired t interval, template balance, position summaries,
and any label-by-position pattern. These diagnostics do not prove absence of
bias, adequate power, or a noise floor. They do not alter the primary stopping
or multiplicity rule.

## Exact artifact and retained evidence

`scripts/run_experiment.py` is the versioned native-binary adapter. It builds
the named `prefix_sums` bench target once, resolves the executable from Cargo
JSON, and hashes the executable before and after every attempt. Direct execution
is allowed only when the target triple equals the recorded host triple and no
effective Cargo target runner is configured. Ambiguous runner configuration
fails closed.

The runner records:

- source commit and dirty diff hash;
- source-tree, benchmark source, runner, and linked-image SHA-256 hashes;
- `rustc -vV`, `cargo -V`, target triple, Cargo profile, complete build argv,
  and explicit compiler flags;
- hostname, operating system, architecture, CPU description, logical CPU
  count, requested and observed affinity when available, and run timestamps;
- the exact controlled environment, cwd, expanded argv, raw JSON, stdout,
  stderr, and parser result for every attempt;
- raw attempt CSV, block CSV, analysis JSON, schedule JSON, a checksum manifest,
  optimized assembly, and available profile or counter output.

The benchmark process receives a small explicit environment rather than the
caller's secret-bearing environment. The exact effective map is safe to retain.
The runner refuses an ambiguous cross-target or configured-runner build.

The `profile` phase emits optimized assembly for the measured bench image. On
Linux it uses `perf stat` when available to request cycles, instructions,
branches, branch misses, cache references, and cache misses. On macOS it retains
`/usr/bin/time -l` resource output and the assembly. Missing permissions or
unsupported counters remain retained failures. Without applicable dynamic
counters or samples, cache and memory-bandwidth explanations remain inferred.

## Commands

Build and check the harness without collecting confirmatory evidence:

```bash
python3 topics/001-prefix-sums-and-scans/scripts/run_experiment.py quick
```

Run one fixed phase:

```bash
python3 topics/001-prefix-sums-and-scans/scripts/run_experiment.py pilot
python3 topics/001-prefix-sums-and-scans/scripts/run_experiment.py main
python3 topics/001-prefix-sums-and-scans/scripts/run_experiment.py aa
python3 topics/001-prefix-sums-and-scans/scripts/run_experiment.py distribution
python3 topics/001-prefix-sums-and-scans/scripts/run_experiment.py profile
```

Run the complete frozen protocol in one evidence directory:

```bash
python3 topics/001-prefix-sums-and-scans/scripts/run_experiment.py all
```

The runner prints the evidence directory. A later report must separate measured
timings and counters, derived work counts, observed assembly, and inferred
mechanisms.
