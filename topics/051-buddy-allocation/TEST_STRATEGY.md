# Test strategy

Scope: serial reservation metadata for one fixed power-of-two arena, three
candidates (scan, buddy, tree) under one placement policy, the trace generator
and the replay runner.
Contract and failure classes: the lowest aligned power-of-two block is reserved.
A zero, overflowing or oversized request, a live or out-of-range ID and an
exhausted arena return None and leave reservations unchanged. A live ID is freed
once, and freeing every ID restores the whole arena. The runner retains every
process attempt with a status and passes only when all 204 are ok.
Primary technique: exact sequences with hand-computed addresses and occupancy
vectors, run on all three candidates (tests/contracts.rs, tests/placement.rs).
Secondary techniques: all 256 eight-step binary request histories compared
across candidates, a Python free-interval oracle on every complete trace
(measure.py), generator domain tables (tests/workload.rs) and stub-process
tests of the runner (test_measure.py).
Oracle: hand-computed values for the sequences. For complete traces, measure.py
keeps disjoint free intervals, splits the selected interval and unions adjacent
free ranges; it shares no Rust placement helper. For the runner tests, an
occupancy-map model in test_measure.py that shares no code with that oracle.
Input/model/schedule domain and bounds: arenas of 8 and 16 units in the
sequences; 256 histories over 4 IDs with requests 1 and 3; 24 complete traces
(3 candidates, 64 and 4096 units, tiny/mixed/holes/churn), all checked before
timing; generator sizes 4 to 4096. Serial calls only.
Replay assets: tests/, measure.py, test_measure.py, examples/replay.rs.
What a green run establishes: agreement on these finite sequences and traces.
The 24 traces hold no zero, oversized or live-ID request and no state that
separates a wrong buddy address from the right one, so those rules rest on the
contract tests. Timed processes repeat the digest, operation count and work of
a verified check process; the digest folds result magnitudes (BENCHMARK.md).
Known gaps and delegated skills: stale caller IDs after reuse, backing memory
access, concurrent lifetimes and arbitrary schedules need separate APIs and
verification. The work counters have no derived oracle. `workload` below 4
units is outside the domain (README.md). No skill is delegated.
Concrete tests to add: with the next measured revision, generated longer
histories against an independent Rust model and a derived oracle for work.

# Invariant-test audit

Mutants are single edits of src/lib.rs at 5e33b3d, run in a scratch copy.

rounding_and_lifecycle
Claim and scope: rounding to a power of two, each rejection class, single free
and restoration, 16 units, all candidates.
Plausible violation: no rounding; a live ID overwritten; a zero request served;
an oversized request clamped; a freed ID kept live; buddy halves not merged.
Observation: every returned address and every rejection, then a 16-unit
allocation at 0 after the frees, which fails if a rejected call reserved units.
Oracle and independence: hand-computed addresses.
Comparator: exact Option and bool equality per call.
Smallest discriminating case: requests 5 then 3 give addresses 0 and 8.
Mutation backstop: 11 mutants fail here, including those six. The live-ID, zero
and oversized mutants pass the other tests and all 24 oracle traces.

alternating_holes_and_restoration
Claim and scope: 8 free units in alternating holes serve no 2-unit request, and
freeing the rest restores one 16-unit block.
Plausible violation: a tree that sums child summaries; a scan that tests the
first unit of a block; a buddy address of a + size; an ID recorded by a failed
allocation.
Observation: the full occupancy vector, the failed 2-unit probe, then a 16-unit
allocation that reuses the probe's ID.
Oracle and independence: hand-computed vector and addresses.
Comparator: exact vector and Option equality.
Smallest discriminating case: the 16-unit arena of this test.
Mutation backstop: 10 mutants fail here, including those four. The a + size
buddy address passes every other test and all 24 oracle traces.

every_small_binary_history
Claim and scope: scan, buddy and tree return equal results on all 256 histories
of 8 steps, 8 units, requests 1 and 3.
Plausible violation: one candidate departs from the shared placement, such as
an unaligned scan or a tree that descends right first.
Observation: every allocation and free result of every step.
Oracle and independence: the scan candidate. The ID and size checks are shared
code, so a defect there passes; the free-interval oracle and the sequences
above cover it.
Comparator: exact equality after each call.
Smallest discriminating case: requests 1 then 3 separate the unaligned scan.
Mutation backstop: 10 mutants fail here; the unaligned scan fails only here
and in the oracle traces.

lowest_address_wins_over_a_smaller_free_block
Claim and scope: with free blocks of 4 units at 0 and 1 unit at 5, a 1-unit
request takes address 0, 8 units, all candidates.
Plausible violation: buddy serves the first sufficient order and returns 5.
Observation: the address and the full occupancy vector.
Oracle and independence: hand-computed.
Comparator: exact equality.
Smallest discriminating case: this 7-call sequence.
Mutation backstop: the first-sufficient-order mutant fails only here; it passes
tests/contracts.rs and all 24 oracle traces.

tests/workload.rs (three tests)
Claim and scope: every request is positive from 4 to 4096 units, churn cycles
are [1, 3, 1, 2], [31, 3, 1, 17] and [2047, 3, 1, 1025] at 4, 64 and 4096 units,
and the measured sizes request at most the arena.
Plausible violation: a churn term of n / 2 or n / 4 - 1, a zero tiny request,
a mixed request of 65.
Observation: the request of every generated allocation.
Oracle and independence: hand-computed cycles and bounds.
Comparator: exact vectors for churn, bounds elsewhere.
Smallest discriminating case: 4 units, where n / 4 - 1 is 0.
Mutation backstop: each of those four mutants fails at least one of the tests.

test_measure.py (seven tests)
Claim and scope: a failed, timed-out or rejected process leaves one line with
its status, command, stdout and stderr and stops the run; the runner refuses
disabled assertions and an existing results file; a timed process agrees with
its check process; a complete run keeps the declared 204-process order.
Plausible violation: the 5e33b3d runner, which lost those attempts, passed a
wrong candidate under -O and accepted a timed process with a different work.
Observation: result lines, the stub launch log, exit status and checks files.
Oracle and independence: stub processes answer from the occupancy-map model.
Comparator: exact record equality for ok lines; exact status, command and
output for the others.
Smallest discriminating case: the first process, scan 64 tiny check.
Mutation backstop: all seven tests fail on the 5e33b3d runner.
