# Correctness strategy and invariant audit

BTreeMap is an independent oracle; it shares no hashing/deletion helper.
Seven library test groups: wraparound/extreme keys/update; all-used termination;
no shift before home;32768 five-step histories for each policy;20000 seeded
operations/policy with growth; snapshot/value mutation; extendible aliases,
update/delete,split/doubling,depth-cap overflow. Two public contract tests
reject duplicate keys after tombstone update and validate same-cap cleanup.
The example asserts exact running-example result and directory accounting.
One compile-fail doctest rejects borrowed iteration plus structural insertion.

Violation -> rejecting check: empty-after-delete -> collision/wrap/history;
first-tombstone insert -> public update/count; unbounded scan -> all-used
probe-count; unconditional shift -> home regression; lost rows in growth ->
seeded oracle; duplicate directory aliases -> split/lookup oracle; unbounded
prefix split -> cap overflow test; mutation during borrow -> compile-fail.

Every benchmark process compares exact operation output vector and complete
final ordered map against BTreeMap, then times a distinct prepared map.
Timings are admitted only after this oracle passes. These are finite checks,
not a proof of all histories, panic behavior or concurrent use.
