# Consistent hashing and bounded-load placement

Place distinct unit items on stable named owners. Separate lookup time,
metadata, item imbalance, request work and endpoint remapping.

## Run

```bash
cargo test -p topic047-placement --all-targets
cargo bench -p topic047-placement
cd topics/047-consistent-hashing-and-bounded-load
python3 run.py
```

The process campaign requires Linux, Python 3 and rustc. For a standalone copy,
use the retained replay crate and `cargo test --all-targets`.

## Contract and example

`Map::new` accepts distinct nonempty u64 owner IDs, canonicalizes their order,
and rejects duplicates. Hash version 1 pins integer fields, constants, wrapping
u64 arithmetic, and a stable-ID tie-break. This hash assumes trusted input.
Equal token positions sort by stable owner ID and then token number.
An exact key/token hash match selects the first tied position; wrap selects the
first token. Every client requires identical IDs, policy and hash inputs.

`assign` returns an owner for every input position. The experiment uses unique
keys. Repeated values represent separate unit items sharing a hash; bounded
replay can assign them to different owners. A key directory must retain the
result of the frozen distinct-key batch. Caller order is part of that batch.

Six items all preferring B on the cyclic order A/B/C illustrate forwarding.
With N=3, K=6 and slack=0.25, capacity is ceil(5K/4N)=3. B takes the first
three and C the next three. Counts become 0/3/3. The maximum/mean ratio is
1.5 because capacity is rounded upward. All virtual nodes share the physical
owner's count. Counts reset for each complete batch replay.

## Alternatives and cost

| Policy | Mechanism | Lookup/batch work | Boundary |
|---|---|---|---|
| Modulo | hash remainder indexes sorted IDs | O(1) / O(K) | Resizing and ID-list compaction remap surviving owners |
| Ring1 | first clockwise of N tokens | O(log N) / O(K log N) | Finite ring ownership can be uneven |
| Ring32 | first clockwise of 32N tokens | O(log(32N)) / O(K log(32N)) | More metadata; an indivisible hot key stays on one owner |
| Rendezvous | maximum key-owner score | O(N) / O(KN) | Stable named-member changes; lookup evaluates all members |
| Bounded | ring32 plus forward probing | batch O(K log(32N)+P) | Shared counts, frozen order and complete batch; P counts token visits |

N is owner count, K item count, V tokens per owner, T=NV total tokens,
and P total token visits. A ring build costs O(T log T), retaining O(T)
metadata. At N=3,V=32,T=96, binary-search comparison scale is log2(96)=6.585;
K=6 rendezvous lookup evaluates KN=18 scores. These scales predict work,
not elapsed time. Vec construction and teardown belong in the measured boundary.

The bounded implementation is deterministic batch replay. The published
bounded-load algorithm's dynamic capacity maintenance and update theorem are
separate revisit work. The lab establishes a batch unit-count cap and records
its actual endpoint changes. Request rate and byte capacity require additional
accounting and enforcement.

Under equal-weight exchangeable random ownership, addition N to N+1 moves
expected 1/(N+1); removal moves expected 1/N. At N=3 those are 1/4 and 1/3.
Modulo 3 to 4 moves 3/4 under uniform residues over the common period.
Measure finite named-owner changes, including byte and request weights.
Physical recovery and executed transfers require their own migration protocol.

## Choose

Use modulo for fixed membership or a fixed logical-slot layer. Use a ring when
arbitrary named-member changes and tunable metadata matter. Use rendezvous
when named-owner behavior or ranked candidates justify N scores per lookup.
Use bounded placement for a hard unit-item count cap with shared batch state.
Choose among the correct measured candidates after applying those requirements.
See measurements for finite workload results and unresolved comparisons.

## Coverage and sources

This visit implements modulo, one/32-token rings, flat rendezvous and bounded
batch replay. Explicit revisits: Jump's contiguous logical buckets; Maglev's
table and flow-state contract; weighted capacities; failure-domain replicas;
full dynamic bounded-load capacity maintenance and migration execution.
The authoritative catalog currently supplies no coverage arrays and the
algorithm index is absent. The earlier metadata follow-up remains separate.

- [Karger et al., 1997](https://people.csail.mit.edu/karger/Papers/web.pdf)
- [Thaler and Ravishankar, 1996](https://www.eecs.umich.edu/techreports/cse/96/CSE-TR-316-96.pdf)
- [Lamping and Veach, 2014](https://arxiv.org/abs/1406.2294)
- [Mirrokni, Thorup and Zadimoghaddam, v3](https://arxiv.org/html/1608.01350v3)
