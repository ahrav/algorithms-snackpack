# Test strategy and invariant audit

Scope: overlapping exact byte records from a fixed nonempty dictionary.
Contract and failure classes: missed suffix/overlap; duplicate loss; incorrect
start/end; chunk reset; empty admission; unsigned offset overflow.
Primary technique: full result multiset differential comparison.
Secondary techniques: explicit public API fixture, every-split replay, emission
order check, first-of-several empty admission, and runner failure retention
(measurements/test_run.py with stub processes).
Oracle: independent start-first, pattern-first scalar byte enumeration. It shares
no transition/failure/output or validation helper with optimized candidates.
Input/model domain: all 14 binary words of length1..3 as each of two patterns;
all 63 binary inputs of length0..5, yielding 12,348 combinations. Each combination
checks three candidates and every two-way chunk split plus single-byte chunks.
Replay assets: source unit tests, tests/contract.rs, experiment fixtures and runner.
A green run establishes agreement in that finite domain and named boundaries.
Known gaps: allocation failures, callback panics, near-usize limits, exhaustive
large dictionaries and recovery quotas. Offset arithmetic is guarded in source.
Concrete future tests: recoverable quota implementation needs failure injection.

| Claim and scope | Plausible violation | Smallest discriminating observation |
|---|---|---|
| Terminal suffix records | omit output link | she must also emit he |
| Duplicate identity | deduplicate bytes/ID | two he IDs both occur |
| Overlap | reset after match | aa in aaa emits at 0 and 1 |
| Stream continuity | reset at chunk boundary | ush + ers emits she and he |
| Absolute offsets | use chunk-local position | he starts at 2 after split |
| Empty admission | silently accept/drop; return last empty index | exact EmptyPattern(1) with empties at 1 and 2, zero callbacks |
| Byte contract | treat zero as terminator | [0,255] has both matches |
| Emission order | buffer and reverse, or group by pattern | end is nondecreasing across emitted hits for all three candidates |
| Size reporting | off-by-one state count | ushers dictionary has 10 states and 20,480 table bytes on 64-bit |
| Complete model agreement | compare count only | full sorted Hit triples retain IDs/spans |

Comparator sorts complete records by derived field order. Multiplicity remains
visible. Handwritten public expected triples are independent of the model.
Mutation backstop: these cases discriminate the listed changes by construction;
mutants were not executed, so no mutation-score claim is made.
