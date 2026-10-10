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
compared to `BTreeMap`; `tests/seeded_history.rs` repeats the seeded schedule
with the complete row set compared after every transition.

Secondary techniques:
Targeted boundary cases (wraparound, `u64::MAX`, all-used table, home-slot
shift guard), a compile-fail doctest for borrowed iteration plus insertion, and
per-process oracle checks inside every benchmark candidate.

Oracle:
`BTreeMap<u64, u64>` for operation results and final content. It shares no
hashing, probing or deletion code with the implementation.

Input/model/schedule domain and bounds:
Integer keys and values. Test tables start at 8 or 32 slots; the seeded run
reaches 144 live keys and grows through 64 and 128 to 256 slots, so growth is
exercised up to capacity 256 in tests and 16384 in benchmark processes. Histories are five steps over keys 7, 15, 23 and 31 in an 8-slot
Identity-hash table, checked at 7, 15, 23, 31, 39. Seeded runs
use one fixed linear congruential sequence. Extendible checks cover split,
alias update, delete and the depth-12 overflow cap. Single-threaded only.

Replay assets:
All fixtures are deterministic source-level values in `src/lib.rs` tests,
`tests/contracts.rs`, `tests/seeded_history.rs` and `examples/probe.rs`; no
random seed is read from the environment. Assertions in the receipt-pinned
`src/lib.rs` print compared values only; the failing policy, history index or
step is recovered by rerunning the deterministic enumeration under a debugger
or with a temporary print. `tests/seeded_history.rs` prints policy, step and
key in every assertion message.

What a green run establishes:
Exact `BTreeMap` agreement on the enumerated finite domains, after every
transition for the five-step histories and the seeded schedule, plus the listed
rejections and the compile-fail borrow contract. Every benchmark process
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
capacities above `1 << 63`, a doc correction or fallible constructor for
allocation failure, and replay context in the pinned `src/lib.rs` assertions. Any minimized future failure becomes a named deterministic
regression in `tests/contracts.rs` before broadening a generator.

## Invariant-test review

Shared facts: the oracle is `BTreeMap<u64, u64>` or an explicit expected value,
sharing no hashing, probing or deletion code with the implementation; the
comparator is exact equality on `Option<u64>` results, `len`, probe counts and
the full ordered row set. Every record below passes against the current source.

### `deletion_update_wrap_and_extremes` (src/lib.rs)
Claim and scope: insert, remove, update and miss over colliding Identity keys
7/15/23 and `u64::MAX` in 8 slots, each policy.
Plausible violation: wraparound at the last slot or `u64::MAX` masking breaks
reachability; update after a tombstone inserts a duplicate.
Observation: return values of every insert/remove and `len() == 3`.
Oracle and independence: literal expected values.
Smallest discriminating case: four keys sharing home slot 7 in 8 slots.
Mutation backstop: dropping the `& mask` in `find` panics on the wrapped probe.

### `all_used_but_not_all_live_terminates` (src/lib.rs)
Claim and scope: lookup terminates after one circuit when every slot is a
tombstone; a later insert reuses a tombstone.
Plausible violation: unbounded scan or false hit on a table with no empty slot.
Observation: `deleted() == 8`, `lookup_probes(16) == 8`, miss then hit.
Oracle and independence: literal values from the 8-slot layout.
Smallest discriminating case: eight insert/remove pairs in 8 slots.
Mutation backstop: treating a `Deleted` slot as a miss in `find` reports
`lookup_probes(16) == 1`.

### `do_not_shift_key_before_its_home` (src/lib.rs)
Claim and scope: Shift moves only entries whose probe path crosses the hole.
Plausible violation: an entry is moved ahead of its home slot and becomes
unreachable.
Observation: keys 2 and 9 remain found with one probe after removing 1.
Oracle and independence: literal values; key 9 homes at slot 1, key 2 at 2.
Smallest discriminating case: keys 1, 2, 9 in 8 Identity slots.
Mutation backstop: inverting the cyclic-distance test loses key 2.

### `bounded_exhaustive_histories` (src/lib.rs)
Claim and scope: all 32768 five-step insert/remove histories over four
colliding keys agree with the oracle after every step, each policy.
Plausible violation: empty-after-delete, tombstone reuse before the probe ends,
or shift errors on specific histories.
Observation: per-step return value, `len`, and `get` for keys 7..39.
Oracle and independence: `BTreeMap` driven by the same history code.
Smallest discriminating case: the enumeration includes every two-step pair.
Mutation backstop: marking a Shift hole `Deleted` instead of `Empty` panics on
the tombstone counter within the first histories.

### `seeded_grow_rebuild_and_shift` (src/lib.rs)
Claim and scope: 20000 seeded operations over 257 keys per policy with growth
8 to 256 slots agree with the oracle on every return value and final rows.
Plausible violation: rows lost or duplicated by rehash, rebuild or shift.
Observation: per-operation return value and `len`; full row set at the end.
Oracle and independence: `BTreeMap` with the same LCG stream.
Smallest discriminating case: not minimized; a failure prints values only.
Mutation backstop: corrupting a shifted value is caught by the next operation
on that key (lib.rs:430) without step context.

### `seeded_rows_match_oracle_after_every_transition` (tests/seeded_history.rs)
Claim and scope: the same seeded schedule with full rows compared after each
of the 20000 steps, final capacity 256.
Plausible violation: a transient row change between operations that an
end-of-run snapshot cannot place.
Observation: return value, `len` and complete row set every step, with policy,
step and key in the message.
Oracle and independence: `BTreeMap` with the same LCG stream.
Smallest discriminating case: the first step whose rows diverge.
Mutation backstop: the same shifted-value mutant fails at `Shift step 82 key
97`, the removal that performed the shift.

### `snapshot_and_value_mutation` (src/lib.rs)
Claim and scope: `visit_values` edits values in place; `snapshot_keys` allows
structural mutation afterwards.
Plausible violation: mutation through a stale key view or a lost row.
Observation: `len() == 3`, `get(101) == Some(7)`, `get(1) == None`.
Oracle and independence: literal values.
Smallest discriminating case: three keys replaced by `k + 100`.
Mutation backstop: `remove` leaving `len` unchanged yields `len() == 6`.

### `extendible_split_alias_update_delete_overflow` (src/lib.rs)
Claim and scope: directory splits, aliased pointers, update, delete and the
depth-12 overflow cap keep every oracle row reachable.
Plausible violation: a split loses or duplicates rows; the directory exceeds
4096 entries.
Observation: every oracle row after each insert and after deletes;
`stats().0 <= 4096`; pointer visits counted.
Oracle and independence: `BTreeMap`.
Smallest discriminating case: keys 4097/8193/12289 forcing deep splits of a
2-row bucket.
Mutation backstop: removing the depth cap fails `stats().0 <= 4096`.

### `existing_key_after_tombstone_is_replaced_once` (tests/contracts.rs)
Claim and scope: update of a key beyond a tombstone replaces it once.
Plausible violation: insert reuses the first tombstone and leaves two rows.
Observation: `insert` returns the old value, `len() == 2`, one row for key 17.
Oracle and independence: literal values; `iter` counts rows directly.
Smallest discriminating case: keys 1, 9, 17 in 8 Identity slots, remove 9.
Mutation backstop: reusing the first tombstone yields two rows for 17.

### `rebuild_cleans_without_changing_live_map` (tests/contracts.rs)
Claim and scope: Rebuild rehashes at the same capacity when deleted slots
exceed one eighth, keeping every live key.
Plausible violation: a rebuild drops rows or never triggers.
Observation: `deleted() == 0`, `work().rebuilds == 1`, keys 8..19 found.
Oracle and independence: literal values.
Smallest discriminating case: 20 inserts, 8 removes, one insert in 32 slots.
Mutation backstop: changing the one-eighth threshold leaves `deleted() == 8`.
