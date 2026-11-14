# Timing wheels, heaps, and timer trees

Choose a timer index from its clock, cancellation and deadline distribution.
Run `cargo test -p topic-049-timers` and
`cargo run -p topic-049-timers --release --example replay -- heap 4096 near 90`.

## Contract

Time starts at zero. IDs occupy a fixed array of at most 65,536 slots. Arming
replaces the ID's deadline. Deadlines must lie strictly after current time and
at most 1,000,000 ticks. Advances are nondecreasing, within the same bound.
Cancellation of an absent ID is idempotent. An advance emits every due live ID
once, ordered by deadline then ID. Each output records polling time too.
All input validation precedes replay. Allocation failure follows Rust's allocator
behavior. The finite replay owns all retained state and drops it before returning.

```text
BEFORE, now=0     DURING, rearm A       AFTER, advance(5)
A=(5,serial1) ->  A=(9,serial3)     ->  B emits (poll5,deadline5)
B=(5,serial2)     old A is stale       A remains live at 9
```

## Portfolio

| Candidate | Mechanism | Selection condition | Main cost/risk |
|---|---|---|---|
| Scan | Current deadline per ID; scan each poll | Tiny ID space | ID capacity times poll count |
| Heap | Reverse ordered immutable records; serial checks | Sparse exact deadlines | Stale storage and cleanup |
| B-tree | Ordered live keys; remove old key on change | Direct cancellation and ordered access | Tree updates and node storage |
| Hashed wheel | 64 buckets; absolute deadlines; serial checks | Dense near deadlines | Empty ticks and distant record revisits |

The comparison uses a lazy wheel and heap. Their records retain metadata through
cancellation until visited or replay teardown. The tree removes canceled records
immediately. ID arrays remain allocated in all four candidates. `peak_records`
counts physical records, while scan reports its allocated ID slots. It measures
neither allocated capacity nor bytes. Work counters are deterministic inspection
counts; they have different meanings across candidates.

## Cost model

Let U be ID capacity, P polls, A arms including rearms, C cancels, H maximum
physical heap records, L maximum live timers, E emitted timers, T elapsed ticks,
V wheel record visits, and e[p] output count at poll p. Validation adds one pass
over the input; ID-array initialization/final-state construction add O(U).

- Scan: O(A+C+UP+sum(e[p] log(e[p]+1))).
- Heap: O((A+E+S) log(H+1)+C+P), where S is popped stale records.
- Tree: O((A+C+E+P) log(L+1)); each poll descends to the minimum key.
- Wheel: O(A+C+T+V+sum(e[p] log(e[p]+1))).

For two IDs armed A@5/B@5, rearm A@9, then poll 5 and 9: U=2,
P=2, A=3, C=0, H=3, L=2, E=2, S=1, T=9, V=3. Scan inspects four
slots. Heap pops three records. Wheel visits nine ticks and three records.
Tree removes two due keys. Complexity terms describe growth, not timing.

Never-early quantization is q=delta*ceil(deadline/delta). Deadline 5 with delta=4
becomes 8, adding three time units before polling delay. Here delta=1 gives 5.
Equal-deadline bursts still require E outputs. Polling late changes observed
lateness, independently of the timer index's bookkeeping duration.

## Coverage and revisit scope

Implemented: scan, lazy binary min-heap, ordered B-tree and hashed wheel;
replacement identity, cancellation, equal deadlines, revolution boundaries,
large jumps, bounded horizon and monotonic simulated clock.
Revisit: indexed d-ary heaps, intrusive red-black timer trees, direct-removal
wheel handles, hierarchical wheels/cascades, empty-slot bitmaps and tick skipping,
periodic missed-tick/drift policy, concurrent callback admission and real wakeups.
The next measured revision of this source also has three items. Scan and tree
keep the (deadline, serial) ID slot they share with heap and wheel: 24 bytes on
64-bit targets, where the deadline alone takes 8. A same-deadline rearm removes
and reinserts the tree key. `Work` counters are `usize`; 65,536 IDs polled
65,536 times reach 2^32 scan inspections, which the 64-bit measured hosts hold.
The catalog currently supplies title/slug/ID without a coverage array;
ALGORITHM_INDEX.md is unavailable at the inspected base.

## Selection and limits

See measurements/README.md for measured selections. An exact timer structure
orders deadlines; service wakeup and callback execution require separate latency
evidence. A long-running lazy queue needs stale-storage and cleanup budgets.
A hierarchical wheel can reduce distant-record revisits; it has its own transfer
and burst costs. These candidates use a finite, single-threaded replay.

## Primary sources

- [Varghese and Lauck: hashed and hierarchical wheels](https://www.cs.columbia.edu/~nahum/w6998/papers/sosp87-timing-wheels.pdf)
- [Rust BinaryHeap: ordering and Reverse](https://doc.rust-lang.org/std/collections/struct.BinaryHeap.html)
- [Rust BTreeSet: ordered B-tree set](https://doc.rust-lang.org/std/collections/struct.BTreeSet.html)
- [Linux hrtimers: ordered red-black tree and timeout tradeoffs](https://docs.kernel.org/timers/hrtimers.html)
