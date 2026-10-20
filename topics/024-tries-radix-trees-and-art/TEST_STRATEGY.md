# Contract-directed verification

Independent oracle: latest-write HashMap, filter by starts_with, sort by byte key.
Candidates never call it. Exact benchmark queries use a reverse input scan.
Check before warmup; reject any response disagreement before timing.

Library groups: arbitrary byte/empty/terminal/duplicate semantics; every fanout
boundary including0,1,4,5,16,17,48,49,255,256 and every first-byte lookup;
80sampled80-write binary histories with all stored-key prefix lengths;
sparse page occupancy and owned Arc lookup after removal. Integration regression
checks zero values and0xff prefixes without constructing a successor key.
Example checks car/cart/carbon/cab order. Histories are bounded tests, not proof.

Discriminating compiled mutants: retain first duplicate; reject prefix ending
inside compressed path; treat slot0 as missing in Node48; omit radix terminal
output. All rejected by original tests. Receipts/raw outputs external.

Sparse userspace model is not a kernel XArray or concurrent algorithm test.
Arc cloned while RwLock is held models ownership transfer; it does not model
optimistic validation, RCU grace periods, or kernel API error handling.
Long hostile recursion is an explicit service-level bound and revisit item.
