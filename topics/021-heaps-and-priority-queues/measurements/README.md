# Exact committed-source measurements

Source: `c3947b28b937aad93cc403acd1bd0de50a148634`. All three measured inputs have matching
local committed, transferred and remote metadata SHA-256 identities. Initial and
committed campaigns remain separate. Each completed576 fresh processes per required
host, all per-pop/final-state/invariant oracles pass. Initial/exact output checksums match.

| Cell | Arm lowest median, ms [min,max] | x86 lowest median, ms [min,max] |
|---|---|---|
| tiny | four 0.124 [0.124,0.128]; unresolved with eight | eight 0.117 [0.114,0.121] |
| small | four 0.316 [0.312,0.319] | lazy 0.309 [0.302,0.312]; unresolved with eight, compact |
| mixed | four 0.762 [0.758,0.766] | lazy 0.743 [0.730,0.754]; unresolved with compact |
| updates | scan 0.376 [0.362,0.387] | scan 0.258 [0.250,0.268] |
| ties | four 0.633 [0.628,0.636] | eight 0.668 [0.662,0.675]; unresolved with four |
| stale | scan 0.169 [0.165,0.171] | scan 0.132 [0.128,0.144] |
| build_random | lazy 0.690 [0.687,0.693]; unresolved with compact | lazy 0.732 [0.716,0.742]; unresolved with compact |
| build_descending | compact 0.341 [0.336,0.349]; unresolved with lazy | scan 0.511 [0.492,0.533] |

Lowest median is an observation. A winner is resolved only when every rival/winner
ratio exceeds1.03 in all8 paired blocks. These descriptive ranges are not confidence
intervals. The identical-binary A/A rows use the same command, binary, schedule and
parser; they diagnose mechanical symmetry, not null calibration or a noise floor.

Arm: `dev-dsk-ahrav-2b-7dc7bd93.us-west-2.amazon.com`, aarch64,64 available CPUs,
Rust1.98.1/LLVM22.1.8. x86: runtime `xxl` resolved to
`dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com`, x86_64,192 available CPUs,
Intel Xeon Platinum8488C, Rust1.99.0/LLVM23.1.1. Kernel, flags, features,
CPU inventory, affinity, image hashes and all samples are retained externally.

All candidates use the same fixed-universe identity and FIFO contract. `updates`
contains99%sets and1%cancels (no pops), plus final peek; `stale` ends24576 hot-ID
updates with one pop and final peek. This distinction explains the tested scan
economics without claiming a universal CPU mechanism. Raw lazy peak records and
stale-pop/rebuild counts are measured separately from timings; record count is not
allocated capacity or RSS. No candidate-only PMU or instruction-count claim.

All8 Rust workspace gates pass, including all-target tests, strict Clippy/docs and
benchmark compilation. Four compiled counterexamples to FIFO age, reverse positions,
priority-only freshness and delayed compaction are rejected by the actual tests.

Frozen population: these exact hosts, toolchains, input traces and warm unpinned
process windows. No cold-cache, production payload, concurrent, tail or ISA-only claim.
Build cells measure construction+first minimum+drop, not a drain-all queue lifecycle.

Complete raw campaigns, hashes, attempts, tests, schedule and research are retained at:
`/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-021/20261017`. Compact receipt records selections and A/A ratios; every candidate summary remains external.
