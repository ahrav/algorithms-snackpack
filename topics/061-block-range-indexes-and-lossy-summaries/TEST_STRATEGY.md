# Test strategy and invariant audit

An independent oracle filters raw rows by inclusive range membership without
chunking or summaries. Enumerate 3,125 five-element sequences, five widths
and 100 endpoint pairs. This gives 1,562,500 cases per implementation.

| Invariant | Test | Rejects |
|---|---|---|
| Summary intervals include both endpoints | Fixed point queries at 20 and 91 | Strict interval comparison |
| Admitted blocks undergo exact recheck | Query 53 admits a block with zero matches | Counting candidate rows as matches |
| Signed extremes and final partial blocks preserve coverage | MIN, MAX and repeated 7 fixtures | Narrow gap arithmetic or dropped tails |
| A second interval refines the original bounds | Compare work over the exhaustive cases | Extra admitted rows outside min/max coverage |
| Empty and reversed inputs return zero | Fixed and exhaustive cases | Spurious counts or nonexistent block access |

The Python benchmark oracle verifies the per-query count on each process.
