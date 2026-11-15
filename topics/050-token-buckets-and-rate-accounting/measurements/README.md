# Two-host exact-source results

Frozen source: `c88066d26429e0c339ac20cc572b938d2b3ee940`. All five executable inputs match this
commit and the final publication head. Each host ran 396 fresh processes:
48 complete-output correctness, 48 warmup, 288 measured, 12 A/A. Every process
checksum and all 48 full output vectors match the independent ledger. Both
native contract suites pass. Rust history-oracle tests compare 24,576 candidate
decisions across 8,192 generated attempts plus explicit boundary checks.

Warm shared Linux hosts, pinned CPU0, native optimized builds. Arm: 64 available
CPUs, ARM implementer 0x41/part 0xd40/r1p1, Rust1.98.1, kernel6.12.115.
x86: 192 available CPUs, Intel Xeon Platinum8488C, Rust1.99.0, kernel6.12.115.
Full uname, affinity, target feature lists, compiler and CPU records are external.
Compiler and host differences prevent hardware-only attribution.

Wide arithmetic wins 11/16 Arm cells; five are unresolved. x86's nominal screen
selects wide in 7/16 cells; nine are unresolved. A/A ratios Arm0.975..1.020,
x860.520..2.212. The x86 control variability prevents a stable speed recommendation.
Keep the declared screen as a finite observation and use wide as the simplest
validated default. Neither alternative wins a declared cell under the paired rule.

Wide and bounded state occupy32 bytes/flow, virtual48 bytes/flow (excluding Vec
metadata and allocator overhead). For4096 flows:131072/131072/196608 bytes.
The tiny4096-flow cells mostly initialize unused state. Burst/steady/idle share
identical traces when N32/F4096, since each visited flow receives one attempt.
These intentionally expose state setup costs rather than refill behavior.

Emitted benchmark assembly contains wide multiply and division operations.
This is generated-code evidence; comparative hardware causes remain unmeasured.
No cold-cache, clock acquisition, map lookup, synchronization, service latency,
distributed quota, RSS or concurrent fairness measurement is claimed.

## All declared cells

Times are median [min,max] microseconds per complete replay across six process
samples. Screen winner requires every paired block >5% faster than both peers.
Inner replays are timing stabilization, not independent samples.

### arm

| N | flows | arrivals | wide us | bounded us | virtual us | screen |
|---:|---:|---|---:|---:|---:|---|
| 32 | 1 | burst | 0.165 [0.164,0.166] | 0.230 [0.230,0.230] | 0.198 [0.198,0.199] | wide |
| 32 | 1 | steady | 0.166 [0.165,0.171] | 0.240 [0.240,0.241] | 0.199 [0.198,0.203] | wide |
| 32 | 1 | idle | 0.171 [0.170,0.171] | 0.246 [0.246,0.246] | 0.198 [0.198,0.200] | wide |
| 32 | 1 | weighted | 0.169 [0.168,0.172] | 0.241 [0.239,0.250] | 0.199 [0.198,0.200] | wide |
| 32 | 4096 | burst | 2.839 [2.796,2.914] | 2.928 [2.824,4.703] | 3.861 [3.823,3.918] | unresolved |
| 32 | 4096 | steady | 2.829 [2.799,2.864] | 2.832 [2.817,2.950] | 3.855 [3.836,5.005] | unresolved |
| 32 | 4096 | idle | 2.854 [2.807,4.012] | 2.838 [2.811,2.867] | 3.836 [3.826,3.863] | unresolved |
| 32 | 4096 | weighted | 2.820 [2.806,2.836] | 2.831 [2.823,5.356] | 3.841 [3.825,3.879] | unresolved |
| 32768 | 1 | burst | 152.745 [151.988,157.339] | 219.381 [218.707,225.274] | 184.904 [183.619,193.325] | wide |
| 32768 | 1 | steady | 151.983 [151.270,152.752] | 229.923 [229.197,236.256] | 186.273 [184.150,192.214] | wide |
| 32768 | 1 | idle | 161.828 [158.836,164.762] | 236.889 [236.706,237.686] | 184.564 [184.215,188.086] | wide |
| 32768 | 1 | weighted | 155.930 [154.907,157.481] | 230.681 [229.867,238.317] | 185.216 [184.741,191.596] | wide |
| 32768 | 4096 | burst | 101.766 [99.701,105.960] | 118.393 [117.192,123.943] | 156.793 [154.165,163.181] | wide |
| 32768 | 4096 | steady | 101.588 [100.942,103.516] | 115.035 [114.423,115.950] | 154.793 [154.429,157.112] | wide |
| 32768 | 4096 | idle | 101.930 [99.796,111.323] | 115.025 [114.020,116.388] | 157.874 [154.807,162.601] | unresolved |
| 32768 | 4096 | weighted | 99.962 [99.646,101.342] | 114.368 [114.137,115.804] | 154.866 [154.413,164.080] | wide |

### x86

| N | flows | arrivals | wide us | bounded us | virtual us | screen |
|---:|---:|---|---:|---:|---:|---|
| 32 | 1 | burst | 0.140 [0.126,0.195] | 0.164 [0.151,0.282] | 0.193 [0.166,0.261] | unresolved |
| 32 | 1 | steady | 0.130 [0.092,0.167] | 0.175 [0.109,0.183] | 0.185 [0.130,0.266] | unresolved |
| 32 | 1 | idle | 0.138 [0.089,0.161] | 0.179 [0.107,0.204] | 0.188 [0.130,0.217] | unresolved |
| 32 | 1 | weighted | 0.095 [0.089,0.213] | 0.109 [0.109,0.227] | 0.130 [0.127,0.271] | wide |
| 32 | 4096 | burst | 4.211 [4.160,5.056] | 4.233 [4.181,6.153] | 4.832 [4.793,5.970] | unresolved |
| 32 | 4096 | steady | 9.684 [7.947,15.124] | 9.812 [6.601,13.680] | 8.756 [7.474,12.527] | unresolved |
| 32 | 4096 | idle | 4.914 [4.846,5.450] | 4.946 [4.922,16.336] | 5.615 [5.544,13.433] | unresolved |
| 32 | 4096 | weighted | 6.385 [4.868,9.793] | 5.781 [4.880,10.233] | 6.420 [5.593,12.947] | unresolved |
| 32768 | 1 | burst | 96.856 [74.971,182.630] | 164.548 [94.662,223.608] | 151.281 [117.462,219.908] | unresolved |
| 32768 | 1 | steady | 75.096 [74.471,77.088] | 95.763 [94.225,96.474] | 118.076 [117.371,126.723] | wide |
| 32768 | 1 | idle | 76.257 [76.140,78.304] | 93.614 [93.072,95.579] | 118.426 [117.593,121.776] | wide |
| 32768 | 1 | weighted | 75.450 [74.752,184.393] | 96.084 [95.142,96.885] | 118.371 [117.920,196.001] | unresolved |
| 32768 | 4096 | burst | 78.775 [77.996,80.143] | 98.901 [98.324,99.533] | 122.379 [120.097,123.779] | wide |
| 32768 | 4096 | steady | 81.207 [79.803,86.626] | 101.027 [100.522,102.569] | 121.146 [120.190,209.981] | wide |
| 32768 | 4096 | idle | 80.881 [79.955,84.130] | 98.878 [98.313,100.431] | 120.259 [119.770,122.040] | wide |
| 32768 | 4096 | weighted | 80.227 [79.203,80.839] | 101.190 [100.231,105.635] | 123.457 [121.204,184.536] | wide |

## Replay

Extract frozen-source.tar.gz from the retained delivery, then run
`python3 benches/run.py --source c88066d26429e0c339ac20cc572b938d2b3ee940` on Linux.
The INPUTS.json in that archive binds source, tests, example, benchmark and runner.
See BENCHMARK.md for controls and the receipt for raw evidence identities.
