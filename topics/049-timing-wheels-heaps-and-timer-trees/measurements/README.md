# Measured finite timer replay

Six paired fresh-process blocks per cell, 16 timed replays/process. Each value is median [min,max] microseconds per complete replay. Winner requires >5 percent faster than every rival in all six blocks. All cells pass complete-output and work-counter oracles.

| Host | IDs/shape/cancel% | Selection | Scan us | Heap us | B-tree us | Wheel us |
|---|---|---|---|---|---|---|
| arm | 16/near/0 | heap | 3.675 [3.629,3.850] | 1.295 [1.250,1.348] | 1.771 [1.749,1.834] | 2.738 [2.614,3.752] |
| arm | 16/near/90 | unresolved | 3.111 [3.092,3.139] | 1.192 [1.148,1.641] | 1.595 [1.546,1.669] | 2.178 [2.106,2.215] |
| arm | 16/far/0 | heap | 3.162 [3.123,3.235] | 1.188 [1.140,1.218] | 1.623 [1.564,1.662] | 225.462 [195.831,308.025] |
| arm | 16/far/90 | heap | 3.058 [3.053,3.077] | 1.068 [1.029,1.098] | 1.582 [1.549,1.597] | 196.597 [194.424,313.190] |
| arm | 16/burst/0 | scan | 0.229 [0.220,0.244] | 0.725 [0.695,0.726] | 1.058 [1.015,1.093] | 302.212 [265.598,347.093] |
| arm | 16/burst/90 | scan | 0.161 [0.151,0.170] | 0.601 [0.566,1.014] | 1.018 [0.990,1.024] | 233.020 [198.878,245.913] |
| arm | 4096/near/0 | wheel | 787.598 [784.274,792.223] | 485.017 [482.259,487.113] | 613.908 [610.798,617.886] | 165.484 [163.898,169.346] |
| arm | 4096/near/90 | wheel | 698.790 [697.984,706.522] | 453.647 [450.750,460.988] | 856.272 [846.533,859.982] | 91.191 [88.556,96.035] |
| arm | 4096/far/0 | heap | 804.409 [802.957,806.480] | 401.795 [399.468,409.218] | 597.343 [592.615,613.185] | 9413.902 [9396.527,9498.593] |
| arm | 4096/far/90 | heap | 712.793 [710.758,717.709] | 370.409 [369.547,374.252] | 785.385 [780.976,792.674] | 1155.410 [1146.060,1164.391] |
| arm | 4096/burst/0 | scan | 46.777 [43.759,49.000] | 393.423 [391.281,399.278] | 546.716 [543.906,547.266] | 17507.735 [17372.300,17613.090] |
| arm | 4096/burst/90 | scan | 24.034 [23.678,25.083] | 361.047 [359.984,364.550] | 690.818 [687.187,694.806] | 1972.007 [1919.418,2023.024] |
| x86 | 16/near/0 | unresolved | 3.924 [3.907,3.960] | 1.333 [1.261,1.635] | 1.766 [1.680,2.786] | 2.076 [2.073,2.203] |
| x86 | 16/near/90 | heap | 2.664 [2.623,5.156] | 1.147 [1.072,1.194] | 1.659 [1.597,2.445] | 1.596 [1.524,2.343] |
| x86 | 16/far/0 | heap | 2.679 [2.611,2.719] | 1.291 [1.245,1.719] | 1.607 [1.532,2.296] | 140.399 [118.668,157.192] |
| x86 | 16/far/90 | heap | 2.612 [2.584,4.625] | 1.158 [1.132,1.276] | 1.589 [1.537,2.461] | 129.817 [109.030,151.715] |
| x86 | 16/burst/0 | scan | 0.229 [0.217,0.341] | 0.803 [0.788,0.860] | 0.970 [0.961,1.386] | 153.532 [140.413,168.507] |
| x86 | 16/burst/90 | scan | 0.194 [0.180,0.212] | 0.752 [0.675,1.336] | 1.011 [0.928,1.426] | 117.645 [105.331,132.421] |
| x86 | 4096/near/0 | wheel | 768.524 [763.633,774.412] | 389.974 [378.349,397.952] | 603.543 [602.798,802.646] | 137.881 [137.285,163.440] |
| x86 | 4096/near/90 | wheel | 590.872 [576.473,794.229] | 351.229 [342.343,373.210] | 844.494 [835.230,868.737] | 81.031 [80.355,84.596] |
| x86 | 4096/far/0 | heap | 742.083 [734.274,746.222] | 285.566 [276.105,310.338] | 578.101 [570.165,590.924] | 6531.896 [6502.398,6599.230] |
| x86 | 4096/far/90 | heap | 560.907 [559.408,563.074] | 246.464 [240.570,258.779] | 747.433 [744.367,757.047] | 833.839 [813.069,859.540] |
| x86 | 4096/burst/0 | scan | 47.339 [46.646,51.833] | 310.832 [306.443,332.745] | 508.334 [500.060,534.439] | 12781.826 [12625.144,13021.606] |
| x86 | 4096/burst/90 | scan | 25.502 [24.313,40.750] | 293.221 [287.885,294.767] | 651.147 [646.926,656.834] | 1397.526 [1382.432,1408.378] |

## Interpretation

The scan wins all one-poll burst cells. Large near-deadline cells select the wheel. Large far-deadline cells select the heap. The Arm 16-ID near/90 cell remains unresolved; its nominal fastest median is heap. All other tiny spread cells select heap. The B-tree pays eager update/removal costs in this workload, including arming every ID before cancellation. Its direct-removal semantics may matter under a different arrival/update mix.

At 4096/far/0, scan inspects 1,048,576 slots, heap pops 5,120 records, tree removes 4,096 due keys and wheel inspects 1,987,584 records while crossing 65,536 ticks. These counts support the stated work model. They do not isolate hardware causes. All candidates include an ID array; heap/wheel peak at 5,120 records, tree at 4,096 live keys. Actual allocation bytes/capacity and resident memory were not measured.

Near/far use 256 polls; burst uses one. Comparing those shapes changes the complete service workload. The wheel walks every elapsed tick; a hierarchy or empty-slot skipping needs a separate implementation and experiment. This wheel is a deliberately simple hashed candidate.

## Environment and limits

Arm: aarch64, 64 available CPUs, native Rust1.98.1/LLVM22.1.8. x86: Intel Xeon Platinum8488C, x86_64,192 available CPUs, native Rust1.99.0/LLVM23.1.1. Both use Linux6.12.113-138.209.amzn2023, CPU0 affinity, edition2024,opt-level3,target-cpu=native. Full hostname, uname, target features, compiler and affinity are retained in each raw archive. Arm implementer/part metadata is retained separately in the delivery evidence.

Arm A/A ratios0.991..1.010; x86 A/A0.960..1.023. Shared hosts, fixed synthetic input, warm code/data, logically empty queues, different native toolchains. Six-block descriptive selection supports these particular builds and cells. Callback lateness, kernel wakeups, cold state, concurrent use, peak RSS and universal optimum require separate evidence. Assembly from both native builds was inspected for allocation/memcpy calls; it establishes code presence, without attributing timing to hardware events.

## Replay and validation

From the topic directory on either Linux host, run `bash benches/native.sh`. This writes a new native/ directory and retains all stdout/stderr, schedule, environment, input hashes, binary, assembly and contract results. Use a fresh directory for another run. The Python runner independently checks every process output; the task verifier rechecked archives, complete outputs and deterministic counters locally. The timed 16 deterministic replay results are consumed and dropped; each process emits its untimed warm replay result for independent checking.

All seven required workspace gates plus workspace all-target tests pass under Rust1.93.0. Both native contract suites pass. Three deliberate semantic mutations (stale arm acceptance, early wheel firing, reversed output order) were rejected. Python syntax and shell syntax checks pass. See EVIDENCE_RECEIPT.json for immutable source and retained raw identities.
