# Monotonic stacks and deques

Keep entries only while they can affect a future answer. A forward stack tracks
unanswered next-greater queries. A deque also expires candidates from a moving
window. This crate uses totally ordered `i64` samples and original indices.

## Contract and example

- `next_greater_*` returns the first later **strictly greater** index, or `None`.
- `window_*` returns the **newest** tied maximum's index for each full window.
- Width zero or larger than the input length returns `None`, including empty
  input. Width one returns every index. Inputs are immutable.
- The input is available as a slice. The deque algorithm needs no future values,
  but this API collects output and is not a streaming service.

| Index | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Value | 5 | 3 | 3 | 4 | 2 | 6 | 1 | 6 |
| Next greater index | 5 | 3 | 3 | 5 | 5 | none | 7 | none |

Width-three window maxima use indices `[0, 3, 3, 5, 5, 7]`.

| Stack update at index 3 | Waiting indices, bottom to top | Values | Effect |
|---|---|---|---|
| Before arrival | 0, 1, 2 | 5, 3, 3 | All questions unresolved |
| During arrival of 4 | Pop 2, then 1 | Remove both 3s | Both answers become 3 |
| After insertion | 0, 3 | 5, 4 | Index 0 still waits |

| Deque update at index 5 | Window | Indices, front to back | Values |
|---|---|---|---|
| Before arrival | 2–4 | 3, 4 | 4, 2 |
| After expiration | 3–5 | 3, 4 | 4, 2 |
| During dominance removal | 3–5 | empty | New 6 beats both values |
| After insertion | 3–5 | 5 | 6 |

Read each candidate list left to right. The stack top is last; the deque's answer
is first. At index 7 the newer equal 6 replaces index 5. The stack retains equal
values because its query requires strict inequality.

## Portfolio and costs

| Method | Work | Extra storage excluding output | Boundary or failure |
|---|---|---|---|
| Window scan | `(n-w+1)(w-1)` value comparisons | Constant | Repeats overlapping comparisons; useful for tiny windows |
| Window deque | Linear total work | At most `w` live indices | One arrival can remove nearly a whole window |
| Block prefix/suffix | Linear preprocessing and output | Two `n`-index arrays | Batch preparation, partial blocks, and tie handling |
| Next-greater scan | Up to `n(n-1)/2` comparisons | Constant | Repeated suffix search |
| Next-greater stack | Linear total work | At most `n` live indices | One arrival can resolve a long waiting suffix |

Here `n` is input length and `w` is window width. For the running example,
`n=8`, `w=3`, and full-window count `n-w+1=8-3+1=6`.
The scan uses `6*(3-1)=12` comparisons. The deque uses 10: six successful
back comparisons and four unsuccessful ones. One index expires at the front.
The next-greater scan uses 14 comparisons; its stack uses 12.
The worst-case suffix-scan bound at this length is `8*7/2=28`.

Each candidate is pushed once and removed at most once. Deque value comparisons
are at most removals plus one unsuccessful comparison per arrival. At `n=8`,
the loose bound is `8+7=15`. This is an amortized total bound, not constant
per-arrival latency. A decreasing width-64 window followed by 100 causes 63
deque back removals and 64 stack removals in the regression test.

The block method computes each block's prefix and suffix maximum indices, then
combines the left suffix and right prefix of each window. `n=8` needs two
eight-index arrays plus six output indices. Comparisons include tie-index
selection; comparison counts are not processor instruction counts.

## Run and choose

From the repository root:

```bash
cargo test -p topic-006-monotonic-stacks-and-deques --all-targets
cargo run --release -p topic-006-monotonic-stacks-and-deques --example experiment
cargo run --release -p topic-006-monotonic-stacks-and-deques --example experiment -- \
  --measure deque random 32768 256 16
```

The default example checks answers and prints operation counts. A process sample
validates against the independent scan, warms three calls, and times sixteen
calls including output allocation and destruction. Its setup time is separate.
The frozen two-host campaign is specified in [BENCHMARK.md](BENCHMARK.md).

Use a stack for nearest-boundary queries, a deque for forward-moving online
extrema, and compare blocks for batch input. Benchmark direct scans for small
windows. A heap or tree can support richer queries, but lazy heap expiration can
retain buried stale entries and grow with the full stream length.

Do not apply this integer ordering to floating-point not-a-number values without
a policy. Moving the window backwards or editing old samples can require
discarded entries. Preallocation prevents growth, not removal bursts. This crate
does not promise hard real-time update latency.

## Evidence and sources

Operation counts and complexity bounds are derived or separately instrumented.
Timing results are host-specific process observations. Exhaustive small-domain
agreement is finite evidence, not proof over all inputs. See
[measurements](measurements/README.md) and [test strategy](TEST_STRATEGY.md).

- [Cornell CS 2110 stacks and queues](https://www.cs.cornell.edu/courses/cs2110/2026sp/lectures/lec15/): monotonic stack and next-greater queries.
- [MIT amortized-analysis notes](https://ocw.mit.edu/courses/6-046j-design-and-analysis-of-algorithms-spring-2012/resources/mit6_046js12_lec11/): push/pop accounting.
- [Lemire, Streaming Maximum-Minimum Filter](https://arxiv.org/abs/cs/0610046): related monotonic extrema and block methods. Its combined-filter comparison bound is not claimed for this crate.
- [Rust `VecDeque`](https://doc.rust-lang.org/std/collections/struct.VecDeque.html): ring-buffer representation and API. Local validation uses Rust 1.93.0; Linux compiler identities are in receipts.
