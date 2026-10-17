# Test strategy

Scope: sequential fixed-ID keyed min queues, FIFO birth-preserving update,
removal/reinsertion and stale-record validity across seven candidates.
Contract and failure classes: full comparator, one live record per ID, repaired
reverse positions, checked fresh counters, exact rejection and compaction count.
Primary technique: finite state-machine exploration with independent ordered-tree
oracle. Secondary techniques: boundary examples and direct representation checks.
Oracle: independent BTreeMap/BTreeSet state and clocks, no heap helpers.
Input/model/schedule domain and bounds: 7^5 traces, 2IDs, 5steps, priorities0/1/2;
0..34 bulk lengths; u64 extrema and private clock exhaustion; declared benchmark traces.
Replay assets: src/lib.rs, tests/public_contract.rs, benches/priority.rs, scripts/run.py.
What green establishes: tested sequential semantics and exact benchmark outputs.
Known gaps: allocation-failure injection, arbitrary payloads, concurrency, unbounded
trace space, sparse ID allocation and durable namespaces.
Concrete tests to add: fallible storage admission and bounded maintenance if introduced.

## Invariant-test audit

| Claim and scope | Plausible violation | Observation and independent comparator | Smallest case / mutation backstop |
|---|---|---|---|
| FIFO live update/reinsert | renew birth on update, or reuse on reinsertion | exact popped IDs and oracle birth keys | equal-priority A/B/C, update A, remove/reinsert B |
| Stale freshness | accept priority alone | exact popped stamp/ID, per-transition state | A5->A1->A5 then pop twice |
| Reverse map/order | omit swap-map repair or use only downward repair | valid() plus independent tree outputs after every step | 2IDs with priority change0/2; delete last and partial child groups |
| Rejection unchanged | mutate clock/state before admission | exact Error and snapshot equality | invalid ID; MAX stamp; absent ID with MAX arrival |
| Bulk order | skip root/last parent or assume full child groups | complete drained ordered-tree equality and valid() | lengths0,1,2,3 plus fanout boundaries |
| Compaction record cap | check threshold before rather than after mutation, lose latest version | valid() and exact final latest priority | one ID,1000 increasing updates |
| Empty public API | return stale record or admit out-of-universe ID | exact None/Error, empty state and valid() | universe0 |

The finite oracle shares the public Entry representation only. It independently
maintains order, membership and both clocks; it calls no candidate repair helper.
Mutation witnesses and their actual results are retained externally when run.
