# Test strategy and invariant audit

Independent model: tests use an ID-keyed BTreeMap, collect all due entries and
sort by deadline/ID. It shares no placement, serial or dispatch logic with the
candidates. The Python runner independently constructs workload state and
compares complete expiry triples and final live pairs from every process. It
also derives each candidate's three work counters from the workload and requires
them to match.

Five contract groups cover exact rejection errors, exact boundary witnesses,
exact work counters, workload generation and 96 seeded
80-event traces checked after every prefix across all four candidates. Seeds
are fixed and wrapping pseudorandom arithmetic is explicit. Witnesses include
repeated same-deadline rearming, cancellation/reinsertion, duplicate advance,
empty state, same-slot deadlines 1/65, tick 64, large jumps and maximum horizon.

Invariant violations discriminated by these tests: accepting a stale same-ID arm
duplicates output; firing a bucket on its first visit emits deadline 65 at tick 1;
reversing equal-deadline ID order breaks complete output equality. Whole-input
validation rejects backward time, past/present arms, horizon excess and invalid IDs
with the exact error, including an invalid event after a valid prefix; 65,536 IDs
are accepted and 65,537 rejected. Work counters are pinned for the README trace,
a deadline-65 wheel revisit and a cancelled record. Source mutants these tests
reject: capacity limit 65,535, a swapped error string, an uncounted heap pop,
wheel tick, tree peak or stale wheel visit, replacement offset 16 and workload
capacity 65,535.

Runner: benches/test_run.py starts benches/run.py on stub processes whose answers
come from a step simulation of the four mechanisms, written separately from the
runner's closed-form counter oracle. It checks that -O, -OO and PYTHONOPTIMIZE
stop the runner before any output; that a nonzero exit, an unavailable CPU 0, a
timeout and 22 unusable outputs (text, bytes that are not UTF-8, wrong shape,
wrong fired/live, `ns` that is not a positive integer, each counter off by one
or of the wrong type) each leave one attempts.jsonl line with its status, the
stdout/stderr bytes and no later process; that a complete 348-process campaign
follows the declared schedule and commands; and that the runner's models equal
the step simulation in all 48 candidate/cell combinations and the published
4096/far/0 counts.

These are bounded sequential tests. Concurrent ownership, wall-clock conversion,
real wakeup lateness, process resource ceilings and periodic policy require their
own contracts and tests. Tests retain error outcomes as part of the public API.
