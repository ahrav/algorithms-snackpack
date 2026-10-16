# Exact committed-source results

Source commit `0b2d8ac3f438dce577749f880528f5d7743aa498`. The receipt hashes name `src/lib.rs`, `benches/portfolio.rs` and `runner.py` at that commit; later review commits changed `benches/portfolio.rs` and `runner.py` as listed below without rerunning the campaigns. Initial and committed campaigns each completed256 fresh processes/host; all pre-timing query/final-state oracles pass. They are retained separately, not pooled.

Times below are median microseconds per whole workload replay, with descriptive min/max over8 independent process aggregates, divided by8 internal replays. They are not confidence intervals. Every winner label beat every rival by>5% in every paired block under the predeclared rule. Full medians/ranges/paired ratios for all four candidates are in summary.json. `report.py` regenerates summary.json and this table from the two raw per-host summaries; against this commit's artifacts it reproduces summary.json byte for byte and every table row.

The committed campaigns predate the identical-artifact A/A control, the linked-image hash, the `-O` cfg query, the attempt log with timeout, the per-campaign `raw/` reset and the integration-test step now in `runner.py`. Their winner labels are therefore provisional until a campaign with the control runs through this pipeline. The archived manifests' `features` came from an unoptimized `rustc --print cfg`, so their `debug_assertions` line describes the query, not the `-O` artifact; the receipt's `binary_sha256` fields were computed outside the runner. Source changes after the campaigns are the argument guard in `benches/portfolio.rs` (`args.len() < 3` prints usage so `cargo bench` exits cleanly) and the runner evidence changes above; `src/lib.rs` and the timed loops are unchanged.

| Workload | Winner label (provisional), both hosts | Arm us [min,max] | x86 us [min,max] |
|---|---|---:|---:|
| tiny | scan | 5.18 [4.50,6.21] | 4.45 [4.34,4.56] |
| read_only | prefix | 5.97 [5.69,6.33] | 5.86 [5.84,5.94] |
| mixed | lazy | 412.12 [408.35,415.12] | 452.59 [445.96,459.78] |
| short | scan | 25.45 [24.36,26.59] | 15.97 [15.86,18.20] |
| update_heavy | lazy | 479.09 [475.88,485.84] | 504.76 [496.47,510.52] |
| hot | scan | 23.66 [22.24,25.29] | 17.36 [15.80,18.94] |
| adversarial | lazy | 112.40 [111.18,116.97] | 126.32 [123.82,127.83] |
| lifecycle | scan | 32.21 [32.12,32.70] | 27.38 [26.50,29.22] |

Steady columns exclude constructor/drop; lifecycle includes new/run/drop. Input generation, oracle and printing are excluded. Separate setup_ns and logical-range element counts remain in raw rows. At n32768 retained payload is262144 Scan bytes,524296 Prefix bytes,1048576 Eager bytes,3145728 Lazy bytes; these exclude allocator metadata. Each run retains exact update counts and spans.

## Machines and limits

- arm: `dev-dsk-ahrav-2b-7dc7bd93.us-west-2.amazon.com`, `aarch64`, kernel`6.12.110-135.202.amzn2023.aarch64`, 64 available CPUs, ARM implementer0x41/part0xd40/variant1/revision1 (reported model identity; no marketing model inferred), `rustc 1.98.1 (48a229cea 2026-09-01)`. Full uname/features/affinity in retained manifest.
- x86: `dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com`, `x86_64`, kernel`6.12.111-136.204.amzn2023.x86_64`, 192 available CPUs, Intel Xeon Platinum8488C, `rustc 1.99.0 (b940084d7 2026-09-28)`. Full uname/features/affinity in retained manifest.

Generated fold/query/change symbols were inspected in both retained disassemblies: recursive boundary checks and call paths remain visible. This is code-shape evidence, not candidate-only PMU or proof of a timing mechanism. No hardware-counter claim is made.

Default target -O, different compiler versions, no pinning/frequency control, no A/A calibration in the retained campaigns, warm initialized allocations. No cold cache/device, tail percentile, production library/database, ISA-only or universal threshold claim. The full-range adversarial workload intentionally rewards compatible whole-node tags; it is not an average production range distribution. Eager and Prefix include educational update policies, not the best possible specialized alternatives. Production vectorized window APIs, exact holistic states, persistent/Beats and concurrent snapshots remain revisit scope.

Raw archive paths, sizes, byte verification and binary/source hashes are in EVIDENCE_RECEIPT.json. Replay assets and campaign orchestrator are retained at /Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-020/20261016.
