# Measurement review

status: complete_with_adjudication_limits

## Frozen design review

- [x] The exact source snapshot and linked image match the benchmark manifest.
- [x] All four candidates satisfy the exact-output canary for every cell.
- [x] The fixture and assignment seed is `2026083002`.
- [x] The family contains exactly 18 predeclared primary contrasts.
- [x] Pilot collection contains four complete blocks per contrast and does not
  enter the main estimate.
- [x] Main collection contains the 12 fixed complete block IDs per contrast.
- [x] Every template allocation is balanced between `ABBA` and `BAAB`.
- [x] No result was inspected before the fixed schedule and analysis were
  frozen.

## Attempt integrity

- [x] Every period uses a fresh native process and the recorded affinity.
- [x] The timer includes one public candidate call and excludes only the frozen
  fixture, canary, hash, and destruction boundaries.
- [x] Every partial, failure, timeout, signal interruption, reset failure, parse
  failure, hash mismatch, and count mismatch remains in raw evidence.
- [x] No failed or incomplete block was silently replaced.
- [x] Any signal retry reuses the same block ID, records `retry_index = 1`, and
  retains the original partial block.
- [x] Work counts match the candidate and fixture contract.

## A/A review

- [x] Both labels resolve to the same `apply_in_place_difference` linked code.
- [x] Source, binary, settings, fixture, work counts, schedule, parser, and
  missing-data paths are identical across labels.
- [x] Twelve fixed A/A blocks completed.
- [x] Mechanical integrity is reported separately from null calibration.
- [x] The A/A interval is treated as a diagnostic, not a noise floor.

## Analysis review

- [x] The analysis unit is one complete four-period block contrast.
- [x] Inner timed calls remain subsamples.
- [x] Candidate-to-in-place orientation is consistent in every row.
- [x] The two-sided log-scale interval uses Bonferroni alpha `0.05 / 18`.
- [x] The 5% practical boundaries are applied to simultaneous bounds.
- [x] Residual time order and a nonparametric sensitivity analysis are shown.
- [x] Wide or assumption-violating intervals are reported as inconclusive.
- [x] Reliability outcomes include every attempt.

## Mechanism review

- [x] Counter support, scaling, multiplexing, and exit status are recorded.
- [x] The optimized profile covers the measured linked image and workload.
- [x] Linked assembly is retained for all four candidates.
- [x] Measured, derived, observed, and inferred claims are separated.
- [x] A mechanism is not claimed from elapsed time alone.

## Archive and publication review

- [x] The external archive contains every raw attempt and manifest.
- [x] Archive SHA-256, byte count, member count, manifest hashes, run IDs,
  linked-image hash, host, toolchain, and completion state were verified.
- [x] `EVIDENCE_RECEIPT.json` matches the verified archive.
- [x] `RESULTS.json` contains only compact aggregates.
- [x] The staged set contains no run directory, per-attempt file, executable,
  profile dump, counter dump, raw stdout or stderr, bundle, or archive.

## Adjudication

Mechanical integrity:

```text
PASS. The final run completed 1,204 valid attempts and 300 valid blocks. Every
fixed schedule, canary, parser, hash, and A/A mechanical check passed.
```

Primary result:

```text
Use reference for cache_short_shuffled and large_sparse_sorted. Use in_place
for small_full_repeated, cache_clustered, and large_wide_shuffled. The whole
tiny_point_sorted timing cell is inconclusive. Dense versus in-place in
cache_short_shuffled has no justified 5 percent winner.
```

Evidence limits:

```text
The result applies only to source tree
0162fbb0ea65379a573e62766265977d35f0d2dc837bb227893512d0899c4e2c,
linked image
fc988e8cd07ae5f7407905a120883b67766a801867e88610246c6e19e78c1e32,
the Apple M1 Pro host, the frozen workloads, and the recorded run window.
Tiny-cell timing is clock-quantized. Dynamic resource totals and hardware
counters are unavailable. Mechanism and cross-machine claims remain inferred.
```
