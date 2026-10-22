# Measurements and evidence

Initial campaign: 450 fresh processes per required Linux host. The committed-source campaign is recorded separately. Never pool them as independent replications. All observations are from the safe instrumented wrapper, including request accounting and common dispatch; they are not timings of unwrapped Vec::push alone.

Sources, flags, kernel, architecture, compiler, CPU features, allowed CPUs and linked binary hashes are in each external identity.json. Arm: dev-dsk-ahrav-2b-7dc7bd93, aarch64, Rust 1.98.1, 64 available CPUs. x86: runtime-resolved xxl = dev-dsk-ahrav-2c-32182091, Xeon Platinum 8488C, Rust 1.99.0, 192 CPUs. Kernel 6.12.111-136.204.amzn2023 on both; native target flags; unpinned. Different toolchains prevent an ISA-only causal comparison.

| Host | Case | Selection | Lowest median ns/op | Observed range |
|---|---|---|---:|---|
| arm | tiny | reserved | 5.571 (reserved) | 5.488..5.636 |
| arm | medium | unresolved: reserved / std,double | 3.725 (reserved) | 3.670..3.865 |
| arm | large | unresolved: reserved / std,onehalf,double | 3.751 (reserved) | 3.708..3.869 |
| arm | wide | unresolved: onehalf / reserved | 22.898 (onehalf) | 22.529..24.161 |
| arm | reuse | unresolved: reserved / std,onehalf,double | 3.626 (reserved) | 3.588..3.692 |
| arm | oscillate | unresolved: std / quarter | 3.493 (std) | 3.452..3.632 |
| arm | drain | unresolved: std / quarter | 3.370 (std) | 3.349..3.635 |
| x86 | tiny | reserved | 3.743 (reserved) | 3.707..4.023 |
| x86 | medium | unresolved: reserved / std,onehalf,double | 2.440 (reserved) | 2.413..2.807 |
| x86 | large | unresolved: reserved / std,onehalf,double | 2.798 (reserved) | 2.749..2.938 |
| x86 | wide | unresolved: reserved / exact,onehalf | 48.820 (reserved) | 43.237..93.646 |
| x86 | reuse | unresolved: reserved / std,onehalf,double | 2.409 (reserved) | 2.391..2.484 |
| x86 | oscillate | unresolved: std / quarter | 2.328 (std) | 2.207..2.673 |
| x86 | drain | std | 2.382 (std) | 2.342..2.465 |

All candidate medians, process samples, paired ratios and counters are in preteach-summary.json. A sole selection requires every rival >3% slower in every paired block; otherwise unresolved. Ranges are descriptive, not confidence intervals. A/A ranges are diagnostic; no calibrated statistical/power, universal, PMU, cold allocator, p99, physical copy-byte or resident-memory claim. Request counts need not equal allocator calls; address changes do not reveal physical copying. Counter snapshots are from the last inner repetition of each process. See BENCHMARK.md.

The reserved candidate receives a correct peak hint. Reuse includes the first fill and sixteen clear/refill cycles; it is not an isolated zero-allocation steady-state loop. Wide includes full 256-byte records and output scans. Large stops just beyond a power-of-two capacity boundary deliberately.

Raw replay assets and archives: /Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-026/20261022
