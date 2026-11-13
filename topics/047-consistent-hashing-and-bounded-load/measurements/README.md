# Native measurements

Frozen executable source: `a32d4584dae47cf05de38e5ad314e1218ce7af5d`.
The final documentation commit preserves every transferred executable blob.
Run `python3 run.py` from this topic directory on Linux. Source archive,
exact flags, hashes, environments and verified external raw archives are in
EVIDENCE_RECEIPT.json. Compact full-cell statistics and endpoint ledgers are
in results.json. Raw archives remain outside Git.

Each host executed 341 fresh processes: 282 main, 47 warmup, 12 A/A.
All full assignment vectors, all three membership-transition vectors, source,
runner, linked binary, archive/member hashes and balanced schedule checks pass.
Rust unit and independent linear-oracle integration tests pass on both hosts.
Both hosts use their first allowed CPU, native optimization and warm inputs.
Arm rustc 1.98.1 and x86 rustc 1.99.0 differ, so timings compare within a host.

## Batch decisions

Modulo wins all seven unrestricted stateless cells on each host. Ring1 wins
all seven stable named-owner cells on each host. Bounded replay is the only
implemented count-cap policy; its feasibility supplies no comparative speed
claim. Three map-build contests per host are unresolved under the declared
all-six-paired >5% rule, including modulo/HRW near ties. See BENCHMARK.md for
boundaries and the stopping rule. All times below include output allocation
and teardown; bounded includes its count allocation/reset/teardown.

Uniform N=32,K=32768 batch times, microseconds, median [min,max]:

| Policy | Arm64 | x86-64 | Max item count / mean | Add-owner key movement |
|---|---:|---:|---:|---:|
| modulo | 129.580 [128.470,131.843] | 176.157 [138.710,199.465] | 1.072 | 96.896% |
| ring1 | 816.571 [815.452,820.252] | 303.315 [300.800,425.733] | 4.683 | 1.801% |
| ring32 | 1643.959 [1637.284,1680.186] | 795.623 [556.207,913.980] | 1.474 | 2.573% |
| hrw | 3716.839 [3711.377,3744.396] | 4965.293 [4933.260,6629.480] | 1.057 | 3.174% |
| bounded | 1658.526 [1629.950,1665.365] | 741.377 [589.647,1100.055] | 1.250 | 3.085% |

Ring32 retains 33,024 vector payload bytes at N=32; ring1 retains 1,280;
modulo and HRW retain 256. These exclude struct/allocator overhead, capacity
slack and transient output. The K=32768 output alone holds 262,144 payload
bytes; bounded additionally holds 256 count bytes on these 64-bit targets.
Ring1's speed comes with its observed 4.683 maximum/mean ratio in this cell.
Rendezvous's observed ratio is 1.057. Choose requirements before comparing speed.

Targeted N=32,K=4096 keys all map to one ring32 owner. Ring32's maximum is
4096 items; bounded limits every owner to 160. Independent reference work
counts 4096 ring32 token visits and 10,579 bounded token visits. At uniform
N=32,K=32768 the counts are 32,768 and 33,243. These are model operation counts,
not isolated hardware counters. HRW evaluates KN=1,048,576 scores in that
uniform cell. Generated assembly is retained externally; its map-assignment
symbol and allocation calls were inspected. Hardware causality is unmeasured.

The first item's declared work weight is 9K and each other item's weight is
one. Its indivisible work fraction is 9K/(10K-1), over 90%. Bounded's targeted
maximum work share is 90.390%, while its item ratio is capped at 1.25. Weighted
movement also differs: uniform N=32 modulo removal changes 96.924% of keys
and 99.692% of work under these weights. These calculations describe assigned
weights, not measured request execution, bytes or transfer traffic.

A/A ring32 timing ratios B/A:

- arm: 1.0100, 0.9831, 0.9994, 0.9407, 0.9959, 1.0048

- x86: 1.0104, 0.9965, 1.0019, 0.9951, 0.9955, 1.0041

## Evidence limits and selection

These six-block comparisons are descriptive screens on one fixed hash map,
not confidence intervals or population-wide guarantees. Small inputs are
included; cold-cache and startup are outside timing. Shared-host scheduling,
toolchain differences, synthetic keys, trusted noncryptographic hashes and
fixed replay order limit external validity. Build has a separate boundary;
full service latency, peak RSS and actual data migration remain unmeasured.
No rerun was selected for a more favorable result.

Choose modulo for fixed membership or a stable slot layer. Choose ring1 when
its measured geometric imbalance is acceptable and named-owner monotonicity
matters. Choose ring32 or HRW when their realized balance justifies their
lookup cost; HRW is better balanced in the declared uniform N=32 cell.
Choose the validated bounded replay for this batch unit-count cap. A live
bounded-load service needs counter ownership, assignment publication,
capacity maintenance and migration protocols.
