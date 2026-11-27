# Measurements

Frozen executable commit `e7387828e052fe26abdeda7b00010439d69bfcea`. Both Linux hosts used the
workspace's Rust 1.93.0 in release mode with `-C target-cpu=native`. Arm
used 64 available AArch64 CPUs. x86-64 used 192 available Xeon Platinum
8488C CPUs. The processes used one thread and were not pinned to a core.
Kernel, CPU and compiler details are in the [raw receipt](EVIDENCE_RECEIPT.json).

Each host ran 231 processes (126 timed, 84 A/A controls and 21 warmups).
Every process passed the independent expected-count check for all eight
predicates. Both hosts passed the Rust suite and the example.

Median (min–max) nanoseconds per query. Build is median microseconds in
full / minmax / two order. A/A is the largest difference between adjacent
same-candidate control processes. See the [selection rule](../BENCHMARK.md).

| Host | Workload | Full | Min/max | Two ranges | Build µs | A/A | Winner |
|---|---|---:|---:|---:|---:|---:|---|
| arm | tiny | 33.6 (33.3–33.7) | 15.4 (15.3–16.2) | 17.4 (17.3–17.6) | 0.1 / 0.3 / 0.6 | 4.2% | minmax |
| arm | ordered | 117583.9 (117396.7–118233.8) | 1548.3 (1523.3–1631.9) | 2025.5 (2020.7–2027.6) | 0.1 / 31.2 / 336.1 | 4.0% | minmax |
| arm | ordered-wide-block | 117372.0 (117209.7–117945.4) | 1113.4 (1104.1–1127.0) | 1178.4 (1176.4–1306.9) | 0.1 / 24.2 / 345.0 | 0.8% | minmax |
| arm | shuffled | 117553.1 (117186.6–117670.3) | 100743.7 (100542.9–101091.9) | 62552.1 (62441.0–63024.8) | 0.1 / 31.1 / 1783.3 | 0.6% | two |
| arm | bimodal-gap | 117603.6 (117304.6–117717.6) | 121100.2 (120529.6–121171.6) | 1906.0 (1902.8–1909.7) | 0.1 / 34.1 / 1940.0 | 0.8% | two |
| arm | bimodal-hit | 117448.8 (116589.0–117928.7) | 121289.3 (120896.3–121478.8) | 121200.0 (120988.7–121643.7) | 0.1 / 33.6 / 1942.1 | 0.7% | unresolved |
| arm | wide | 117399.5 (117356.4–117615.1) | 105242.0 (105046.9–105756.0) | 105336.7 (105038.9–105625.5) | 0.1 / 32.4 / 336.1 | 0.3% | unresolved |
| x86 | tiny | 15.0 (14.1–17.7) | 16.2 (16.1–17.1) | 17.2 (17.2–18.1) | 0.1 / 0.5 / 1.0 | 32.5% | unresolved |
| x86 | ordered | 41217.3 (41028.2–41426.1) | 960.2 (888.7–1020.3) | 1358.5 (1355.8–1362.1) | 0.1 / 30.6 / 248.6 | 12.0% | minmax |
| x86 | ordered-wide-block | 41275.7 (41085.8–42509.9) | 455.9 (448.8–465.3) | 512.6 (510.3–514.5) | 0.1 / 27.0 / 213.1 | 3.3% | minmax |
| x86 | shuffled | 41102.6 (41027.1–41983.9) | 34978.4 (34723.2–35275.8) | 21723.0 (21635.5–21904.4) | 0.1 / 32.5 / 1572.7 | 1.7% | two |
| x86 | bimodal-gap | 41258.5 (41034.4–42195.5) | 41835.2 (41673.6–41950.5) | 1498.7 (1465.1–1667.8) | 0.1 / 31.6 / 1816.5 | 1.9% | two |
| x86 | bimodal-hit | 41090.3 (41018.8–41271.2) | 41914.2 (41717.0–42894.4) | 41900.8 (41669.3–42400.6) | 0.1 / 31.3 / 1822.6 | 3.0% | unresolved |
| x86 | wide | 41340.3 (41020.6–41728.9) | 36330.7 (36161.0–37096.7) | 36547.3 (36249.4–36646.6) | 0.1 / 33.5 / 250.6 | 1.2% | unresolved |

Both indexed candidates checked 256 or 2,048 rows per query in the two
ordered cases. In the permuted case min/max checked an average of 217,088
rows and two ranges checked 131,136, versus 262,144 for the full scan.
In the bimodal gap case two ranges checked zero rows and visited 1,024
summaries per query. All candidates checked 262,144 rows for the bimodal
hit workload. The wide workload admitted 226,560 rows on average for both
indexes. At width 256 the summary payload was 8 KiB for min/max and 16 KiB
for two ranges. At width 2,048 it was 1 KiB and 2 KiB.

The benchmark measures resident count queries, including summary traversal,
row checks and work counters. It excludes file reads, data generation,
index build and teardown from query timings. Build timing includes index
allocation. Separate linked disassembly is retained. Row counts show which
work was skipped. The timing data does not isolate a hardware cause.

From a full checkout run `python3 topics/061-block-range-indexes-and-lossy-summaries/benches/run.py /tmp/topic061-results`.
For native replay, extract the frozen path-limited archive and run
`bash topics/061-block-range-indexes-and-lossy-summaries/benches/replay.sh`
from the extracted root. The replay derives a one-topic workspace manifest
from the frozen root and retains the toolchain and lint settings.
