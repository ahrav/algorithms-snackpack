# Measurement record

status: complete_with_dynamic_profile_limit

The final fixed campaign, `all-committed-45b79fc`, completed 1,423 valid
harness attempts and 348 valid complete blocks on an Apple M1 Pro. All 21
primary contrasts, the descriptive reference phase, the 12-block
identical-artifact A/A campaign, and the linked-image inspection completed.
[RESULTS.json](RESULTS.json) contains the compact aggregate.
[BENCHMARK.md](../BENCHMARK.md) contains the frozen design.

The published aggregate is smaller than the frozen design's reporting list, in
two places. Each of the 21 `primary_results` rows stops at the composite-time
ratio, simultaneous interval, block count, and classification; it omits the
build, contains, and intersection aggregates that `analyze_main` retains, so a
reader here cannot see which component drove a ranking. The `aa` object
likewise carries the mean ratio, the unadjusted two-sided 95% paired interval,
template balance, and the complete-block count, but not the 12 individual A/A
block log contrasts or the label-by-position summaries.

Both sets exist only in the archived `analysis/main.json` and
`analysis/aa.json` for run `all-committed-45b79fc`, which is outside this
repository. Treat the published rows as the decision aggregate and those
diagnostics as archive-only. [REVIEW.md](REVIEW.md) tracks both omissions.

The primary estimand is the geometric mean of complete-block composite-time
ratios, oriented as candidate B divided by candidate A. The composite is the
predeclared build, membership, and intersection batch for each cell. The
Bonferroni intervals were constructed for at least 95% simultaneous coverage
of the 21-member family under the frozen paired-t assumptions. Those
assumptions treat complete-block log contrasts as independent, identically
distributed, and approximately normal. Balanced `ABBA`/`BAAB` blocks address
additive linear position drift only when treatment carryover is negligible.

The scoped portfolio is:

| Cell | Timing order or decision |
|---|---|
| `tiny_sparse_shuffled` | `dense < sorted < adaptive` |
| `wide_sparse_low_overlap` | `sorted < adaptive < dense` |
| `skewed_sparse` | Sorted versus adaptive is unresolved; both are lower than dense. |
| `dense_local_high_overlap` | `dense < adaptive < sorted` |
| `dense_wide_medium_overlap` | `dense < adaptive < sorted` |
| `long_runs` | `dense < adaptive < sorted` |
| `mixed_chunk_shapes` | `dense < adaptive < sorted` |

The symbols `<` above mean lower composite time only for this exact workload,
binary, host, and collection window. They are not universal representation
rankings. Twenty contrasts crossed a simultaneous symmetric `1.05` factor
boundary. Sorted versus adaptive in the skewed sparse cell did not.

The A/A mechanical checks passed. Its B/A estimate was `0.9966507143`, with
an unadjusted 95% interval of `[0.9810100112, 1.0125407845]`. This is a
diagnostic for this campaign, not a universal noise floor.

## Reproduction commands

The runner requires an absolute output path outside the repository that does
not already exist:

```bash
python3 topics/003-density-adaptive-bitmaps/scripts/run_experiment.py plan
python3 topics/003-density-adaptive-bitmaps/scripts/run_experiment.py self-check
python3 topics/003-density-adaptive-bitmaps/scripts/run_experiment.py quick \
  --output-dir /absolute/external/new-directory
python3 topics/003-density-adaptive-bitmaps/scripts/run_experiment.py all \
  --output-dir /absolute/external/new-directory
```

The measured source commit was
`45b79fccf8d52cb892b602329a4e0a7d935a6f00`. Its source-tree digest was
`1100309780c57db157bb6279395995ea5c09c5213481896b64b5fccca07f285d`.
The release benchmark image, built with `-C target-cpu=native`, had SHA-256
`d1e6b77120fd041f617efcfd1bba4110eefdba0090bf0d3745913b0d199f75d9`.

That digest and the archived snapshot cover the 14 files the runner enumerated
during collection. They exclude the repository's `rust-toolchain.toml`, which
selects the channel and profile the build resolves, so the digest alone does
not attest to it. Two independent records bound the toolchain for this
campaign: the receipt records the resolved `rustc 1.93.0 (254b59607 2026-01-19)`
with LLVM 21.1.8 and `cargo 1.93.0 (083ac5135 2025-12-15)`, and
`rust-toolchain.toml` at `45b79fc` is
`8bc51ecab82415fddd8489604f2424e137d71856e7f65cbdcfaa48850d794b46`, which
pins channel `1.93.0` and matches the file at the tip unchanged. Later
campaigns enumerate 15 files and carry the toolchain file inside the digest and
snapshot; the recorded evidence above is not restated, because a new campaign
is a new run with its own identity.

The source-to-image link rests on hashing the tree at sampled instants, not on
building from the archived snapshot. This campaign sampled the manifest twice,
before the build and after the final phase, as `source_tree_sampled_at` in the
receipt records; it had no post-build gate, so an edit confined to the build
itself would have passed. The current runner samples three times, adding one
immediately after the build and requiring all three to agree, which bounds that
window for later campaigns but says nothing about this one. Neither schedule can
exclude an edit made and reverted between two samples. Treat the digest as
evidence that the tree matched at those instants, and the recorded clean commit
as the source identity, rather than as proof that Cargo read exactly the
archived bytes.

## Raw evidence boundary

Raw attempts, binaries, stdout, stderr, disassembly, symbols, profiler bundles,
and source snapshots remain outside Git. The sole retained archive is:

```text
/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-003/topic003-runner-all-20260831.tar.gz
```

Its SHA-256 is
`aea801b861a9cec1ab54b82f82b1f7a7934b84ed95e515a0c0129044ab3bf0e7`.
The archive contains eight run bundles, including the intact record of one
early failed build attempt. All 7,025 embedded manifest entries verified after
a clean extraction. [EVIDENCE_RECEIPT.json](EVIDENCE_RECEIPT.json) records the
archive identity, member counts, run IDs, manifest hashes, source, linked
image, host, toolchain, and verification gates.

Do not commit raw run paths, executables, profiles, per-attempt files, run
bundles, or archives.

## Evidence limits

Measured evidence includes elapsed time and harness reliability. Payload sizes
and work counts are derived from the recorded representations and fixtures.
Final-image symbols and disassembly are observed code shape.

All three `xctrace` dynamic-profile attempts were retained as
`ATTEMPTED_UNAVAILABLE`. Each failed before producing a validated target
record. They provide no cache, branch, allocation, vectorization, or
instruction evidence. The host was unpinned, CPU frequency was uncontrolled
and unobserved, and allocator calls were not traced. Component timings and
reference costs are descriptive, not additional primary decisions.
