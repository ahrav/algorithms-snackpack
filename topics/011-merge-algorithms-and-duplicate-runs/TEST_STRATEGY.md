# Correctness strategy

The independent stable concatenate-sort oracle preserves input origin ordering. Exhaustive small
joins use a nested equality-filter oracle. Benchmark joins use a BTreeMap keyed by the right key
and independent left-major expansion. No merge cursor or co-ranking helper appears in either
oracle. Benchmark validation runs outside timing for every implemented candidate and workload.

Four library checks: 3136 pairs from all56 sorted ternary-key vectors of length0..5; every possible
output diagonal against the stable-sort prefix; tie cuts/empty partitions with1,2,4,32 workers;
cascade stability with empty/skewed runs; 200 seeded differential sizes and u64 extremes.
Two integration checks: valid merge partitions that lose every join pair; cardinality budgeting
for512x512 all-equal rows. These are bounded evidence, not proof over every input.

Invariant audit: changing left <= to < fails duplicate identity ordering. Making both co-rank
comparisons weak fails the stable prefix check. Advancing both cursors after one equal-key pair
fails Cartesian-product agreement. Reordering cascade runs fails identity ordering. Using merge
partitions as local join inputs fails the explicit zero-vs-four regression. Count-only agreement
rejects lost or duplicated pairs independently of payload order.

Input sorting is a documented precondition. No unsafe code. Ordinary allocator failure is not
converted to a domain error. count_join uses u128, sufficient for products of usize-sized lengths
on supported64-bit hosts. Zero workers and an out-of-range output rank panic by documented
contract. Thread spawn/join cost and initialized output are intentional parts of this API.

Required validation includes whitespace, workspace format/lib/examples/doctests/alltargets,
Clippy with warnings denied, benchmark compilation and warning-free rustdoc. Linux campaigns
compile and run both library and integration contracts from frozen committed input hashes.
