# Correctness strategy

Independent authority: BTreeSet and direct canonical-key equality,not any filter
hash or fingerprint. Library runs cardinalities0,1,31,256,4096 plus10000 absent
probes per size; all present keys must pass and exact rechecked answers must agree.
Public contract covers duplicates,0,u64::MAX,empty filters,tiny rounded layouts.

Discriminating invariants: complete failed cuckoo admission leaves every bucket
unchanged including zero budget; two admitted copies survive one known deletion;
an absent colliding key deliberately breaks the delete precondition and reproduces
a false negative. Binary equations verify every original row and reject contradictory
or zero-left/nonzero-right rows. Each benchmark process checks all admitted members
and exact results for its complete query list before timing.

Limits: bounded examples are not proof for every key/capacity; probabilistic error
models need a hash contract and sampling beyond one workload. Mutable compact QF,
concurrency,persistence,adversarial hashing and production Ribbon are not implemented.
