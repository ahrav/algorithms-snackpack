# Dynamic programming as state-space design

Select items at most once to maximize total value within a weight budget.
The bounded lab accepts up to 20 items, unsigned 16-bit weights, values and
capacity, including zero, and returns a 32-bit score. The empty set is always
legal. The sum of 20 maximum values fits in the result type.

With items (2,4), (3,5), (4,8) and capacity 6, take items one and three for 12.
For each prefix and capacity, compare the previous prefix at that capacity
with the previous prefix at capacity minus the item's weight plus its value.
The previous prefix guarantees at-most-once selection. A full table keeps
every row. A descending one-row loop preserves all unread source cells.

The value-indexed solver stores the lightest reachable weight per total value
and chooses the largest feasible value. The sparse solver merges the old
sorted frontier with the same states shifted by one item. At the same item
stage, a lighter state with at least as much value supports each continuation
of a heavier state, so the heavier state can be discarded. This argument
applies to an upper-bound budget. Exact-fill would need distinct reachability
by weight, including a representation for impossible states.

| Strategy | Time | Logical storage |
| --- | --- | --- |
| Exhaustive include/exclude tree | O(2^n) visited nodes, with overweight branches pruned | O(n) call stack |
| Full capacity table | O(n(C+1)) | (n+1)(C+1) score cells |
| Rolling capacity | O(n(C+1)) upper bound | C+1 score cells |
| Minimum weight per value | O(n(V+1)) upper bound | V+1 weight cells |
| Sorted frontier | O(sum F_i) over prior-prefix frontiers | O(max F_i) pairs in two reusable vectors |

Here n is item count, C is capacity, V is total value, and F_i counts
nondominated pairs after i items. For the example, n=3, C=6, V=17. Table,
rolling, and value storage are 28, 7, and 18 cells. Frontier sizes are
1, 2, 4, 6, including the empty prefix. Capacity and value bounds are
pseudopolynomial because their numeric magnitude can grow exponentially
with the number of bits needed to encode them.

Use the measured fixture results before selecting a score solver. A full
table can support backward reconstruction with an explicit tie rule.
For a later round, measure top-down memoization, witness reconstruction
with decisions or checkpoint/recompute, exact-fill reachability, and
meet-in-the-middle for large numeric axes. The lab covers state sufficiency,
dense and sparse storage, rolling-state dependencies, and the cost of
retaining or recomputing history. The catalog currently supplies a title
and slug for this topic.

From the repository root:

```bash
cargo test -p topic-054-dynamic-programming
cargo run -p topic-054-dynamic-programming --example focus
cargo run --release -p topic-054-dynamic-programming --example compare -- frontier wide-sparse 24
```

Sources: [Bellman, Dynamic Programming, chapter III](https://gwern.net/doc/statistics/decision/1957-bellman-dynamicprogramming.pdf),
[MIT 6.006, Lecture 21](https://www.ocw.mit.edu/courses/6-006-introduction-to-algorithms-fall-2011/06fe52fd13d711cb32dd0e5c6c8f0980_MIT6_006F11_lec21_orig.pdf),
[Horowitz and Sahni, Computing Partitions](https://www.cise.ufl.edu/~sahni/papers/computingPartitions.pdf).
