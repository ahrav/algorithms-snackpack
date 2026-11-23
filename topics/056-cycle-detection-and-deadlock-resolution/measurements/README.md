# Native correctness evidence

Arm and x86-64 Linux each passed four contract groups. Each agreement check
covered 512 directed graphs, 1,600 reusable-resource snapshots and 4,800
single-rollback policies. The example produced identical reports, with cycle
A, B, A and exclusive victim B. Pooled completion was C, A, B.

| Host | CPUs available | Kernel | Rust |
|---|---:|---|---|
| Arm Linux | 64 | 6.12.115-140.210.amzn2023.aarch64 | 1.98.1 |
| x86-64 Linux | 192 | 6.12.115-140.210.amzn2023.x86_64 | 1.99.0 |

The locked input archive contains the exact library, test, example and runner
from the source commit. Both hosts verified its hash and all named input
hashes. The runner builds Rust 2024 code with warnings as errors, optimization
level 2, overflow checks and the host's native CPU features. Results retain
the toolchain, CPU, affinity, flags and executable hashes. See the
[evidence receipt](EVIDENCE_RECEIPT.json).

From the extracted input archive, run the following with the host's
architecture in place of `aarch64` when needed.

```sh
bash topics/056-cycle-detection-and-deadlock-resolution/measurements/run.sh aarch64
sha256sum -c RESULT_SHA256SUMS
```

This is bounded correctness evidence for a fixed snapshot model. The harness
reported test durations as part of its normal output. Selection follows
the proved resource contracts and verified decisions.
