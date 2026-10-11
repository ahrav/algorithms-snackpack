# Correctness strategy and invariant audit

Scope: new/get/get_counted/insert/remove/len/is_empty/layout/valid under all three
policies and mixed/identity/constant hash modes. Single-threaded fixed capacity.

Oracle: independent std BTreeMap, sharing no hash/probe/displacement logic.
Primary method: seeded LCG history,4000 transitions per policy/hash/capacity,
capacities2/8/64,97-key domain. Compare mutation results/count and every stored
value after every transition. Benchmark processes compare every query against
BTreeMap; churn replays the exact timed rounds against BTreeMap and checks the
timed checksum against that replay. Pair checksums must agree. Literal
expectations backstop topology witnesses.

Green means agreement on bounded tested histories and exact witnesses. `valid`
is an internal diagnostic and not an independent oracle. Tests are not a proof
for all histories. No unsafe/concurrency tools are warranted. Known gaps: exhaustive
operation schedules, arbitrary payload ownership/destructors, compact-DIB overflow,
allocation failure injection, invalid-capacity panic and production resizing.

## Invariant-test audit

Shared facts: comparators are exact equality on `Option<u64>`,
`Result<Option<u64>, Full>`, `(Option<u64>, usize)`, `usize` or `Vec<(usize, u64, usize)>`.
Every mutation backstop was applied to a scratch copy of the committed source, observed
to fail the named test with the quoted output, and discarded; repository source was
never mutated. The three backstops from the original campaign are also retained externally
under evidence/topic-015/20261011/mutation-backstops.

### `independent_oracle_mixed_operations` (src/lib.rs)
Claim and scope: for every policy, hash mode and capacity 2/8/64,4000 LCG-driven
insert/remove/get steps over a97-key domain agree with BTreeMap on every result, `len`
and every stored value after each transition; a new key on a full table returns `Err(Full)`.
Plausible violation: a backward shift or swap that strands or duplicates a resident after
removal, or a full-table rejection that still mutates.
Observation: per-step result equality, `len`, `valid()`, and `get` of every oracle key.
Oracle and independence: `BTreeMap<u64, u64>` driven by a separate LCG; `mix` is unused.
Comparator: exact equality after every transition, not only at the end.
Smallest discriminating case: capacity2 under Constant hash saturates within a few steps;
capacity8 under Identity crosses the wrap.
Mutation backstop: `e.dib >= gap` to `e.dib > gap` in `remove` fails `assertion failed: t.valid()`;
a `remove` that leaves the slot occupied fails `left: Ok(None) right: Ok(Some(158))`.

### `displacement_is_not_globally_sorted` (src/lib.rs)
Claim and scope: Robin placement of Identity keys0,8,2 in capacity8 yields
`[(0,0,0),(1,8,1),(2,2,0)]`, and a miss for16 inspects exactly three slots.
Plausible violation: lookup stops at a resident whose DIB equals the probe distance, or
assumes DIB is sorted across homes.
Observation: `layout()` literal and `get_counted(16) == (None, 3)`.
Oracle and independence: hand-derived literal layout and probe count.
Comparator: exact equality.
Smallest discriminating case: three keys, one displaced between two at-home residents.
Mutation backstop: `e.dib < d` to `e.dib <= d` in `locate` fails `left: (None, 1) right: (None, 3)`.

### `swap_and_wrap_delete` (src/lib.rs)
Claim and scope: Robin insertion of6,7,14 displaces7 across the wrap to slot0 with DIB1;
removing6 shifts14 and7 back to DIB0 and `valid()` holds.
Plausible violation: reversed swap priority; backward shift that fails across the wrap or
does not decrement DIB.
Observation: `layout()` before and after `remove(6)`, plus `valid()`.
Oracle and independence: hand-derived literal layouts.
Comparator: exact equality.
Smallest discriminating case: three keys whose cluster crosses the last slot.
Mutation backstop: `incoming.dib > resident.dib` to `incoming.dib < resident.dib` fails
`left: [(0, 14, 2), (6, 6, 0), (7, 7, 0)] right: [(0, 7, 1), (6, 6, 0), (7, 14, 1)]`.

### `linear_must_skip_at_home_then_continue` (src/lib.rs)
Claim and scope: linear deletion of0 with residents1 (at home) and8 (displaced past1)
keeps8 reachable, moving it into slot0 while1 stays.
Plausible violation: deletion scan ends at an at-home resident and strands8 behind the hole.
Observation: `get(8) == Some(8)` and `layout() == [(0,8,0),(1,1,0)]`.
Oracle and independence: hand-derived literal layout.
Comparator: exact equality.
Smallest discriminating case: keys0,1,8.
Mutation backstop: removing the `!= Policy::Linear` guard on the DIB0 stop fails
`left: None right: Some(8)`.

### `full_failure_and_update_preserve_contents` (src/lib.rs)
Claim and scope: for every policy, a full8-slot table rejects a new key with `Err(Full)`
and unchanged layout, updates an existing key in place returning the old value, keeps the
absent key absent, and still removes and validates afterward.
Plausible violation: Full check before the update lookup; relocation before failing; update
reporting `Ok(None)`.
Observation: `insert(16, 99) == Err(Full)`, `layout()` equals its pre-call snapshot,
`insert(8, 99) == Ok(Some(8))`, `get(8) == Some(99)`, `get(16) == None`, `remove(1) == Some(1)`, `valid()`.
Oracle and independence: literal values and the pre-call layout snapshot.
Comparator: exact equality.
Smallest discriminating case: eight Identity keys over homes0..3 twice.
Mutation backstop: moving the Full check ahead of the lookup fails `left: Err(Full) right: Ok(Some(8))`;
update returning `Ok(None)` fails `left: Ok(None) right: Ok(Some(8))`.

### `empty_absent_and_reinsert` (tests/contracts.rs)
Claim and scope: for every policy under Constant hash, an empty table reports empty, `get`
and `remove` of an absent key return `None` without effect, `u64::MAX` coexists with
colliders0,8,16, remove then reinsert of `u64::MAX` returns `Some(u64::MAX)` then `Ok(None)`,
and the new value reads back with `valid()`.
Plausible violation: absent remove dereferences a slot; reinsertion treated as an update.
Observation: the listed results in order, then `valid()`.
Oracle and independence: literal values.
Comparator: exact equality.
Smallest discriminating case: `remove(0)` on the empty table.
Mutation backstop: `locate(key).0?` to `locate(key).0.unwrap_or(0)` in `remove` panics at
`expect("located entry")` on the empty table. A `remove` that leaves the slot occupied is
masked here by the backward shift under Constant hash; `independent_oracle_mixed_operations`
catches it (see above).

### `robin_early_exit_does_not_hide_present_key` (tests/contracts.rs)
Claim and scope: Robin lookup over Identity keys0,1,8,16,3 in capacity8 finds every
present key and reports24 absent.
Plausible violation: early exit when a resident's DIB equals the probe distance.
Observation: `get(k) == Some(k)` for all five keys, `get(24) == None`, `valid()`.
Oracle and independence: literal values.
Comparator: exact equality.
Smallest discriminating case: `get(0)` with0 at home.
Mutation backstop: `e.dib < d` to `e.dib <= d` in `locate` fails `left: None right: Some(0)`.
