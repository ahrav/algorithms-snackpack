# Measurement record

status: complete_with_dynamic_profile_limit

The fixed campaign compared `oracle`, `packed`, `events`, and `btree` with the
wide `flat` baseline in four workload cells and two operation phases. The ratio
orientation is candidate time divided by flat time.

No candidate had a simultaneous upper bound below `1 / 1.05`. Flat had a
simultaneous 1.05-factor advantage in 23 of 32 contrasts. Nine contrasts were
unresolved at the predeclared boundary.

| Candidate | Build: flat advantage | Build: unresolved | Build plus membership: flat advantage | Build plus membership: unresolved |
|---|---:|---:|---:|---:|
| Point oracle | 4 | 0 | 4 | 0 |
| Packed runs | 1 | 3 | 0 | 4 |
| Event sweep | 4 | 0 | 3 | 1 |
| Range tree | 3 | 1 | 4 | 0 |
| Total | 12 | 4 | 11 | 5 |

Flat sort-and-merge is the justified default for this exact matrix. Packed
runs still use half the endpoint payload of wide runs by construction. This
campaign did not establish a packed timing advantage.

## Integrity and uncertainty

- Pilot: 512 of 512 harness attempts valid.
- Main: 1,536 of 1,536 harness attempts valid.
- Identical-artifact A/A: 48 of 48 harness attempts valid.
- Profile canaries: 5 of 5 harness attempts valid.
- Total decision-campaign harness attempts: 2,101 of 2,101 valid.
- Complete analysis blocks: 524 of 524.
- Invalid attempts, timeouts, external interruptions, and zero samples: zero.

The identical-flat A/A candidate-to-flat ratio was `1.0041908756`. Its
Bonferroni interval was `[0.9870439638, 1.0216356633]`; its unadjusted 95%
interval was `[0.9951038772, 1.0133608539]`. Mechanical integrity passed. One
A/A campaign does not estimate a false-positive rate or define a noise floor.

All main results retain their 12 block log contrasts, time-order diagnostics,
and distribution-free median sensitivity in
[RESULTS.json](RESULTS.json). All pilot results retain the predeclared
prospective 12-block width calculations at `0.5x`, `1x`, `1.5x`, and `2x` the
pilot standard deviation.

The minimum main position median was 208 ns. Timer overhead can be material at
that scale. The simultaneous intervals measure process-block variation but do
not remove systematic timing-boundary bias. The largest absolute time-order
slope was `0.0738682` log-ratio units per block for event sweep versus flat in
the clustered build-plus-membership cell. That contrast was unresolved.

Every candidate was timed through a `&dyn IntervalSet` view, so each timed
`cardinality` and `contains` call was an indirect call that could not be inlined
or specialized to the concrete representation. The cost applied to the flat
baseline as well, so the ratios remain a symmetric comparison, but they estimate
membership through a uniform dynamic interface rather than the best form each
representation could reach after inlining. This campaign therefore does not
exclude a packed or tree advantage that depends on inlining a short membership
path. Establishing that would require a fresh campaign against a monomorphized
timed loop.

## Profile boundary

The exact linked image retained all eight required symbol substrings. Symbol
and disassembly capture completed with an unchanged image hash. All five
same-image profile canaries passed.

All five macOS `xctrace` Time Profiler attempts returned code 2 without a
validated target record. Their partial trace bundles and stderr remain in raw
evidence. They are `ATTEMPTED_UNAVAILABLE`, not dynamic mechanism evidence.
Elapsed time does not establish cache, branch, allocation, instruction, or
bandwidth mechanisms.

## Raw evidence boundary

The verified external archive is:

```text
/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-004/20260901T152408Z-29f91e6-v2.tar.gz
```

It contains three bundles:

1. a failed-closed quick run whose source changed during the build;
2. a complete 40-position quick parser run;
3. the complete fixed pilot, main, A/A, and profile campaign.

The first failed archive build was rejected because macOS inserted AppleDouble
members. The retained replacement has no AppleDouble files, unsafe paths,
duplicate names, or links. Gzip integrity, clean extraction, extracted-copy
comparison, and all 11,496 embedded manifest entries passed.

[EVIDENCE_RECEIPT.json](EVIDENCE_RECEIPT.json) records the archive hash, size,
member counts, bundle identities, manifest hashes, source commit, linked image,
host, toolchain, and evidence limits. Git contains no raw attempt, stdout,
stderr, executable, disassembly, trace bundle, or archive.

## Scope

The measurements apply only to source commit
`29f91e6a3237797c876d359a6f9b326d11070e43`, source-tree digest
`e82c86c533565c5897e7ad72fe48b99f754b6b265d22d3a489dd3bbc41f93633`,
linked image
`657e60d2e2b4eb9e235b502bc67a71138f30eb4791bc9bcaf4264ac603e70b8d`,
the recorded Apple M1 Pro host, Rust 1.93.0, fixed workloads, assignment, and
2026-09-01 run window. They do not establish a universal representation
winner or a production service result.
