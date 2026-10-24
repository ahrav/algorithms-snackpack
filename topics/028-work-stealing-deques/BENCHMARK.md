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

Build once, then run fresh processes:

```bash
cargo build --release -p topic028-work-stealing-deques --example compare
python3 topics/028-work-stealing-deques/scripts/run_campaign.py \
  --binary target/release/examples/compare --repetitions 8 --out campaign.json
```

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
and `B`, with the order alternating per repetition. The treatment pair and
the control pair alternate first position by repetition parity, so each pair
occupies the first two process slots in four repetitions and the last two in
the other four, and a host effect tied to slot position reaches both pairs
alike. These identical-artifact controls pass through the same scheduling,
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

The runner copies `--binary` into a private directory before the first
process and executes that copy throughout, so every row runs the artifact
whose hash is recorded; it re-hashes the copy at the end and marks the
document `invalid` if the hash changed, the copy could not be made, the
`rustc` and `cargo` identity probes returned nothing, or the source tree is
not reproducible (no commit, a dirty or untracked path, or a frozen input
that could not be hashed); an `invalid` document carries no selection and
keeps every row and ratio. Child output is decoded with
replacement so a process that emits invalid UTF-8 is retained as a `failed`
row. It records the identity of each run
in `environment`: source commit
and worktree dirtiness, SHA-256 of the frozen topic and workspace inputs,
SHA-256 of the measured binary, `rustc -Vv`, `cargo -V`, `RUSTFLAGS`, host
name, uname, CPU model, available CPUs and the sorted CPU affinity set. Each host verifies the source
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
