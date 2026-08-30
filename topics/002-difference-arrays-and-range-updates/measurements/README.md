# Measurement record

status: complete_with_adjudication_limits

Final run: `topic002-runner-all-20260830d`. The run completed 1,204 valid
attempts and 300 valid blocks on an Apple M1 Pro. All 18 primary contrasts,
the 12-block identical-artifact A/A campaign, and the linked-image profile
phase completed. [BENCHMARK.md](../BENCHMARK.md) contains the frozen design.
[RESULTS.json](RESULTS.json) contains the compact aggregate.

Candidate-to-in-place ratios use simultaneous intervals across the 18-member
family. The scoped portfolio is:

| Cell | Selection | Limit |
|---|---|---|
| `tiny_point_sorted` | Timing inconclusive | Clock quantization invalidates all three formal timing decisions. |
| `small_full_repeated` | `in_place` | Exact recorded cell and artifact only. |
| `cache_short_shuffled` | `reference` | Dense versus in-place has no justified 5% winner. |
| `cache_clustered` | `in_place` | Dense effect size is model-sensitive. |
| `large_sparse_sorted` | `reference` | Exact recorded cell and artifact only. |
| `large_wide_shuffled` | `in_place` | Exact recorded cell and artifact only. |

The A/A point ratio was `0.9971025093`, with simultaneous interval
`[0.9710396151, 1.0238649367]`. Every mechanical check passed. The A/A run is
a diagnostic for this campaign, not a universal noise floor.

## Workload and commands

The primary family contains three candidate-to-in-place comparisons across six
cells. Fixture and assignment seed: `2026083002`. The runner accepts these
phases:

```bash
python3 scripts/run_experiment.py quick --output-dir /absolute/outside/repo/new-dir
python3 scripts/run_experiment.py pilot --output-dir /absolute/outside/repo/new-dir
python3 scripts/run_experiment.py main --output-dir /absolute/outside/repo/new-dir
python3 scripts/run_experiment.py aa --output-dir /absolute/outside/repo/new-dir
python3 scripts/run_experiment.py profile --output-dir /absolute/outside/repo/new-dir
python3 scripts/run_experiment.py all --output-dir /absolute/outside/repo/new-dir
```

`--output-dir` must resolve to an absolute, nonexistent directory outside this
repository. The runner records each native command and uses a 120-second timeout
per attempt.

## Identity required for a valid run

- source commit and dirty-state check;
- `rustc -Vv`, Cargo version, build profile, target, and compiler flags;
- benchmark executable SHA-256 and exact linked-image hash;
- host name, architecture, CPU model, logical CPU set, affinity, kernel, and
  frequency policy;
- allocator and relevant environment variables;
- fixture generator version, seed, input hash, update hash, and expected-output
  hash for every cell;
- start and end timestamps for every attempt and phase.

On Darwin, the runner records the CPU model through a sanitized
`system_profiler` probe. Only the six hardware fields named in
`BENCHMARK.md`, the exact command, status, return code, and a sanitized reason
may enter the raw record. Raw profiler output and device identifiers are
forbidden.

## Raw evidence boundary

Raw attempts live only in the task-owned external run directory until they are
archived. The archive path must have this form:

```text
/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-002/<run-id>.tar.gz
```

The archive must include every complete, partial, failed, timed-out,
interrupted, and reset-failed record. Verify the archive before removing the
source directory. `EVIDENCE_RECEIPT.json` records the absolute archive path,
SHA-256, byte count, member count, run IDs, original manifest hashes, linked
image hash, host and toolchain identity, and completion state.

The verified archive is:

```text
/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-002/topic002-runner-all-20260830d.tar.gz
```

Its SHA-256 is
`067350497eca5db9c94b17881932dad357afdd0dab41352d904cb913a1dfadf1`.
It contains every complete, partial, failed, and superseded run bundle from
this topic. [EVIDENCE_RECEIPT.json](EVIDENCE_RECEIPT.json) records the verified
member counts and original run-manifest hashes.

Do not commit `measurements/runs/`, raw stdout or stderr, executables, profiles,
counter dumps, per-attempt files, run bundles, or the external archive. The only
allowed compact result file is `RESULTS.json`.

## Runner revisions after collection

`RESULTS.json` and `EVIDENCE_RECEIPT.json` pin the runner source hash that
produced them, so the runner in this tree is expected to differ from that hash
once it is corrected. Reproducing a recorded run requires the runner revision
its receipt names; a promotable re-collection uses the current runner.

Three corrections landed after `topic002-runner-all-20260830d`:

- `perf stat` counter rows are parsed, and a completed target whose requested
  counters are unsupported, never counted, or absent is `UNAVAILABLE` rather
  than `COMPLETE`.
- A requested CPU list is applied by `sched_setaffinity` in the forked child
  before `exec` instead of by a `taskset` wrapper, so the recorded
  `observed_affinity` reflects the mask the harness ran under.
- The frozen plan states the sample and warmup counts the `quick` phase
  actually executes.

None of the three can change the numbers in `RESULTS.json`. The final run
completed `pilot`, `main`, `aa`, and `profile` on an Apple M1 Pro: `quick` was
not among its phases, the profile phase used `/usr/bin/time -l` and `otool`
rather than `perf`, and its recorded `affinity` is `null` because no CPU list
was requested.

## Result boundary

The compact result separates:

- measured elapsed times, hardware counters, allocation observations, and
  reliability outcomes;
- derived work counts and complexity;
- observed optimized assembly and profile attribution;
- inferred mechanisms and untested generalizations.

All fixed main blocks and A/A blocks completed. Linked assembly retained all
four candidate symbols and the timing boundary. Dynamic `/usr/bin/time -l`
resource collection was unavailable because macOS denied `kern.clockrate`.
The target calls completed and matched their expected identities, but the
partial tool output is not resource evidence. Cache, branch, allocation,
instruction-count, and bandwidth explanations remain inferred.

The final run retained 645 zero-duration subsamples with positive position
medians. The tiny-cell medians fell on a few clock quanta. The formal model
does not include timer quantization, so every tiny-cell timing decision is
inconclusive.
