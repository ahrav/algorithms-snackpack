# Test strategy

Scope: the safe locked scheduler models `global_fifo` and `locked_work_stealing`
with both placements, the sequential `oracle`, and the campaign runner.

Contract and failure classes: every preloaded task executes exactly once and the
run terminates; `Outcome` reports the executed count and the wrapping checksum
of deterministic per-task values. A zero worker count panics at the public
boundary. Failure classes are a lost task, a duplicated task, a wrong per-task
value, and non-termination.

Primary technique: exact-boundary agreement with the independent sequential
oracle under a 30 second deadline per run.

Secondary techniques: a rejection test for zero workers; the `compare` example
asserts the oracle on every measured process, so each campaign row is also a
contract check; `scripts/test_run_campaign.py` drives the runner end to end
against a planned stub binary.

Oracle: `oracle` folds `task_value` over the task range on one thread. It
shares the per-task value function and no queue, thread, counter or placement
code with either candidate.

Input/model/schedule domain and bounds: workers 1 through 6; tasks 0, 1, 2, 17
and 256; leaf work 0, 1 and 7; both placements. Schedules are whatever the host
scheduler produces for unpinned threads; each case is one run, so interleavings
are sampled, not enumerated.

Replay assets: `cargo test -p topic028-work-stealing-deques`;
`python3 topics/028-work-stealing-deques/scripts/test_run_campaign.py`; the
campaign commands in `BENCHMARK.md`; the retained rows in `measurements/`.

What a green run establishes: count and checksum agreement with the oracle, and
termination within the deadline, for the listed finite cases on the running
host. It does not establish lock-free atomic ordering, retired-array
reclamation, behaviour under dynamic spawning, or any schedule the host did not
produce.

Known gaps and delegated skills: no exhaustive schedule exploration and no
model checker, so a race that the host rarely schedules can pass. Steal
direction and victim order are policy, and the oracle cannot observe them. A
Chase-Lev candidate needs atomic-ordering review and a reclamation argument
before it enters this crate.

Concrete tests to add: a scoped-thread harness that interleaves owner pops and
thief steals on a two-task deque at every lock boundary; a test that observes
steal direction through an instrumented queue; a property over random
(workers, tasks, leaf_work) triples once a Chase-Lev candidate exists.

## Invariant-test audit

`all_candidates_match_the_independent_oracle`

Claim and scope: for every listed case, each candidate returns the oracle's
`Outcome` and terminates.

Plausible violation: a task is never enqueued, a thief and owner both execute
one task while another is dropped, or a worker exits before the last task runs.

Observation: the returned `Outcome` compared field by field, with a 30 second
deadline so a scheduler that spins on `remaining` fails instead of hanging.

Oracle and independence: the sequential fold above; it has no queues or threads.

Comparator: `assert_eq!` on `Outcome`, which compares `completed` and `checksum`.

Smallest discriminating case: one task with zero leaf work. `task_value(0, 0)`
is 0, so a dropped first task leaves the checksum unchanged and is caught only
by `completed`, which is why both fields are compared.

Mutation backstop, run against the committed test with mutated `src/lib.rs`
and then restored:

| Mutant | Result |
| --- | --- |
| `global_fifo` enqueues `1..task_count` | Fails: `completed` 0 versus 1 at tasks=1, leaf_work=0 |
| `locked_work_stealing` enqueues `1..task_count` | Spins on `remaining` forever; the deadline fails the test after 30 s. Without the deadline the test hung past 900 s |
| Thief peeks `front()` under one lock and pops under a second lock | Duplicated and lost steals desynchronise `remaining`; the deadline fails the test after 30 s in three of three runs |
| Stealing worker exits when one victim scan finds nothing, ignoring `remaining` | Survives. With a preloaded closed task set, an empty scan implies every task was claimed, so this mutant is equivalent for the contract. `remaining` matters once tasks can be spawned during the run |

`zero_workers_is_rejected` and `zero_workers_is_rejected_by_work_stealing`

Claim and scope: `global_fifo(0, ..)` and `locked_work_stealing(0, ..)` panic
with exactly `workers must be positive`.
Plausible violation: a zero-worker run returns an empty `Outcome`, fails
later with a different panic, or the diagnostic drifts. Observation and
comparator: `catch_unwind` captures the payload and `assert_eq!` compares the
complete string, so a longer message containing the phrase fails.
Independence: the message is asserted, not computed from the implementation.
Smallest case: the single call in each test. Mutation backstops, executed:
removing `checked_args` from `locked_work_stealing` makes it panic at the
`id as usize % workers` division instead, and the work-stealing test fails
while the `global_fifo` test still passes; changing the message to
`workers must be positive (got {workers})` fails both tests, where the
earlier `should_panic(expected = ...)` substring form passed.

## Runner checks

`scripts/test_run_campaign.py` replaces the compare binary with a stub whose
plan controls each invocation. It asserts: the summary applies the paired 3
percent rule and reports `unresolved` for a mixed cell; failed and timed-out
processes are retained as rows and the campaign continues; the output file
exists with `status: running` while a process is still executing; a row whose
process echoes different arguments or a non-positive `elapsed_ns` is retained
as `failed` and excluded from ratios; a failed warmup is retained and forces
its cell to `unresolved`; the A and B
control rows exist with alternating order and ratios of exactly 1 for a
constant stub; and `environment` records the binary hash, source hashes,
commit, toolchain and CPU identity, where `cpu_model` keeps the Arm
implementer, variant, part and revision from the first processor block and
the x86 model name. Each assertion failed on the previous
runner before the corresponding change landed.
