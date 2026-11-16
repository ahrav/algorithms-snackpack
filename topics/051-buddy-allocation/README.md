# Buddy allocation and hierarchical free-space tracking

Reserve the lowest aligned power-of-two block in a fixed power-of-two arena.
`Pool::alloc` rejects zero, overflow, oversized requests, duplicate live IDs and
out-of-range IDs without changing reservations. `free` consumes a live ID once.
The serial API returns unit offsets; backing storage and byte alignment belong
to the caller. Reusing an ID gives it a new allocation; clients must prevent
stale operations across that reuse.

For 16 units, requests 5 and 3 reserve [0,8) and [8,12). Freeing the second
merges [8,12) with [12,16); freeing the first restores [0,16).

```text
before: [                 free 16                 ]
during: [ A: 5 used + 3 slack ][ B: 3 + 1 ][ free 4 ]
after:  [                 free 16                 ]
```

| Candidate | Mechanism | Cost and selection constraint |
|---|---|---|
| scan | inspect aligned occupancy ranges | O(U) allocation, O(s) free; compact simple state |
| buddy | ordered maximal-free-block sets by order | O(log(U) log(F+1)) conservative bound; dynamic metadata allocation |
| tree | largest-free-block summary per subtree | O(log U) path, 2U-1 useful summary nodes |

U is arena units, s is rounded request size, F is free block count. For U=16,
r=5, s=8, height=4, reservation slack=3. Buddy address is `a ^ s` relative
to the arena. Only free equal-order siblings merge. Alternating holes can
leave U/2 units free with no eligible two-unit block. Placement is identical
across these candidates; conventional first-sufficient-order lists differ.

```bash
cargo test -p topic051-buddy-allocation
python3 topics/051-buddy-allocation/test_measure.py
RUSTFLAGS='-C target-cpu=native' cargo build --release -p topic051-buddy-allocation --example replay
taskset -c 0 python3 topics/051-buddy-allocation/measure.py target/release/examples/replay /tmp/topic051/results.jsonl
```

The last two commands apply the declared controls on Linux: native target and
CPU 0, which the replay processes inherit. They use the workspace toolchain; the
archived hosts used Rust 1.98.1 and 1.99.0, so a replay checks the procedure and
builds a different image. The runner exits when the results file or the
checks/ directory beside it exists: archive both and remove them first.

`workload(n, "churn")` yields positive requests from 4 units. For 1 unit,
`n / 2 - 1` underflows: a debug build panics and a release build requests
`usize::MAX`, which `alloc` rejects. For 2 units each cycle starts with request
0, which `alloc` rejects. The measured sizes are 64 and 4096.

Use measured workload winners in measurements/README.md. Include construction,
metadata allocation and backing-memory work when those belong to the application.
Revisit: arbitrary runs and alignment, word/hierarchical bitmaps, intrusive
constant-time buddy membership, first-sufficient-order placement, reserved holes,
non-power-of-two arenas, slab frontends and concurrent sharded free lists.
For the next measured revision: a churn bound check in `workload`, a replay
digest that keeps the sign of each result, an untimed replay inside each
measured process, and A/A arms inside the cell permutation schedule under
distinct arm labels.
The authoritative catalog currently supplies no coverage array or algorithm index.

Sources: [Linux physical memory](https://docs.kernel.org/mm/physical_memory.html),
[buddy mechanics](https://www.kernel.org/doc/gorman/html/understand/understand009.html),
[generic bitmap pool](https://kernel.org/doc/html/next/core-api/genalloc.html).
Historical kernel constants in the buddy reference are version-specific.
