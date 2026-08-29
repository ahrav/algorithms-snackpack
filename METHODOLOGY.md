# Methodology

The goal is not to collect textbook implementations. Each topic asks what the
best implementation means for one explicit workload, contract, and machine.

## Daily sequence

1. State the public contract, failure classes, semantic equality, and workload.
2. Research primary sources, production implementations, and known lower bounds.
3. Write the complete lesson in the task before changing the repository.
4. Implement the simplest independent reference model.
5. Implement serious candidates without sharing correctness-critical logic with
   the model.
6. Count abstract operations, comparisons, branches, bytes touched, allocations,
   and synchronization events where relevant.
7. Build tests at the smallest boundary that can observe each promise.
8. Audit every invariant-bearing test against a plausible wrong implementation.
9. Inspect optimized intermediate representation or assembly when code shape is
   part of the explanation.
10. Run controlled benchmarks only after correctness gates pass.
11. Retain raw evidence, state exactly what passed, and list untested domains.

## Required test-strategy record

Every topic contains a compact record with these fields:

```text
Scope:
Contract and failure classes:
Primary technique:
Secondary techniques:
Oracle:
Input/model/schedule domain and bounds:
Replay assets:
What a green run establishes:
Known gaps and delegated skills:
Concrete tests to add:
```

Prefer exact examples and boundary tables for finite contracts. Use generated
properties for large value domains, state-machine sequences for mutable data
structures, raw-byte fuzzing at decoder boundaries, and concurrency exploration
only when schedules are part of the contract. Generated and fuzzed runs are
sampled evidence. A reference model must be simpler and independently
implemented; calling the optimized implementation from the oracle is circular.

## Required invariant-test audit

For every test whose name claims an invariant, rejection rule, state-machine
property, replay guarantee, model agreement, metamorphic relation, snapshot
contract, or concurrent consistency property, record:

```text
Claim and scope:
Plausible violation:
Observation:
Oracle and independence:
Comparator:
Smallest discriminating case:
Mutation backstop:
```

A test is inadequate when the violating behavior cannot reach the assertion,
the expected value is circular, a proxy assertion replaces the promised state,
or canonicalization erases a contract-relevant difference. Rejection tests must
check the exact error and absence of forbidden effects. Stateful tests compare
after each transition, not only at the final snapshot when history matters.

## Performance contract

“Optimal” is always scoped. A topic must name:

- the operation mix and input distribution;
- latency or throughput estimand, including tail behavior when relevant;
- cold, warm, steady-state, and construction boundaries;
- memory, allocation, code-size, and preprocessing budgets;
- architecture, compiler, flags, source commit, and linked binary identity;
- the experimental unit and why inner-loop iterations are not independent;
- ordering, warmup, stopping, exclusions, and retained raw samples;
- a baseline, an independent correctness oracle, and at least one A/A control;
- which claims are measured, derived, observed in code generation, or inferred.

Count work before timing it. Depending on the topic, that includes comparisons,
hash computations, probe lengths, swaps, loads, stores, branches, bytes read and
written, allocations, cache-line crossings, atomics, and retries. Nanoseconds
matter only after the timing boundary and noise controls are credible.

No single winner is universal. Keep a portfolio when input shape, construction
cost, mutation rate, or hardware changes the best choice.
