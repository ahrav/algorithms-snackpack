# Replay evidence

Run the isolated example and independent tests on Linux.

```sh
bash measurements/run.sh
```

The runner prints machine and compiler identities, effective target features,
executable source hashes, example output, and the contract results. Time
advances by simulated unit ticks. The experiment uses identical task inputs
and cost assumptions for each policy.

## Exact-source results

Source commit `c1549a95baec7beb232ce5fd26beacd630146c1a` builds and passes five
contract groups on both Linux hosts. The replay checks 3,375 finite three-job
sets against exhaustive feasibility and 900 task pairs against enumerated
periodic releases. Arm uses Rust 1.98.1 with 64 available CPUs. x86-64 uses
Rust 1.99.0 with 192 available CPUs. The runner uses edition 2024, optimization
level 2, and the effective native target. The [receipt](EVIDENCE_RECEIPT.json)
pins inputs and logs.

Both hosts report zero EDF misses and one fixed-priority miss. Tightening
A's deadline makes four ticks due in three. Immediate refill finishes the
five-tick request at 9. Fixed-boundary refill finishes at 15. These are
simulated times, with no measured production latency or runtime ranking.
