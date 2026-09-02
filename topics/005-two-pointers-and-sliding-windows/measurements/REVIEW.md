# Measurement review

status: incomplete_prefix_contrasts_superseded

## Frozen design and exact artifact

- [x] `benchmark_required: true` is justified by runtime and crossover claims.
- [x] The primary family contains exactly 12 candidate-to-direct contrasts.
- [x] Ten cells cover tiny work, all-fit scans, immediate rejection, zeros,
  periodic oversized separators, and frequent separators.
- [x] The treatment, randomization, analysis, subsample, sampling, and
  generalization units are distinct.
- [x] The nuisance model limits `ABBA` and `BAAB` to additive linear position
  drift with negligible carryover.
- [x] Four pilot blocks per contrast are excluded from main estimates.
- [x] Main stopping is fixed at 12 complete blocks per contrast.
- [x] Bonferroni intervals cover the 12-member family.
- [x] The symmetric practical factor is 1.05, with equality inconclusive.
- [x] The first quick run failed closed on a zero-nanosecond one-call sample.
- [x] The 64-call timed batch was frozen before any pilot or main block ran.
- [x] Source commit `0eecf67dcc299d665bc57773374b6a2983b7d91f`
  was clean at collection.
- [x] Source digest
  `4befbddae044f904abe77f1113d20d0deff437bea24b714093d811d33540f8e9`
  remained unchanged before build, after build, and after collection.
- [x] Linked-image hash
  `372315c83f8d0ef948433fe9149d00fb2509041710f2c876d956fffe83c8245a`
  remained unchanged.

## Collection and reliability

- [x] Pilot has four complete blocks per contrast: 192 of 192 attempts valid.
- [x] Main has 12 complete blocks per contrast: 576 of 576 attempts valid.
- [x] Main templates balance six `ABBA` and six `BAAB` blocks per contrast.
- [x] A/A has 12 complete balanced blocks: 48 of 48 attempts valid.
- [x] Profile canaries are 5 of 5 valid.
- [x] The decision campaign has 821 of 821 valid harness attempts and 204 of
  204 complete blocks.
- [x] No decision-campaign attempt timed out, received an external signal,
  emitted a zero sample, failed parsing, changed the linked image, or failed a
  canary.
- [x] The failed quick run and all later complete validation runs are retained.
- [x] There were no automatic retries or silent replacements.
- [ ] The prefix records revalidate under the current parser. They do not: each
  box above reflects the parser at commit `0eecf67`, which accepted any positive
  `candidate_value_visits`, and the benchmark then reported a modeled prefix
  comparison total rather than the count `slice::partition_point` charges. The
  three `prefix_vs_direct` contrasts and their classifications are superseded
  pending regeneration. The other nine contrasts are unaffected.

## Primary adjudication

The ratio orientation is candidate time divided by direct time. Four
contrasts had a simultaneous upper bound below `1 / 1.05`:

| Candidate | Workload | Ratio | Simultaneous interval |
|---|---|---:|---|
| Quadratic | `n64_immediate_reject` | 0.4036 | `[0.3805, 0.4281]` |
| Quadratic | `n4096_immediate_reject` | 0.3747 | `[0.3544, 0.3962]` |
| Reset | `n65536_oversized_every64` | 0.7544 | `[0.7386, 0.7705]` |
| Reset | `n65536_half_oversized_alternating_zero` | 0.4442 | `[0.4371, 0.4515]` |

Direct had a simultaneous lower bound above `1.05` for both reference cells,
the tiny and all-fit quadratic cells, and the reset all-fit cell. No primary
contrast was unresolved. The three prefix cells are superseded pending
regeneration and support no conclusion, so this adjudication rests on nine of
the twelve contrasts.

Direct is the justified default for the nine contrasts that stand. Quadratic
early exit is favored only in the two immediate-rejection cells. Reset is
favored only in the two frequent-separator cells. Prefix is unadjudicated
until the campaign is regenerated. The experiment does not locate a crossover
or justify a public dispatch rule.

All exact estimates, intervals, classifications, 12 block log contrasts,
distribution-free median sensitivities, time-order summaries, and pilot width
sensitivities are in [RESULTS.json](RESULTS.json). There was no post hoc family
change, component timer, candidate deletion, or added block.

## A/A and diagnostics

- [x] Both A/A labels execute the same `direct` path in the same image.
- [x] Fixture, output, work, canary, parser, and missing-data paths match.
- [x] The A/A candidate/direct ratio is `0.9861430702`.
- [x] Its Bonferroni interval is `[0.9363522652, 1.0385815157]`.
- [x] Its unadjusted 95% interval is `[0.9553996375, 1.0178757837]`.
- [x] All 12 A/A block contrasts and label-by-position summaries are published.
- [x] Mechanical integrity is separate from null calibration.
- [x] One A/A run is not called a noise floor or false-positive calibration.

The simultaneous A/A interval extends beyond the symmetric 5% region on the
lower side. The unadjusted interval stays within that region. This diagnostic
does not invalidate the correctly executed schedule, but it limits how much
confidence to place in effects near 5% on this host. Every primary interval in
this campaign is well separated from the boundary.

The minimum position median was 583 ns for 64 candidate calls. Timer overhead
can matter at that scale. The largest absolute time-order slope was
`-0.0108664` log-ratio units per block for prefix versus direct on the small
zero-heavy cell. Its first-half and second-half ratios were `3.6207` and
`3.4800`. The retained diagnostics do not repair nonlinear drift, dependence,
carryover, nonnormality, or systematic boundary bias.

## Static and dynamic profile evidence

- [x] The exact image retains all five candidate workloads, the timed loop,
  result consumer, and clock boundary.
- [x] `nm` and complete `otool` disassembly succeeded with no missing symbol.
- [x] The image hash was unchanged before and after static inspection.
- [x] Every dynamic target first passed a same-image correctness canary.
- [x] Five `xctrace` Time Profiler attempts ran and remain visible.
- [x] Every dynamic attempt returned code 2 without a validated target record.
- [x] Partial trace bundles and stderr are retained as
  `ATTEMPTED_UNAVAILABLE`.
- [x] No elapsed-time result is used as cache, branch, instruction, allocation,
  or bandwidth mechanism proof.

Static code shape is observed evidence. Dynamic mechanism attribution is
unavailable for this campaign.

## Correctness, test, and visual review

- [x] The independent cubic reference shares no movement or prefix helper with
  optimized candidates.
- [x] Eleven focused tests cover exact boundaries, wide arithmetic, zeros,
  duplicate prefixes, deep maximum-value separators, and all five candidates.
- [x] The exhaustive gate checks exactly 109,220 array-budget cases.
- [x] The fixed generated stream checks 640 larger cases and prints replay data.
- [x] Reversal, budget monotonicity, positive scaling, and all-fit properties
  have the scope and oracle limits stated in `TEST_STRATEGY.md`.
- [x] The independent invariant audit found no blocking correctness defect.
- [x] Its one generator coverage warning was converted into the deep
  `u64::MAX` separator literal regression before measurement.
- [x] All five README visual tables were rendered directly from Markdown in
  light and dark themes.
- [x] All ten rendered copies have consistent row cell counts, no clipping,
  no wrapping, explicit backgrounds, readable contrast, and no color-only
  meaning.
- [x] The first candidate-comparison render was too wide; the corrected
  four-column table passed the second complete render.

## Archive and publication

- [x] One external archive contains all six failed, superseded, quick, profile,
  and decision-campaign bundles.
- [x] Gzip integrity, safe relative paths, unique names, no links, no
  AppleDouble members, and clean extraction passed.
- [x] Extracted-copy comparison passed for all six roots.
- [x] All 5,240 embedded manifest entries passed SHA-256 verification.
- [x] The archive has 6,587 members: 5,246 files and 1,341 directories.
- [x] Archive hash, bytes, run IDs, manifest hashes, source, image, host, and
  toolchain match [EVIDENCE_RECEIPT.json](EVIDENCE_RECEIPT.json).
- [x] [RESULTS.json](RESULTS.json) contains aggregate evidence, not raw process
  samples, executables, stdout, stderr, or trace data.
- [x] The explicit staged set contains no run directory, attempt file,
  executable, disassembly, trace bundle, stdout, stderr, manifest, CSV, or
  archive.

## Scope

The result covers one Apple M1 Pro host, Rust 1.93.0, one exact linked image,
ten synthetic cells, one count phase, and the recorded 2026-09-02 window. It
does not establish a universal algorithm winner, production latency, another
memory budget, a stable crossover, another host result, or a dynamic machine
mechanism.
