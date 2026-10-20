# Tries, radix trees, ART, and sparse radix arrays

Find exact byte keys and materialize prefix matches in byte-key order. Empty
keys, embedded NUL/0xff, prefix-of-another keys and zero values are legal.
Identical keys retain the last input value. Input order is otherwise arbitrary.

`car=10, cart=20, carbon=30, cab=40`: exact `ca` misses; prefix `ca` emits
`[40,10,30,20]`. A terminal can have children. Prefixes may stop inside a
compressed path. [The split visual](radix-split.svg) shows before/during/after.

## Candidates and costs

- `flat`: stable-sort/deduplicate then binary search or lower-bound/scan.
- `tree`: standard BTreeMap; incremental ordered insertion and ordered range.
- `trie`: arena byte trie with sorted sparse child vectors.
- `radix`: full-prefix bulk-built scalar adaptive model. Fanouts1..4/5..16 use
  vectors reserved to4/16,17..48 use a256-byte slot index,49..256 direct256
  arena indices. Empty leaves allocate no child vector. This is not the
  canonical packed ART layout, SIMD implementation, or mutable ART algorithm.

Let n=distinct keys,L=query bytes,p=prefix bytes,z=output values,h=compressed
nodes,f_i=fanout. Comparison search can inspect O(L log n) bytes. Trie search
consumes L bytes plus child-selection work (binary edge vectors add sum log f_i).
Radix checks relevant bytes over h nodes with scalar/index/direct child selection.
Prefix cost includes locate(p), visited subtree slots/nodes and Omega(z) output.
For four keys log2(n)=2; car has L=3. After splitting at ca, car visits h=2
compressed nodes. Prefix ca has p=2,z=4 and32 payload bytes. A256-slot array
of eight-byte indices uses2048 bytes before other fields. Broad scans still
visit absent direct slots. No universal time or space winner follows.

Linux XArray indexes unsigned-long integers, not arbitrary byte strings.
Its clustered sparse slots, marks, reservations and multi-index entries have
API-specific semantics. xa_load returning NULL does not prove an ID is free;
xa_insert may reject a reservation. xa_store returns the old entry or an
encoded error. Normal synchronization protects structure access; object
ownership after lookup needs a separate reference/locking or RCU protocol.
The userspace sparse64-page/Arc test illustrates sparse occupancy and owned
lookup only. It does not execute Linux XArray or prove RCU safety. A dense
array through ID2^40 would require about8TiB at8bytes/slot; IDs0,63,64,2^40
occupy three64-slot model pages,1536slot bytes plus directory overhead.

## Run

```bash
cargo run -p topic024-tries-radix-trees-and-art --example dictionary
cargo test -p topic024-tries-radix-trees-and-art
cargo bench -p topic024-tries-radix-trees-and-art --bench dictionary -- radix shared
cd topics/024-tries-radix-trees-and-art
python3 scripts/run.py /tmp/topic024-results
```

The campaign runner requires Linux,lscpu,Python3,rustc. No third-party dependencies.
See BENCHMARK.md,TEST_STRATEGY.md and measurements/README.md for bounded results.
Only frozen lookup/build/output are compared. Online insertion/deletion,
reclamation, Unicode normalization and semantic composite encodings are not
implemented. Keys are bytes; recursion depth remains bounded by available
stack, so a service must constrain hostile key depth. No stable serialized or
Rust memory-layout ABI is promised.

## Selection

Use flat as the frozen baseline, especially for build-heavy or broad-output
work. Consider radix for repeated selective reads when preprocessing and
capacity costs amortize. Compare early-miss tries explicitly. Use a general
mutable map when online updates are required; those costs were not timed.
Do not extrapolate this scalar arena model to production ART, kernel XArray,
all CPUs, or all key distributions. Topic025 revisits immutable automata/FSTs;
online ART growth/shrink and kernel concurrent lifetime need distinct evidence.

## Primary sources

- [ART,2013](https://db.in.tum.de/~leis/papers/ART.pdf): adaptive layouts/compression.
- [Linux v6.12 XArray](https://raw.githubusercontent.com/torvalds/linux/v6.12/Documentation/core-api/xarray.rst): pinned API/lifetime semantics.
- [RCU](https://docs.kernel.org/6.12/RCU/whatisRCU.html): removal versus reclamation.
- [Rust BTreeMap](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html): ordered map/range contract.
