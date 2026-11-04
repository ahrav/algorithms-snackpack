# Test strategy

Scope: first-match arbitrary bytes; five implementations and both counter modes.
Contract and failures: earliest offset, empty at zero, m>n absent, final window,
zero/high bytes, unsafe skips, periodic-memory loss, wrong fallback offsets.
Primary technique: exhaustive binary texts lengths0..9 and patterns0..6 against
independent slice-window equality. Secondary:2000 seeded larger inputs, all256
bytes, named periodic fixtures, and exact diagnostic witnesses.
Oracle: standard-library windows and slice equality, sharing no shift logic.
Negative controls: explicit assertions reject last-match and omitted-final-window
answers; suffix trap discriminates unit shifts and adaptive budget enforcement.
Green scope: bounded agreement and named contracts on executed toolchains.
Replay: cargo test plus the standalone rustc runner. Complexity arguments remain
separate from sampled/exhaustive bounded correctness evidence.
Known gaps: all-occurrence, streaming, transformed text, allocation failure and
exhaustive machine-size inputs. Concrete tests are in tests/contracts.rs.

## Invariant audit

- Earliest offset: plausible last-match mutation returns1 for aaa/aa; expected0
  from independent oracle and an explicit inequality reject it.
- Final alignment: xabc/abc must return1; an exclusive upper bound returnsNone.
- Model agreement: exhaustive texts0..9 and patterns0..6 compare exact Options,
  retaining offsets and absence rather than only boolean existence.
- Repetition: suffix trap independently requires (n-m+1)*m Horspool comparisons.
  A wrong table or verification order changes the exact count.
- Fallback: suffix traps for m={4,31,32,63,64,127} require one fallback and
  comparison count <=4n+2m. Repeated demotion or lost offset changes observations.
- Two-Way period memory: seeded and alternating periodic cases compare exact
  offsets against the oracle in both instrumented and uninstrumented modes.

Named negative-control answers were rejected by executed assertions. Mutating
compiled implementation source and a full mutation-tool campaign are unrun.
