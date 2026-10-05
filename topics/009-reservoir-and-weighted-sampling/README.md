# Reservoir and weighted sampling

Keep three distinct IDs from A..H as records arrive. A uniform reservoir needs
admission probability K/t after t arrivals; weighted sampling needs a declared
probability law before choosing a container.

| State | slot0 | slot1 | slot2 | Decision |
|---|---|---|---|---|
| Before D | A | B | C | K=3,t=3 |
| During D | A | D | C | t=4,draw1; replace slot1 |
| After E | A | D | C | t=5,draw4; skip E |

The same states were rendered and inspected in the external lesson evidence.

## Contract and finite implementation

`uniform(n,k,seed,method)` samples synthetic index-stream IDs 0..n and returns
min(n,k) distinct ascending IDs. n must fit u32. These wrappers know the test
stream's termination; Algorithm R's admission recurrence itself requires only
current count. No payload decoding or source I/O is implemented. Weighted APIs
validate all weights, including k=0; accept positive finite f64 weights; reject
zero, negative, NaN and infinity with None. Output IDs are sorted for presentation,
not in weighted draw order. No caller input is mutated.

`Rng` freezes SplitMix64, state initialization and rejection/remainder range
reduction. Independent uniform source-word assumptions justify the ideal law;
this PRNG does not prove them. Open-unit values use 52-bit midpoint conversion.
X is finite-precision product inversion, bounded by source exhaustion, including
k=1. Integer-priority ties use ID. Weighted keys use
`ln(-ln(U))-ln(w)` to avoid division overflow; finite rounding can change close
ranks. Cross-host/libm bit replay is not promised.

| Candidate | Mechanism and cost | Choose / limitation |
|---|---|---|
| R | one bounded admission draw per postfill record; O(n) plus output sorting, O(k) entries | simple count-based stream; does not save decoding |
| X | one uniform skip draw then sequential survival-product factors | fewer random calls, still O(n) count work; finite rounding |
| Uniform priority sort | sort (word,ID), retain smallest k | simple materialized baseline; O(n log n), O(n) keys |
| Uniform priority heap | retain smallest k in max-heap | O(n + accepted*log k) after fill, bounded entries; finite ties |
| Weighted sort/heap | smallest retained log exponential clocks | successive weight-proportional draws without replacement; not general proportional final inclusion |

Expected R replacements are k*sum(1/t,t=k+1..n), about k*ln(n/k).
For n=8,k=3 these are2.654 and2.942. X's skip survival after t=4,k=3 for
s=2 is(2/5)*(3/6)=1/5. Z's ideal sampling-control scale
k*(1+ln(n/k)) is5.942 here. Z uses rejection-corrected skip proposals;
PostgreSQL18.0 switches X to Z after t>22k. Those CPU controls exclude traversal.
An uncapped X skip at k=1 has infinite mean; this implementation caps at n.

For weights(1,1,1,1,4,1,1,1), E is first with probability4/11 under
successive weighted choice. Final-inclusion3*4/11 is impossible. With U=.5,
E's direct clock is.173287; a weight-one clock is.693147. Choose a separate
proportional-inclusion or with-replacement design when required.

`combine` preserves assigned keys and one comparator. Preconditions: disjoint
unique IDs, finite scores, each local sample keeps at least global k or all its
items. Any locally discarded item has k better peers, so cannot be global Top-K.
Unequal shards are safe under these conditions. Equal per-shard quotas, key
regeneration, insufficient local capacity or deduplication after truncation can
invalidate this argument. The tests partition already-assigned global keys;
restarting each worker's PRNG is not worker-count invariant.

## Run

```sh
cargo test -p sampling-lab --lib --examples --tests
cargo run -p sampling-lab --example stream
cargo bench -p sampling-lab --no-run
```

The sampling benchmark takes METHOD N K SHAPE REPS SEED. Methods:
r,x,sort,heap,wsort,wheap. `measurements/campaign.py` is the frozen standalone
Linux rustc runner: place src/tests/benches/examples under topic/, include its
source-manifest.json, then run `python3 campaign.py`. Exact packaging commands,
hashes, raw observations and environment are in the external receipt. See
BENCHMARK.md and measurements/README.md for comparisons and their limits.

## Revisit scope and sources

Algorithm R, X, weighted keys, seeded identity and shard combination are runnable.
Z's complete rejection implementation, seekable versus obligatorily decoded
sources, proportional-inclusion/with-replacement candidates, arbitrary-stream
API/lifetime integration and distributed communication costs remain explicit
revisit scope. No speed claim for those unimplemented variants.

- Vitter1985, [Algorithms R/X/Z](https://doi.org/10.1145/3147.3165), sections2-8.
- [PostgreSQL18.0 sampling.c](https://github.com/postgres/postgres/blob/REL_18_0/src/backend/utils/misc/sampling.c), reservoir_get_next_S.
- Efraimidis [arXiv1012.0256v2](https://arxiv.org/pdf/1012.0256), definitions2/3 and A-ES.
- Hübschle-Schneider/Sanders [arXiv1903.00227v3](https://arxiv.org/pdf/1903.00227), section2.3 clocks.
- [SplitMix64 author reference](https://prng.di.unimi.it/splitmix64.c).
