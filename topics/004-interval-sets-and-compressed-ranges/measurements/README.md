# Measurement record

status: frozen_before_collection

No decision-grade timing result is recorded yet. [BENCHMARK.md](../BENCHMARK.md)
freezes the workload, candidates, timing boundary, assignment, fixed stopping,
analysis, A/A, profile, and retention rules before collection.

## Frozen family

Arm A is `flat`. Arm B is `oracle`, `packed`, `events`, or `btree`. Four
candidate comparisons cross four workload cells and two operation phases:

```text
4 candidates x 4 cells x 2 phases = 32 primary contrasts
```

The root assignment and fixture seed is `2026090104`. The fixture generator is
`topic004-fixture-v1`. Every position uses one warmup and three timed samples.
The main outcome is the position median. One complete `ABBA` or `BAAB` block
log contrast is an analysis unit.

## Runner commands

Plan and self-check do not collect timing evidence:

```bash
python3 scripts/run_experiment.py plan
python3 scripts/run_experiment.py self-check
```

Collection requires a new absolute directory outside the repository:

```bash
python3 scripts/run_experiment.py quick --output-dir /absolute/new/quick
python3 scripts/run_experiment.py pilot --output-dir /absolute/new/pilot
python3 scripts/run_experiment.py main --output-dir /absolute/new/main
python3 scripts/run_experiment.py aa --output-dir /absolute/new/aa
python3 scripts/run_experiment.py profile --output-dir /absolute/new/profile
python3 scripts/run_experiment.py all --output-dir /absolute/new/all
```

`quick` checks every candidate, workload cell, and operation phase once. It is
descriptive and cannot enter the primary result. `all` runs four pilot blocks
and 12 main blocks for every contrast, 12 identical-artifact A/A blocks, five
static-profile canaries, linked-image inspection, and five dynamic profile
attempts.

## Required identity

A promotable run records:

- source revision, branch, dirty diff hashes, source manifest, and source
  snapshot;
- Cargo and rustc versions, target triple, Cargo configuration hashes,
  compiler flags, profile fields, and `Cargo.lock`;
- Cargo's exact JSON artifact message and the retained linked-image SHA-256;
- runner and benchmark-source hashes;
- host, operating system, architecture, CPU identity, affinity policy,
  frequency-policy boundary, allocator boundary, and complete controlled
  environment;
- exact schedules, derived fixture seeds, fixture and output hashes, canaries,
  work records, parser results, and attempt timestamps;
- every valid, invalid, partial, timed-out, signaled, and profile record;
- verified pre-analysis phase manifests and the verified final bundle manifest.

## Raw evidence boundary

Raw attempts stay in the task-owned external run directory until they are
archived. The verified archive must use this location:

```text
/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-004/<run-id>.tar.gz
```

The archive must retain every complete, partial, failed, timed-out, signaled,
and profile record. Verify the archive before removing its source directory.
`EVIDENCE_RECEIPT.json` must record the absolute archive path, SHA-256, byte
count, member count, run IDs, original manifest hashes, exact linked-image
hash, host and toolchain identity, and completion state.

Do not commit raw attempt directories, stdout or stderr, retained executables,
disassembly, profile data, run bundles, or the raw archive. Git receives only
the compact result, receipt, and review files named by the automation gate.

## Result boundary

The eventual result will separate:

- measured elapsed times and reliability outcomes;
- untimed logical count passes and elementary derivations;
- observed optimized linked code and dynamic profile samples;
- inferred mechanisms and untested generalizations.

`canonical_binary_search_comparisons` is an untimed search over the canonical
semantic projection. It is not the candidate's actual membership path.
`result_scalar_slots` counts logical retained result fields. It is not an
allocation or transient-memory measurement.

No candidate selection is justified until all fixed main blocks, A/A
mechanical checks, profile gates, external archive checks, and compact receipt
checks pass.
