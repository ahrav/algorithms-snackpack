# Test strategy and invariant audit

Scope: complete-buffer decoding of shortest-form unsigned u32 varints (`scalar`,
`unrolled`) and four-byte little-endian integers (`fixed`) under a caller output cap.
Contract and failure classes: truncation; overflow past u32 or five bytes; redundant
final zero payload; output cap reached. Each error carries the integer's starting
offset. Output is cleared on entry and keeps the valid prefix on error.
Primary technique: differential comparison of the result and the full output against
the reference, each decoder starting from its own stale buffer.
Secondary techniques: handwritten boundary table with exact error, offset and prefix;
stale-output table for `fixed`; valid roundtrip; public API fixture; runner failure
retention (measurements/test_run.py with stub processes).
Oracle: the independent reference locates the terminal group, reconstructs with u64
multiplication, checks conversion, and derives minimal length by division. Candidates
use checked byte-at-a-time u32 shifts. Oracle and candidates share no parser/encoder
helper. Expected values in tests/contract.rs are handwritten. Python independently
validates all workload encodings.
Input/model/schedule domain and bounds: all 65,793 byte strings of length 0..=2 at
output caps 0, 1 and 2. 10,000 random four-byte strings and 12,000 random strings of
length 1..=12 at cap 3, where decoding stops at the third value; that loop also repeats
the empty string 1,000 times. 16,000 random strings of length 1..=16 at a cap equal to
the length. A 10,011-value roundtrip including every u32 encoded-length threshold.
Replay assets: src/lib.rs unit tests, tests/contract.rs, measurements/test_run.py, the
experiment example and the runner. Generators use fixed seeds.
What a green run establishes: agreement with the reference in that finite and sampled
domain, and the exact outcomes in the boundary tables. Random strings are sampled
evidence.
Known gaps and delegated skills: allocation failure, inputs near usize limits and
coverage-guided fuzzing are untested. Full-buffer decoding requires callers to assemble
framing before this API. No skill is delegated.
Concrete tests to add: SIMD or streaming implementations would require independent
tail and every-split tests.

| Claim and scope | Plausible violation | Smallest discriminating case |
|---|---|---|
| Byte five holds at most four payload bits | accept a payload above 0f | ff ff ff ff 10 is Overflow at 0 |
| Five bytes end an integer | read or skip a sixth byte | 80 80 80 80 81 is Overflow at 0 |
| Shortest form at every length | check only the two-byte form | 80 80 00, 80 80 80 00 and 80 80 80 80 00 are Noncanonical |
| Truncation | return Ok on a short tail | 80 is Truncated at 0; 01 7f 80 is Truncated at 2 with prefix [1,127] |
| Offset starts the failing integer | report the failing byte | 01 80 00 is Noncanonical at 1; 01 ff ff ff ff 10 is Overflow at 1 |
| Valid prefix retained | clear output on error | both cases above keep prefix [1] |
| Cap checked before each integer | read first, then check | 01 at cap 0 is Limit at 0 with empty output; 01 02 at cap 1 is Limit at 1 with prefix [1] |
| Output cleared on entry | reject before clearing; append to stale output | fixed 01 00 00 00 at cap 0 from stale [77,78,79] is Limit at 0 with empty output |
| Fixed-width tail | drop or zero-pad a short chunk | 01 00 00 00 02 is Truncated at 4 with prefix [1] |
| Model agreement | compare the result only | result and full output compared for every generated string |
| Running example | redundant or wrong-order encoder | exact bytes 01 7f 80 01 ac 02 ff ff ff ff 0f |

Observation: the returned Result and the complete output vector after each call.
Oracle and independence: handwritten expected errors and prefixes for the tables; the
reference for generated strings.
Comparator: exact equality of Result, error offset, error kind and output sequence.
Mutation backstop: six handwritten mutants each fail at least one test: `fixed` without
clear; `fixed` without clear and push; `fixed` rejecting cap 0 before clearing; `scalar`
and `unrolled` checking redundancy only in the two-byte form; `unrolled` accepting
truncation after three values. No mutation-score claim is made.
