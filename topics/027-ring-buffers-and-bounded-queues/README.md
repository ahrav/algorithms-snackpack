# Topic 027: Ring buffers and bounded queues

A ring preserves FIFO order while reusing storage. Define full behavior and ownership
before selecting indexing arithmetic. This artifact implements single-owner storage.

## Contract and running example

All five candidates enforce exact logical capacity C. Full insertion returns `Err(value)`
and leaves queued values unchanged. Empty pop returns `None`. Pop transfers ownership;
destruction drops remaining owners. Zero capacity always rejects. Construction follows
ordinary infallible allocation behavior; there is no transactional allocation API.

```text
Physical index       0    1    2    3       Logical FIFO
Before: full        10   11   12   13       10,11,12,13  head=0 len=4
During: pop ->10      .   11   12   13       11,12,13     head=1 len=3
After: push(14)      14   11   12   13       11,12,13,14  head=1 len=4
```

The dot is a free slot. Read from head, wrapping at the backing end. If P is backing
slots, h is head and L is live length, `0 <= L <= C <= P`. An admitted push uses
`(h+L) mod P`: `(1+3) mod 4 = 0` above. Length distinguishes full from empty without
reserving an unused logical slot. Full is checked before touching the insertion slot.

## Alternatives and costs

| Candidate | Mechanism | Main cost or rejection condition |
|---|---|---|
| `Shift` | `Vec::remove(0)` | Every successful pop shifts n-1 live values |
| `Deque` | Explicit bound plus `VecDeque` | Reservation alone does not limit admission |
| `Ring<T,0>` | Remainder by backing length | Target/compiler-dependent arithmetic |
| `Ring<T,1>` | Conditional wrap | Branch and bounds work |
| `Ring<T,2>` | Mask by power-of-two backing length | Padding and optional-slot storage |

For n live elements and inline size s, a front pop models `(n-1)*s` moved bytes.
For n=4 and s=8, that is 24 logical bytes, not measured DRAM traffic. A ring touches
one slot plus metadata. Construction initializes P optional slots; destruction processes
live owners. For C=1000, the mask ring has P=1024 and still rejects the 1001st insertion.
`Option<T>` permits safe move-out with `take()`, with possible layout overhead. A slot
count does not bound allocations owned by payloads. No unsafe code is used.

`as_slices` returns two FIFO-ordered slices of initialized optional slots. The example
prints `([Some(11), Some(12), Some(13)], [Some(14)])`. A contiguous-slice requirement can
introduce rearrangement costs; it is a different operation from inspecting both spans.

## Run

From repository root:

```sh
cargo test -p topic027-ring-buffers-and-bounded-queues
cargo run -p topic027-ring-buffers-and-bounded-queues --example wrapped
cargo bench -p topic027-ring-buffers-and-bounded-queues --bench portfolio -- mask steady_odd
```

On Linux, run `python3 run.py` from this topic directory for the full fixed schedule. It
writes every result to `raw/` and refuses to start while `raw/` holds a previous campaign;
archive that directory elsewhere first.
The independent oracle uses an ordered map of insertion tickets, with no ring helpers.
See [test strategy](TEST_STRATEGY.md), [protocol](BENCHMARK.md) and
[results](measurements/README.md) for evidence and exact replay boundaries.

## Selection and revisit scope

Use bounded `VecDeque` for ordinary application FIFO storage. Admit a custom ring for a
specific backing-allocation or span-control requirement. Choose masking only when its
measured hot-path benefit justifies padding. Keep close measured candidates unresolved.
A fixed queue cannot absorb sustained producer overload forever.

Topic 027 has no additional catalog coverage array. This round covers sequential storage,
exact admission, wrap, two spans and owning payloads. SPSC/MPSC/SPMC/MPMC publication,
blocking wakeups, overwrite loss accounting, variable-sized records and batch throughput
remain explicit revisit scope. SPSC means one producer/one consumer; MPMC means multiple
producers/multiple consumers. Sharing requires both write-to-read publication and
completed-read-to-reuse synchronization. A mask or atomic counter proves neither progress
nor thread safety. Blocking, overwrite and rejection implement different contracts.

## Primary sources

- [Rust VecDeque](https://doc.rust-lang.org/std/collections/struct.VecDeque.html): growable
  ring, reservation, two slices and contiguous rearrangement. Artifact builds on Rust 1.93;
  measured host compilers are recorded separately.
- [Linux circular buffers](https://www.kernel.org/doc/html/latest/core-api/circular-buffers.html):
  reserved-slot convention and producer/consumer publication. Cursor naming differs by source.
- [Linux tracing ring design](https://docs.kernel.org/trace/ring-buffer-design.html): rejection
  versus overwrite and its particular execution assumptions. No concurrent implementation
  guarantee transfers to this single-owner artifact.
