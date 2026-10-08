# Merge algorithms and duplicate-run joins

Choose output semantics before a merge kernel. This lab uses sorted immutable u64 keys with
16-byte `(key, id)` rows. Merge preserves every row and source order on ties. Inner equijoin emits
every matching identity pair in left-major/right-major order. Count-only returns u128 cardinality.
Inputs must be ascending; kernels exclude validation scans. Output uses ordinary Vec allocation;
allocation failure follows Rust's allocator behavior. Budget duplicate output before materializing.

```text
Before: A=[1a,5b,5c,9d]  B=[2w,5x,5y,5z,8v]
During: d=4 -> i=3,j=1 -> [1a,2w,5b,5c] | [5x,5y,5z,8v,9d]
After merge: [1a,2w,5b,5c,5x,5y,5z,8v,9d]
After join: (b,x),(b,y),(b,z),(c,x),(c,y),(c,z)
```

Two-pointer merge advances one cursor per output. Equality chooses A. Galloping searches and
copies winning prefixes: `A <= B_head`, `B < A_head`. Alternating inputs can make those searches
cost more than scalar comparisons. Parallel-4 uses stable co-ranks and disjoint output slices,
including initialization, allocation, scoped thread launch and joining in its API cost.

For output prefix d, find i+j=d with endpoint-guarded A[i-1]<=B[j] and B[j-1]<A[i]. The example
checks 5<=5 and 2<9. Symmetric inequalities can misplace ties. Equal merge partitions cannot define
join partitions: the example's split separates every matching key-5 row across the partitions.

Duplicate-run join finds both equal-key runs and replays the right run for every left row. If
key k occurs p_k and q_k times, it emits p_k*q_k pairs. PostgreSQL's versioned executor provides
a production mark/restore example; this lab implements only integer inner equality.

| Candidate | Contract and cost | Failure or selection boundary |
|---|---|---|
| Stable concatenate-sort | Stable merge oracle; allocation/copy plus library sort | Test as baseline; existing sorted runs can help |
| Linear merge | O(m+n), N output rows | Sequential output/decision path |
| Galloping merge | O(N) output plus prefix searches | Short alternating winning blocks add overhead |
| Parallel-4 | Roughly N/4 rows plus logarithmic co-ranks | Thread startup, output initialization, bandwidth limits |
| Probe join | O(m*(1+log(n+1))+z) worst-case work | Searches repeat for duplicate left keys |
| Replay join | O(m+n+z) worst-case work | z may be quadratic; right run must be replayable |
| Left cascade | Repeated accumulated-prefix copies | Quadratic-in-run-count writes for equal runs |
| Adjacent balanced cascade | O(N log k) merge rows | Leaf clones/allocations included; equal-depth tree may miss weighted optimum |
| Heap cascade | N final writes, O(k+N*(1+log(h+1))) work | Heap overhead; key includes source-run precedence |

m,n are input lengths, N=m+n, z=sum(p_k*q_k), k is supplied run count and h is active nonempty run count. Example: m=4,n=5,N=9,z=6.
Linear merge uses at most 8 comparisons. Read-plus-write row bytes are 2*9*16=288 logical bytes,
not measured hardware traffic. Count-only can return 6 without storing six pairs; its contract
cannot compete for the row-join timing title.

Split the example into runs of lengths 2,2,2,3. Left-fold output lengths 2,4,6,9 write 21 rows.
Balanced internal outputs 4,5,9 write 18, plus 9 leaf-clone rows = 27 in this implementation.
For equal runs of r rows, left writes r*k*(k+1)/2; power-of-two balanced writes N*(1+log2 k),
including clones. For k=32,r=2 these are 1056 and 384. Arbitrary shortest-first merging can reorder
equal-key source runs; use adjacent/alphabetic scheduling or an explicit source-order tie-breaker.

## Run

```bash
cargo test -p merge-lab
cargo run --release -p merge-lab --example portfolio
cargo run --release -p merge-lab --example experiment -- join_equal1024 replay 8
cargo run --release -p merge-lab --example experiment -- merge_interleaved1m parallel4 2
```

Expect six tests, a nine-row stable example merge, six join pairs, `row_bytes: 16`, and
`output_len: 262144` for the all-equal benchmark join. Timing varies. Use the balanced process
campaign in [BENCHMARK.md](BENCHMARK.md) for selection, not one illustrative command.

Read [measured results](measurements/README.md) before choosing a kernel. Begin with linear merge;
compare galloping for long winning blocks and parallel merge for large outputs. Budget z for joins.
For many runs compare adjacent balanced, heap and library sorting under actual sizes and tie rules.

Revisit scope: weighted adjacent merge scheduling, reusable output buffers, persistent workers,
variable-size payloads, join output tiling, rewindless streams/spill, SQL null/collation/predicate
semantics, and hardware-counter attribution. Snapshot/tombstone semantics belong to Topic059.

## Sources

- [Siebert and Traeff, stable co-ranking, arXiv1303.4312v2 section2](https://arxiv.org/html/1303.4312)
- [Green, Odeh and Birk, Merge Path, arXiv1406.2628](https://arxiv.org/abs/1406.2628)
- [PostgreSQL18 nodeMergejoin.c](https://github.com/postgres/postgres/blob/REL_18_0/src/backend/executor/nodeMergejoin.c)
- [Rust partition_point](https://doc.rust-lang.org/std/primitive.slice.html#method.partition_point)
- [Rust scoped threads](https://doc.rust-lang.org/std/thread/fn.scope.html)

Cost counts above are derived from this lab. Timings are observations; mechanism explanations
remain hypotheses unless separately measured.
