# LRU, CLOCK, and workload-aware eviction

A full cache trades future misses against replacement bookkeeping. This crate
replays equal-size, equal-cost objects with sequential dense integer keys. Every
miss admits. A lookup returns prior residency. Capacity zero always misses.
Keys outside the declared universe return an error before mutation. Expiry,
loading, invalidation, writes, pinning and concurrency need separate contracts.

## Mechanisms and cost

Timestamp LRU records request positions and scans residents on full-cache misses.
Indexed LRU maintains an oldest-to-newest doubly linked list. CLOCK sets hit bits
and advances a hand across set bits during eviction. Segmented LRU promotes
probation hits to protected, demotes protected overflow, and evicts probation's
oldest. Protected quota is floor(capacity/2); probation borrows unused space.
This stated segmented variant has its own semantics and parameters.

With N requests, M misses, C slots and U possible keys, timestamp LRU costs
O(N+MC), indexed LRU and segmented LRU cost O(N), and CLOCK has amortized O(N)
work with an O(C) single eviction. All use O(U+C) metadata for direct lookup.
Dense-universe lookup removes hashing from this comparison. Sparse identifiers
require a separately measured index. CLOCK uses Rust's byte-sized bool entries.

```text
capacity 3; trace A B A B | C D E | A B
LRU oldest first: [A B] | [C D E] | [E A B] (7 misses)
CLOCK slots: A1 B1 _; hand2 | D1 E1 C0; hand2 | B1 E0 A0; hand1 (7)
SLRU probation/protected: [A]/[B] | [D E]/[B] | [E A]/[B] (6)
```

The two LRU realizations must match every hit and final order. Different policies
can produce different valid hit streams. The independent models validate each
policy's specified transitions rather than imposing LRU results on CLOCK.

## Run

```sh
cargo test -p topic048-eviction
cargo run --release -p topic048-eviction --example measure -- linked tiny 3 9
cargo run --release -p topic048-eviction --example measure -- segmented scan 64 32768
python3 topics/048-lru-clock-and-eviction/measurements/run.py \
  target/release/examples/measure /tmp/topic048-new-run
```

The tiny LRU run reports 2 hits and final queue [[4,0,1]]. CLOCK examines 13
slots including fills; timestamp LRU examines 12 candidates on four full misses.
The full schedule writes fresh-process records with independently verified hit
streams, final ordered residents, reference bits, hand and work counters.

## Selection

Use measured miss counts and policy CPU together. For CPU t per request, miss
fraction m and added miss cost P, the serial projection is t+mP. The running
trace compares t_LRU+(7/9)P with t_SLRU+(6/9)P. Segmented LRU repays a positive
extra CPU cost when P>9(t_SLRU-t_LRU). Backend and end-to-end costs require their
own measurement, including concurrency and queueing.

Exact LRU fits a recency requirement. Timestamp scans can suit small hit-heavy
caches. CLOCK shifts work into eviction. Segmentation can preserve reused keys
through scans, and can retain old protected keys during a phase change. Sweep
capacity and phase behavior before selecting a policy. See measurements for
observed winners, full ranges and unresolved cells.

## Coverage and explicit revisits

This visit implements timestamp LRU, indexed LRU, CLOCK and the stated segmented
LRU variant. Revisit FIFO/Random, Belady's equal-size offline bound, full 2Q,
ARC, LIRS, CLOCK-Pro, SIEVE, S3-FIFO, variable-size and heterogeneous-cost objects,
shards, concurrency and delayed maintenance. Topic 063 owns TinyLFU admission.
The inspected catalog contains no coverage arrays or ALGORITHM_INDEX.md; the
existing metadata follow-up remains separate from this lesson.

## Primary sources

- [Arpaci-Dusseau, replacement policies](https://pages.cs.wisc.edu/~remzi/OSTEP/vm-beyondphys-policy.pdf): LRU, CLOCK, misses and cost.
- [Johnson and Shasha, 2Q](https://www.vldb.org/conf/1994/P439.PDF): reuse-sensitive queue policies and exact 2Q rules.
- [Megiddo and Modha, ARC](https://www.usenix.org/legacy/events/fast03/tech/full_papers/megiddo/megiddo.pdf): adaptive resident and ghost lists for equal-size paging.
- [S3-FIFO authors](https://s3fifo.com/blog/2023/08/01/fifo-queues-are-all-you-need-for-cache-eviction/): small/main/ghost queues and capped counters.
