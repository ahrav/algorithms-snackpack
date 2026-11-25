# Exact-source measurements

Executable source commit: `c7b59fecde0b95eb9915a63303ca8e573e04139c`.
Both pinned Linux hosts passed four tests, including 12,288 generated snapshot
views per host. All seven benchmark fixtures returned identical full vectors
on both hosts. Each host ran 224 timed, 112 identical-binary control, 28 warmup,
and seven correctness processes. Source, runner, linked binaries, raw data, and
machine identities are in [the evidence receipt](EVIDENCE_RECEIPT.json).

Each timing cell lists median [first quartile, third quartile] in microseconds
per complete scan. The paired rule in [the benchmark plan](../BENCHMARK.md)
governs selection. Unresolved cells retain the fastest sample median below.

| Host | Fixture | Replay | Flat | Heap | Loser | Selection |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| arm | tiny | 1.010 [1.009, 1.013] | 0.434 [0.434, 0.435] | 0.499 [0.497, 0.501] | 0.537 [0.536, 0.540] | flat |
| arm | low-fan-in | 1313.837 [1312.811, 1315.774] | 584.259 [581.916, 584.996] | 300.890 [298.924, 301.715] | 280.205 [279.341, 281.702] | loser |
| arm | mixed-8 | 1315.608 [1312.881, 1318.644] | 597.098 [595.224, 598.384] | 516.886 [514.985, 517.529] | 627.291 [626.489, 627.909] | heap |
| arm | mixed-32 | 1317.900 [1316.107, 1319.917] | 608.371 [605.143, 609.272] | 747.414 [746.821, 749.970] | 983.231 [981.257, 986.358] | flat |
| arm | overlap-32 | 1371.266 [1370.311, 1373.990] | 619.722 [616.463, 624.685] | 850.624 [848.642, 857.259] | 1099.109 [1098.202, 1101.174] | flat |
| arm | skew-64 | 1345.020 [1342.303, 1346.642] | 558.831 [557.790, 560.510] | 317.309 [314.780, 319.863] | 511.024 [509.953, 512.349] | heap |
| arm | disjoint-32 | 1315.416 [1312.443, 1320.256] | 102.068 [101.532, 103.693] | 202.250 [201.259, 204.756] | 333.039 [332.712, 334.301] | flat |
| x86 | tiny | 1.110 [0.997, 1.126] | 0.517 [0.467, 0.532] | 0.543 [0.496, 0.559] | 0.703 [0.624, 0.716] | flat |
| x86 | low-fan-in | 1663.016 [1592.154, 1683.721] | 716.669 [659.908, 723.924] | 323.427 [300.246, 350.075] | 359.234 [312.187, 367.812] | unresolved |
| x86 | mixed-8 | 1420.216 [1406.915, 1513.541] | 558.155 [549.171, 621.016] | 521.936 [511.554, 532.266] | 773.892 [695.146, 784.123] | heap |
| x86 | mixed-32 | 1649.603 [1634.490, 1662.263] | 714.896 [688.291, 728.820] | 711.209 [706.233, 720.510] | 1294.398 [1236.618, 1315.764] | unresolved |
| x86 | overlap-32 | 1791.221 [1650.462, 1932.312] | 635.058 [630.608, 666.380] | 752.652 [747.803, 833.368] | 1221.577 [1192.744, 1309.266] | flat |
| x86 | skew-64 | 1385.074 [1373.214, 1425.500] | 503.501 [495.013, 545.665] | 277.289 [274.462, 290.111] | 557.430 [550.654, 567.139] | heap |
| x86 | disjoint-32 | 1664.459 [1648.848, 1680.222] | 130.788 [128.672, 133.841] | 239.789 [238.370, 244.697] | 487.638 [468.011, 493.425] | flat |

## Point selector work and allocator calls

Point comparison counts are identical across hosts. Flat sorting recognized
the already ordered disjoint input. The loser tree used fewer contests than
the heap in mixed-32, while its complete scans took longer on both hosts.
Comparisons count one head decision, including absent-head decisions in the
loser tree. An allocation diagnostic on the exact linked binaries matched
on both hosts. It interposed glibc allocation calls during one untimed scan
per arm and workload and observed two monotonic timer reads in each process.
Each allocation cell is new calls plus resize calls. Aligned allocations were
zero, and non-null frees matched new calls. This is call count, not peak bytes.

| Fixture | Returned rows | Flat comparisons | Heap comparisons | Loser contests | Replay alloc | Flat alloc | Heap alloc | Loser alloc |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tiny | 17 | 82 | 31 | 33 | 6+3 | 4+3 | 4+3 | 5+3 |
| low-fan-in | 5760 | 266366 | 16382 | 16385 | 792+13 | 4+15 | 4+15 | 5+15 |
| mixed-8 | 5760 | 235926 | 65448 | 49159 | 792+13 | 4+15 | 4+16 | 5+15 |
| mixed-32 | 5760 | 240774 | 120949 | 81951 | 792+13 | 4+15 | 4+18 | 5+15 |
| overlap-32 | 39 | 241390 | 120881 | 81951 | 1256+16 | 4+22 | 4+25 | 5+22 |
| skew-64 | 5760 | 244104 | 47626 | 98367 | 792+13 | 4+15 | 4+19 | 5+15 |
| disjoint-32 | 5792 | 16383 | 31324 | 81951 | 805+13 | 4+15 | 4+18 | 5+15 |

Use flat sort on these tiny, overlap-dense and disjoint fixtures when point
materialization fits. Use the heap on the mixed eight-run and skewed 64-run
fixtures. Use the loser tree for this measured Arm low-fan-in fixture.
The mixed 32-run fixture favored flat sort on Arm. The x86
low-fan-in and mixed 32-run cases were unresolved. Full paired ratios and A/A
dispersion are archived. Median A/A ratios ranged from 0.986 to 1.001 on Arm
and 0.980 to 1.012 on x86. Both native binaries were disassembled. The
timings do not isolate the cause of differences in branch or cache behavior.
Different compilers and CPUs rule out an architecture-only comparison.

The native runner builds the exact source using rustc with optimization and
target-cpu=native, pins the process family to an available CPU and archives
environment and raw samples. Source and build flags are retained. Startup
and input generation are present in process wall time only. There is no cold
storage, repeated-seek, compaction or end-to-end engine result.
