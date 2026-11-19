# Two-host measurement

Frozen source `0b171624485e871c10e5be5b3f21d1b6d3e4b675`. Arm ran Rust 1.98.1.
x86-64 ran Rust 1.99.0 on Intel Xeon Platinum 8488C. Each native release
build ran pinned to CPU 0. The [receipt](EVIDENCE_RECEIPT.json) records
hostnames, kernels, build flags and verified raw evidence.

Both hosts completed three contract groups, the focused example, and 390
fresh processes each. Each host ran 300 measured processes, 30 warmup
processes and 60 duplicate rolling controls. The runner enumerated all
subsets independently for all six full fixtures, checked every result and
recorded the complete fixture inputs. The first campaign stopped at a
Python 3.9 compatibility error and contributes no samples.

## Elapsed time in nanoseconds per solve

Each cell is median [minimum, maximum] across ten fresh processes. The
winner clears a five percent advantage against every rival in all ten
paired blocks. Each selected winner is bold.

| Host | Fixture | Exhaustive | Table | Rolling | Value | Frontier |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| arm | tiny | **26.7 [26.5, 29.7]** | 55.9 [55.9, 57.9] | 47.3 [47.1, 49.9] | 50.7 [50.4, 51.0] | 139.6 [138.6, 144.6] |
| arm | tight | 221.2 [220.5, 227.2] | 219.4 [218.7, 243.0] | **140.3 [139.5, 146.4]** | 17241.9 [17188.2, 17280.8] | 415.4 [409.1, 428.6] |
| arm | medium | 488568.4 [482512.4, 498150.0] | **1028.8 [1007.0, 1083.2]** | 4645.4 [4622.1, 4669.2] | 16866.7 [16808.4, 17090.6] | 3335.2 [3278.5, 3597.8] |
| arm | small-value | 232953.5 [230665.1, 239884.8] | 315534.1 [309947.6, 336643.9] | 1123990.8 [1117232.0, 1139351.0] | **405.8 [400.1, 423.0]** | 2050.1 [1975.3, 2158.1] |
| arm | wide-sparse | 6696.9 [6482.4, 7229.9] | 289514.8 [283364.1, 300719.5] | 885451.1 [881784.5, 895255.8] | 536936.3 [533285.6, 542978.9] | **1486.5 [1445.6, 2208.9]** |
| arm | wide-dense | 217960.4 [214987.5, 219404.6] | 387660.5 [385038.7, 448337.7] | 1163879.2 [1145528.6, 1189457.7] | **124163.8 [122621.3, 126482.4]** | 487893.4 [484461.2, 494147.9] |
| x86 | tiny | **34.3 [33.6, 36.6]** | 71.8 [69.5, 72.9] | 49.5 [46.1, 50.2] | 53.7 [51.6, 54.3] | 181.2 [179.0, 185.2] |
| x86 | tight | 301.5 [292.4, 360.4] | 314.2 [307.2, 329.0] | **134.2 [130.6, 136.3]** | 15449.7 [15155.8, 15699.3] | 587.5 [569.0, 602.8] |
| x86 | medium | 592177.5 [587206.3, 599037.6] | **1225.2 [1084.4, 1255.7]** | 4789.7 [4635.3, 5385.9] | 14772.1 [12958.4, 15042.3] | 4464.9 [3917.1, 4615.4] |
| x86 | small-value | 284295.9 [276570.7, 291698.4] | 500297.9 [476919.5, 545989.4] | 965631.9 [927775.2, 1001148.0] | **407.9 [368.4, 496.0]** | 2746.5 [2627.0, 3185.0] |
| x86 | wide-sparse | 8524.9 [8434.7, 8822.0] | 465263.3 [446503.2, 479206.0] | 739300.3 [715544.5, 774122.9] | 449772.9 [436517.9, 466694.4] | **1926.4 [1787.2, 2092.8]** |
| x86 | wide-dense | 187376.7 [184853.8, 193651.2] | 487424.1 [470814.6, 559161.1] | 715361.2 [710603.0, 727667.9] | **65720.6 [64182.7, 68301.2]** | 456960.7 [454770.6, 663711.8] |

## Logical work

Table and rolling visit the same take slots. Table additionally copies
each complete prior row. Counts are derived from the frozen fixture and
independent frontier replay. They describe work, not memory traffic.

| Fixture | Items | Capacity | Score | Table slots | Rolling slots | Take updates | Value slots | Value updates | Frontier inputs | Final frontier |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tiny | 3 | 6 | 12 | 28 | 7 | 12 | 18 | 16 | 12 | 6 |
| tight | 18 | 20 | 210 | 399 | 21 | 67 | 1554 | 12133 | 72 | 6 |
| medium | 18 | 212 | 1257 | 4047 | 213 | 3371 | 1554 | 12133 | 766 | 50 |
| small-value | 16 | 49151 | 25 | 835584 | 49152 | 711317 | 32 | 246 | 400 | 23 |
| wide-sparse | 16 | 49151 | 21756 | 835584 | 49152 | 544768 | 51542 | 344634 | 263 | 12 |
| wide-dense | 16 | 49151 | 49151 | 835584 | 49152 | 720897 | 65536 | 65535 | 114686 | 49152 |

The arm rolling A/A ratio ranged from 0.946 to 1.017.

The x86 rolling A/A ratio ranged from 0.870 to 1.110.

The assembly has packed maximum instructions in the full table update
on both hosts, Scalable Vector Extension (SVE) `umax` on Arm and `vpmaxud` on x86-64. The arrays for
source and destination come from disjoint rows. This shows emitted code.
The campaign provides no isolated instruction or hardware-counter
attribution. The x86 A/A spread is wider than the selection margin in
some cells, so use the paired results as fixture-specific evidence.

For these inputs, choose exhaustive for three items, rolling for the tight
budget, full table for the medium budget, a value row for narrow values and
the dense-frontier fixture, and a frontier for sparse reachable weights.
The dense frontier grows to 49,152 pairs. The value-axis solver performs
65,535 take updates because successive item values double. Its observed
win applies to this fixture. Storage and speed decisions also depend on
witness requirements, input distribution, and numeric bounds.

## Replay

Run from the repository root with Python 3.9 or later.

```bash
RUSTFLAGS='-C target-cpu=native' cargo build --release -p topic-054-dynamic-programming --example compare
taskset -c 0 python3 topics/054-dynamic-programming-state-space-design/measure.py target/release/examples/compare /tmp/topic054-results.json
```

Use a CPU from the current affinity mask. For the exact recorded build and
Linux commands, use the retained native driver and verified input archive.
Process launch, input construction and printing sit outside the solver
timer. Allocation and drop are within each solver call. The campaign
uses score-only requests and warmed repeated calls, with no claim about
isolated cold latency, reconstruction or deployed workloads.
