# Correctness and invariant audit

The independent Oracle stores (member,score) unsorted, edits by member scan,
and sorts a fresh swapped-field vector for order queries. It shares the public
contract but no path, rank, span, binary search or tree helper with candidates.
Every benchmark response and final snapshot are compared before warmup.

Library checks:59049 nine-action length-five histories;4000 seeded mutations
with every-step whole-state/get/rank/select/range checks; eight seeds with1024
nodes and staggered deletion checking all spans. External tests cover the full
running example, missing members, equal scores, integer extremes, zero limit,
maximum invalid rank and10000insert/remove cycles retaining one arena slot.
Tests are bounded evidence. Exhaustive history checking compares final states;
seeded churn checks every mutation. No floating-score/concurrent/OOM claim.

Discriminating invariants: removing an untouched-level span decrement breaks
rank metadata despite sorted bottom traversal; removing untouched-level
insertion increment breaks the reverse case; retaining old ordered keys on
upsert breaks member uniqueness; replacing full tuple order with score-only
breaks equal-score identity. Compiled mutation witnesses are retained outside
Git when executed. validate independently reconstructs positions and checks
active terminal spans, map equality, strict ordering, slot occupancy/free-list
bijection and successor heights. Snapshot-only tests cannot substitute for it.
