# Dynamic arrays and growth policies

An event stream needs contiguous indexed storage without knowing its final length. Compare standard Vec growth, exact incremental reservation, rounded1.5x, doubling and a known-peak reservation. All preserve append/pop/clear sequence semantics. Compare retain/tight/quarter-threshold shrink on oscillating and full-drain workloads.

Contract: nonzero-width inline u64 records; safe Vec backing; checked capacity target; fallible growing reserve before append; caller hint is not an admission bound; pop empty returns None; clear retains storage; shrink requests do not promise physical release. No raw-pointer stability guarantee. Zero-sized records/general Drop/panic semantics and injected allocator failures remain revisit scope.

## Cost and selection

Spare capacity postpones growth. In an ideal exact geometric sequence with factor g>1, initial capacity C0 and final Cf, copying all live elements at every growth costs M=(Cf-C0)/(g-1). For nine8-byte events, capacities4,8,16 imply M=12old records=96bytes, final slack(16-9)*8=56bytes. Separate old+new final buffers would request(8+16)*8=192bytes excluding metadata. Rounded1.5 capacities4,6,9 imply10old records; rounding changes the ideal formula. Incremental full-copy growth costs n(n-1)/2, or36records at n=9. These are conditional copy models, not observed allocator bytes. Amortized constant append does not bound one growth pause.

T≈R*a+B/b+U: R allocation requests, a average request overhead, B actual moved bytes, b effective bandwidth, U other work. For the doubling example3requests and hypothetical96bytes give3a+96/b+U. Neither a nor b was isolated. Actual growth can occur in place. Address changes do not measure physical copies; reported capacity is not resident memory.

Standard Vec is the default. Reserve a trustworthy bound once; reuse across batches when retention is acceptable. Smaller geometric growth trades more requests for less slack. Avoid tight shrinking on alternating pop/push; quarter occupancy then half capacity leaves a gap before growth. Preallocate an enforced bound or consider segmented/incrementally migrated storage for hard latency limits. Those alternatives, allocator contention/cold state, memory pressure, batch APIs, inline vectors and general object lifetimes remain explicit revisits.

## Run

From repository root:

```bash
cargo test -p algorithms-growth
cargo run -p algorithms-growth --example ninth_event
cargo bench -p algorithms-growth --bench portfolio
cd topics/026-dynamic-arrays-and-growth-policies
python3 run.py
```

`run.py` uses rustc directly and freezes -O -C target-cpu=native; retain its raw directory outside Git. See BENCHMARK.md and measurements/README.md for timing boundaries, all candidate results and limits.

Rust reserve_exact additional counts from length, not capacity: if len5/cap8/target16, request11additional. with_capacity is not a maximum; clear does not shrink. Vec growth factor, exact allocator size and shrink release are not contractual.

## Primary sources

- [Rust Vec guarantees and reservation/shrink APIs](https://doc.rust-lang.org/stable/std/vec/struct.Vec.html)
- [MIT6.006 dynamic arrays and amortized analysis](https://ocw.mit.edu/courses/6-006-introduction-to-algorithms-spring-2020/79a07dc1cb47d76dae2ffedc701e3d2b_MIT6_006S20_lec2.pdf)
- [CPython3.14.0 list resizing](https://github.com/python/cpython/blob/v3.14.0/Objects/listobject.c), a pointer-list implementation illustration, not a transferred growth recommendation
