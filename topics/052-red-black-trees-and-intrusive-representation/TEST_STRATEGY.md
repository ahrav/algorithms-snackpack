# Test strategy and invariant audit

Exact set semantics use independent Python set/sort operations, full insertion
booleans, complete ordered output and every query bit. Each measured process
must reproduce its correctness-mode cardinality and checksum.

Native Rust tests check all720 permutations of six keys after every insertion,
ascending/descending257 keys, duplicate-heavy1024 attempts, empty input,0 and
u64::MAX, growth from zero capacity and arena birth-slot identity. Ordered
prefixes use a simple vector oracle with linear membership and sorting.

Invariant checks reject a red root, right-red link, consecutive reds, unequal
black height, ordering violations, unreachable arena records and repeated slot
reachability. Deletion/reuse/concurrency are outside the implemented contract.
Mutation probes must show that skipped root recoloring, skipped left rotation
and overwritten duplicate keys are rejected. The tests are bounded evidence.
