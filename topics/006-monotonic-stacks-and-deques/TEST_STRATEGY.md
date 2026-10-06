# Test strategy

Scope: immutable integer samples; newest-tie rolling maxima and first strictly
greater indices. No concurrency, floating-point policy, historical updates, or
allocation-failure behavior is covered.

Contract and failure classes: reject zero and oversized widths; preserve original
indices; distinguish strict next-greater from equal maxima; expire old indices;
retain second-best candidates; handle partial blocks and extreme integer values.

Primary technique: independent direct-model differential checks. Secondary
techniques: exact boundaries, counted operations, and adversarial removal bursts.
Oracle: separate direct window loops and direct suffix search. Neither calls an
optimized helper. Compare complete index vectors and exact `None` results.

Input/model/schedule domain and bounds: all 3,280 arrays of lengths 0 through 7
over `{-1,0,1}`; 27,884 array-width cases, including invalid widths. Focused tests
add integer extrema, `usize::MAX`, expiration, and a 64-entry removal burst.
Replay assets: tests and fixed benchmark generator; no random test seed.
What a green run establishes: agreement on this finite domain and named cases,
not proof of every input or wall-clock update bounds.
Known gaps: arbitrary long arrays, allocation failure, stream index rollover,
unordered values, and hardware-specific mechanisms.
Concrete tests to add: a streaming API would need transition-by-transition
state checks and a rollover policy before reuse.

## Invariant-test audit

| Test and claim | Plausible violation | Smallest observation / comparator | Independence and backstop |
|---|---|---|---|
| Running example and counts | Count failed comparisons incorrectly | Exact output; deque 10 and stack 12 comparisons | Literal index vectors and hand trace; change counted increment |
| Duplicate contracts | Use `<=` in forward next-greater stack, or `<` in newest-tie deque | `[3,3,4]`; `[6,1,6]` | Literal expected indices distinguish identity; both operators are discriminating mutations |
| Expiration / second best | Keep stale maximum or discard all smaller candidates | `[9,8,7]`, width 2 gives `[0,1]` | Literal indices, peak and front count; omit front eviction or retain only maximum |
| Invalid widths / extrema | Subtract before validating; use a value sentinel | Zero, `usize::MAX`, integer extrema | Exact `None` and index result; no effects exist on immutable input |
| Removal burst | Assert bounded constant arrival work | 64 descending values then 100 | Literal 63 deque and 64 stack back pops; removing one candidate per arrival fails oracle checks |
| Exhaustive model agreement | Incorrect tie, expiry, partial-block reset, or block-boundary combine | All complete output vectors; counted peak at most width | Independent direct models; compare instrumentation and noninstrumented outputs |

Mutation backstops are explicit discriminating cases, not a claimed automated
mutation score. Final validation also runs these contracts natively on both
Linux hosts from the exact committed source.
