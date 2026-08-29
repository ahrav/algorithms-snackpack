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
10. Make and record the mandatory benchmark decision. Run controlled benchmarks
    and profiles after correctness gates pass whenever runtime, allocation,
    throughput, latency, crossover, or candidate selection is part of the lesson.
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

## Mandatory benchmark decision

Every topic includes `BENCHMARK.md`, even when it does not run a benchmark. The
file begins with exactly one of:

```text
benchmark_required: true
benchmark_required: false
```

Use `true` whenever the lesson compares candidate speed, latency, throughput,
allocation behavior, memory traffic, preprocessing economics, scaling,
contention, or an input-size crossover. This is the default for algorithm
topics. Use `false` only when timing cannot answer the topic's central claim;
state the reason and the evidence used instead. “The implementation is small,”
“the asymptotic complexity is known,” or “tests passed” are not sufficient
reasons to omit profiling.

When benchmarking is required, `BENCHMARK.md` must predeclare:

- the performance claim, target population, workload matrix, input generation,
  operation mix, and semantic equivalence of all candidates;
- construction, mutation, query, cold, warm, and steady-state boundaries;
- treatment-application, randomization, analysis, subsample, and generalization
  units, plus the interference and reset argument;
- pilot variance, independent-unit count, allocation, sensitivity, effect
  boundary, confidence-bound direction, and fixed or valid sequential stopping;
- the assignment law and recorded seed. Use complete ABBA/BAAB blocks only when
  additive linear position drift is the declared nuisance model and carryover is
  negligible; retain every partial, failed, timed-out, and reset-failed attempt;
- an identical-artifact A/A run through the same build, label, schedule, parser,
  missing-data, stopping, and analysis paths, with mechanical integrity kept
  separate from null-calibration evidence;
- the benchmark family and multiplicity rule before results are inspected;
- exact source, compiler, flags, linked-image hash, host, architecture, affinity,
  environment, and raw-output location.

Benchmark every serious candidate against the independent reference model or a
clearly named baseline. Cover the input dimensions that can change the winner:
size, distribution, duplicates, ordering, hit rate, mutation ratio, skew,
adversarial structure, and memory budget as applicable. Add crossover sizes
instead of reporting one convenient point.

Profile the measured implementation rather than inferring its bottleneck from
elapsed time. Record the counters or profile that can test the mechanism, such
as instructions, cycles, branches, branch misses, cache or translation misses,
bytes, allocations, atomics, retries, or generated code. If a required counter
is unavailable, record that limitation and keep the mechanism as inference.

Only complete independent analysis-unit contrasts enter uncertainty estimates.
Inner Criterion samples, loop iterations, requests in one process, and repeated
reads of the same warmed state are subsamples. More subsamples cannot replace
process, build, host, or time-window replication when the claim varies there.

The retained result separates:

- measured timings, counters, and allocation observations;
- elementary operation-count and complexity derivations;
- observed optimized intermediate representation or assembly;
- inferred mechanisms and untested generalizations.
