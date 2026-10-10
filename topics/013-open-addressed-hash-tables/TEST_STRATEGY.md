# Open-addressed hash table test strategy

Scope:
The public `LinearMap` API (`new`, `insert`, `get`, `remove`, `len`, `is_empty`,
`capacity`, `deleted`, `work`, `lookup_probes`, `iter`, `visit_values`,
`snapshot_keys`) under the Lazy, Rebuild and Shift deletion policies with the
Mixed and Identity hash modes, and the `ExtendibleMap` operation-count model.

Contract and failure classes:
`insert` replaces an existing key once and reports the old value; a tombstone
is reused only after the probe sequence proves the key absent. `get` and
`remove` stop at a true empty slot or after one full circuit. Live occupancy
stays at or below seven eighths; Rebuild cleans when deleted slots exceed one
eighth. Shift moves only entries whose cyclic probe path crosses the hole.
Borrowed iteration excludes structural mutation at compile time. Failure classes:
empty-after-delete breaking reachability, duplicate keys after tombstone update,
unbounded scan when every slot is used, shifting a key before its home, rows lost
in growth, aliased directory entries, unbounded prefix splitting.

Primary technique:
Exact examples plus bounded exhaustive five-step histories (32768 per policy)
and 20000 seeded insert/remove/get operations per policy with growth, each
compared to `BTreeMap`.

Secondary techniques:
Targeted boundary cases (wraparound, `u64::MAX`, all-used table, home-slot
shift guard), a compile-fail doctest for borrowed iteration plus insertion, and
per-process oracle checks inside every benchmark candidate.

Oracle:
`BTreeMap<u64, u64>` for operation results and final content. It shares no
hashing, probing or deletion code with the implementation.

Input/model/schedule domain and bounds:
Integer keys and values; capacities 8 to 32 in tests, up to 16384 in benchmark
processes. Histories are five steps over keys 7, 15, 23 and 31 in an 8-slot
Identity-hash table, checked at 7, 15, 23, 31, 39. Seeded runs
use one fixed linear congruential sequence. Extendible checks cover split,
alias update, delete and the depth-12 overflow cap. Single-threaded only.

Replay assets:
All fixtures are deterministic source-level values in `src/lib.rs` tests,
`tests/contracts.rs` and `examples/probe.rs`; no random seed is read from the
environment. A failing assertion prints the policy, history index or key needed
to replay the disagreement.

What a green run establishes:
Exact `BTreeMap` agreement on the enumerated finite domains, the listed
rejections, and the compile-fail borrow contract. Every benchmark process
compares its exact operation output vector and complete final ordered map
against `BTreeMap` before timing a distinct prepared map, so timings are
admitted only after this oracle passes. These are finite checks, not a proof
over all histories, all capacities, panic behavior or concurrent use.

Known gaps and delegated skills:
No unsafe code, so Miri is not selected; no shared state, so Loom and Shuttle
are not selected; typed integer input, so fuzzing is not selected. Panic and
abort paths are untested: allocation failure runs the default allocation error
handler, which aborts the process, although the `new` and `insert` docs state a
panic; in release builds `new` with capacity above `1 << 63` returns a
zero-capacity map instead of panicking. The receipt-pinned `src/lib.rs` keeps
both until the next committed replay. `invariant-test-review` is applied below.

Concrete tests to add:
With the next committed replay: a release-mode construction test rejecting
capacities above `1 << 63`, and a doc correction or fallible constructor for
allocation failure. Any minimized future failure becomes a named deterministic
regression in `tests/contracts.rs` before broadening a generator.

## Invariant-test review

Seven library test groups: wraparound/extreme keys/update; all-used termination;
no shift before home; 32768 five-step histories for each policy; 20000 seeded
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
Comparator is exact equality on `Option<u64>` results, `len`, and the full
ordered row set; the smallest discriminating cases are the 8-slot Identity
tables above; the mutation backstop is the seeded 20000-operation oracle run.
