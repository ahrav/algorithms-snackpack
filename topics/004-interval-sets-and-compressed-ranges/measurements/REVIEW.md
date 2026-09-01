# Measurement review

status: complete_with_dynamic_profile_limit

## Frozen design and exact artifact

- [x] `benchmark_required: true` is justified by runtime and crossover claims.
- [x] The family contains exactly 32 candidate-to-flat contrasts.
- [x] Four cells vary size, order, duplicates, overlap shape, and output runs.
- [x] `build` and `build_membership` are separate primary outcomes.
- [x] The treatment, randomization, analysis, subsample, sampling, and
  generalization units are distinct.
- [x] The nuisance model limits `ABBA` and `BAAB` to additive linear position
  drift with negligible carryover.
- [x] Pilot blocks are excluded from main estimates.
- [x] Main stopping is fixed at 12 complete blocks per contrast.
- [x] Bonferroni intervals cover the 32-member family.
- [x] The symmetric factor boundary is 1.05, with equality inconclusive.
- [x] Source commit `29f91e6a3237797c876d359a6f9b326d11070e43`
  was clean at collection.
- [x] Source digest
  `e82c86c533565c5897e7ad72fe48b99f754b6b265d22d3a489dd3bbc41f93633`
  remained unchanged before build, after build, and after collection.
- [x] Linked-image hash
  `657e60d2e2b4eb9e235b502bc67a71138f30eb4791bc9bcaf4264ac603e70b8d`
  remained unchanged.

## Collection and reliability

- [x] Pilot has four complete blocks per contrast: 512 of 512 attempts valid.
- [x] Main has 12 complete blocks per contrast: 1,536 of 1,536 attempts valid.
- [x] Main templates balance six `ABBA` and six `BAAB` blocks per contrast.
- [x] A/A has 12 complete balanced blocks: 48 of 48 attempts valid.
- [x] Profile canaries are 5 of 5 valid.
- [x] The decision campaign has 2,101 of 2,101 valid harness attempts and 524
  of 524 complete blocks.
- [x] No harness attempt timed out, received an external interruption, emitted
  a zero sample, failed parsing, changed the linked image, or failed a canary.
- [x] The earlier source-change quick failure and later 40-position quick pass
  are both retained. The failed run contributes no timing result.
- [x] There were no automatic retries or silent replacements.

## Primary adjudication

The ratio orientation is candidate time divided by flat time. No candidate had
a simultaneous upper bound below `1 / 1.05`. Flat had a simultaneous lower
bound above `1.05` for 23 contrasts. Nine contrasts were unresolved.

The unresolved contrasts were:

| Phase | Cell | Candidate | Ratio | Simultaneous interval |
|---|---|---|---:|---|
| build | tiny sparse | packed | 1.1401 | `[1.0221, 1.2717]` |
| build | tiny sparse | btree | 1.0330 | `[0.7111, 1.5006]` |
| build | clustered duplicates | packed | 1.0230 | `[0.9885, 1.0587]` |
| build | adjacent coalescing | packed | 0.9806 | `[0.9130, 1.0532]` |
| build plus membership | tiny sparse | packed | 1.0740 | `[1.0092, 1.1430]` |
| build plus membership | clustered duplicates | packed | 1.0367 | `[0.9148, 1.1749]` |
| build plus membership | clustered duplicates | events | 1.5501 | `[1.0084, 2.3827]` |
| build plus membership | large sparse reverse | packed | 1.0151 | `[0.8322, 1.2383]` |
| build plus membership | adjacent coalescing | packed | 1.0002 | `[0.9317, 1.0736]` |

All exact estimates, intervals, classifications, 12 block log contrasts,
distribution-free median sensitivities, and time-order summaries are in
[RESULTS.json](RESULTS.json). The same file publishes all 32 predeclared pilot
width sensitivities. Build and build-plus-membership are separate outcomes;
there is no post hoc subcomponent timer or candidate ranking.

Flat sort-and-merge is the justified default for this fixed matrix. The packed
form's smaller payload is derived from its representation. The timing evidence
does not establish a packed speed advantage.

## A/A and diagnostics

- [x] Both A/A labels execute the same `flat` path in the same image.
- [x] Fixture, output, work, canary, parser, and missing-data paths match.
- [x] The A/A ratio is `1.0041908756`.
- [x] Its Bonferroni interval is `[0.9870439638, 1.0216356633]`.
- [x] Its unadjusted 95% interval is `[0.9951038772, 1.0133608539]`.
- [x] All 12 A/A block contrasts and label-by-position summaries are published.
- [x] Mechanical integrity is separate from null calibration.
- [x] One A/A run is not called a noise floor or false-positive calibration.

The minimum main position median was 208 ns. Timer overhead can be material at
that scale. The largest absolute time-order slope was `0.0738682` log-ratio
units per block for events versus flat in clustered build-plus-membership. Its
first-half and second-half ratios were `1.3134` and `1.8295`; the simultaneous
interval remained wide and the result stayed inconclusive. The retained
diagnostics do not repair nonlinear drift, dependence, carryover, nonnormality,
or systematic timing-boundary bias.

## Static and dynamic profile evidence

- [x] The exact image retains all five candidate paths, the timed loop, result
  consumer, and clock boundary.
- [x] `nm` and complete `otool` disassembly succeeded with no missing symbol.
- [x] The image hash was unchanged before and after static inspection.
- [x] Every dynamic target first passed a same-image correctness canary.
- [x] Five `xctrace` Time Profiler attempts ran and remained visible.
- [x] Every dynamic attempt returned code 2 without a validated target record.
- [x] Partial trace bundles and stderr are retained as
  `ATTEMPTED_UNAVAILABLE`.
- [x] No elapsed-time result is used as cache, branch, instruction, allocation,
  or bandwidth mechanism proof.

Static code shape is observed evidence. Dynamic mechanism attribution is
unavailable for this campaign.

## Archive and publication

- [x] One external archive contains the failed quick, complete quick, and
  complete decision campaign.
- [x] Gzip integrity, safe relative paths, unique member names, no links, clean
  extraction, and extracted-copy comparison passed.
- [x] All 11,496 embedded manifest entries passed SHA-256 verification.
- [x] The archive has 13,860 members: 11,499 files and 2,361 directories.
- [x] Archive hash, bytes, run IDs, manifest hashes, source, image, host, and
  toolchain match [EVIDENCE_RECEIPT.json](EVIDENCE_RECEIPT.json).
- [x] An initial archive containing macOS AppleDouble members was rejected and
  removed only after the clean replacement passed extraction verification.
- [x] [RESULTS.json](RESULTS.json) contains compact aggregate evidence, not raw
  process samples or per-attempt files.
- [x] The explicit staged set contains no run directory, attempt file,
  executable, disassembly, profile dump, stdout, stderr, manifest, CSV, bundle,
  or archive.

## Scope

The result covers one Apple M1 Pro host, Rust 1.93.0, one exact linked image,
four synthetic interval shapes, two operation phases, and the recorded
2026-09-01 window. It does not establish a universal representation winner,
production latency, online-update behavior, another memory budget, or another
host. The unavailable dynamic profiles also prevent mechanism claims.
