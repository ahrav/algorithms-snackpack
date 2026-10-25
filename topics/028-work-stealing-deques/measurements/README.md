# Linux measurements

Both required hosts ran exact committed source `200b157100cb9aa25c4622a545daaab9c329a627`.
All 160 measured and 20 discarded warmup processes passed their oracle checks.
Each cell has eight paired observations, with candidate order split 4:4.
The initial source also passed tests and three focused invocations on both hosts.

The timer covers scheduler queues, threads, work, shared counters, joins and
equality checking. The sequential oracle runs before timing. Read
[the frozen protocol](../BENCHMARK.md) for controls and selection. The runner
at the measured commit had no identical-artifact control arm, so these hosts
have no A/A ratios; `BENCHMARK.md` records that limit.

Times below are median [minimum, maximum] in microseconds. Ratios divide the
stealing duration by the global duration within each repetition. A selection
requires all eight ratios beyond the same 3 percent boundary. Lowest median
rankings remain descriptive when selection is unresolved. The "Selection"
column records the rule's outcome on this campaign; because the campaign had
no A/A control, every entry other than "Unresolved" is provisional until the
rerun described in `../BENCHMARK.md` passes its null check.

| Host | Workload | Global FIFO us | Locked stealing us | Paired ratio range | Lowest median | Selection |
| --- | --- | ---: | ---: | ---: | --- | --- |
| arm | tiny_balanced | 134.821 [123.773, 155.339] | 140.530 [129.422, 157.371] | 0.861..1.183 | Global | Unresolved |
| arm | tiny_skewed | 132.625 [120.706, 141.649] | 133.795 [129.967, 142.281] | 0.932..1.126 | Global | Unresolved |
| arm | medium_balanced | 2575.575 [2145.165, 2692.646] | 1685.665 [1627.761, 1765.101] | 0.613..0.776 | Steal | Steal |
| arm | medium_skewed | 2608.695 [2462.817, 2769.190] | 3284.731 [2752.066, 3457.002] | 1.117..1.382 | Global | Global |
| arm | coarse_skewed | 1028.186 [981.060, 1081.002] | 1226.745 [1201.693, 1359.304] | 1.114..1.320 | Global | Global |
| x86 | tiny_balanced | 143.404 [126.548, 151.554] | 146.865 [116.322, 193.435] | 0.833..1.359 | Global | Unresolved |
| x86 | tiny_skewed | 119.846 [109.003, 184.268] | 129.773 [108.919, 199.130] | 0.601..1.450 | Global | Unresolved |
| x86 | medium_balanced | 5070.350 [4022.070, 6078.882] | 2716.511 [1769.228, 3486.822] | 0.378..0.727 | Steal | Steal |
| x86 | medium_skewed | 5080.444 [3725.726, 5867.035] | 5357.240 [4596.361, 6841.520] | 0.884..1.610 | Global | Unresolved |
| x86 | coarse_skewed | 2368.871 [1840.257, 2639.322] | 1687.072 [1566.359, 1864.734] | 0.601..1.013 | Steal | Unresolved |

## Environment and limits

Arm: `dev-dsk-ahrav-2b-7dc7bd93.us-west-2.amazon.com`, aarch64, 64 available
CPUs, Rust 1.98.1. Linux 6.12.111-136.204.amzn2023.aarch64. The exposed CPU
identity is implementer `0x41`, part `0xd40`, variant `0x1`, revision 1;
the host supplies no model-name label.

x86: alias `xxl` resolved to `dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com`,
x86_64, Intel Xeon Platinum 8488C, 192 available CPUs, Rust 1.99.0.
Linux 6.12.111-136.204.amzn2023.x86_64.

Both builds use Cargo release defaults, generic targets and empty RUSTFLAGS.
Full uname, CPU identification, target configuration, binary SHA-256, symbol
list, replay workspace and test logs are retained in the external archives.
All seven executable/workspace input hashes match the measured commit. The
`target_cfg` field in the receipt came from `rustc --print cfg` without an
optimization flag, so its `debug_assertions` line describes that probe and
the release build has the flag off; the `target_*` lines identify the target.
`src/lib.rs`, `examples/compare.rs` and the crate `Cargo.toml` are byte for
byte identical to the measured commit. The campaign runner and the contract
tests changed after the campaign and are outside the measured binary.

## Reproduction

The runner builds the example itself and records the build, so two commands
serve two purposes.

Replay of a retained artifact. The retained hosts built commit `200b157` with
Rust 1.98.1 (Arm) and 1.99.0 (x86); the current tree's root manifest,
lockfile, runner and contract tests differ from `frozen_inputs_sha256`, so a
replay builds the retained revision in a separate worktree:

```bash
git worktree add ../topic028-replay 200b157100cb9aa25c4622a545daaab9c329a627
# Arm host (retained: 1.98.1); use --toolchain 1.99.0 on the x86 host.
python3 topics/028-work-stealing-deques/scripts/run_campaign.py \
  --source ../topic028-replay --toolchain 1.98.1 \
  --repetitions 8 --out campaign.json
```

`--toolchain` adds `+<toolchain>` to the build and routes the `rustc` and
`cargo` probes through `rustup run`, so `environment.rustc` reports the
compiler that built the binary. The run is a replay when these fields match
the host's entry in `EVIDENCE_RECEIPT.json`: `environment.binary_sha256`
against `binary_sha256`; `environment.rustc` against `environment.rustc`
after trimming surrounding whitespace on both sides (the receipt values end
with a newline and the runner strips its probe output);
`environment.hostname` against `environment.hostname`; `environment.uname_a`
against `environment.uname` (both are `uname -a` strings);
`environment.cpu_model` against the identity fields of the first processor
block in the receipt's `environment.cpu_identification` (`model name` on x86;
`CPU implementer`, `CPU variant`, `CPU part` and `CPU revision` on Arm, which
`cpu_model` joins as `CPU implementer 0x41, CPU variant 0x1, CPU part 0xd40,
CPU revision 1` for the retained Arm host); and `environment.available_cpus`
against `environment.available_cpus`. The selections are host-specific, so a
matching binary on a different machine is a new measurement.
`environment.source_*` describe the `--source` tree the binary was built
from, and `environment.runner_sha256` identifies the runner script that
drove the campaign.

New measurement of the current tree. The default `--source` is this
repository under the pinned toolchain; the run is a new campaign and its
selections replace the retained ones only with a fresh receipt.

```bash
python3 topics/028-work-stealing-deques/scripts/run_campaign.py \
  --repetitions 8 --out campaign.json
```

`campaign.json` holds the build record, every process row, the per-cell
paired ratios and the selection. The runner emits a selection only for the
frozen protocol's eight repetitions from a clean, identified source tree;
other runs report ratios and `unresolved`.

## Derived operation counts

These counts follow from reading `src/lib.rs`; they were not measured.
Each executed task costs one mutex acquisition for its successful pop or
steal. `global_fifo` performs two shared atomic read-modify-writes per task
(`checksum`, `completed`); `locked_work_stealing` performs three (`checksum`,
`completed`, `remaining`). Setup differs: `global_fifo` builds its queue
before wrapping it in a mutex, so it acquires zero locks while populating;
`locked_work_stealing` acquires one lock per task while populating, for
256, 16384 or 4096 single-threaded acquisitions inside the timed region. On
this Arm host that populate step took a median 4.5 us for 256 tasks and
259 us for 16384 tasks in an isolated probe, against measured cell medians of
134 us and 1686 us. Successful steals, empty victim scans and `yield_now`
calls are unmeasured; counting them requires instrumenting the measured
source and rerunning both hosts.

Timing includes setup and shared counters. Different hosts and compilers,
unpinned scheduling and uncontrolled caches limit interpretation. These
measurements select among the locked teaching candidates for the listed
workloads. Lock-free throughput, production tails, allocator bytes and causal
hardware mechanisms require separate evidence.

Raw archives and identities are in [EVIDENCE_RECEIPT.json](EVIDENCE_RECEIPT.json).
The original 70-process macOS campaign remains separate in LOCAL_SMOKE.md.
