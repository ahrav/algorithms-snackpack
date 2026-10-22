# Dynamic arrays and growth policies

An event stream needs contiguous indexed storage without knowing its final length.
Compare standard `Vec` growth, exact incremental reservation, rounded 1.5 growth,
doubling, and a known-peak reservation. All preserve the same append/pop/clear
sequence. Compare retain, tight, and quarter-threshold shrinking separately.

## Contract

Records contain a nonzero fixed number of inline `u64` fields. The implementation
uses safe `Vec` storage and checked capacity targets. Growing reservation errors
return before append. The capacity hint is a reservation, not an admission bound.
Empty pop returns `None`. Clear retains capacity. Shrinking does not promise
physical memory release. Mutation does not guarantee stable raw pointers.

Zero-sized records, general objects with destructors, injected allocation failures,
concurrency, batch reservation, inline vectors, cold allocators, and strict tail
latency remain explicit revisit scope. Segmented storage or incremental migration
needs a separate access and ownership contract.

## Cost model

Spare capacity postpones growth. For exact geometric capacities, growth factor
`g > 1`, initial nonzero capacity `C0`, and final capacity `Cf`, copying every live
prefix costs `M = (Cf - C0) / (g - 1)` old records. Nine events of eight bytes,
with capacities 4, 8, 16, imply 12 copied records, or 96 bytes. Final slack is
`(16 - 9) * 8 = 56` bytes. Separate old/new buffers at the final growth would
request `(8 + 16) * 8 = 192` bytes, excluding metadata and other allocations.

Rounded 1.5 growth reaches 4, 6, 9 and copies 4 + 6 = 10 old records under the same
model. Rounding changes the ideal formula. Incremental full-copy growth costs
`n * (n - 1) / 2`, or 36 old records for nine appends. Actual allocation can grow
in place; these formulas do not measure copied bytes or resident memory.

Elapsed time can be modeled as `T ≈ R*a + B/b + U`: requests `R`, average request
overhead `a`, actual moved bytes `B`, effective transfer rate `b`, and other work
`U`. The doubling example gives `3*a + 96/b + U` only under its copy assumptions.
Neither allocator overhead nor copy bandwidth was isolated. Amortized constant
append does not bound a single pause.

## Choose a policy

Start with ordinary `Vec` for unknown input length. Reserve a trustworthy bound
once. Reuse across batches when high-water retention is acceptable. Measure a
smaller geometric factor when spare capacity matters. Avoid tight shrinking on
alternating pop/push. Quarter occupancy followed by a half-capacity request leaves
a gap before growth. A phase boundary is another useful time to consider release.

`reserve_exact` counts additional capacity from length: length 5, capacity 8,
target 16 requires 11 additional slots. `with_capacity` is not a maximum, and
`clear` does not shrink. The library growth factor, exact allocator size, and
physical release after shrinking are not contractual.

## Run

From the repository root:

```bash
cargo test -p algorithms-growth
cargo run -p algorithms-growth --example ninth_event
cargo bench -p algorithms-growth --bench portfolio
cd topics/026-dynamic-arrays-and-growth-policies
python3 run.py
```

The runner compiles with `rustc -O -C target-cpu=native`. Keep its raw directory
outside Git. See BENCHMARK.md and measurements/README.md for workload dimensions,
controls, all candidate results, and measurement limits. Timings include the safe
wrapper and request accounting, not just unwrapped `Vec::push`.

## Primary sources

- [Rust Vec guarantees and reservation/shrink APIs](https://doc.rust-lang.org/stable/std/vec/struct.Vec.html)
- [MIT 6.006 dynamic arrays](https://ocw.mit.edu/courses/6-006-introduction-to-algorithms-spring-2020/79a07dc1cb47d76dae2ffedc701e3d2b_MIT6_006S20_lec2.pdf)
- [CPython 3.14.0 list resizing](https://github.com/python/cpython/blob/v3.14.0/Objects/listobject.c), a pointer-list example, not a transferred policy recommendation
