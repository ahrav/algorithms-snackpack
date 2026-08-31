# Measurement review

status: complete_with_dynamic_profile_limit

## Frozen design and integrity

- [x] The exact clean source commit, source snapshot, and release image are pinned.
- [x] Every candidate passed the independent semantic canary in all seven cells.
- [x] The family contains exactly 21 predeclared optimized-candidate contrasts.
- [x] Pilot collection contains four complete blocks per contrast and is excluded from main inference.
- [x] Main collection contains twelve complete blocks per contrast.
- [x] Every contrast balances six `ABBA` and six `BAAB` blocks.
- [x] The stopping rule is fixed, timeouts are 180 seconds, and automatic retries are disabled.
- [x] The independent analysis unit is one complete four-position block contrast.
- [x] The log-scale interval uses Bonferroni alpha `0.05 / 21`.
- [x] Paired-t coverage assumes iid, approximately normal block-log contrasts.
- [x] `ABBA`/`BAAB` addresses additive linear drift only with negligible carryover.
- [x] The symmetric `1.05` factor boundary is applied to simultaneous bounds.
- [x] Every partial, failed, timed-out, or unavailable record is retained.

## Primary adjudication

The ratio orientation is B/A. `B_TIME_AT_MOST_1_OVER_1_05_OF_A` means B
uses at most A divided by `1.05`, about 4.76% less time relative to A.
`B_TIME_AT_LEAST_1_05_TIMES_A` means B uses at least 5% more time relative
to A.

| Cell | Adjudication |
|---|---|
| `tiny_sparse_shuffled` | Dense, then sorted, then adaptive |
| `wide_sparse_low_overlap` | Sorted, then adaptive, then dense |
| `skewed_sparse` | Sorted versus adaptive is unresolved; each is lower than dense |
| `dense_local_high_overlap` | Dense, then adaptive, then sorted |
| `dense_wide_medium_overlap` | Dense, then adaptive, then sorted |
| `long_runs` | Dense, then adaptive, then sorted |
| `mixed_chunk_shapes` | Dense, then adaptive, then sorted |

There are 20 simultaneous factor separations and one unresolved contrast.
The unresolved skewed-sparse sorted-versus-adaptive estimate is B/A
`0.8374539653`, with interval `[0.6674166055, 1.0508116493]`.
The point estimate alone is not a selection.

## Reliability and A/A

- [x] Pilot: 336 of 336 attempts valid.
- [x] Main: 1,008 of 1,008 attempts valid.
- [x] Descriptive reference: 28 of 28 attempts valid.
- [x] A/A: 48 of 48 attempts valid.
- [x] Profile canaries: 3 of 3 attempts valid.
- [x] Total: 1,423 of 1,423 harness attempts and 348 of 348 blocks valid.
- [x] No invalid attempt, timeout, nonzero harness exit, or zero-duration total sample occurred.
- [x] A/A used the identical adaptive artifact with six `ABBA` and six `BAAB` blocks.
- [x] A/A mechanical integrity is separate from its unadjusted null diagnostic.
- [x] The A/A result is not presented as a universal noise floor.

## Mechanism review

- [x] The linked image retained all seven required symbol substrings.
- [x] Symbols and complete disassembly were captured from the measured image.
- [x] Three same-image dynamic-profile targets were attempted.
- [x] All three failed with `xctrace` return code 2 and no validated target record.
- [x] Failed trace bundles are retained but are not dynamic mechanism evidence.
- [x] Elapsed time, derived payload and work, observed code shape, and inferred mechanisms remain separate.

Dense also had the lowest composite time in the long-run cell even though the
adaptive form used run containers. The timing is measured. Any explanation in
terms of dense word-wise intersection, cache behavior, branch behavior,
allocation, or instruction mix remains an inference because dynamic profiling
was unavailable.

## Archive and publication review

- [x] The external archive contains all eight complete and incomplete run bundles.
- [x] Gzip integrity, tar enumeration, safe paths, unique member names, clean extraction, and extracted-copy comparison passed.
- [x] All 7,025 embedded manifest entries passed SHA-256 verification.
- [x] Archive SHA-256, bytes, members, run IDs, manifest hashes, source, image, host, and toolchain match the receipt.
- [x] `RESULTS.json` contains only compact aggregates.
- [x] The repository contains no raw run directory, executable, profiler dump, stdout, stderr, or archive.

## Scope

The primary conclusion applies to one fixed synthetic composite workload
family, source tree
`1100309780c57db157bb6279395995ea5c09c5213481896b64b5fccca07f285d`,
linked image
`d1e6b77120fd041f617efcfd1bba4110eefdba0090bf0d3745913b0d199f75d9`,
and the recorded Apple M1 Pro window. It does not establish a universal
representation winner, a production latency claim, or a result for another
host, build, allocator, distribution, operation mix, or concurrency level.
