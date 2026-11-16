# Test strategy and invariant audit

Critical observable: each returned address, failure and final reservation state.
Python's independent oracle maintains disjoint free intervals, splits a selected
interval and unions adjacent free ranges. It shares no Rust placement helper.
All 24 complete candidate/workload/size traces are checked before timing.

Contracts pin rounding, zero/overflow, duplicate/out-of-range IDs, exact fits,
exhaustion, repeated free, alternating holes and full coalescence. The 256 small
request histories compare implementations; those comparisons supplement the
independent full-trace interval oracle. Full occupancy vectors distinguish holes
that total-free counts would miss. Restoration admits a whole-arena allocation,
which detects stale tree summaries and incomplete buddy coalescence.

The interface is serial reservation metadata. Stale caller IDs after reuse,
backing memory access, concurrent lifetimes and arbitrary schedules require
separate APIs and verification.
