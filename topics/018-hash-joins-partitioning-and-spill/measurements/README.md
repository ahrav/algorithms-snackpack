# Exact committed measurements

Measured source commit: `b2f96008cad45e9dbc632aa318ef1fba4addbdc7`. Six frozen files match both remote manifests.
Raw evidence: `/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-018/20261014`.

| Case | Arm choice; median [min,max] microseconds | x86 choice; median [min,max] microseconds |
|---|---|---|
| disjoint | merge: 146.534 [144.339,148.727] | merge: 126.866 [125.204,129.563] |
| duplicates | merge: 61.215 [60.881,61.519] | merge: 49.156 [48.855,50.060] |
| heavy | merge: 11.867 [11.844,12.289] | merge: 10.058 [10.005,10.483] |
| large | merge: 3446.495 [3437.140,3478.966] | merge: 3267.829 [3252.858,3274.421] |
| small_build | unresolved; chunk: 1208.936 [1175.508,1279.208]; hash: 1218.140 [1187.484,1237.586] | unresolved; chunk: 1218.635 [1209.016,1242.011]; hash: 1210.850 [1207.060,1239.718] |
| sorted | merge: 25.326 [25.294,25.333] | merge: 21.632 [21.570,22.138] |
| tiny | merge: 0.081 [0.081,0.084] | merge: 0.075 [0.069,0.090] |
| uniform | merge: 163.804 [162.770,165.926] | merge: 141.256 [138.936,145.847] |

Both exact256-process campaigns pass every independent oracle. Two library groups,
three external file contracts and example pass per host. Initial256-process campaigns
also pass, with x86 uniform unresolved between hash/merge; retained independently.
Final data is the exact committed campaign, with no repeat to seek a winner.

Arm verifiedaarch64,64CPUs, Linux6.12.110; model fieldCPU part retained in raw.
x86 verifiedx86_64,192CPUs, IntelXeonPlatinum8488C, Linux6.12.110.
Bothrustc1.98.1 LLVM22.1.8; optimization3/native features. Full target cfg and uname
are retained per manifest. Schedulers unpinned. See BENCHMARK.md for sample controls.

Merge selectedtiny/uniform/disjoint/duplicates/heavy/large/sorted on bothhosts.
Small_build leaveshash/chunk unresolved. Frequency compression and optimized
standard sorting make this a count-only, fixed-key workload claim. Residentpartition
has extra arrays and no bytebound. Chunk has a rowcap and can rescan64times.

File spill uses actual temporary files and checks logical reads/writes. OS cache,
physical media, durable completion, byte admission, full payload output and cancellation
are unmeasured. Assembly is retained; no hardware mechanism claim is inferred.
