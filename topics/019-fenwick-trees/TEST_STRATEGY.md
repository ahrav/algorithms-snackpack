# Correctness and invariant checks

The oracle sums logical leaves directly; it shares no bucket, prefix-repair or
segment-walk helper. All small ternary arrays through length seven check every
range before and after each modular negative point update. This discriminates
wrong endpoint conventions, lowbit walk direction, linear parent build and
range cancellation. Exhaustive nonnegative arrays through length six compare
all valid and first-invalid count ranks with a linear cumulative scan. Zeros,
empty input and maximum total discriminate <= skip and monotonicity boundaries.

Overflowing construction, invalid replacement and failed increment admission
leave count state unchanged. Range-add checks every range after every boundary
pair through length seventeen with alternating positive/modular negative deltas.
Invalid point/range/prefix boundaries and wrapping sums have explicit witnesses.
The public integration test checks the rendered example. Every benchmark
candidate additionally compares each query in the declared streams with Scan
before its timing is admitted. These finite tests are bounded evidence.

Invariant audit: each test names a distinct plausible mutation above; no test
infers source mechanism or universal complexity from a passing timing.
