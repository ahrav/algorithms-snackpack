# Exact-source native measurements

Frozen source 0941e07a0517518958495e7c241eb9bf5b06f2e0. All three executable
input blobs match both native runs. Every candidate verified the full independent
oracle multiset before timing. Each host passed 3 contract groups (including
12,348 exhaustive pairs), running example and 474 processes:396 measured main,
66 warmup,12 A/A. Archive and every member hash verified locally. The linked
binaries and generated assembly are retained externally through EVIDENCE_RECEIPT.

Arm host dev-dsk-ahrav-2b-7dc7bd93: aarch64,64 allowed CPUs, rustc1.98.1,
model1 steppingr1p1, Linux6.12.113-138.209.amzn2023. The system exposes ARM model
fields rather than a marketing model name. xxl resolved to
dev-dsk-ahrav-2c-32182091: x86-64,192 allowed CPUs, Intel Xeon Platinum8488C,
rustc1.99.0, Linux6.12.113-138.209.amzn2023. Both pinned CPU0; edition2024,
opt-level3,target-cpu=native. Full environment/features are retained externally.

The frozen selection screen requires >=5% advantage over every alternative in
every paired block. Dense wins10/11 Arm scan cells,9/11 x86 scan cells; direct
wins x86 dense-output case9; one cell per host remains unresolved. Direct wins
all22 build cells under its owned-clone construction boundary. Build and scan
are separate endpoints. Whole-lifetime rankings require an actual reuse count.

| Host | Case | n | r | z | Selection | Direct µs | Sparse µs | Dense µs |
|---|---:|---:|---:|---:|---|---:|---:|---:|
| arm | 0 | 64 | 2 | 0 | dense | 0.342 | 0.260 | 0.168 |
| arm | 1 | 64 | 32 | 0 | dense | 4.190 | 0.976 | 0.186 |
| arm | 2 | 32768 | 2 | 0 | dense | 191.537 | 130.023 | 86.451 |
| arm | 3 | 32768 | 32 | 0 | dense | 2798.347 | 613.371 | 97.572 |
| arm | 4 | 32768 | 128 | 0 | dense | 10552.689 | 1418.831 | 139.034 |
| arm | 5 | 262144 | 128 | 0 | dense | 84481.570 | 11255.618 | 1109.974 |
| arm | 6 | 32768 | 32 | 0 | dense | 3478.543 | 764.774 | 86.968 |
| arm | 7 | 32768 | 128 | 0 | dense | 13523.521 | 1984.014 | 93.285 |
| arm | 8 | 4096 | 8 | 32740 | unresolved | 92.470 | 114.020 | 85.709 |
| arm | 9 | 4096 | 32 | 130576 | dense | 435.331 | 414.243 | 379.196 |
| arm | 10 | 32768 | 32 | 1048080 | dense | 3496.764 | 3326.814 | 3094.238 |
| x86 | 0 | 64 | 2 | 0 | dense | 0.408 | 0.232 | 0.158 |
| x86 | 1 | 64 | 32 | 0 | dense | 3.707 | 0.584 | 0.215 |
| x86 | 2 | 32768 | 2 | 0 | dense | 217.771 | 108.961 | 91.956 |
| x86 | 3 | 32768 | 32 | 0 | dense | 2999.696 | 483.732 | 96.787 |
| x86 | 4 | 32768 | 128 | 0 | dense | 12092.322 | 1341.043 | 131.293 |
| x86 | 5 | 262144 | 128 | 0 | dense | 96079.922 | 10938.734 | 1046.369 |
| x86 | 6 | 32768 | 32 | 0 | dense | 3002.817 | 443.072 | 92.150 |
| x86 | 7 | 32768 | 128 | 0 | dense | 12046.476 | 1430.865 | 92.180 |
| x86 | 8 | 4096 | 8 | 32740 | dense | 80.265 | 80.366 | 72.734 |
| x86 | 9 | 4096 | 32 | 130576 | direct | 294.552 | 356.592 | 346.178 |
| x86 | 10 | 32768 | 32 | 1048080 | unresolved | 2379.267 | 2877.896 | 2805.376 |

Numbers are medians in microseconds per scan. results.json includes six-process
min/max dispersion, all build medians/ranges, table sizes and full cell selections.
For random32768/r32, dense median/range is97.572[93.248,100.444]us Arm and
96.787[96.606,96.928]us x86. Sparse is613.371/483.732us; direct2798.347/2999.696us.

Tiny n64/r2 dense search is0.168us Arm,0.158us x86, while construction is
8.365/12.041us. Direct construction is0.181/0.306us with search0.342/0.408us.
A median-only total-cost projection gives direct at one scan; dense overtakes
direct at47 scans on both hosts (crossover46.91 Arm,46.81 x86). This is a derived
projection, with no joint total-lifetime timing claim. Dense versus sparse
crossover is64 scans Arm,127 scans x86. Large random r128 tables occupy3,102,720
payload bytes.

Dense output case9 emits130,576 records: x86 direct294.552us versus dense346.178us.
Case10 emits1,048,080 records; its x86 outcome stays unresolved despite direct's
lower median. A/A direct/direct ratios range1.004..1.042 Arm,0.902..1.004 x86.
These are descriptive diagnostics. They establish no universal noise threshold.

Evidence limits: correctness-warmed deterministic synthetic input, one artifact
and host per architecture, different toolchains, no cold-cache control, no
production-library comparison, no total allocation/RSS/peak-memory measurement.
Build timing runs after oracle verification with the input text and both automata
resident in the process, so build medians carry that resident-state effect: x86
direct build for identical r32 pattern sets is1.543us at n64 (case1) and5.151us at
n32768 (case6). The A/A control reuses the direct treatment string, so it covers
scheduling, process order and parsing; the binary's candidate dispatch is outside
its coverage. The control has no discarded warmup block of its own: its twelve
retained processes run after the 462 main-schedule processes of the same binary,
so first-block effects within the control cell are part of its descriptive ratios.
results.json carries per-cell medians, min/max and selections; the
per-block paired ratios behind each selection live only in the external raw rows.
Assembly is retained for replay; hardware causal attribution remains unmeasured.
Transition n and dense payload 256*S*sizeof(usize) are source-derived counts.
Use the full results and memory budget to choose for a declared workload.

Replay: python3 measurements/run.py from this topic or the root command in README.
The runner prints its external temporary evidence directory and writes eight files:
the three copied inputs, the rlib, the experiment and unit-test binaries,
environment.json and raw.jsonl. tests/contract.rs runs through cargo test. The
aggregation into results.json and the archive's remaining members were produced
outside the runner. The receipt hashes bind lib.rs, experiment.rs and run.py at
commit 0941e07; run.py at HEAD adds failed-attempt retention with the same schedule
and binary inputs. All original
failed local documentation checks are preserved in daily outputs; corrected
focused checks and the full repository gates passed.
