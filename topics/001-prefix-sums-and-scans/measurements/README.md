# Measurement evidence

The fixed benchmark and profiling protocol completed on 2026-08-29 Pacific
time. The protocol is frozen in [`../BENCHMARK.md`](../BENCHMARK.md). The raw
bundles under `measurements/runs/` remain authoritative when rounded values in
this file differ from their machine-readable values.

## Collected runs

- `20260830T001429Z-quick-adapter-failure` retains the first quick-run failure.
  The adapter resolved the `rustc` proxy symlink to `rustup`, so host-triple
  detection failed before build or timing. The later adapter preserves proxy
  paths and checks both tool identities in `self-check`.
- `20260830T001641Z-all-358de3b8` is the complete fixed run. It contains 835
  valid process attempts, 204 block records, all five phases, and a verified
  3,384-file manifest with SHA-256
  `e7785208c41af1a68966457ee6d74490a60ed1d795359f3ad0e5fee4be031cb5`.
- `20260830T001954Z-profile-f4f3fc40` repeats only the profile phase outside
  the restricted sandbox. All four `/usr/bin/time -l` collections, linked-image
  disassembly, and the one-second macOS sample completed. Its verified 56-file
  manifest has SHA-256
  `9a179a80472ea35cea8e6a863cafea5834b25a9a48be0329098e3ebfdb4af2df`.

The complete run and profile rerun used linked-image SHA-256
`0144d4ef720e2f721643a00f3b5d4775b65d753c34781efe6c1e7b714d9dd4a9`.
They used `rustc 1.93.0`, LLVM 21.1.8, Cargo 1.93.0, release mode, and
`-C target-cpu=native` for `aarch64-apple-darwin`. The host was an Apple M1 Pro
with 10 logical CPUs, macOS 26.6.2 build 25G83. macOS exposed no affinity mask,
so the run makes no placement or isolation claim.

## Workload

The confirmatory input uses `pattern=mixed`, seed `2026082901`, and exact
`i64::wrapping_add` semantics. The 12 contrast IDs, lengths, block size 16,384,
and worker counts 2, 4, and 8 are fixed in `BENCHMARK.md` and in
`scripts/run_experiment.py`.

The descriptive audit uses length 262,144 and the patterns `zero`, `constant`,
`ascending`, `alternating`, and `mixed`. It runs `linear`, `blocked` with block
size 16,384, and `parallel` with four workers. Audit observations do not enter
the 12-contrast confirmatory family.

## Counted work

The Rust harness computes these counts before any warmup or timed call. The
Python adapter computes the same formulas independently and rejects a mismatch.
Let `n` be the input length.

- `reference` performs `n(n+1)/2` wrapping additions and input-element reads.
  It writes `n` result elements and has one explicit result `Vec` allocation
  when `n > 0`.
- `linear` performs `max(n-1, 0)` wrapping additions. It reads and writes `n`
  elements and has one explicit result `Vec` allocation when `n > 0`.
- `blocked` uses `q = ceil(n / block_size)` and
  `b0 = min(n, block_size)`. Its wrapping-add count is `0` when `q = 0`,
  `n-1` when `q = 1`, and `2n-b0-2` otherwise. It reads `n` input elements.
  It writes `n + (n-b0)` output elements, reads `n-b0` output elements during
  offset repair, reads and writes `q` block totals, and uses two explicit `Vec`
  allocations when `n > 0`.
- `parallel` uses `k = min(workers, n)` and
  `b0 = ceil(n / k)` when `k > 0`. Its wrapping-add count is `0` when `k = 0`,
  `n-1` when `k = 1`, and `2n-b0-2` otherwise. For `k > 1`, it reads `n`
  input elements, writes `2n + (n-b0)` output or chunk elements, and reads
  `n + (n-b0)` chunk elements during repair and final concatenation. Its
  auxiliary reads and writes each count `k` chunk totals plus `k-1` offsets.
  It starts and joins `2k-1` scoped threads. The explicit `Vec` count is `k+6`,
  excluding allocator work inside the Rust thread runtime and operating system.

These counts describe source-level work. They are not instruction, cache,
allocation-trace, or bandwidth measurements.

## Timing boundary

Each schedule position is a fresh native benchmark process. Input construction,
one candidate warmup, semantic validation, and checksums are outside the primary
timer. Each of the three timed subsamples starts immediately before the candidate
call and stops immediately after the function returns. Result allocation is
inside. Result destruction is outside because the caller owns the returned
`Vec`.

One warmup call defines a post-warmup regime. It does not prove a general steady
state or thermal stability. Cold startup and long-duration thermal behavior are
outside the primary claim.

The position outcome is the median of the three subsamples. A complete
`ABBA` or `BAAB` block is one independent analysis unit. Inner calls and fresh
processes are not host, build, or time-window replicates.

## Run commands

Check the schedule, analysis implementation, and fixed counts without building
or timing the artifact:

```bash
python3 topics/001-prefix-sums-and-scans/scripts/run_experiment.py plan
python3 topics/001-prefix-sums-and-scans/scripts/run_experiment.py self-check
```

Run exploratory harness checks. Quick output cannot become confirmatory
evidence:

```bash
python3 topics/001-prefix-sums-and-scans/scripts/run_experiment.py quick
```

Run the complete fixed experiment:

```bash
python3 topics/001-prefix-sums-and-scans/scripts/run_experiment.py all
```

The separate `pilot`, `main`, `aa`, `distribution`, and `profile` phase commands
are useful for bounded execution and diagnosis. A phase-only directory is
partial evidence. It does not satisfy the complete protocol by itself.

On Linux, `--cpu-list` applies `taskset` and rejects a mask outside the granted
affinity. `taskset` does not claim CPU isolation. Without `--cpu-list`, the run
records the inherited affinity and makes no placement claim.

## Artifact identity and environment

The adapter builds `prefix_sums` once with Cargo JSON output. It resolves exactly
one native executable, copies the linked image into the run directory, and runs
that retained copy. The default compiler flag is `-C target-cpu=native`.
`metadata/build.json` records the actual `rustc -vV`, Cargo version, build argv,
build environment, profile, flags, host triple, executable paths, tool hashes,
and linked-image hash. Configured Cargo target runners or a non-host build target
make direct execution fail closed.

Each benchmark process receives this explicit environment:

```text
LANG=C
LC_ALL=C
TZ=UTC
RUST_BACKTRACE=1
```

The caller's secret-bearing environment is not inherited by benchmark
positions. Each attempt records its exact environment, cwd, expanded argv,
start and end time, return code, timeout or signal, executable hashes, parser
result, semantic canary, and input and output checksums.

## Raw bundle layout

Each complete or incomplete run contains:

```text
run-status.json
protocol.json
attempts.csv
blocks.csv
manifest.sha256
metadata/
  build.json
  build.stdout
  build.stderr
  benchmark-environment.json
  git.json
  host.json
  source-tree-before.json
  source-tree-after.json
artifacts/
  prefix_sums-<sha256>
raw/attempts/<attempt-id>/
  launch.json
  stdout.jsonl
  stderr.txt
  result.json
analysis/
  pilot.json
  main.json
  aa.json
  distribution.json
  profile.json
profiles/
```

Invalid, failed, partial, timed-out, reset-failed, parse-failed, checksum-failed,
and profile-failed records stay in the same bundle. Only an external `SIGINT`,
`SIGTERM`, or `SIGHUP` permits one whole-block retry. The original partial block
remains in `blocks.csv` and under `raw/attempts/`.

`manifest.sha256` covers every other retained file after the final run status is
written. The adapter verifies each entry immediately after creating the
manifest.

The topic's `.gitattributes` disables line-ending normalization and whitespace
diagnostics under `measurements/runs/`. Some raw CSV files use CRLF records, and
captured source-tree diffs can contain intentional trailing whitespace. These
files remain byte-for-byte evidence covered by their manifests. Authored source
and documentation remain subject to the repository's normal whitespace checks.

## Profile evidence

The profile phase keeps disassembly of the exact linked image. On Linux it also
attempts `perf stat` for cycles, instructions, branches, branch misses, cache
references, and cache misses. On macOS it retains `/usr/bin/time -l` output and
attaches `/usr/bin/sample` for one second to a 256-subsample parallel run. Tool
absence, permission denial, early process exit, timeout, and nonzero status are
retained as profile failures.

Assembly shows generated code. Counters and samples describe only their exact
recorded run. Elapsed time alone does not prove a cache, vectorization, allocator,
threading, or memory-bandwidth mechanism.

## Result and evidence boundary

The table reports `B / linear`. Each interval is the predeclared simultaneous
two-sided Bonferroni interval from 12 complete block log contrasts. A smaller
ratio favors `B`.

| Contrast | Point ratio | Simultaneous interval | Predeclared decision |
|---|---:|---:|---|
| reference, `n=64` | 6.720 | [5.503, 8.204] | linear faster |
| reference, `n=512` | 33.039 | [30.681, 35.577] | linear faster |
| reference, `n=4,096` | 338.831 | [328.574, 349.409] | linear faster |
| blocked, `n=4,096`, `B=16,384` | 1.369 | [1.327, 1.413] | linear faster |
| blocked, `n=262,144`, `B=16,384` | 1.083 | [0.755, 1.555] | no justified 5% winner |
| blocked, `n=4,194,304`, `B=16,384` | 1.324 | [1.100, 1.592] | linear faster |
| parallel 2, `n=262,144` | 1.585 | [1.010, 2.485] | no justified 5% winner |
| parallel 4, `n=262,144` | 1.298 | [0.901, 1.869] | no justified 5% winner |
| parallel 8, `n=262,144` | 1.488 | [0.987, 2.243] | no justified 5% winner |
| parallel 2, `n=4,194,304` | 1.082 | [0.906, 1.292] | no justified 5% winner |
| parallel 4, `n=4,194,304` | 0.754 | [0.572, 0.994] | no justified 5% winner |
| parallel 8, `n=4,194,304` | 0.614 | [0.483, 0.781] | parallel 8 faster |

The last decision means only that the interval lies below `1 / 1.05` in that
declared cell. The parallel-4 interval lies below 1 at its upper endpoint but
does not clear the predeclared 5% practical boundary.

The identical-artifact A/A path passed every mechanical check. Its diagnostic
point ratio was 1.108 with an unadjusted 95% interval of [0.735, 1.669]. The
wide null interval and maximum absolute block log contrast of 1.102 show high
run-window dispersion. They do not invalidate the fixed analysis mechanically,
but they limit precision and make replication important before adopting a
dispatch threshold.

The descriptive distribution audit used one process per cell and is not an
inferential comparison. Its variation, despite value-independent control flow,
also warns against treating a single timing as a mechanism.

The unrestricted profile rerun recorded whole-process resource totals for one
warmup plus three measured calls. The linear, blocked, and four-worker parallel
processes had maximum resident sizes of 68,829,184, 68,829,184, and 102,547,456
bytes. Those totals include input construction, validation, hashing, runtime,
and teardown, so they are not kernel-only allocation measurements. The sample
observed `pthread_create`, `pthread_join`, and `_platform_memmove` below
`parallel_inclusive`. This establishes that thread lifecycle, joins, and final
chunk copying execute in the measured artifact. It does not establish their
share of timed latency or a cache or bandwidth bottleneck.

The report keeps four claim types separate:

- measured position times, counters, resource totals, and samples;
- derived operation, read, write, allocation-site, and thread counts;
- observed linked-image assembly;
- inferred mechanisms and untested generalizations.

The experiment uses one source image, one compiler build, one host, and one run
window. It cannot estimate build, host, architecture, fleet, or long-term
variation. A/A mechanical integrity and null diagnostics do not prove absence
of bias or define a noise floor. The justified local portfolio is scalar for
the cells where linear cleared the boundary, eight-worker parallel only for
the 4,194,304-element cell, and no declared winner for the other cells. No
unmeasured input size, block size, worker count, thread pool, or machine inherits
that result.
