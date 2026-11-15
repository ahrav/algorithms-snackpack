# Correctness strategy

Independent oracle: accepted suffix costs fit B+R*elapsed. Rust scans every
accepted suffix using u128, independently of refill and virtual representations.
16 capacity/rate combinations times 512 deterministic attempts cover zero,
capacity, oversized and random costs. Separate tests exercise rejected attempts,
backward-time state preservation, long idle saturation, zero capacity/rate and
maximum u64 time/rate/cost. Python uses arbitrary-precision cumulative admitted
cost and suffix-envelope intercepts, with direct history cross-checks.

Invariant audit: removing the capacity cap fails long idle; charging a rejected
cost fails the running example; dropping the backward guard fails state
preservation; narrowing the unguarded multiplication fails maximum arithmetic.
These are executable contract checks, not an exhaustive interleaving proof.
Serial scope gives the caller responsibility for synchronization and composition.
