# Measured selection

Source: `395cba1437c6072325c05ad2d96d8ec23c1ecf2f`.

All 421 scheduled processes per host completed; 360 measured,45 warmup,16 A/A.
Three candidates passed 3969 exhaustive pairs per host and every measured workload passed independent minimum-cost and exact replay checks. Source/runner/binary/plan identities verified.

Times are median [min,max] microseconds per complete operation, eight process samples.

|Host|m|shape|target n|D|Matrix|Hirschberg|Myers|selection|
|---|---:|---|---:|---:|---|---|---|---|
|arm|16|identical|16|0|0.705 [0.669,0.763]|1.914 [1.866,1.938]|0.169 [0.164,0.203]|myers|
|arm|16|sparse|16|2|0.713 [0.696,0.786]|1.913 [1.852,2.004]|0.246 [0.225,0.279]|myers|
|arm|16|unrelated|16|32|0.673 [0.666,3.146]|1.223 [1.215,1.279]|3.159 [3.114,3.312]|unresolved|
|arm|16|repeat|16|16|0.574 [0.562,0.580]|1.408 [1.402,1.416]|1.144 [1.110,1.226]|matrix|
|arm|16|unequal|1|15|0.161 [0.155,0.176]|0.341 [0.325,0.392]|0.977 [0.962,0.987]|matrix|
|arm|256|identical|256|0|171.523 [171.119,172.243]|282.288 [281.381,298.716]|1.710 [1.385,2.236]|myers|
|arm|256|sparse|256|8|172.495 [171.428,185.298]|282.731 [281.978,292.296]|2.675 [2.103,2.729]|myers|
|arm|256|unrelated|256|512|173.838 [173.013,174.480]|304.027 [303.294,312.547]|716.183 [702.576,735.841]|matrix|
|arm|256|repeat|256|256|129.318 [128.848,137.118]|202.504 [201.967,212.842]|120.803 [119.925,121.631]|myers|
|arm|256|unequal|16|240|9.603 [9.527,9.647]|12.441 [12.409,13.002]|108.231 [107.869,128.560]|matrix|
|arm|1024|identical|1024|0|2834.240 [2828.069,2868.314]|4641.262 [4637.362,4664.760]|8.086 [5.792,8.125]|myers|
|arm|1024|sparse|1024|32|2834.820 [2823.211,2842.923]|4643.325 [4638.769,4650.039]|10.855 [9.588,11.914]|myers|
|arm|1024|unrelated|1024|2048|2848.157 [2844.245,2853.247]|4960.838 [4956.117,4968.842]|12060.003 [11981.842,12334.362]|matrix|
|arm|1024|repeat|1024|1024|2128.802 [2101.333,2317.298]|3077.086 [3071.068,3080.137]|1796.080 [1780.679,1809.804]|myers|
|arm|1024|unequal|64|960|179.194 [177.667,181.771]|241.390 [239.698,253.269]|2418.978 [2389.037,2460.398]|matrix|

arm A/A ratio range: [0.962561695896238, 1.0422013379021573].
Compiler: rustc 1.98.1 (48a229cea 2026-09-01). CPU model: see external environment. Kernel: 6.12.113-138.209.amzn2023.aarch64. Available CPUs: 64; pinned CPU 0.
|x86|16|identical|16|0|0.774 [0.765,0.898]|2.452 [2.436,2.524]|0.231 [0.223,0.234]|myers|
|x86|16|sparse|16|2|0.785 [0.779,0.914]|2.471 [2.458,2.582]|0.305 [0.294,0.312]|myers|
|x86|16|unrelated|16|32|0.739 [0.735,0.743]|1.362 [1.354,1.371]|4.129 [3.930,5.133]|matrix|
|x86|16|repeat|16|16|0.805 [0.798,0.953]|1.751 [1.739,1.809]|1.579 [1.531,1.736]|matrix|
|x86|16|unequal|1|15|0.212 [0.208,0.216]|0.487 [0.484,0.493]|1.367 [1.327,1.434]|matrix|
|x86|256|identical|256|0|201.474 [200.667,218.248]|330.962 [329.294,331.650]|2.107 [2.063,2.177]|myers|
|x86|256|sparse|256|8|200.791 [200.515,201.229]|330.104 [329.507,346.932]|3.090 [3.003,3.147]|myers|
|x86|256|unrelated|256|512|205.308 [205.080,205.865]|356.940 [355.819,361.853]|1048.049 [1033.164,1054.722]|matrix|
|x86|256|repeat|256|256|179.772 [179.445,184.403]|239.238 [238.260,247.224]|190.186 [183.706,192.389]|unresolved|
|x86|256|unequal|16|240|10.799 [10.680,11.190]|15.061 [14.964,15.550]|189.078 [187.684,195.336]|matrix|
|x86|1024|identical|1024|0|3046.620 [3042.135,3120.792]|4773.006 [4765.637,4795.038]|4.679 [4.665,4.745]|myers|
|x86|1024|sparse|1024|32|3051.573 [3045.580,3057.912]|4781.547 [4775.984,4792.154]|7.259 [7.038,7.316]|myers|
|x86|1024|unrelated|1024|2048|3173.569 [3170.647,3194.163]|5243.908 [5237.922,5258.543]|10220.451 [10082.011,10792.640]|matrix|
|x86|1024|repeat|1024|1024|2306.587 [2272.826,2337.767]|3205.416 [3198.300,3219.329]|1431.555 [1419.639,1503.300]|myers|
|x86|1024|unequal|64|960|171.976 [170.629,173.306]|223.913 [222.756,225.719]|2203.072 [2190.743,2223.396]|matrix|

x86 A/A ratio range: [0.9911778448260181, 1.0516953735638048].
Compiler: rustc 1.99.0 (b940084d7 2026-09-28). CPU model: Intel(R) Xeon(R) Platinum 8488C. Kernel: 6.12.113-138.209.amzn2023.x86_64. Available CPUs: 192; pinned CPU 0.

Warm native synthetic process screen, fixed deterministic rotation/reversal, one host/build per architecture. Toolchains differ; cross-host times have no hardware-causal interpretation. Full operation includes counters, allocations and teardown; oracle and setup are excluded. Matrix was median-fastest for Arm unrelated16, but one block prevents a declared win; x86 repeat256 also remains unresolved. Hirschberg buys smaller live score storage and won no timing cell. See BENCHMARK.md for criterion and limits.

For sparse1024: score slots are {'arm': {'matrix': 1050625, 'hirschberg': 2050, 'myers': 561}, 'x86': {'matrix': 1050625, 'hirschberg': 2050, 'myers': 561}}. At eight bytes per score this is 8405000 matrix bytes, 16400 Hirschberg bytes and 4488 Myers bytes; add vector headers, allocation metadata, matching pairs, inputs and output. These are deterministic score-capacity counts, not RSS.

Raw verified external retention: `/Users/ahrav/.codex-amazon/automations/algorithms-daily-curriculum/evidence/topic-042/20261107`. Frozen runner and source hashes are in EVIDENCE_RECEIPT.json. Final evidence-only commit must retain identical executable/runner blobs.
