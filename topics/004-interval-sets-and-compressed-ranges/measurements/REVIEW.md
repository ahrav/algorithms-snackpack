# Measurement review

status: frozen_before_collection

## Frozen design

- [x] `benchmark_required: true` is justified by runtime and crossover claims.
- [x] The baseline is wide-endpoint flat sort-and-merge.
- [x] The family contains exactly 32 candidate-to-flat contrasts.
- [x] Four cells vary size, order, duplicates, overlap shape, and output runs.
- [x] `build` and `build_membership` have separate primary contrasts.
- [x] The timing boundary, position outcome, ratio orientation, and population
  are fixed before collection.
- [x] Treatment, randomization, analysis, subsample, and generalization units
  are distinct.
- [x] `ABBA` and `BAAB` are limited to additive linear position drift with
  negligible carryover.
- [x] Pilot blocks cannot enter the main estimate.
- [x] The main horizon is fixed at 12 complete blocks per contrast.
- [x] Bonferroni simultaneous intervals cover the 32-member family.
- [x] The symmetric practical factor is 1.05, with equality inconclusive.

## Schema and work-count review

- [x] Every valid attempt emits exactly one schema-v1 JSON record.
- [x] Canonical intervals, cardinality, membership, and the `2^32` endpoint are
  checked before timing.
- [x] Fixture, output, image, parser, and work identities are retained.
- [x] `canonical_binary_search_comparisons` is labeled as an untimed canonical
  projection proxy, not an actual candidate hot-path count.
- [x] `result_scalar_slots` is labeled as logical retained fields, not an
  allocation, capacity, transient-memory, or allocator measurement.
- [x] Elapsed time does not establish a cache, branch, instruction,
  allocation, or bandwidth mechanism.

## Runner integrity before collection

- [x] Relative output directories fail closed.
- [x] Existing output directories fail closed by contract.
- [x] Output directories inside the repository fail closed by contract.
- [x] Cargo resolves exactly one optimized bench artifact from JSON in a new
  target directory.
- [x] Ambiguous Cargo runners and non-native targets fail closed.
- [x] A fresh process is used for every schedule position.
- [x] Timeouts kill the whole attempt process group.
- [x] `SIGINT`, `SIGTERM`, and `SIGHUP` are relayed and retained.
- [x] There are no automatic retries or silent replacements.
- [x] Raw phase inputs are checksummed and verified before reduction.
- [x] The final bundle manifest is checksummed and verified.

## Collection gates

- [ ] The measured source commit is clean and matches the source snapshot.
- [ ] Four complete pilot blocks exist for every contrast.
- [ ] Pilot sensitivity is reported at `0.5x`, `1x`, `1.5x`, and `2x` pilot
  standard deviation.
- [ ] Twelve complete main blocks exist for every contrast.
- [ ] Every primary template allocation has six `ABBA` and six `BAAB` blocks.
- [ ] No timing, estimate, or interval changed the fixed main horizon.
- [ ] Every invalid, partial, timeout, signal, parse failure, canary failure,
  and reset failure remains in raw evidence.
- [ ] Reliability outcomes include every started attempt.

## A/A gates

- [ ] Both labels execute the same `flat` function path and linked image.
- [ ] Twelve fixed A/A blocks complete with balanced templates.
- [ ] Fixture, output, work, canary, parser, and missing-data paths are equal
  across labels.
- [ ] Mechanical integrity is reported separately from null diagnostics.
- [ ] A/A spread is not called a noise floor or full null calibration.

## Analysis gates

- [ ] Only complete main blocks enter the primary estimates.
- [ ] Candidate-to-flat orientation is consistent in every result.
- [ ] Each two-sided interval uses alpha `0.05 / 32`.
- [ ] The 1.05 symmetric boundary uses simultaneous bounds.
- [ ] Block order and the distribution-free median diagnostic are retained.
- [ ] Drift, carryover, dependence, nonnormality, and clock resolution are
  reviewed before interpreting a candidate selection.
- [ ] Wide or invalid intervals remain inconclusive.

## Static and dynamic profile gates

- [ ] Linked symbols and disassembly retain all five candidate paths, the
  timed loop, result consumer, and clock boundary.
- [ ] The linked image hash is unchanged before and after static inspection.
- [ ] Every dynamic target first passes a same-image correctness canary.
- [ ] Linux `perf record` or macOS `xctrace` is attempted for every declared
  target.
- [ ] Unsupported, denied, or failed dynamic collection stays visible as
  `ATTEMPTED_UNAVAILABLE`.
- [ ] Whole-process dynamic samples are not described as timed-region counters.
- [ ] Measured, derived, observed, and inferred claims remain separate.

## Archive and publication gates

- [ ] The external archive contains every raw attempt and manifest.
- [ ] Archive SHA-256, byte count, member count, run IDs, original manifest
  hashes, linked-image hash, host, toolchain, and completion state are verified.
- [ ] `EVIDENCE_RECEIPT.json` matches the verified archive.
- [ ] `RESULTS.json` contains only the compact aggregate.
- [ ] The staged set contains no run directory, attempt file, executable,
  disassembly, profile dump, raw stdout or stderr, bundle, or archive.

## Adjudication

Mechanical integrity: pending collection.

Primary result: pending collection.

Evidence limits: pending collection. Any result will apply only to the exact
linked image, workloads, host, toolchain, assignment, and recorded run window.
