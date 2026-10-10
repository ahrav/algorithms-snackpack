# Measurement evidence

Exact frozen source: `42cdb5ed9436ad196353e1d7342b86092f25164b`. All five input hashes in EVIDENCE_RECEIPT.json agree with that commit and the final working inputs of the campaign. Later commits change only the `#[cfg(test)]` module of `src/lib.rs` (excluded from the measured rlib and bench binary, SHA256 `2038ddb0…`), the runner's identity recording, and the analyzer's schedule check; see `post_measurement_changes` in the receipt.

Both local campaigns completed360 processes with every oracle passing. Initial observations are retained separately. Final committed-input selection below is authoritative for this source/campaign. Required Arm/x86 Linux SSH plus one retry each failed before execution due expired Midway authentication. No remote identity/transfer/success is claimed.

| Workload | Unfiltered median | Scalar median | Word median [min,max] ns/query | Final selection |
|---|---:|---:|---:|---|
| bad_tag | 64.2 | 64.1 | 60.3 [58.9,63.0] | unresolved |
| build_probe | 78.6 | 42.5 | 42.2 [40.2,44.0] | unresolved |
| cluster | 665.2 | 131.2 | 76.3 [73.4,77.7] | word |
| deleted | 57.3 | 25.1 | 21.4 [21.3,21.9] | word |
| first_batch | 58.4 | 22.5 | 21.7 [20.3,24.9] | unresolved |
| large87 | 147.3 | 46.3 | 38.7 [36.9,41.3] | word |
| small_hit | 36.9 | 20.1 | 16.7 [16.2,17.4] | word |
| small_miss | 38.1 | 21.7 | 19.0 [18.2,20.8] | unresolved |
| tiny | 27.8 | 13.0 | 10.4 [9.8,10.9] | word |
| wide_miss | 64.4 | 25.7 | 22.0 [21.2,23.0] | word |

The word candidate is selected for cluster/deleted/large87/small_hit/tiny/wide_miss. bad_tag/build_probe/first_batch/small_miss remain unresolved. small_hit changed from initial unresolved to selected; small_miss changed from selected to unresolved. Retain both classifications and do not extrapolate a universal threshold.

Selection uses all12 paired process ratios exceeding1.05 against every rival of the lowest median. Observed ranges are dispersion, not confidence intervals. Lower median alone does not establish a winner. No A/A, tail/cold-cache or PMU evidence was obtained; METHODOLOGY.md requires an identical-artifact A/A run, so every selection above is provisional until that control runs through this pipeline. Local Apple M1 Pro, Darwin25.6.0, Rust1.93.1/LLVM21.1.8, native optimization; no CPU affinity. The committed `identity.target_cfg` in results.json was printed without `-C opt-level=3`; its `debug_assertions` line describes that probe, not the measured binary. Linux verification is pending, not replaced by local evidence.

Wide misses use14.95 baseline equalities/query versus0.12 filtered; constant tags retain14.95 for all three. These are separate untimed diagnostic counts. Assembly confirms the word branch uses carry-isolated integer operations on two halves; it does not establish runtime cache causes.

Reproduce using a fresh raw directory: `python3 measurements/runner.py`, then `python3 measurements/analyze.py raw`. Exact runner and source are retained in committed-input. See BENCHMARK.md for boundary, warmup, ordering and workload controls. See EVIDENCE_RECEIPT.json for archive/source hashes and concrete missing host actions.

External retained archive: `/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-014/20261010/local-campaigns-and-source.tar.gz` (SHA256 `1e9ea956f3e275d169f513ad16d9c3161ed5d0125aa264683bf5056d5d7f34d8`). Raw binaries, assembly, samples and replay assets stay outside Git.
