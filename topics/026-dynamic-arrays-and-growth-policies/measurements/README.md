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

Candidate medians, ranges, paired-ratio ranges and counters are in preteach-summary.json. Individual samples remain in the external raw archives. A sole selection requires every rival >3% slower in every paired block; otherwise unresolved. Ranges are descriptive, not confidence intervals. A/A ranges are diagnostic; no calibrated statistical/power, universal, PMU, cold allocator, p99, physical copy-byte or resident-memory claim. Request counts need not equal allocator calls; address changes do not reveal physical copying. Counter snapshots are from the last inner repetition of each process; the committed summaries report the block-0 process's counters for each candidate, and the current runner records every distinct value across the ten processes. See BENCHMARK.md.

The reserved candidate receives a correct peak hint. Reuse includes the first fill and sixteen clear/refill cycles; it is not an isolated zero-allocation steady-state loop. Wide includes full 256-byte records and output scans. Large stops just beyond a power-of-two capacity boundary deliberately.

Raw replay assets and archives: /Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-026/20261022

## Exact committed-source campaign

Source commit `51b2cf40013aa0ba5716b2917765c6d0ec602243`. Each host completed 450 processes; all source/runner hashes, library tests, oracle checks and cross-candidate/cross-campaign output checksums match. These results are reported separately from the initial campaign. Later commits in this change alter `main()` argument handling and the untimed oracle in benches/portfolio.rs, a doc comment and a `Debug` derive in src/lib.rs, and run.py; both campaigns were compiled from 51b2cf4 (file hashes in EVIDENCE_RECEIPT.json) and were not rerun on the final sources.

| Host | Case | Selection | Lowest median ns/op | Observed range |
|---|---|---|---:|---|
| arm | tiny | reserved | 5.550 (reserved) | 5.487..5.683 |
| arm | medium | unresolved: reserved / std,double | 3.779 (reserved) | 3.660..3.987 |
| arm | large | unresolved: reserved / std,onehalf,double | 3.755 (reserved) | 3.712..3.841 |
| arm | wide | unresolved: onehalf / reserved | 22.818 (onehalf) | 22.719..23.812 |
| arm | reuse | unresolved: onehalf / std,double,reserved | 3.630 (onehalf) | 3.587..3.718 |
| arm | oscillate | unresolved: std / quarter | 3.503 (std) | 3.444..3.767 |
| arm | drain | unresolved: std / quarter | 3.483 (std) | 3.358..3.686 |
| x86 | tiny | reserved | 3.761 (reserved) | 3.723..4.100 |
| x86 | medium | unresolved: reserved / std,double | 2.433 (reserved) | 2.411..2.497 |
| x86 | large | unresolved: reserved / std,onehalf,double | 2.774 (reserved) | 2.741..2.809 |
| x86 | wide | unresolved: onehalf / reserved | 46.556 (onehalf) | 39.971..65.824 |
| x86 | reuse | unresolved: reserved / std,onehalf,double | 2.405 (reserved) | 2.383..2.436 |
| x86 | oscillate | std | 2.323 (std) | 2.209..2.453 |
| x86 | drain | std | 2.372 (std) | 2.348..2.448 |

Known-size reservation wins tiny on both hosts. Retaining capacity wins x86 oscillate/drain; other cells remain unresolved under the fixed rule. See committed-summary.json for every candidate. Candidate order is not retuned.

All nine workspace/example checks passed. Four compiled semantic mutants are rejected: lost append, clear releasing storage, premature quarter shrink, and odd-capacity rounding. Raw gates/mutants remain external. The initial compile/lint fixes are retained; exact replay uses the final lint-clean executable bytes.
