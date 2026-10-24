# Test strategy

Scope: frozen byte-key dictionaries (`flat`, `tree`, `trie`, `radix`): exact
lookup, ordered prefix materialization, bulk construction and drop.
Contract and failure classes: arbitrary bytes, empty keys, embedded NUL/0xff,
prefix-of-another keys, zero values, last duplicate wins, byte-key output
order; failures are wrong value, missing/extra prefix output, wrong order,
first-duplicate retention, and prefix ending inside a compressed path.
Primary technique: exact boundary examples plus independent-model agreement.
Secondary techniques: sampled random binary histories at every stored-key
prefix length; benchmark-time response checks before warmup; compiled mutants.
Oracle: latest-write HashMap, filter by `starts_with`, sort by byte key.
Candidates never call it. Exact benchmark queries use a reverse input scan.
Input/model/schedule domain and bounds: fanouts 0, 1, 4, 5, 16, 17, 48, 49,
255, 256 with every first-byte lookup; 80 sampled 80-write binary histories
with keys of length 0..=6; the running example; zero values with all-0xff
prefixes; a 5-key sparse-page model with 64-slot pages.
Replay assets: `src/lib.rs`, `tests/contracts.rs`, `examples/dictionary.rs`,
`benches/dictionary.rs`, `scripts/run.py`, `measurements/test_runner.py`.
What a green run establishes: the tested boundaries and sampled histories
agree with the independent model for all four candidates, and every timed
benchmark response matched the model before timing. Histories are bounded
evidence, not proof.
Known gaps and delegated skills: allocation-failure injection, online
insertion/deletion, Unicode normalization, hostile recursion depth (a
service-level bound), kernel XArray or RCU behavior, concurrency.
Concrete tests to add: a depth bound or iterative traversal test if hostile
key depth enters the contract; mutation-based online update tests if a
mutable ART lands.

## Invariant-test audit

| Claim and scope | Plausible violation | Observation | Oracle and independence | Comparator | Smallest discriminating case | Mutation backstop |
|---|---|---|---|---|---|---|
| `arbitrary_bytes_terminals_last_write_and_prefix_inside_path`: terminals, duplicates and in-path prefixes for all candidates | retain first duplicate; drop a terminal with children; reject a prefix ending inside a compressed path | `prefix` vector and `get` option per query | HashMap latest-write filter/sort; reverse input scan for `get` | exact `Vec<u64>` and `Option<u64>` equality | `car=1` then `car=9`; prefix `ca` inside the `car` path | retain-first-duplicate and reject-in-path-prefix mutants rejected |
| `every_fanout_boundary_and_miss_agrees_with_independent_model`: every child-class boundary and every first-byte miss | slot 0 treated as missing in the 17..48 class; off-by-one at 4/16/48 | `prefix(&[])` vector and 256 `get` results per size | HashMap oracle; linear `find` over rows | exact equality | n = 17 with byte 0 present | slot-0-missing mutant rejected |
| `sampled_binary_histories_match_hash_scan_sort`: all stored-key prefix lengths over random binary keys | omit radix terminal output; wrong order under embedded NUL/0xff | `prefix(q)` for every stored prefix length | HashMap oracle | exact equality | an empty key and a one-byte key sharing a history | omit-terminal-output mutant rejected |
| `zero_is_a_value_and_ff_prefix_needs_no_successor_key` (`tests/contracts.rs`): zero payloads and all-0xff prefixes | treat 0 as absent; build a successor key that overflows past 0xff | `get` options and `prefix(&[255])` | literal expected values | exact equality | keys `[]`, `[255]`, `[255,0]`, `[255,255]` | rejected by the literal expectations |
| `sparse_pages_and_owned_lookup_survive_removal`: sparse 64-slot page occupancy and owned lookup after removal | page index arithmetic off by one; lookup result invalidated by removal | page count, per-key slot values, held `Arc` after `remove` | HashMap model of the same keys | exact equality; `pages.len() == 4` | keys 63 and 64 straddling a page edge | none; descriptive model test |

Discriminating compiled mutants: retain first duplicate; reject prefix ending
inside compressed path; treat slot 0 as missing in the 17..48 class; omit radix
terminal output. All four were rejected by the original tests. Receipts and raw
outputs are external (`measurements/EVIDENCE_RECEIPT.json`).

Example checks `car`/`cart`/`carbon`/`cab` order. The sparse userspace model is
not a kernel XArray or concurrent algorithm test. An `Arc` cloned while the
`RwLock` is held models ownership transfer; it does not model optimistic
validation, RCU grace periods, or kernel API error handling. Long hostile
recursion is an explicit service-level bound and revisit item.
