# Documentation and evidence rules

Each topic lives at `topics/NNN-slug/` and is a self-contained Rust crate unless
another language is essential to the algorithm's boundary.
Add every new Rust topic crate explicitly to the root workspace member list.

Required topic files:

- `README.md`: problem, plain-language mental model, precise contract, candidate
  comparison, cost model, selection guide, limitations, and primary sources;
- `src/lib.rs`: documented implementation and reference model;
- `tests/`: exact boundaries, independent-model agreement, properties, and
  invariant-discriminating regressions as justified by the contract;
- `benches/`: benchmarks only for predeclared performance claims;
- `BENCHMARK.md`: the mandatory benchmark decision. When it is `true`, include
  the claim, workloads, unit roles, assignment, controls, stopping rule, counters
  or profiles, and evidence boundary. When it is `false`, include the concrete
  reason timing cannot answer the central claim and name the evidence used
  instead;
- `measurements/README.md`: workload, artifact identity, environment, commands,
  external raw-data location, operation counts, results, and evidence
  boundaries;
- `measurements/EVIDENCE_RECEIPT.json`: when a benchmark runs, the external
  archive path, archive hash, size, member count, run IDs, manifest hashes,
  linked-image hash, environment identity, and verification state;
- `TEST_STRATEGY.md`: the required test-strategy and invariant-test-audit records.

Keep full lesson transcripts in the task chat. Repository prose should be short,
direct, executable, and sufficient to reproduce the claims.

Rust requirements:

- public APIs have rustdoc;
- examples and expected output are checked where practical;
- optimized code and reference models do not share correctness-critical helpers;
- unsafe code is forbidden by default and requires an explicit topic-level
  justification plus dedicated verification if a future topic enables it;
- benchmarks consume results and pin the intended code path;
- a performance-sensitive topic benchmarks the reference model and every
  serious candidate across the workload dimensions that can change the winner;
- elapsed time is paired with operation counts and profiling or hardware-counter
  evidence when the claimed mechanism depends on branches, cache behavior,
  instructions, allocations, contention, or code shape;
- raw samples are retained in the automation-owned evidence store outside Git;
  the repository keeps a compact summary and verified receipt, not per-attempt
  files or raw archives.

Before publication run:

```bash
git diff --check
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo bench --workspace --no-run
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
```
