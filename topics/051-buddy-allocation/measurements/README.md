# Exact-source results

Frozen executable commit: `5e33b3ddb2e7570b68a731e3d66fe58dd656cdef`. The receipt hashes bind src/lib.rs, tests/contracts.rs, examples/replay.rs and measure.py at this commit. The first three are byte-identical at HEAD; measure.py differs as listed under Replay.

| Host | Units/trace | Winner | scan us | buddy us | tree us |
|---|---|---|---|---|---|
| arm | 64/tiny | scan | 10.291 [9.537, 10.384] | 21.178 [18.637, 21.778] | 14.355 [14.312, 14.501] |
| arm | 64/mixed | tree | 9.275 [8.638, 10.696] | 9.620 [8.570, 13.300] | 5.521 [5.344, 5.856] |
| arm | 64/holes | tree | 7.407 [7.181, 8.032] | 11.686 [11.152, 15.201] | 4.261 [4.133, 4.399] |
| arm | 64/churn | tree | 3.050 [2.885, 3.125] | 4.952 [4.826, 5.588] | 2.494 [2.326, 2.588] |
| arm | 4096/tiny | tree | 22774.408 [22648.953, 22914.223] | 2032.527 [2012.778, 2151.390] | 1795.181 [1780.038, 1822.717] |
| arm | 4096/mixed | tree | 21396.904 [21369.328, 21541.033] | 559.245 [554.321, 569.903] | 322.253 [314.142, 350.410] |
| arm | 4096/holes | tree | 18589.315 [18578.754, 18868.533] | 793.056 [789.221, 816.627] | 465.325 [463.165, 493.647] |
| arm | 4096/churn | tree | 61.380 [60.975, 61.450] | 7.374 [7.049, 7.735] | 3.964 [3.824, 4.116] |
| x86 | 64/tiny | scan | 8.752 [7.892, 9.316] | 16.715 [16.325, 23.028] | 13.825 [13.747, 13.973] |
| x86 | 64/mixed | tree | 7.212 [6.786, 8.361] | 9.316 [8.965, 9.579] | 5.891 [5.793, 6.007] |
| x86 | 64/holes | tree | 6.131 [5.796, 7.245] | 12.362 [11.887, 15.816] | 4.295 [4.058, 4.758] |
| x86 | 64/churn | tree | 3.011 [2.653, 3.995] | 5.131 [4.464, 6.037] | 2.711 [2.350, 2.931] |
| x86 | 4096/tiny | tree | 15684.977 [14929.125, 16224.108] | 2027.194 [1942.968, 2462.035] | 1725.063 [1698.140, 1798.983] |
| x86 | 4096/mixed | tree | 15198.246 [14431.089, 19854.810] | 518.614 [508.741, 723.138] | 359.406 [336.716, 434.212] |
| x86 | 4096/holes | tree | 20924.503 [19174.159, 21701.165] | 1114.158 [1052.004, 1147.813] | 582.556 [545.113, 626.177] |
| x86 | 4096/churn | tree | 75.561 [68.042, 86.222] | 10.516 [9.808, 11.037] | 5.674 [5.125, 5.944] |

Times are median [minimum, maximum] microseconds for one complete trace. Each host passes 24 complete independent interval-oracle traces, three native contract tests and 204 processes: 24 correctness, 24 warmup, 144 measured and 12 A/A. All source/archive/member/binary identities verify. Full outputs are rechecked locally.

Arm: Neoverse V1, 64 available CPUs, AArch64 Linux 6.12.115, Rust 1.98.1. x86: Intel Xeon Platinum 8488C, 192 available CPUs, x86-64 Linux 6.12.115, Rust 1.99.0. Exact uname, compiler/LLVM and target features are retained in environment.txt. Native CPU code generation, optimization, serial execution pinned CPU0.

A/A B/A range: Arm 0.970..1.024; x86 0.798..1.093. The control is 12 scan/4096/mixed processes after all cells with alternating labels. It does not pass through the three-candidate permutation order or the other cells, so it screens order asymmetry between two consecutive identical processes. The x86 control varies materially. The cell winners satisfy the fixed finite screen; stable cross-window x86 recommendations need new evidence.

All candidates preserve identical placement, so this compares metadata cost under one policy. It does not measure a general-purpose allocator, cold service latency, concurrent ownership, backing-memory traffic or hardware causation. Logical counters are representation-specific. Construction medians and raw process ranges are retained externally; construction/destruction are outside the operation interval. Each timed interval is the first replay in a fresh process, so allocator growth, first-touch pages and first execution of the operation code are inside it; steady-state repetition inside one process is unmeasured.

External evidence: /Users/ahrav/.codex-amazon/automations/algorithms-daily-curriculum/evidence/topic-051/20261116/delivery. Reproduce with measure.py and the commands in README.md.

## Replay

measure.py keeps the 204-process schedule and the replay command lines, and differs from the archived runner as follows. It exits at startup when assertions are disabled or the results file exists; the archived runner overwrote the file. Each process has a 60 second limit (TOPIC_051_PROCESS_TIMEOUT_S). Every attempt appends one line with a status (`ok`, `failed`, `timeout`, `rejected`); a line that is not `ok` keeps the command, stdout and stderr, and the run stops there. The archived runner raised before it wrote such a line. Each process must echo its kind, n and shape, and a warmup, measured or A/A process must repeat the digest, `ops` and `work` of its check process; the archived runner compared the digest. Its oracle holds its own definition of the four traces and requires each check output to be those operations in order, one state line and a summary whose `ops` equals the trace length; the archived oracle verified the answers of the operations that the binary printed. It hashes the binary before the first launch and before every later launch and stops when the hash changes; the archived runner hashed the binary once, after the last process. The digest folds result magnitudes, so -1 and 1 add the same term; complete answers are verified in the check processes. The archived runs used the 5e33b3d checks.

`python3 topics/051-buddy-allocation/test_measure.py` checks the HEAD runner with stub processes. A replay of HEAD with the README commands on an aarch64 host under Rust 1.93.0 accepted all 204 processes and 24 complete traces; that replay checks the tooling and adds no measurement. tests/workload.rs and tests/placement.rs are HEAD additions to the three archived contract tests.
