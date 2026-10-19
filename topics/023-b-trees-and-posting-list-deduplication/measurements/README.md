# Exact committed-source results

Source commit: `cb7e3eb97adc9879ec05566e65f5b77e96700f34`. Three measured files match the final topic tree.
504fresh processes per host, every exact query/oracle and cross-candidate checksum passes.
Initial504/host and overlap504/host remain separate. Primary alone selects winners.
The earlier quadratic-oracle prototype was stopped; no accepted result is taken from it.

## Selection and dispersion

Times below are microseconds per declared batch, median[min,max] across8processes.
Lowest median is named even when unresolved; only resolved rows meet the fixed all-pairs3% rule.
These ranges are descriptive, not confidence intervals. Candidate repeats differ by cell.

### arm

- tiny: flat 236.68,231.61,240.38; resolved.
- distinct: flat 22.18,21.76,22.73; resolved.
- duplicates: packed32 87.40,86.62,89.63; unresolved versus packed8.
- hot: packed32 1878.21,1855.49,1918.32; unresolved versus packed8.
- range: packed32 120.41,119.98,121.16; unresolved versus packed8.
- broad: flat 111.40,111.29,112.39; resolved.
- build_distinct: flat 405.54,401.60,413.73; resolved.
- build_duplicates: flat 847.52,842.91,856.92; resolved.
- small_lifecycle: flat 70.36,69.49,71.28; resolved.

A/A paired ratio range across cells: 0.920..1.150.

Host `dev-dsk-ahrav-2b-7dc7bd93.us-west-2.amazon.com`; architecture `aarch64`; available CPUs 64.
rustc 1.98.1 (48a229cea 2026-09-01).
Kernel: 6.12.111-136.204.amzn2023.aarch64.
CPU identity: Arm implementer0x41/part0xd40/variant1/revision1; friendly model name not exposed by lscpu.

### x86

- tiny: flat 253.47,250.16,263.30; unresolved versus packed8,packed32.
- distinct: posting 25.57,25.23,26.24; resolved.
- duplicates: packed32 76.17,75.86,76.55; unresolved versus packed8.
- hot: flat 2094.48,2087.51,2333.39; unresolved versus packed8,packed32.
- range: packed32 107.46,107.20,107.81; resolved.
- broad: flat 111.03,110.50,111.29; resolved.
- build_distinct: flat 341.37,334.56,349.98; resolved.
- build_duplicates: flat 742.57,738.28,754.15; resolved.
- small_lifecycle: flat 71.99,71.22,72.69; resolved.

A/A paired ratio range across cells: 0.843..1.104.

Host `dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com`; architecture `x86_64`; available CPUs 192.
rustc 1.99.0 (b940084d7 2026-09-28).
Kernel: 6.12.111-136.204.amzn2023.x86_64.
CPU identity: Intel Xeon Platinum8488C.

## Controls and limits

Input/workload, nativeflags, source and binary SHA256, cpuinfo/lscpu/kernel, affinity,
compiler/features, schedule, attempts and all raw observations are retained externally.
Unpinned scheduling, different toolchains and no frequency/cache-flush controls limit cross-host
interpretation. No cold-storage, tail, ISA-only, PMU, concurrent or production database claim.
Initial x86 duplicate-key cell resolved packed32; primary does not. Keep the uncertainty.
The overlap replay shared hosts with allocation diagnostics and is excluded from selection.

Allocation diagnostics use a separate unshipped System allocator forwarding adapter.
Unsafe is confined to exact trait delegation, no pointer dereferences. Counted build and
64query windows exclude input/oracle/formatting. No instrumented timings select winners.
At duplicates32768/128 both hosts: flat build 1 alloc/0 realloc; posting 146/768; packed-32
6/18. The query batch for flat/posting/packed-32 has 34 alloc/0 realloc; pair-tree 34/204.
Logical bytes:flat 524,288; posting 266,240; packed-32 265,280. Requested/encoded bytes are
not live/peak/RSS, reserved capacity or allocator overhead.30fresh diagnostic processes pass.

Exact binaries were linked-code inspected. Index::build and Index::range remain distinct
symbols and dispatch among enum variants; flat construction scales pair bytes by16 on x86.
Inlined lower-group work is inside Index::range. No candidate-only PMU or elapsed-time
attribution is claimed. Full symbol/disassembly extracts remain external.

## Retention

Raw root: `/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-023/20261019`. Initial, committed-overlap and committed-primary are distinct.
Frozen source, replay wrapper, manifests, raw attempts, tests/mutants, diagnostics and
visual review are retained before worktree or remote scratch cleanup.
Compact aggregate.json and EVIDENCE_RECEIPT.json identify all accepted inputs/results.
