# Correctness strategy and invariant audit

Independent oracle uses stable comparison sorting and division/modulo digit arithmetic;
optimized candidates use shifts/masks and histogram placement. Both are tested before
candidate timings can enter selection. Exact boundaries plus canonicalized full rows
reject missing/duplicated records and payload loss. Direct bucket membership rejects
right counts/wrong placement. Stable rows match oracle exactly; cycles exempt from order.

Nine lib groups: running example; empty/singleton/empty buckets; zero bits/high bit/max
key/payload;513 equal keys and512 structured keys;21,845 exhaustive vectors lengths0..7
alphabet4;128 seeded length/shift/width shapes; three invalid digit cases.
Two external regressions verify caller-owned cycle permutation and stable digit grouping
that does not sort complete keys. Example asserts boundaries and exact source identities.

Invariant-discriminating failures: inclusive prefix -> overlap/gap; cursor advanced before
write -> missing first/overflow last; payload-only swap -> canonical row mismatch; global
high-then-low digit passes -> incorrect boundary/order; unstable ties in stable scatter ->
exact oracle order mismatch; treating u64MAX as sentinel -> boundary tests.

Bounded checks are not mathematical proof or exhaustive u64 validation. Allocation failure
uses standard Vec policy. No unsafe, concurrency or arbitrary-width key support is claimed.
