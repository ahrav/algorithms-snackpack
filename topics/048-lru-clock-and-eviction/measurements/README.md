# Measured results and replay boundary

Frozen source d6d74abc7bd7a437551e04ce13d1dc76d58324be. Each host ran 460
processes: 384 main measurements, 64 warmups and 12 A/A controls. Three native
contract groups and the doctest passed. Full hit streams, ordered residents,
CLOCK bits/hand and work counts passed independent Python oracles on the host
and again after archive retrieval. All 12 frozen topic files, archive hashes,
linked image hashes, schedules and stdout records were checked.

Arm: aarch64, Linux6.12.113-138.209.amzn2023, CPU implementer0x41/part0xd40,
64 online CPUs, Rust1.98.1 LLVM22.1.8. x86: x86_64, same kernel release,
Intel Xeon Platinum8488C, 192 online CPUs, Rust1.99.0 LLVM23.1.1. Both pinned
CPU0. Flags: --edition=2024 -C opt-level=3 -C target-cpu=native. Full target
features, hostnames and CPU inventory are retained externally. Compiler versions
differ, so cross-host timing differences have several possible causes.

Request CPU screen: all six paired blocks must beat every alternative by >5%.
Minimum misses refer to these four tested candidates. All equal-miss ties are
retained. All four use direct dense-key indexing; backend, value storage and
concurrency require separate experiments. Warm CPU caches, logically empty
cache state, one trace/process. Setup and output lie outside request timing.

| Workload | C | Minimum misses and policies | Arm CPU screen | x86 CPU screen |
|---|---:|---|---|---|
| tiny | 3 | 6: segmented | unresolved | scan |
| hot | 4 | 1815: scan, linked, segmented | unresolved | unresolved |
| scan | 4 | 8194: segmented | clock | unresolved |
| loop | 4 | 32768: scan, linked, clock, segmented | clock | unresolved |
| phase | 4 | 8: scan, linked, clock, segmented | unresolved | unresolved |
| uniform | 4 | 24916: scan, linked | clock | clock |
| hot | 64 | 1803: scan, linked, segmented | clock | clock |
| scan | 64 | 8224: segmented | clock | clock |
| loop | 64 | 32768: scan, linked, clock, segmented | clock | unresolved |
| phase | 64 | 128: scan, linked, clock, segmented | unresolved | unresolved |
| uniform | 64 | 24578: segmented | linked | unresolved |
| hot | 1024 | 2263: segmented | clock | clock |
| scan | 1024 | 8704: segmented | clock | clock |
| loop | 1024 | 32768: scan, linked, clock, segmented | clock | unresolved |
| phase | 1024 | 2048: scan, linked, clock, segmented | clock | clock |
| uniform | 1024 | 24711: scan, linked | unresolved | unresolved |

## CPU and setup dispersion

Times are microseconds per complete trace. Each row is median [min,max] across
six fresh processes. These ranges describe the observed runs.

### arm

| Trace | C | Policy | Misses | Request us | Setup us |
|---|---:|---|---:|---|---|
| tiny | 3 | scan | 7 | 0.492 [0.489,0.587] | 0.568 [0.531,0.630] |
| tiny | 3 | linked | 7 | 0.633 [0.577,0.937] | 0.534 [0.494,0.613] |
| tiny | 3 | clock | 7 | 0.487 [0.374,0.533] | 0.529 [0.443,0.822] |
| tiny | 3 | segmented | 6 | 0.716 [0.605,0.908] | 0.489 [0.368,0.655] |
| hot | 4 | scan | 1815 | 245.827 [236.398,251.785] | 0.719 [0.600,0.882] |
| hot | 4 | linked | 1815 | 415.466 [381.242,420.488] | 1.858 [1.760,1.925] |
| hot | 4 | clock | 1915 | 229.234 [221.869,236.741] | 0.604 [0.489,0.860] |
| hot | 4 | segmented | 1815 | 398.757 [385.503,416.040] | 1.718 [1.569,2.133] |
| scan | 4 | scan | 10242 | 338.113 [334.233,353.987] | 0.698 [0.572,0.731] |
| scan | 4 | linked | 10242 | 382.817 [374.566,406.633] | 1.903 [1.580,2.156] |
| scan | 4 | clock | 10242 | 233.397 [227.789,236.831] | 0.673 [0.489,0.703] |
| scan | 4 | segmented | 8194 | 398.519 [379.381,427.523] | 1.874 [1.733,9.843] |
| loop | 4 | scan | 32768 | 598.107 [561.501,705.892] | 0.612 [0.544,0.740] |
| loop | 4 | linked | 32768 | 406.301 [390.998,408.315] | 0.577 [0.488,0.614] |
| loop | 4 | clock | 32768 | 254.026 [248.511,254.632] | 0.696 [0.595,0.861] |
| loop | 4 | segmented | 32768 | 405.800 [370.617,574.153] | 0.568 [0.520,0.698] |
| phase | 4 | scan | 8 | 221.126 [219.219,241.279] | 0.643 [0.540,0.900] |
| phase | 4 | linked | 8 | 388.738 [368.357,425.146] | 0.517 [0.505,0.594] |
| phase | 4 | clock | 8 | 211.492 [198.315,222.311] | 0.581 [0.526,0.770] |
| phase | 4 | segmented | 8 | 401.071 [368.392,413.848] | 0.544 [0.432,0.629] |
| uniform | 4 | scan | 24916 | 544.561 [540.254,554.942] | 0.705 [0.609,0.818] |
| uniform | 4 | linked | 24916 | 456.555 [448.300,465.236] | 1.728 [1.611,2.886] |
| uniform | 4 | clock | 24941 | 332.049 [329.472,338.855] | 0.685 [0.613,0.739] |
| uniform | 4 | segmented | 25011 | 527.694 [510.125,541.606] | 1.621 [1.502,1.967] |
| hot | 64 | scan | 1803 | 511.200 [501.976,514.104] | 1.837 [1.736,2.335] |
| hot | 64 | linked | 1803 | 418.312 [411.078,468.140] | 2.856 [2.656,3.243] |
| hot | 64 | clock | 1823 | 231.981 [224.995,232.711] | 1.744 [1.630,1.921] |
| hot | 64 | segmented | 1803 | 404.808 [386.510,430.528] | 2.797 [2.603,11.592] |
| scan | 64 | scan | 10272 | 1690.823 [1686.957,1716.806] | 2.631 [2.291,2.711] |
| scan | 64 | linked | 10272 | 390.462 [373.891,452.121] | 3.577 [3.246,4.080] |
| scan | 64 | clock | 10272 | 236.406 [224.073,246.786] | 1.621 [1.550,2.007] |
| scan | 64 | segmented | 8224 | 403.419 [396.668,415.247] | 3.422 [3.249,3.836] |
| loop | 64 | scan | 32768 | 4915.690 [4895.908,4963.457] | 1.744 [1.718,10.671] |
| loop | 64 | linked | 32768 | 397.130 [388.469,415.997] | 1.674 [1.433,9.506] |
| loop | 64 | clock | 32768 | 282.376 [275.127,284.530] | 1.764 [1.631,11.807] |
| loop | 64 | segmented | 32768 | 400.480 [370.892,727.206] | 1.671 [1.593,2.072] |
| phase | 64 | scan | 128 | 231.488 [220.298,583.384] | 2.034 [1.711,2.417] |
| phase | 64 | linked | 128 | 405.229 [377.839,413.362] | 2.801 [2.724,3.633] |
| phase | 64 | clock | 128 | 205.934 [196.662,234.034] | 2.013 [1.875,2.386] |
| phase | 64 | segmented | 128 | 387.030 [373.739,408.291] | 2.865 [2.756,3.279] |
| uniform | 64 | scan | 24612 | 3833.446 [3826.575,3871.316] | 1.848 [1.539,2.538] |
| uniform | 64 | linked | 24612 | 432.663 [424.272,451.578] | 2.638 [2.471,2.798] |
| uniform | 64 | clock | 24586 | 497.260 [486.597,1353.856] | 1.784 [1.420,2.521] |
| uniform | 64 | segmented | 24578 | 504.286 [500.814,1093.338] | 2.728 [2.473,2.984] |
| hot | 1024 | scan | 2264 | 3340.419 [3331.323,3606.231] | 13.592 [12.722,14.268] |
| hot | 1024 | linked | 2264 | 418.449 [390.030,438.915] | 33.791 [31.585,45.243] |
| hot | 1024 | clock | 2515 | 227.919 [218.395,238.558] | 12.991 [12.166,14.413] |
| hot | 1024 | segmented | 2263 | 408.279 [397.875,431.662] | 34.881 [33.070,45.092] |
| scan | 1024 | scan | 10752 | 23945.809 [23688.457,24400.132] | 17.782 [16.921,18.812] |
| scan | 1024 | linked | 10752 | 395.925 [385.009,411.434] | 49.827 [44.208,56.840] |
| scan | 1024 | clock | 10752 | 235.590 [226.944,240.630] | 16.953 [16.126,17.553] |
| scan | 1024 | segmented | 8704 | 387.419 [372.217,405.480] | 52.906 [44.346,58.545] |
| loop | 1024 | scan | 32768 | 76850.076 [76480.524,77134.861] | 6.192 [6.148,6.633] |
| loop | 1024 | linked | 32768 | 404.072 [368.732,412.859] | 6.521 [6.214,6.887] |
| loop | 1024 | clock | 32768 | 286.592 [279.097,289.438] | 5.295 [5.198,5.869] |
| loop | 1024 | segmented | 32768 | 384.103 [367.400,408.382] | 6.657 [6.255,7.017] |
| phase | 1024 | scan | 2048 | 2710.304 [2689.819,2742.166] | 10.763 [10.502,12.267] |
| phase | 1024 | linked | 2048 | 387.505 [370.685,410.935] | 22.350 [20.282,233.074] |
| phase | 1024 | clock | 2048 | 202.198 [200.154,211.722] | 9.943 [9.511,11.894] |
| phase | 1024 | segmented | 2048 | 416.856 [394.544,429.498] | 22.450 [20.988,30.103] |
| uniform | 1024 | scan | 24711 | 57739.980 [57497.973,57781.361] | 11.976 [11.593,13.178] |
| uniform | 1024 | linked | 24711 | 475.135 [456.370,479.232] | 26.072 [22.663,34.841] |
| uniform | 1024 | clock | 24749 | 476.144 [473.118,572.197] | 11.396 [10.840,11.437] |
| uniform | 1024 | segmented | 24805 | 522.732 [519.855,539.968] | 25.720 [23.436,27.379] |
### x86

| Trace | C | Policy | Misses | Request us | Setup us |
|---|---:|---|---:|---|---|
| tiny | 3 | scan | 7 | 0.389 [0.352,0.419] | 0.541 [0.507,0.635] |
| tiny | 3 | linked | 7 | 0.526 [0.461,0.558] | 0.512 [0.493,0.567] |
| tiny | 3 | clock | 7 | 0.457 [0.424,0.499] | 0.658 [0.552,0.738] |
| tiny | 3 | segmented | 6 | 0.615 [0.549,0.753] | 0.531 [0.492,0.676] |
| hot | 4 | scan | 1815 | 155.498 [154.365,253.513] | 0.616 [0.572,0.870] |
| hot | 4 | linked | 1815 | 261.024 [255.288,296.650] | 2.039 [1.786,2.291] |
| hot | 4 | clock | 1915 | 163.268 [153.363,178.672] | 0.666 [0.632,0.801] |
| hot | 4 | segmented | 1815 | 293.596 [291.010,448.582] | 1.968 [1.600,2.835] |
| scan | 4 | scan | 10242 | 175.285 [173.703,185.406] | 0.617 [0.603,0.654] |
| scan | 4 | linked | 10242 | 280.923 [275.870,288.877] | 1.919 [1.580,2.197] |
| scan | 4 | clock | 10242 | 172.632 [168.770,258.712] | 0.660 [0.616,0.957] |
| scan | 4 | segmented | 8194 | 295.552 [286.834,511.400] | 1.900 [1.755,2.997] |
| loop | 4 | scan | 32768 | 272.091 [253.734,283.040] | 0.636 [0.550,0.712] |
| loop | 4 | linked | 32768 | 273.813 [258.943,381.039] | 0.571 [0.557,0.678] |
| loop | 4 | clock | 32768 | 263.089 [235.881,651.124] | 0.828 [0.579,1.027] |
| loop | 4 | segmented | 32768 | 257.046 [253.556,447.505] | 0.586 [0.564,0.841] |
| phase | 4 | scan | 8 | 148.283 [145.112,155.910] | 0.675 [0.586,0.753] |
| phase | 4 | linked | 8 | 263.514 [254.546,456.898] | 0.570 [0.530,0.741] |
| phase | 4 | clock | 8 | 132.957 [128.408,211.515] | 0.691 [0.589,0.960] |
| phase | 4 | segmented | 8 | 263.752 [258.674,275.284] | 0.542 [0.524,0.600] |
| uniform | 4 | scan | 24916 | 423.529 [407.554,543.224] | 0.650 [0.588,1.080] |
| uniform | 4 | linked | 24916 | 403.382 [395.201,434.249] | 1.925 [1.559,2.053] |
| uniform | 4 | clock | 24941 | 336.548 [327.373,352.859] | 0.664 [0.585,0.778] |
| uniform | 4 | segmented | 25011 | 467.993 [461.575,656.333] | 2.005 [1.881,2.995] |
| hot | 64 | scan | 1803 | 292.351 [290.861,301.290] | 2.205 [1.741,2.344] |
| hot | 64 | linked | 1803 | 269.017 [266.740,452.344] | 3.477 [2.956,4.813] |
| hot | 64 | clock | 1823 | 156.272 [151.363,158.587] | 1.813 [1.680,2.346] |
| hot | 64 | segmented | 1803 | 289.082 [275.863,375.311] | 3.421 [3.008,3.895] |
| scan | 64 | scan | 10272 | 888.728 [874.682,951.056] | 2.752 [2.583,3.304] |
| scan | 64 | linked | 10272 | 263.430 [263.056,280.002] | 4.567 [4.321,4.682] |
| scan | 64 | clock | 10272 | 172.171 [160.290,247.052] | 1.811 [1.717,2.396] |
| scan | 64 | segmented | 8224 | 281.341 [269.227,473.466] | 4.771 [4.300,5.742] |
| loop | 64 | scan | 32768 | 2559.211 [2456.427,3065.525] | 2.223 [1.638,10.811] |
| loop | 64 | linked | 32768 | 262.894 [257.547,461.250] | 2.277 [2.054,2.691] |
| loop | 64 | clock | 32768 | 245.066 [235.077,266.971] | 2.042 [1.732,2.243] |
| loop | 64 | segmented | 32768 | 273.607 [259.968,463.442] | 2.010 [1.615,3.125] |
| phase | 64 | scan | 128 | 153.163 [151.758,155.834] | 2.174 [1.688,14.185] |
| phase | 64 | linked | 128 | 262.428 [258.090,293.857] | 3.373 [3.082,15.409] |
| phase | 64 | clock | 128 | 133.593 [131.196,197.398] | 2.027 [1.766,2.386] |
| phase | 64 | segmented | 128 | 279.029 [264.225,460.630] | 3.786 [2.812,4.036] |
| uniform | 64 | scan | 24612 | 2935.028 [2542.060,3128.586] | 2.255 [1.692,2.519] |
| uniform | 64 | linked | 24612 | 420.592 [368.239,525.212] | 3.622 [2.997,3.966] |
| uniform | 64 | clock | 24586 | 541.844 [528.289,593.633] | 2.172 [1.797,2.451] |
| uniform | 64 | segmented | 24578 | 453.940 [433.845,538.187] | 3.561 [2.985,10.801] |
| hot | 1024 | scan | 2264 | 1385.522 [1276.008,1413.817] | 17.523 [16.465,18.365] |
| hot | 1024 | linked | 2264 | 289.399 [280.489,306.235] | 44.411 [42.492,52.995] |
| hot | 1024 | clock | 2515 | 151.385 [146.429,235.435] | 16.366 [15.695,21.001] |
| hot | 1024 | segmented | 2263 | 298.341 [290.086,465.634] | 50.167 [39.226,56.805] |
| scan | 1024 | scan | 10752 | 9879.533 [9688.713,12230.702] | 22.070 [21.492,23.389] |
| scan | 1024 | linked | 10752 | 278.933 [272.472,289.208] | 64.105 [57.270,67.534] |
| scan | 1024 | clock | 10752 | 168.554 [164.876,171.380] | 21.012 [20.281,22.020] |
| scan | 1024 | segmented | 8704 | 279.392 [274.150,284.814] | 64.990 [56.342,75.892] |
| loop | 1024 | scan | 32768 | 32976.556 [31252.372,36954.995] | 7.550 [7.199,8.475] |
| loop | 1024 | linked | 32768 | 264.651 [254.680,290.736] | 7.403 [7.140,10.034] |
| loop | 1024 | clock | 32768 | 252.222 [228.074,265.631] | 6.239 [6.115,7.099] |
| loop | 1024 | segmented | 32768 | 260.995 [254.584,265.711] | 8.427 [7.011,10.496] |
| phase | 1024 | scan | 2048 | 1156.099 [1108.412,1163.210] | 13.201 [13.019,13.848] |
| phase | 1024 | linked | 2048 | 268.841 [262.242,636.690] | 30.301 [26.627,49.117] |
| phase | 1024 | clock | 2048 | 134.858 [134.238,206.650] | 12.247 [11.844,16.154] |
| phase | 1024 | segmented | 2048 | 282.697 [276.425,470.673] | 31.479 [25.585,41.671] |
| uniform | 1024 | scan | 24711 | 24238.079 [23780.028,28546.948] | 14.582 [14.216,18.569] |
| uniform | 1024 | linked | 24711 | 428.942 [423.555,572.178] | 36.151 [29.759,44.449] |
| uniform | 1024 | clock | 24749 | 511.717 [509.319,513.741] | 13.415 [13.144,14.097] |
| uniform | 1024 | segmented | 24805 | 496.495 [486.206,709.629] | 40.172 [30.050,45.978] |

## Interpretation and limitations

At capacity64 on the scan trace, segmented has8224 misses versus10272 for LRU
and CLOCK. CLOCK has the fastest screened CPU on both hosts. Extra segmented
CPU divided by2048 avoided misses gives a median-only serial break-even
projection of81.55ns/miss on Arm and53.31ns/miss on x86, excluding setup.
These projections use deterministic miss counts and observed median CPU;
backend and end-to-end performance remain unmeasured.

All candidates miss every request on the capacity-plus-one loop. A faster
implementation there improves bookkeeping while preserving zero cache hits.
At capacity1024/hot, CLOCK has2515 misses, LRU2264 and segmented2263. The
CLOCK CPU win therefore needs the miss-cost tradeoff before policy selection.

A/A B/A ratios: Arm0.922..1.124, x860.970..1.109. Mechanical image/settings,
parser and oracle integrity pass. These finite controls show variation and
provide no population-level calibration or noise-floor estimate. Timing
screens remain exploratory, host/workload/implementation-specific.

The first Arm campaign stopped at process28: the bool-vector filter aggregate
reported16382 hits while its exact emitted stream contained30953 and matched
the independent oracle. Root cause remains unestablished. The retained runner
now derives its reported hit total from the emitted stream using string matching.
Both hosts were rerun from the corrected committed runner and all checks pass.
Initial failed Arm and initial passing x86 archives remain external; selection
uses only the corrected campaign. Assembly and work counters are retained;
no hardware-counter or compiler-root-cause claim is made.

Replay: cargo run --release -p topic048-eviction --example measure -- linked tiny 3 9.
Then python3 topics/048-lru-clock-and-eviction/measurements/run.py
 target/release/examples/measure /tmp/topic048-new-run. On Linux, run sh
measurements/host.sh from the frozen topic root. EVIDENCE_RECEIPT.json records
verified archive locations, sizes/member counts and exact source/image hashes.
