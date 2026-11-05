# Boyer-Moore, Two-Way, and adaptive string search

Find the first exact byte occurrence in resident text. Empty patterns return
`Some(0)`; patterns longer than text return `None`. Offsets index original bytes.
Search borrows inputs, performs no allocation, and preserves the earliest match.

## Candidates and costs

`Direct` verifies left to right at every start. `Horspool` verifies right to left
and shifts by the last aligned text byte. `BoyerMoore` combines bad-character
and strong good-suffix shifts. `TwoWay` uses two maximal-suffix passes, a critical
split, and periodic comparison memory. `Adaptive` selects direct search for
patterns shorter than four or texts shorter than 64; otherwise it allows `2n`
Horspool comparisons before permanently continuing with Two-Way at the current
unresolved start. Preparation for an adaptive pattern of length at least four
allocates its table even when a later short text takes the direct route.

For text length `n`, pattern length `m`, and `A=max(0,n-m+1)` legal starts,
direct and Horspool can require `A*m` comparisons. Two-Way preparation and
search require `O(m+n)` work with constant auxiliary words. The hybrid bounds
its fast phase by `2n`, prepares Two-Way once on fallback, and searches a suffix
of the original text. Earlier starts were excluded by safe shifts. It therefore
also requires `O(m+n)` work. Its thresholds are educational experiment policy.
The full Boyer-Moore first-match implementation uses linear table preparation;
all-occurrence enumeration would require a separate progress analysis.

Preparation costs matter. With build cost `B`, per-query cost `Q`, and `R`
queries, total cost is `B+R*Q`. For `ABCD` in `ABAAABCD`, `n=8`, `m=4`, and
`A=5`. The direct scan compares 12 bytes. Horspool compares six bytes at starts
0, 3, and 4. Its table gives A=3, B=2, C=1, and four for other bytes.

```text
Before, start 0:  ABAAABCD     final A mismatches D; shift 3
                  ABCD
During, start 3:  ABAAABCD     final C mismatches D; shift 1
                     ABCD
After, start 4:   ABAAABCD     return Some(4)
                      ABCD
```

Use direct search when setup dominates. Consider Horspool for useful skips,
full Boyer-Moore when matched suffixes justify its tables, and Two-Way for a
linear resident search. Inspect both lifecycle endpoints in measurements before
selecting a candidate. A repeated-prefix miss hurts direct search; a repeated-
suffix miss hurts Horspool. A timing difference alone does not identify CPU
branch, vectorization, or cache mechanisms.

## Run

```sh
cargo test -p topic-039-adaptive-string-search
cargo run --release -p topic-039-adaptive-string-search --example search
python3 topics/039-adaptive-string-search/measurements/run.py /tmp/search039-run
```

The Python replay requires Linux, rustc, Python 3, taskset, lscpu, and objdump.
Its output directory must be new. Read BENCHMARK.md before interpreting results.

## Coverage and revisit scope

This visit implements Horspool, full bad-character plus good-suffix Boyer-Moore,
Two-Way with periodic memory, and bounded adaptive fallback against direct search.
The authoritative catalog at the base revision has no coverage arrays and no
ALGORITHM_INDEX.md. Existing metadata follow-up remains separate. Revisit
Galil-style retained comparisons for all occurrences, vector prefilters, reverse
search, streaming, Unicode transformations, and production-library comparisons.

## Primary sources

- [Boyer and Moore, 1977](https://www.cs.utexas.edu/~moore/publications/fstrpos.pdf): first-occurrence backward comparison and two shift rules.
- [Horspool, 1980](https://doi.org/10.1002/spe.4380100604): simplified last-byte shift search.
- [Crochemore and Perrin, 1991](https://www.cin.ufpe.br/~paguso/courses/if767/bib/Crochemore_1991.pdf): critical factorization and linear constant-space matching.
- [CPython fastsearch](https://github.com/python/cpython/blob/main/Objects/stringlib/fastsearch.h): production adaptation and maximal-suffix formulation. This crate is an educational implementation, with its own thresholds.
