# Test strategy and invariant audit

Scope: exact byte scripts for Matrix, Hirschberg and Myers at retained source.
Contract: consume all source, emit exact target, minimize unit insert/delete cost.
Primary: exhaustive integration test over all 63 binary strings of length 0..5,
3969 pairs, three candidates. Oracle: separately implemented forward edit-graph
shortest distance. Replay checker compares complete consumed and emitted bytes.
Secondary: empty, identical, unequal, repeated, high-byte and ambiguous alignment
examples plus each timed workload checked outside timing. Rustdoc fixes cost 5
for ABCABBA/CBABAC. Generated workload execution occurs on Arm64 and x86-64.

Negative controls executed in semantic_controls_and_boundaries: a wrong kept byte
fails target replay; a substitution-cost-one premise fails exact cost; delete+
insert on identical input fails minimum cost. Tests accept different optimal ties.
These controls discriminate semantics rather than merely detecting a panic.
Exhaustion establishes the bounded binary domain, with larger synthetic checks.
Input/resource exhaustion, production tokenization, human readability and unsafe
concurrency are outside this sequential safe-byte contract. No dependencies added.
Replay commands and frozen transfer identities live in measurements/runner.py.
