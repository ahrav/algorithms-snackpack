benchmark_required: true

# Benchmark decision

Timing is required for choosing among these safe locked scheduler models.
The baseline is one global FIFO. Each workload compares it with per-worker
owner-bottom, thief-top deques using the stated initial placement.

| Workload | Workers | Tasks | Leaf iterations | Placement |
| --- | ---: | ---: | ---: | --- |
| tiny_balanced | 2 | 256 | 0 | Round-robin |
| tiny_skewed | 2 | 256 | 0 | All on worker zero |
| medium_balanced | 4 | 16384 | 8 | Round-robin |
| medium_skewed | 4 | 16384 | 8 | All on worker zero |
| coarse_skewed | 4 | 4096 | 512 | All on worker zero |

The runner builds the example and then runs fresh processes:

```bash
python3 topics/028-work-stealing-deques/scripts/run_campaign.py \
  --repetitions 8 --out campaign.json
```

It runs `cargo clean --release -p topic028-work-stealing-deques` and then
`cargo build --release -p topic028-work-stealing-deques --example compare -vv
--message-format=json-render-diagnostics` in `--source` (default: this
repository; `--toolchain` adds `+<toolchain>`), and records the build
command, exit code and output. From that build it takes the example's path
from Cargo's `compiler-artifact` message (so a configured target triple or
redirected target directory is honoured); reads the wrapper, compiler and
linker executables, the `--target` and the environment Cargo passed from the
verbose `Running` line and hashes each executable, resolving bare names
through the `PATH` of that invocation (or the runner's) with relative entries
taken from the build directory, and including the Cargo executable Cargo
names as `CARGO`, which the `cargo -V` probe then runs; hashes a
custom target specification (a `.json` path, or a name found through the
`RUST_TARGET_PATH` the compiler saw, including one set by Cargo `[env]`
config) together with any linker that specification names, and asks the
compiler whether a named target is builtin, requiring a specification
otherwise; probes `rustc -Vv` and `cargo -V` on the executables Cargo
invoked, under the environment Cargo passed them;
hashes the built example and identifies the source tree it came from. The
build runs in its own process group and a timed-out build is killed with its
descendants.

Each of these ends the campaign as `invalid` with no rows: a build that
exits successfully without reporting an artifact (`invalid: build reported
no artifact`); more than one reported artifact or compiler invocation for
the example (`invalid: build artifact ambiguous`); an explicit `--binary`
that differs from the reported artifact, or, when the build reports none,
whose content did not change during the build (`invalid: build artifact
mismatch`); and any build, including one run through `--build-command`,
whose compiler chain cannot be read or hashed (`invalid: compiler identity
unavailable`), so a custom build command has to print Cargo's verbose
`Running` line for the example; a compiler or Cargo probe that prints
nothing counts as unavailable. Of the environment Cargo passes to rustc and of the
runtime variables the runner records, only an exact allowlist of known
build and runtime knobs (`CARGO`, `CARGO_PKG_*`, the `RUSTC*`, `RUSTFLAGS`
and `CARGO_BUILD_*` tool settings, `CARGO_PROFILE_*` profile keys,
`CARGO_TARGET_<triple>_{RUSTFLAGS,LINKER,RUNNER,AR}`, `PATH`, loader paths,
`RUST_TARGET_PATH`, `RUST_BACKTRACE`, `RUST_MIN_STACK`, `RUST_LOG`, and the
named glibc, macOS and allocator tunables) is recorded verbatim; any other
variable, including one with a provenance-like prefix such as
`RUST_API_TOKEN`, is recorded as a hash and redacted from the retained build
output. An empty `RUSTC_WRAPPER` or `RUSTC_WORKSPACE_WRAPPER` is recorded as
a disabled wrapper, as Cargo treats it. Files that compiler flags name as
inputs (`-C profile-use`, `-C profile-sample-use`, bare `@path` response
files, `@` response files and path-like objects or archives in link
arguments, the operands of file-taking linker switches such as `-T`,
`--script=` or `--version-script=` including those packed into `-Wl,`, the
files those response files name, four levels deep, and the native libraries
`-l` selects from explicit `-L` search directories, `+verbatim` names
resolved exactly) are hashed under
`rustflags.external_inputs` for the environment flag sources and under
`build.external_inputs` for the flags in Cargo's effective invocation
(which includes flags from Cargo config), and a missing one invalidates the
campaign. The builtin-target probe runs under the same invocation
environment as the compiler probe.
When no linker is named on the command line or in the target specification,
the inferred default (`cc`, or `link.exe` for MSVC targets) is resolved,
hashed and probed with `--version` as `environment.linker`. A named target
the compiler lists as builtin is taken as builtin even when a same-named
`.json` file is on the target path, as rustc does. Libraries injected by
`LD_PRELOAD`, `LD_AUDIT` or `DYLD_INSERT_LIBRARIES` are hashed under
`environment.runtime_libraries`, resolved against `--source`, which is also
the working directory of every measured process (recorded per row as
`cwd`); an unreadable one invalidates the campaign. Configuration files a
build command passes to Cargo with `--config <path>` are hashed under
`build.config_sha256`. The `rustc -Vv` and `--print target-list` probes run
through the wrapper chain Cargo used, so a wrapper that substitutes a backend
answers for that backend. A build command runs with the runner's own
environment; one that visibly changes it (`env`, `export`, `unset`, a
nested shell, `sudo`, a `NAME=value` prefix, matched by basename, through
wrappers such as `command`, `nohup` or `timeout`, and after attached shell
separators and redirections; once a wrapper opens a command segment every
word of that segment is examined) is refused so the identity probes observe
the same environment the build did. The artifact a build reports must come from
a manifest inside `--source`; any other manifest is an artifact mismatch. A flag string the shell
cannot parse is split on whitespace and recorded. The shared objects the loader resolves for the
pinned binary and for each injected library, from the source tree under the
runner's environment (`LD_LIBRARY_PATH` included), are hashed under
`environment.loaded_objects`
via `ldd`; without `ldd`, a set loader search variable invalidates the
campaign. Both the preload and loaded-object digests are recomputed after
the last row (`runtime_libraries_final`, `loaded_objects_final`) and any
difference ends the campaign as `invalid: loaded objects changed during the
campaign`. Beneath the target directory, only build-script `out/` trees are exempt
from the input scan, so a Cargo input anywhere else under the target
directory, `deps/` included, marks the tree dirty. Tracked symlinks that
resolve to a directory or outside the repository are recorded under
`environment.external_symlinks` and invalidate the campaign, since Cargo
follows them to files the commit never recorded. Git status and listings are read
NUL-delimited and decoded with `surrogateescape`, so paths with spaces or
bytes outside UTF-8 are filtered and recorded verbatim. Cargo's coloured output is stripped of ANSI sequences before the
`Running` line is parsed and redacted. `--out` is refused when either its
lexical or its resolved path names a tracked file or a Cargo input. `CARGO_PROFILE_*` values are kept verbatim only for the known
profile keys; other names are hashed. The document is written through a
uniquely named scratch file beside `--out`, so no fixed sibling path is
touched, and the dirty check lists untracked or ignored Cargo inputs placed
beneath the target directory (Cargo's own build-script outputs excepted). `--out` has to name a generated path: the runner refuses an
output path that is a symlink, a tracked file in `--source` (checked on the
path as written, before symlink resolution), or a path matching a Cargo
input pattern such as `build.rs`, and refuses a Cargo target
directory that contains tracked inputs, since either would hide source
changes from the dirty check. An identity file that exists but cannot be
read hashes to nothing and invalidates the campaign like a missing one.
Compiler overrides given as bare names resolve through `PATH` from the
build directory, like the tools in the compiler chain. The runner's own
test suite (`scripts/test_run_campaign.py`) runs in CI. The recorded runtime variables include the
macOS loader and allocator controls (`DYLD_*`, `Malloc*`).

The environment also records Cargo's compiler overrides (`RUSTC`,
`RUSTC_WRAPPER`, `RUSTC_WORKSPACE_WRAPPER` and their `CARGO_BUILD_*` forms,
each resolved against `--source` the way Cargo resolves them and recorded
with the hash of the named executable; an override that cannot be hashed
invalidates the campaign, and the `rustc` probe uses the override when set),
every `CARGO_PROFILE_*` override, the hash of a custom target specification
named by `CARGO_BUILD_TARGET` (an unreadable specification invalidates the
campaign), and the loader, allocator and Rust runtime variables (`LD_*`,
`GLIBC_*`, `MALLOC_*`, `RUST_*` and allocator tunables) that every measured
child inherits. Relative `RUST_TARGET_PATH` entries resolve against
`--source`, as rustc resolves them from the build directory. The source
check lists ignored files that match Cargo input patterns (`*.rs`,
`Cargo.toml`, `Cargo.lock`, `.cargo/config*`, `rust-toolchain*`) alongside
tracked changes and untracked files, and lists the files inside ignored
directories (such as a vendored path dependency) that match those patterns,
so an ignored `build.rs` or `vendor/dep/src/lib.rs` also marks the tree
dirty. `--skip-build` measures an existing binary and yields an `invalid`
document with no selection, since that binary's provenance is unverified.

## Protocol

The protocol was frozen before either Linux campaign: one discarded warmup
process per candidate and workload, then eight paired repetitions with strict
alternating order. The runner reports median, minimum and maximum plus the
eight paired steal/global ratios and the decision below for every cell.
The lowest median is a descriptive ranking. Select a candidate only when every
paired ratio lies beyond the same 3 percent boundary: below 1/1.03 for
stealing, above 1.03 for global FIFO. Otherwise record unresolved. Stop after
eight repetitions. The runner accepts other even counts of at least four for
smoke runs and reports their ratios, and it emits a selection only when the
count is exactly eight.

Every process is retained as a row that is written as `running` before
launch and finishes as `ok`, `failed` or `timeout`, together with its command,
exit code and captured output; a binary that cannot start is a `failed` row
carrying the launch error. The runner rewrites `campaign.json` before and
after each process, so an interrupted campaign keeps every completed
observation and records the attempt that was in flight. A repetition whose pair lacks an `ok` observation is
counted in `incomplete_repetitions` and forces `unresolved`; it is never
dropped from the file. Failures are reported per cell as `failures`. A warmup
that fails or times out is retained in `warmups`, counted per cell as
`warmup_failures`, and forces that cell to `unresolved`, since the protocol's
required warmup did not complete.

Each repetition also runs the global FIFO artifact twice under the labels `A`
and `B`. The four process slots follow a four-repetition cycle:
`[A, global, B, steal]`, `[steal, B, global, A]`, `[B, steal, A, global]`,
`[global, A, steal, B]`. Over one cycle each treatment candidate and each
control label occupies every slot once, so a host effect tied to slot
position or parity reaches both pairs alike, and each treatment candidate
follows a control run three times and itself once, so predecessor state is
balanced between the candidates. These identical-artifact controls pass
through the same scheduling,
labelling, parser, missing-data and ratio code as the treatments, and the
runner reports their eight paired A/B ratios as `aa_ratios`. Mechanical integrity is the `ok` status of every control
row, and a repetition whose control pair lacks an `ok` observation forces the
cell to `unresolved`; null calibration is the spread of the A/B ratios around 1. A full
campaign runs 10 warmup, 80 treatment and 80 control processes per host.

## Uncertainty and multiplicity

Pilot variance comes from the retained macOS smoke campaign in
`measurements/LOCAL_SMOKE.md`: observed ranges within a cell spanned up to
20x for tiny workloads and about 2x for coarse work, which motivated fresh
processes, warmups and a paired rule over a mean difference.

The independent unit is one measured process; eight paired repetitions per
cell are the analysis units, and a cell is one host and workload. The rule is
two-sided within a cell, since either arm can be selected. If the sign of
each paired ratio were an independent fair coin under the null, all eight
would fall on one side with probability 2 * (1/2)^8 = 1/128, and the
per-cell false-selection rate would be at most 0.78 percent before the 3
percent margin. That independence is an assumption, and this deterministic
alternating design does not establish it: a host-state drift or an
order effect spanning several repetitions moves paired ratios in the same
direction, and the rate is then unknown. The eight paired A/B control ratios
are the only diagnostic for such shared-direction effects; eight A/B ratios
on one side of 1 are evidence against the assumption for that cell, and the
runner reports that as `aa_null_check: failed` and withholds selection. The
benchmark family is the five cells per host, ten in total, inspected at one
fixed look. No multiplicity correction is applied, and the Bonferroni bounds
that would follow from the assumption, 5/128 per host and 10/128 overall,
hold only when it does. No confidence interval is reported, and the rule
carries no power claim: a true difference below 3 percent is expected to
remain unresolved. The rule is descriptive; the figures above characterise it
under a stated assumption and were derived from the rule, not from data.

## Boundary and identity

The binary timer starts after argument parsing and sequential oracle computation.
It includes queue allocation and population, worker startup, scheduling, task
work, shared accounting, joins and the final equality check. Compilation, process
startup, output and JSON writing remain outside that timer. Fresh threads and
queues are created in every measured invocation. Warmups warm executable pages
and host state. Scheduling is unpinned, and caches are uncontrolled.

The runner copies the built binary into a private directory before the
first process and executes that copy throughout, so every row runs the
artifact whose hash is recorded; it re-hashes the copy at the end and marks
the document `invalid` if the build failed, the binary's provenance is
unverified, the hash changed, the copy could not be made, the `rustc` and
`cargo` identity probes returned nothing, or the source tree is not
reproducible (no commit, a dirty or untracked path, or a frozen input that
could not be hashed); an `invalid` document carries no selection and keeps
every row and ratio. Child output is decoded with
replacement so a process that emits invalid UTF-8 is retained as a `failed`
row. It records the identity of each run in `environment`: source commit,
worktree status and dirtiness, SHA-256 of the frozen topic and workspace
inputs, SHA-256 of the measured binary, `rustc -Vv` from the compiler Cargo
invoked, `cargo -V`, every Cargo flag source (`CARGO_ENCODED_RUSTFLAGS`,
`CARGO_BUILD_RUSTFLAGS`, `RUSTFLAGS`, `CARGO_TARGET_<triple>_RUSTFLAGS`,
`CARGO_BUILD_TARGET`, and hashes of the `.cargo/config` files Cargo reads
from the source directory, its ancestors and Cargo home, with a relative
`CARGO_HOME` resolved against the source directory as Cargo resolves it),
host name, `uname -a`, CPU model and the first cpuinfo block, available CPUs
and the sorted CPU affinity set, plus the hash of the runner script itself
and whether it matches the copy in `--source`. The compiler probes run
inside `--source`, so a `rust-toolchain.toml` there selects the same
toolchain for the probes as for the build. The runner performs the build
itself, so the recorded flag sources are the ones the build saw. Each host
verifies the source
hashes before building the unchanged topic with the same edition, minimum Rust
version and lints. Run the exact committed source on both declared hosts.
Toolchain and host differences prevent an ISA-only interpretation.

## Scope and retained evidence

The experiment covers complete invocations of a mutex-based teaching model
with preloaded independent tasks and cyclic victim scans. Selection applies
within those limits. Dynamic spawning, parking, cancellation, lock-free
throughput, allocator bytes, hardware counters and production tail latency
require further experiments. Shared counters and setup contribute to the
measured costs, so elapsed time alone cannot attribute a mechanism.

The retained Linux campaign in `measurements/` ran commit
`200b157100cb9aa25c4622a545daaab9c329a627`. The runner at that commit wrote
its file once at the end, stopped on the first failed process and had no
control arm; every one of its 160 measured and 20 warmup processes passed the
oracle, so no observation was lost. Its paired ratios and selections were
computed from the retained rows with the same rule. It has no A/A ratios, so
null calibration for those hosts is absent until the campaign is rerun with
the current runner. The seven-repetition macOS smoke campaign is retained
separately; its 4:3 order split and broad ranges support descriptive local
observations.
