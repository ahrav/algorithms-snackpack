# Test strategy and invariant audit

Scope: bounded immutable byte dictionaries with exact `get` and sorted `prefix` enumeration
over four candidates (`Flat`, `Trie`, `Final`, `Pushed`) plus the campaign runner `run.py`.
Contract and failure classes: raw-byte keys of at most 64 bytes, empty dictionary and empty key,
accepting prefixes, bytes 0 and 255, last-write duplicates, `u64::MAX` values, arbitrary arrival
order, no mutation or decoder; `KeyTooLong` is the only construction rejection.
Primary technique: differential checks against an independent hash-map scan/filter/sort oracle.
Secondary techniques: exact worked examples, 64 generated seeds x 80 writes, boundary tests,
structure counts, externally retained compiled mutations, and stub-campaign runner smoke checks.
Oracle: `oracle()` reuses no trie normalization, graph, registry, or output-pushing helper.
Input/model/schedule domain and bounds: keys of 0..=64 bytes over the full byte alphabet; the
generated model draws 80 keys of length 0..7 over 4 symbols per seed with `u64` values; the
runner schedule is 8 mirrored blocks x 8 cases x (4 candidates + 2 A/A controls) = 384 processes
per host with a 60 s per-command timeout.
Replay assets: `cargo test -p topic025-finite-state-transducers`; `python3 run.py` validates every
generated member, late miss, early-miss query, ordered complete enumeration and narrow prefix
before timing; `python3 measurements/test_runner.py` replays the runner scenarios below.
What a green run establishes: exact outputs and enumeration for these finite inputs and the
generated domain above, the rejection boundary, the 9 -> 5 state sharing example, and the runner's
retention, placement, and argument contracts. It does not establish all 64-byte keys, global
minimality, or a production decoder.
Known gaps and delegated skills: allocation failure, serialized hostile inputs, concurrency, cold
pages, fuzzy search, and mutation runs inside the repository.
Concrete tests to add: raw-byte fuzzing of `prefix` against the oracle at longer lengths; an
in-repository mutation backstop for residual subtraction and finality.

## Invariant audit

`tests::exact_outputs_and_prefix_terminals`
Claim and scope: exact final output and finality for every kind, including `None` versus
`Some(0)`, empty key, bytes 0/255, `u64::MAX`, and last-write duplicates.
Plausible violation: ignoring the initial or final output, accepting every prefix, or keeping the
first duplicate value.
Observation: `get("ab")` is 17 after writes 12 then 17; `get([0])` and `get("aba")` are `None`;
every `prefix` call equals the oracle rows.
Oracle and independence: `oracle()` hash-map scan; shares no graph code.
Comparator: `Vec<Record>` equality including bytes and value; `Option<u64>` equality.
Smallest discriminating case: `("", MAX)` plus `("a", 0)`, or `"ab"` written twice.
Mutation backstop: retained externally (residual-subtraction and finality mutants).

`tests::pushing_shares_residual_functions`
Claim and scope: output pushing shares equal residual mappings while finality and outputs stay exact.
Plausible violation: disabling pushing (no sharing) or merging states by accepted language alone.
Observation: trie and final-interned builds hold 9 states, pushed holds 5; `get("cat")` is 20,
`get("ca")` is `None`, full enumeration equals the input rows.
Oracle and independence: literal expected rows; counts come from `stats()` on each build.
Comparator: exact integer and `Vec<Record>` equality.
Smallest discriminating case: bat 10 / bats 12 / cat 20 / cats 22.
Mutation backstop: a state count alone is not a semantic oracle or a minimality proof; retained
externally.

`tests::generated_differential_and_arrival_order`
Claim and scope: model agreement for every kind over 64 seeds of 80 unordered writes.
Plausible violation: wrong residual subtraction, wrong finality, or a first-write duplicate rule.
Observation: full ordered enumeration, every member's `get`, and every member prefix of every
length match the oracle.
Oracle and independence: `oracle()`.
Comparator: `Vec<Record>` and `Option<u64>` equality.
Smallest discriminating case: a seed whose 80 writes repeat a key with a different value.
Mutation backstop: retained externally.

`tests::empty_maximum_and_rejection`
Claim and scope: the empty dictionary rejects every key, 64-byte keys and `u64::MAX` are exact,
and a 65-byte key is rejected with `KeyTooLong`.
Plausible violation: accepting or truncating a 65-byte key, or emitting from a rejecting root.
Observation: `get` on the empty index is `None` and `prefix("")` is empty; `[255; 64]` returns
`Some(MAX)` while `[255; 63]` returns `None`; `build` with a 65-byte key matches
`Err(BuildError::KeyTooLong)`.
Oracle and independence: literal expected values.
Comparator: exact `Option`, `Result` pattern, and emptiness checks.
Smallest discriminating case: one 65-byte key; one 64-byte key beside the empty key.
Mutation backstop: retained externally.

`every_incoming_prefix_emits_its_own_values` (`tests/contract.rs`)
Claim and scope: enumeration follows every incoming path through a shared state and the public
rejection contract holds for each kind.
Plausible violation: a global visited set during enumeration that emits a shared state once.
Observation: prefixes `""`, `b`, `c`, `cat`, `cats`, `catss` each equal the oracle; `bat` and `cat`
return 10 and 20; the 65-byte key is rejected.
Oracle and independence: `oracle()`.
Comparator: `Vec<Record>` equality and the exact error variant.
Smallest discriminating case: prefixes `b` and `c` over bat/bats/cat/cats.
Mutation backstop: retained externally.

## Runner checks (`measurements/test_runner.py`)

Each scenario runs `run.py` in a scratch topic copy whose benchmark is a stub, or runs
`cargo bench` for one cell. `rerun_refuses_existing_campaign` requires a nonzero exit and
untouched `raw/` when a campaign already exists. `build_artifacts_stay_inside_raw` requires the
compiled test, contract, library, and benchmark binaries under `raw/` and none beside the sources.
`failing_contract_test_is_run_and_retained` appends a panicking test to `tests/contract.rs` and
requires an aborted campaign with the failure retained. `timed_out_command_is_recorded_and_aborts`
uses a sleeping benchmark and requires one retained timeout record and no `summary.json`.
`cargo_bench_defaults_candidate_and_case` requires `cargo bench --bench portfolio` to run
pushed/tiny with no arguments and flat/tiny with `-- flat`. `build_checksum_discriminates`
requires a nonzero build checksum for flat and pushed.
