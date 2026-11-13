# Test strategy and invariant audit

- Contract: nonempty unique stable IDs, canonical order, deterministic tie-break,
  exact vector length, independent per-item mapping or complete bounded replay.
- Oracle: Rust linear cyclic-distance selection with a BTreeMap count model;
  Python fixed-width arithmetic, sorted tokens and independent assignment model
  verify every process and all membership-transition vectors.
- Boundaries: empty membership, duplicate IDs, zero items, singleton, u64 maximum,
  sparse IDs, input-order canonicalization, exact collisions and wraparound.
- Discriminating invariants: collision test rejects upper-bound lookup; token
  collision replay rejects per-token counters; cap checks reject missing reset
  or incorrect rounding; named-owner monotonicity rejects ID compaction.
- Scope: five policies, seven owner counts in the Rust differential sweep,
  lengths 0/1/2/7/32/257, and 8192 keys for named-member invariants.
  These are bounded tests. Dynamic bounded-load theorems require separate work.
- Model checks remain outside the timer. Each native process is rejected on a
  vector mismatch before timing is accepted for comparison.
