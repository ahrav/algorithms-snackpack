# Measurements

Exact committed source: `ba617e8e93c08767e94d4da33b28693222c51449`. The source, runner and protocol hashes in
EVIDENCE_RECEIPT.json match the campaign. Final documentation/receipt commits
change no measured Rust, runner, or protocol input.

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

Both required Linux hosts rejected initial execution and one retry due to
expired Midway authentication. xxl resolves to
`dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com`; live x86-64 verification did
not execute. No remote transfer or kernel/CPU/compiler receipt is claimed.
After authentication is restored, verify each declared hostname/architecture,
transfer and hash-check committed-replay.tar.gz, extract it into a new
topic-owned scratch directory, and run this exact script on each host:

```bash
python3 topics/007-binary-galloping-branch-aware-search/scripts/run.py --out /tmp/topic007-results
```

Retain the complete outputs and frozen hashes; update this open PR by the
non-force guarded publisher. Current local selections are not Linux results.
Required runs remain in curriculum pending_work; daily cadence continues.
