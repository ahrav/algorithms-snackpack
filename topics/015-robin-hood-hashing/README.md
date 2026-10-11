# Robin Hood hashing

A first-empty linear map can leave a late-arriving key far from home. Robin Hood
placement gives a contested slot to the entry with the larger displacement.
This lab isolates placement, early miss termination and mutation cost.

## Contract and running example

Single-owner `u64 -> u64` maps, power-of-two fixed capacity of at least two,
exact equality, update returns the old value, erase returns the removed value.
A full-table new insert returns `Full` before relocation; existing keys can still
update. No tombstones, resize, shared access or stable-address guarantee. All
public methods copy values. Allocation can panic. No unsafe code.

At capacity eight with identity homes, insert keys 0, 1, 8. Linear gives
`[(slot0,key0,d0),(slot1,key1,d0),(slot2,key8,d2)]`; Robin gives
`[(slot0,key0,d0),(slot1,key8,d1),(slot2,key1,d1)]`. A miss for key16 inspects four
slots under linear and three under Robin. See `examples/demo.rs` and the labeled
[visual](states.svg).

Distance is `(slot - home) mod capacity`. A Robin miss can stop at EMPTY or when
`resident.dib < query.dib`, because insertion would have displaced that smaller
resident. This is query-relative order, not globally increasing distance:
keys 0,8,2 produce distances 0,1,0. Swaps happen only for strictly larger DIB in
this declared variant; production implementations can also break equal-DIB ties.

Robin deletion shifts displaced successors back one slot, reducing DIB by one,
until EMPTY or DIB0. Ordinary linear deletion must instead use cyclic eligibility:
it may skip an at-home entry and move a later entry over the hole. Tests reject
using the Robin stopping rule on ordinary linear placement. DIB is `usize`;
compact encodings must handle overflow without losing exact order.

## Portfolio and costs

`Linear`: simple first-empty placement and cyclic backward deletion.
`Robin`: distance-prioritized placement and early miss stop.
`RobinScan`: same Robin placement/deletion, but misses scan to EMPTY, controlling
for the effect of the extra stopping predicate.

All candidates retain the same Option<Entry> layout, including usize DIB.
This charges linear for metadata it could omit. Every insert first searches for
an existing key. Deterministic mixed hashing is included in timed operations;
it is not collision-attack protection. Constant and identity modes are witnesses.

Let n be live keys, m capacity, P_j successful probes for key j and mu their mean.
In the example, occupancy n/m=3/8. Linear probes [1,1,3], Robin [1,2,2]:
mu=5/3 for both, but sum((P_j-mu)^2)/n falls from8/9 to2/9. Reduced dispersion
need not reduce mean time, especially with hot keys and per-slot control work.
Lookup cost is hashing plus inspected slots, equality and control; erase adds
payload movement. Long clusters and relocation remain possible. A random-probing
paper's constant-variance result is not a linear-table latency bound.

## Run

```bash
cargo test -p robin-hood-hashing --all-targets
cargo run -p robin-hood-hashing --example demo
cd topics/015-robin-hood-hashing
python3 measurements/runner.py
```

The runner refuses to overwrite `raw/`. Keep raw process evidence externally.
See [benchmark contract](BENCHMARK.md), [measurements](measurements/README.md),
and [test strategy](TEST_STRATEGY.md).

## Selection and boundaries

Use the exact-source results in measurements/README.md. Select by actual hit/miss,
skew, occupancy, build and mutation mix. Probe-count percentiles are not latency
percentiles. The standard map is the production default absent a measured gap.
Required Linux runs remain pending after Midway authentication failed before
execution. Local M1 Pro evidence cannot establish an Arm-Linux or x86 winner.

Catalog015 has no additional named coverage. Revisit: compact-DIB overflow policy,
dense payload indirection, resize tails, alternate probing models, large values,
reference invalidation and synchronized relocation.

## Primary sources

- [Celis thesis, 1986](https://cs.uwaterloo.ca/research/tr/1986/CS-86-14.pdf)
- [Random-probing model and variance](https://arxiv.org/abs/1605.04031)
- [unordered_dense v4.5.0 source](https://github.com/martinus/unordered_dense/blob/v4.5.0/include/ankerl/unordered_dense.h)
- [Rust HashMap](https://doc.rust-lang.org/std/collections/struct.HashMap.html)
