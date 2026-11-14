# Test strategy and invariant audit

Independent model: tests use an ID-keyed BTreeMap, collect all due entries and
sort by deadline/ID. It shares no placement, serial or dispatch logic with the
candidates. The Python runner independently constructs workload state and
compares complete expiry triples and final live pairs from every process.

Three contract groups cover invalid input, exact boundary witnesses and 96 seeded
80-event traces checked after every prefix across all four candidates. Seeds
are fixed and wrapping pseudorandom arithmetic is explicit. Witnesses include
repeated same-deadline rearming, cancellation/reinsertion, duplicate advance,
empty state, same-slot deadlines 1/65, tick 64, large jumps and maximum horizon.

Invariant violations discriminated by these tests: accepting a stale same-ID arm
duplicates output; firing a bucket on its first visit emits deadline 65 at tick1;
reversing equal-deadline ID order breaks complete output equality. Whole-input
validation rejects backward time, past/present arms, horizon excess and invalid IDs.

These are bounded sequential tests. Concurrent ownership, wall-clock conversion,
real wakeup lateness, process resource ceilings and periodic policy require their
own contracts and tests. Tests retain error outcomes as part of the public API.
