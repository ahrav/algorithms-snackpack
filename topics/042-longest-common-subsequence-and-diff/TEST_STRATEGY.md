# Test strategy and invariant audit

Scope: exact byte scripts for Matrix, Hirschberg and Myers at retained source.
Contract: consume all source, emit exact target, minimize unit insert/delete cost.
Primary: exhaustive integration test over all 63 binary strings of length 0..5,
3969 pairs, three candidates. Oracle: separately implemented forward edit-graph
shortest distance. Replay checker compares complete consumed and emitted bytes.
Secondary: empty, identical, unequal, repeated, high-byte and ambiguous alignment
examples plus each timed workload checked outside timing and under `cargo test`
at n=64. Rustdoc fixes cost 5 for ABCABBA/CBABAC. Generated workload execution
occurs on Arm64 and x86-64. Counter contracts under `cargo test`: Matrix holds
(m+1)(n+1) score slots in one allocation and counts every traceback byte
comparison; Hirschberg keeps at most 2(min(m,n)+1) live slots; Myers retains
(D+1)(D+2)/2 history slots in D+1 allocations. An unknown workload shape panics.

Negative controls executed in semantic_controls_and_boundaries: a wrong kept byte
fails target replay alone; a wrong consumed byte fails source replay alone; a
substitution-cost-one premise fails exact cost; delete+insert on identical input
fails minimum cost. Tests accept different optimal ties.
These controls discriminate semantics rather than merely detecting a panic.
Exhaustion establishes the bounded binary domain, with larger synthetic checks.
Input/resource exhaustion, production tokenization, human readability and unsafe
concurrency are outside this sequential safe-byte contract. No dependencies added.
Replay commands and frozen transfer identities live in measurements/runner.py.
