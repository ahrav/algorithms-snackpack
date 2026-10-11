# Measurements and replay

Measured source commit: `9b864643603fa2e48cb6b771cb4ade4e9620ee1a`. Five frozen source/runner SHA256 values are in [receipt](receipt.json). Published evidence-only changes do not alter them.

Local host b0f1d8752aba, Apple M1 Pro, Darwin25.6.0, arm64, ten logical CPUs, unpinned affinity; rustc1.93.1/LLVM21.1.8, edition2024,opt-level3,target-cpu=native. Full target cfg and assembly retained externally.

396 exact-source candidate processes,12 independent paired/order-balanced blocks per candidate/workload. All BTreeMap comparisons, mutation checks, library/external contracts, example and paired checksums pass.

| Workload | Linear median [min,max] | Robin median [min,max] | Scan median [min,max] | Selection |
|---|---:|---:|---:|---|
| tiny | 5.55 [5.34,6.34] | 6.03 [5.76,6.38] | 5.59 [5.28,6.07] | unresolved |
| small_hit | 7.99 [7.57,8.29] | 9.99 [9.17,10.53] | 9.16 [8.79,11.20] | linear |
| high_hit | 17.63 [16.03,19.91] | 22.16 [21.15,25.75] | 20.34 [19.54,23.40] | linear |
| high_miss | 39.34 [37.60,42.52] | 20.71 [19.73,22.00] | 39.21 [38.16,41.08] | robin |
| high_mix | 30.58 [28.80,31.16] | 21.77 [20.29,23.00] | 29.71 [28.40,31.61] | robin |
| large_mix | 35.43 [29.27,52.89] | 30.01 [21.93,36.41] | 34.73 [30.02,42.98] | unresolved |
| skew | 4.28 [4.00,4.76] | 17.49 [16.73,18.15] | 16.12 [14.93,16.39] | linear |
| same_home | 119.79 [116.29,130.76] | 145.31 [142.44,155.81] | 119.37 [116.28,125.71] | unresolved |
| churn | 125.23 [121.97,127.60] | 93.00 [90.11,97.53] | 108.05 [104.57,111.61] | robin |
| build | 18.80 [17.73,19.80] | 24.05 [22.71,25.74] | 27.03 [26.11,29.03] | linear |
| first_batch | 30.60 [28.94,34.52] | 22.19 [20.65,23.15] | 29.70 [28.51,35.46] | robin |

Units: ns/query, ns/remove-reinsert-read triplet for churn, ns/inserted key for build. Min/max is dispersion, not confidence. Median-fastest is retained in JSON for every unresolved cell; no unique choice is claimed there. Unique requires every paired rival/winner ratio>1.03 in all12 blocks.

Block0 high-miss slot inspections: linear/scan571545, Robin83594. High-hit maximum sampled probes: linear370, Robin32, while linear time wins. Counts exclude timed code and are not hardware or latency counters. Assembly retains scalar DIB comparison/conditional branch; no candidate-only PMU/cache/ISA explanation.

Initial scratch396-process campaign remains separate: small_hit unresolved initially versus linear selected in exact replay; all other selections agree. No extra campaign was chosen to force that outcome.

Both required Linux hosts failed authentication before transfer/execution after bounded refresh and retries. Arm host dev-dsk-ahrav-2b-7dc7bd93.us-west-2.amazon.com; xxl resolves to dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com. Architecture of the inaccessible xxl host remains unverified. Required exact-source396-process campaigns on each are pending. Local evidence is supplementary and supplies no Linux or cross-host winner.

Replay: from this topic directory run `python3 measurements/runner.py`. It creates raw/ once and refuses overwrite. Transfer only the five frozen paths; verify required host/architecture, SHA256 and toolchain, then run the exact runner and retain raw externally.

External retained root: `/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-015/20261011`. Raw exact-source data: `/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-015/20261011/committed-input/topics/015-robin-hood-hashing/raw`. Initial source/raw, host attempts, mutation backstops and source-commit identity are retained there.

Timing includes hashing/checksum. Lookup excludes setup/oracle/diagnostic counts/final teardown; build includes allocation/insertion/one read/drop; churn includes the triplet. First batch skips warmup but is not cold/startup. Equal DIB layout and insertion presearch isolate this portfolio; no production-library, generic-payload, resize, collision-security or universal-optimal claim.
