# Measurement record

status: incomplete_prefix_contrasts_superseded

The fixed campaign compared `reference`, `quadratic`, `reset`, and `prefix`
with the `direct` sliding-window baseline in 12 predeclared workload
contrasts. The ratio orientation is candidate time divided by direct time.

Four candidate paths had a simultaneous upper bound below `1 / 1.05`:

- quadratic early exit for immediate rejection at `n = 64` and `n = 4096`;
- oversized reset for a separator every 64 values and for alternating
  oversized values and zeros.

Direct had a simultaneous lower bound above `1.05` in five further contrasts.
No contrast was unresolved at the declared boundary. The three `Prefix` rows
below are superseded pending regeneration and support no conclusion; see
Integrity and uncertainty.

| Candidate | Workload | Candidate/direct | Simultaneous interval | Result |
|---|---|---:|---|---|
| Reference | `n8_zero_heavy_budget0` | 6.8759 | `[6.6709, 7.0871]` | direct faster |
| Reference | `n8_all_fit` | 10.8130 | `[9.8753, 11.8396]` | direct faster |
| Quadratic | `n8_all_fit` | 3.9046 | `[3.7675, 4.0468]` | direct faster |
| Quadratic | `n64_immediate_reject` | 0.4036 | `[0.3805, 0.4281]` | quadratic faster |
| Quadratic | `n4096_immediate_reject` | 0.3747 | `[0.3544, 0.3962]` | quadratic faster |
| Quadratic | `n4096_all_fit` | 2283.8073 | `[2224.5427, 2344.6508]` | direct faster |
| Reset | `n65536_all_fit` | 1.2916 | `[1.2614, 1.3225]` | direct faster |
| Reset | `n65536_oversized_every64` | 0.7544 | `[0.7386, 0.7705]` | reset faster |
| Reset | `n65536_half_oversized_alternating_zero` | 0.4442 | `[0.4371, 0.4515]` | reset faster |
| Prefix | `n64_zero_heavy_budget0` | 3.5496 | `[3.3095, 3.8072]` | superseded, pending regeneration |
| Prefix | `n4096_uniform_moderate` | 7.8220 | `[7.2407, 8.4499]` | superseded, pending regeneration |
| Prefix | `n65536_all_fit` | 22.0626 | `[21.6476, 22.4855]` | superseded, pending regeneration |

Direct remains the default for the exact public API. The fixed matrix supports
quadratic early exit for these synthetic immediate-rejection shapes and the
reset variant for these frequent-separator shapes. The run does not establish
a dispatch threshold or a universal winner.

## Protocol correction before decision collection

The first quick run used one candidate call per timed batch. One of its three
attempts produced a zero-nanosecond sample, so the runner failed closed. Before
any pilot or main block ran, the protocol froze 64 candidate calls per timed
batch. The failed bundle remains in raw evidence. The final quick run and the
decision campaign both use the 64-call boundary.

## Integrity and uncertainty

These counts are as judged by the parser at commit `0eecf67`, and the prefix
figures no longer revalidate. That parser checked `candidate_value_visits` only
for positivity, and the benchmark supplied a modeled comparison total for the
prefix candidate instead of the count `slice::partition_point` charges. Both
were corrected after collection, so the current parser rejects every prefix B
position in this campaign.

Superseded pending regeneration: the three `prefix_vs_direct` contrasts, their
36 of 144 primary analysis blocks, their 3 pilot entries, and the three
`B_TIME_AT_LEAST_1_05_TIMES_A` classifications drawn from them. The remaining
nine contrasts are unaffected, because the reference, quadratic, reset, and
direct counters always reported exact walks rather than models.

- Pilot: 192 of 192 harness attempts valid.
- Main: 576 of 576 harness attempts valid.
- Identical-artifact A/A: 48 of 48 harness attempts valid.
- Profile canaries: 5 of 5 harness attempts valid.
- Total decision-campaign attempts: 821 of 821 valid.
- Complete analysis blocks: 204 of 204.
- Invalid attempts, timeouts, external interruptions, and zero samples in the
  decision campaign: zero.

The identical-direct A/A candidate/direct ratio was `0.9861430702`. Its
Bonferroni interval was `[0.9363522652, 1.0385815157]`; its unadjusted 95%
interval was `[0.9553996375, 1.0178757837]`. Mechanical integrity passed. The
simultaneous interval is wider than the symmetric 5% region. One A/A campaign
does not estimate a false-positive rate, calibrate the null, or define a noise
floor.

The smallest decision-campaign position median was 583 ns for a batch of 64
candidate calls. The largest absolute time-order slope was `-0.0108664`
log-ratio units per block for prefix versus direct on
`n64_zero_heavy_budget0`. Its first-half and second-half ratios were `3.6207`
and `3.4800`; both remain far on the same side of the practical boundary. That
slope comes from a superseded contrast, so it bounds drift only for the run as
collected and carries no weight for the regenerated campaign.
These diagnostics do not repair dependence, nonlinear drift, carryover,
nonnormality, or timing-boundary bias.

[RESULTS.json](RESULTS.json) retains all 12 block log contrasts, simultaneous
intervals, distribution-free median diagnostics, time-order summaries, pilot
width sensitivities, A/A data, and the profile summary.

## Profile boundary

The exact linked image retained all eight required symbol substrings. `nm` and
complete `otool` disassembly succeeded with an unchanged image hash. All five
same-image profile canaries passed.

All five macOS `xctrace` Time Profiler attempts returned code 2 without a
validated target record. Their partial trace bundles and stderr remain in raw
evidence. They are `ATTEMPTED_UNAVAILABLE`, not dynamic mechanism evidence.
Elapsed time does not establish cache, branch, allocation, instruction, or
bandwidth mechanisms.

## Raw evidence boundary

The verified external archive is:

```text
/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-005/20260902T153049Z-0eecf67-v1.tar.gz
```

It contains six bundles: the failed one-call quick run, two superseded
pre-final validation bundles, one pre-visual-fix quick run, the final-source
quick run, and the complete fixed campaign. The archive has 6,587 members:
5,246 files and 1,341 directories. All 5,240 embedded manifest entries passed.
Gzip integrity, safe paths, unique names, no links, no AppleDouble members,
clean extraction, and extracted-copy comparison passed.

[EVIDENCE_RECEIPT.json](EVIDENCE_RECEIPT.json) records the archive hash, size,
member counts, bundle identities, manifest hashes, source commit, linked image,
host, toolchain, and evidence limits. Git contains no raw attempt, stdout,
stderr, executable, disassembly, trace bundle, manifest, or archive.

## Scope

The measurements apply only to source commit
`0eecf67dcc299d665bc57773374b6a2983b7d91f`, source-tree digest
`4befbddae044f904abe77f1113d20d0deff437bea24b714093d811d33540f8e9`,
linked image
`372315c83f8d0ef948433fe9149d00fb2509041710f2c876d956fffe83c8245a`,
the recorded Apple M1 Pro host, Rust 1.93.0, fixed workloads, assignment, and
2026-09-02 run window. They do not establish a universal algorithm winner,
production latency, a stable crossover, another host result, or a dynamic
machine mechanism.
