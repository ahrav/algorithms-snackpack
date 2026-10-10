# Correctness strategy and invariant audit

Independent oracle: std::collections::BTreeMap, with no shared probe/mask helper. Every
candidate-process query checks exact Option<value> before timing and verifies its checksum.
The operation history oracle uses independent ordered keys; it covers120,000 insert/erase/query
steps across narrow/wide equality and normal/cluster/constant-tag hashes.

| Contract | Plausible violation | Discriminating check |
|---|---|---|
| exact tag mask | cross-byte borrow creates an extra lane | all32,768 byte/tag pairs plus alternating0/1 witness |
| candidate before EMPTY | return absent at empty before a later candidate | fabricated group with exact key at lane15 after empty lanes |
| bounded full-table probing | absent lookup loops forever | full table, absent key, tombstone reuse and capacity failure |
| tombstone preserves path | erased slot stops displaced lookup | wrapped full groups, overwrite beyond tombstone |
| duplicate replacement | reuse tombstone before checking existing key | replace key31 after key0 erased; preserve value777 |
| tag collision remains exact | return first matching tag | constant-tag keys, present and absent queries |
| erasure subset consistency | remove one key hides another | all256 subsets of eight deletions among24 colliding keys |
| mutation equivalence | lookup/remove disagree with stored contents |120,000 deterministic BTreeMap history steps |

Five library groups plus one external group pass locally. Tests are bounded evidence; they are
not a proof for every arbitrary key/prehash violation or production table revision. Uniform
synthetic keys do not establish collision-attack resistance. No unsafe code is present.

The initial harness invocation without arguments failed during cargo --all-targets; the
benchmark now returns normally when invoked without explicit workload/candidate arguments.
Literal/style and mathematically bounded conversion lint findings were repaired before commit.
Initial and exact committed campaigns remain separate and their input hashes are retained.
Both Linux exact-source contracts remain pending shared expired Midway authentication.
