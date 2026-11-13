# Test strategy

Scope: sequential equal-size eviction state machines with dense key lookup.
Contract and failure classes: prior residency, bounded residents, exact LRU
order, CLOCK bit clearing/hand advance, segmented promotion/demotion, capacity
zero/one, invalid-key rejection without mutation. Trace length bounds timestamps.
Primary technique: exhaustive transition sequences with independent VecDeque
models and linear CLOCK slot search; compare full state after each transition.
Secondary techniques: hand-computed scan trace, work counts, Python OrderedDict
and linear-slot oracles on every measured full trace.
Oracle: independent queue/state model, separate input generator, complete hit
string, ordered final state, bits, hand and abstract work counters.
Input/model/schedule domain and bounds: all 4^6 length-six sequences, capacities
0..4, all four candidates, six checked prefixes per sequence. 81,920 complete
runs and 491,520 transition-state comparisons. Three contract test groups.
Replay assets: tests/contracts.rs and measurements/run.py.
What a green run establishes: agreement within these finite domains and frozen
workloads. Known gaps: concurrency, expiry, invalidation, sparse indexes, actual
backend/byte costs and unbounded traces. Add these when their contracts exist.

# Invariant-test audit

Claim: exact transition state and hit history. Plausible violations: omit LRU
movement, retain a CLOCK hit bit incorrectly, or skip protected demotion.
Observation: hit prefix plus ordered queues/slots/bits/hand after each operation.
Oracle independence: VecDeque linear search versus array-indexed links and map.
Comparator: exact state and hit vectors. Small cases: cap2 trace0102 for LRU;
cap2 trace01202 for CLOCK; cap3 trace0101 for demotion. Mutation backstop:
task scratch checks deliberately broken LRU/CLOCK/segmented copies against the
exhaustive test; retain actual outcomes externally.

Claim: scan trace and operation counts. Plausible violation: protect new objects
immediately or count only final CLOCK victim. Observation: exact hit count and
13 CLOCK /12 timestamp candidate inspections. Comparator: hand-computed values.
Smallest retained witness: capacity3, trace010123401.

Claim: invalid key preserves state. Plausible violation: evict before key
validation. Observation: exact error and unchanged full state after a resident
insertion, plus empty-universe boundary. Comparator: exact Snapshot equality.
