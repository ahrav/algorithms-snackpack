# Benchmark decision

**Timing benchmark: true.** Test whether skipped resident predicates repay
summary traversal and whether a second interval repays its larger metadata.

The small case uses 64 rows in 16-row blocks. Other cases use 262,144 rows.
Ordered and permuted inputs contain the same values. The permutation is a
fixed odd affine mapping over a power-of-two domain. Bimodal cases share the
same two bands and vary whether queries fall between them or hit one band in
every block. Wide queries admit most ordered rows. Block width is 256 except
for the 2,048-row ordered case.

Each workload has eight fixed predicates. The Python oracle sorts the input
multiset and verifies every predicate count from two binary-search endpoints.
Replay runs the Rust tests before timing. Both hosts use Rust 1.93.0 and
`-C target-cpu=native`.

A fresh process is the sample. Three processes warm each case. Within each
process, fixture generation precedes the build timer. An untimed query pass
checks the answer vector and warms all predicates. The query timer covers
16 batch repetitions or 4,096 for tiny, including counting and operation
counters. Printing and teardown are outside the query timer.

Use all six candidate permutations, rotating the initial order by case.
Alternate running two adjacent min/max processes before or after the
candidates. This same-code A/A control estimates local timing variation.
Report median and min to max nanoseconds per query. The lowest median wins
only if it beats both alternatives in at least five rounds and its median
paired fractional saving exceeds 5% and the largest fractional A/A
difference. Stop after six rounds and record unresolved cells.

Retain row checks, block counts, summary bytes, build timings and linked
disassembly. Source inspection shows zero query buffers, one min/max
summary vector, and one two-range summary vector with a reusable scratch
vector. The comparison measures resident elapsed time.
