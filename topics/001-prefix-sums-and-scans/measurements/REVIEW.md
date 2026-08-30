# Benchmark experiment review

## Verdict

The fixed timing protocol completed without an invalid primary, pilot, A/A, or
distribution attempt. The raw manifests verify. The result supports only the
listed one-build, one-host, one-window cells. It does not support a universal
scan threshold.

## Design audit

- The protocol, 12-contrast family, seed `2026082901`, four excluded pilot
  blocks, 12 fixed main blocks, 12 A/A blocks, practical ratio `1.05`, and
  two-sided Bonferroni familywise alpha `0.05` were frozen before timing.
- One fresh benchmark process is the treatment-application unit. A complete
  four-position `ABBA` or `BAAB` block is the randomization and analysis unit.
  Three calls inside one process are subsamples.
- Six main blocks per contrast received each template. The fixed stop completed
  with no extension, early stop, or result-based exclusion.
- Rust and Python computed work counts independently before timing. Every
  process passed their equality canary, semantic recurrence, input checksum,
  output checksum, parser, and executable-hash checks.
- Every candidate used the same linked image and byte-identical input within a
  workload cell. Input construction, warmup, validation, and checksums stayed
  outside the primary timer. Output allocation stayed inside.
- The profile rerun used the same linked-image SHA-256 as the complete run.
  Disassembly, process-wide resource counters, and a macOS sample completed.

## Retained failures

The first quick run failed before build or timing because the adapter resolved
the `rustc` proxy symlink to `rustup`. Its complete failure bundle remains at
`runs/20260830T001429Z-quick-adapter-failure/`. The adapter now preserves the
proxy path and checks both `rustc` and Cargo identities during `self-check`.

The restricted full run retained four failed `/usr/bin/time -l` collections and
one failed macOS sample. The operating system denied `sysctl` and process
sampling. The same-source, same-binary profile-only rerun outside that
restriction completed all five paths. No failed profile record was overwritten
or promoted into timing evidence.

## A/A audit

The identical-artifact A/A control passed executable, parameter, parser,
checksum, schedule-balance, and attempt-count checks. Its diagnostic `B / A`
ratio was `1.108`, with an unadjusted 95% interval of `[0.735, 1.669]`. The
maximum absolute complete-block log contrast was `1.102`.

This is weak null calibration. It does not establish a label bias because the
interval includes 1, and it does not establish a noise floor because the run is
small and the interval is wide. The primary intervals remain the predeclared
analysis, but the A/A dispersion limits precision and argues for independent
replication before a production dispatch threshold.

## Evidence classes

Measured:
The complete run contains 12 complete main-block contrasts for each cell. The
profile rerun contains process-wide instructions, cycles, resident size, page
events, context switches, and one-second stack samples.

Derived:
The source-level wrapping-add, read, write, explicit `Vec`, thread-spawn, and
join counts come from the implementation. The two independent harness formulas
matched at every position.

Observed:
The linked-image disassembly contains the measured code. The macOS sample
contains `pthread_create`, `pthread_join`, and `_platform_memmove` below
`parallel_inclusive`.

Inferred:
Memory bandwidth, cache behavior, allocator cost inside the timed region, and
the fraction of latency caused by thread lifecycle remain inferred. The dynamic
counters cover the whole benchmark process, including setup, validation,
hashing, and teardown. They are not kernel-only counters.

## Generalization limits

The host was one 10-logical-CPU Apple M1 Pro running macOS 26.6.2. macOS exposed
no affinity mask, so the experiment makes no placement or isolation claim. The
artifact used Rust 1.93.0, LLVM 21.1.8, release mode, and
`-C target-cpu=native`. The run did not replicate builds, hosts, time windows,
block sizes, thread-pool implementations, or unlisted input sizes. The
distribution audit has one process per cell and remains descriptive.
