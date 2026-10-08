# Evidence boundaries

Initial scratch:528 processes and nine lib tests per declared host, source/runner hashes
in external initial receipts. Final committed replay is required and retained separately.
Compact results include median and observed range per candidate, paired ratio ranges,
contract-specific choices and unresolved contenders. Raw process records, host identities,
assembly, failures and replay archives stay outside Git under automation evidence/topic-012.
`runner.py` freezes workload/boundary and source hashes before compilation. It verifies
both the9lib and2integration tests plus running example before benchmarking.

Run from workspace root with `python3 topics/012-radix-partitioning/measurements/runner.py`.
Do not overwrite raw; use a fresh retained exact-source extraction. Source is prehashed,
single-threaded, materialized in memory. Consumer, hashing, storage, thread lifetime and
cross-machine attribution remain outside the measured claim.

## Exact committed selection
All528 process oracle checks, nine lib tests, two external contracts and the example passed per host.
Median leader is unique only under the predeclared all12 paired ratios above1.05 rule.
- tiny: arm stable: scatter unique; arm unordered: scatter unique; x86 stable: scatter unique; x86 unordered: scatter unique.
- small: arm stable: scatter unique; arm unordered: scatter unique; x86 stable: scatter unique; x86 unordered: scatter unique.
- large16: arm stable: scatter unique; arm unordered: scatter unique; x86 stable: scatter unique; x86 unordered: scatter unique.
- large1024: arm stable: scatter unique; arm unordered: scatter unique; x86 stable: scatter unique; x86 unordered: scatter unique.
- large4096: arm stable: scatter unique; arm unordered: scatter unique; x86 stable: scatter unique; x86 unordered: scatter unique.
- skew90: arm stable: scatter unique; arm unordered: scatter unique; x86 stable: scatter unique; x86 unordered: scatter unique.
- same: arm stable: scatter unique; arm unordered: cycles unique; x86 stable: scatter unique; x86 unordered: cycles unique.
- lowzero: arm stable: scatter unique; arm unordered: cycles unique; x86 stable: scatter unique; x86 unordered: cycles unique.
- grouped: arm stable: unresolved scatter,buckets; arm unordered: cycles unique; x86 stable: unresolved buckets,scatter; x86 unordered: unresolved buckets,scatter,cycles.
- wide: arm stable: scatter unique; arm unordered: scatter unique; x86 stable: scatter unique; x86 unordered: scatter unique.
- first_call: arm stable: scatter unique; arm unordered: scatter unique; x86 stable: scatter unique; x86 unordered: scatter unique.
