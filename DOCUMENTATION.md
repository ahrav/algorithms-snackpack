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
- `measurements/README.md`: workload, artifact identity, environment, commands,
  raw-data paths, operation counts, results, and evidence boundaries;
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
- raw samples are retained; summaries never replace them.

Before publication run:

```bash
git diff --check
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo bench --workspace --no-run
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
```
