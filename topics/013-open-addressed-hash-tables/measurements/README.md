# Exact committed-source results

Measured source commit: `944cf9c0be104b7499b314119b29bc35c9c08b88`. Five input hashes match the Git blobs.
Initial scratch and committed campaigns each 360 candidate processes/host.
Correctness: seven library groups, two external tests,example,every process oracle pass.

| Workload | Arm selection | x86 selection | Median microseconds L/R/S, Arm | Median microseconds L/R/S, x86 |
|---|---|---|---|---|
| tiny | unresolved | unresolved | 1.9 / 1.9 / 1.9 | 1.6 / 1.6 / 1.6 |
| clean25 | unresolved | unresolved | 443.0 / 443.8 / 442.5 | 290.4 / 288.4 / 288.0 |
| clean75 | unresolved | unresolved | 1336.0 / 1347.9 / 1337.0 | 1406.8 / 1414.2 / 1406.1 |
| clean87 | unresolved | unresolved | 2177.0 / 2180.4 / 2176.5 | 2019.0 / 2012.7 / 2028.7 |
| churn_read | shift | shift | 32120.6 / 2051.7 / 1380.8 | 50516.1 / 2066.4 / 1452.1 |
| mixed25 | shift | shift | 1294.3 / 1205.0 / 835.9 | 1352.5 / 1259.5 / 860.9 |
| mixed75 | shift | shift | 7187.2 / 2939.1 / 2616.4 | 8260.9 / 3168.4 / 2785.0 |
| cluster | unresolved | unresolved | 3668.4 / 3670.5 / 5649.8 | 2644.4 / 2618.9 / 4355.7 |
| build | unresolved | unresolved | 538.0 / 539.0 / 538.5 | 547.0 / 554.8 / 548.2 |
| first_lookup | unresolved | unresolved | 1393.9 / 1390.5 / 1388.4 | 1457.0 / 1440.1 / 1444.5 |

L/R/S means lazy,rebuild,shift. Selection requires every paired rival/winner
ratio > 1.05 in all 12 blocks. Unresolved cells retain descriptive medians,
observed ranges and paired-ratio ranges in results.json; no confidence
interpretation or universal optimum. Read-only churn excludes prior maintenance.
Mixed includes maintenance; build includes allocation,growth,drop.
First lookup skips warmup but is not startup/cold-cache controlled.

Arm: dev-dsk-ahrav-2b-7dc7bd93.us-west-2.amazon.com, aarch64, rustc 1.98.1.
x86: runtime-resolved xxl dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com, x86_64, rustc 1.99.0.
CPU/kernel/cfg/native flags/affinity/binary/input hashes are in EVIDENCE_RECEIPT.json.
Different compilers prevent isolated ISA comparison. No A/A or candidate PMU
attribution, startup/tail guarantee, wide-value or sparse-iteration timing.

Raw identity, oracles, 360 samples and disassembly per host are externally retained
as committed-arm-raw.tar.gz and committed-x86-raw.tar.gz under:
/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-013/20261009

```bash
python3 measurements/runner.py
```
Run in a fresh copy; raw/ must not exist.

Guide: shift for declared selected churn/mixed workloads; retain unresolved
choices where the frozen criterion does not separate. Hash concentration can make
shift deletion move a whole cluster. Test rebuilding when maintenance spikes
are acceptable and later reads repay it. Extendible model has operation counts
only; compare total latency and memory before production adoption.
