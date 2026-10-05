# Measurements

Exact committed source: `ba617e8e93c08767e94d4da33b28693222c51449`. The source, runner and protocol hashes in
EVIDENCE_RECEIPT.json match the campaign. Final documentation/receipt commits
change no measured Rust, runner, or protocol input. A later review commit in
the same pull request changes only the bench binary's argument-count check
(malformed arguments now report usage on stderr with exit 2; the no-argument
`cargo test` invocation still exits 0) and adds a `[4, 4]` contract fixture;
the library, timed path, runner, and protocol are unchanged, and the receipt
hashes refer to the measured commit.

Supplementary local machine: b0f1d8752aba, Apple M1 Pro, arm64, Darwin 25.6.0,
10 available CPUs, no affinity, Rust 1.93.0/LLVM 21.1.8. Flags: edition 2024,
opt-level 3, debuginfo 1; default target features, unwind panic, no LTO/PGO.
The environment's target_cfg is default discovery. The external actual-build
cfg uses the exact optimization flags and has no debug_assertions.

All 624 independent process attempts completed. Values below are geometric
mean nanoseconds per query. Each cell has 12 processes. RESULTS.json includes
process min/max and simultaneous paired ratio intervals. Selection requires
all competitor/point-fastest lower bounds above 1.05; unresolved is an evidence
limit, not a proven equality.

| Workload | Linear | Binary | Select | Gallop | Standard | Selection |
|---|---:|---:|---:|---:|---:|---|
| tiny_uniform | 3.34 | 3.07 | 2.99 | 4.23 | 3.32 | unresolved: select, binary |
| medium_uniform | 708.90 | 15.60 | 10.65 | 17.82 | 8.08 | standard |
| large_uniform | 163376.77 | 72.76 | 52.79 | 98.67 | 48.01 | unresolved: standard, select |
| medium_low | 3.31 | 17.06 | 10.63 | 4.13 | 8.06 | linear |
| large_low | 3.23 | 54.73 | 33.22 | 4.00 | 30.44 | linear |
| medium_repeat | 699.98 | 15.86 | 10.68 | 16.74 | 8.35 | standard |
| medium_monotone | 694.81 | 15.90 | 10.96 | 16.88 | 8.31 | standard |
| medium_duplicates | 665.41 | 15.97 | 10.90 | 17.22 | 8.55 | unresolved: standard, select |
| medium_endpoints | 689.55 | 17.01 | 10.81 | 10.64 | 8.45 | standard |
| large_first | 165428.61 | 251.31 | 202.82 | 313.36 | 174.91 | unresolved: standard, select |

Examples: medium uniform selects standard: select/standard 1.318,
interval [1.279, 1.358]. Large warm uniform remains unresolved:
select/standard 1.100, [1.004, 1.205]. Large low-rank selects linear:
gallop/linear 1.239, [1.221, 1.256]. A/A B/A 1.017, [0.948, 1.091]. Mechanical
integrity passed; this wide null interval does not establish 5% calibration.
Intervals assume independent normally distributed block log ratios. The
fixed 128-key streams, one time window and lack of affinity limit transfer.

Linked-image disassembly is retained. Ordinary binary already uses csel and
csinc for its bound updates; select and standard also use csel. The hand-written
select retains bounds-check branches that the standard candidate omits in this
build. These are observed code-shape facts. No dynamic PMU counts, branch-site
attribution or causal speedup attribution is claimed. Gallop won no cell under
this experiment; a hint-based or wider-displacement workload is not tested.
All candidates reuse 8*n bytes and allocate zero within search; harness setup
and teardown are recorded separately. First-pass follows construction and is
not a cache-flushed or DRAM-resident measurement.

Required Linux follow-up completed on 2026-10-04 after authentication became
available. Both exact declared hostnames/architectures verified; all frozen
source/protocol/runner SHA256 values match the original source commit above.
Each host passed contracts/example and completed 624 independent processes.
Raw archives and file manifests were retrieved and verified before this update.

Arm: aarch64, 64 available CPUs, ARM r1p1, Rust 1.98.1.
xxl: dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com, x86_64,
Xeon Platinum 8488C, 192 available CPUs, Rust 1.98.0.
Both Linux 6.12.110-135.202.amzn2023, CPU0, opt-level3, debuginfo1,
compiler-default target features. Full environment receipts remain external.

`RESULTS-LINUX.json` contains the committed Linux results, separately from
supplementary local results. Arm selects linear for tiny/low-rank cases,
conditional-select for five medium workload shapes, standard for large first
use; large uniform standard/binary is unresolved. x86 selects linear for both
low-rank cases; every other cell remains unresolved under the declared criterion.

Arm A/A B/A 0.9823 [0.9353,1.0317]; x86 1.0592 [0.6354,1.7657].
The x86 control is wide and does not establish stable 5% calibration. Preserve
its unresolved cells, do not pool machines or substitute the earlier M1 ranking.
These model-based intervals and linked code do not prove candidate-only dynamic
branch mechanisms. No universal branchless/galloping advantage is claimed.
The required missing execution is now complete; publication and cleanup are
verified separately in the automation state.
