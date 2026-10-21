# Test strategy and invariant audit

Scope: bounded immutable byte dictionaries, exact values and prefix enumeration.
Contract/failures: raw bytes<=64, empty dictionary/key, prefix terminal, binary0/255,
last-write duplicates, max u64, arbitrary arrival order, no mutation/decoder.
Primary technique: independent hash-map scan/filter/sort differential checks.
Secondary: exact examples,64 generated seeds*80 writes, boundary tests and compiled mutations.
Oracle independence: no trie normalization, graph, registry, or output-pushing helpers reused.
Replay: cargo test -p topic025-finite-state-transducers; run.py validates every generated
member, late miss, early-miss query, ordered complete enumeration and prefix before timing.
A green run establishes these finite inputs, not all64-byte keys or a production decoder.
Gaps: allocation failure, serialized hostile inputs, concurrency, cold pages and fuzzy search.

Claim: exact output/finality. Violation: ignore initial/final output or accept every prefix.
Observation: bat10/cat20/cats22 and absent ca; oracle independent; equality includes bytes/value.
Smallest case: empty key plus ab or bat/bats. Mutation backstop retained externally.
Claim: duplicate policy. Violation: retain first value. Observation: ab12 then ab17 ->17.
Claim: sharing. Violation: disable output pushing. Observation: example9->5 states and exact
full output enumeration. Count alone is not a semantic oracle or global minimality proof.
Claim: independent incoming paths. Violation: global visited-set during enumeration.
Observation: both b and c prefixes emit distinct correct outputs; full ordered rows compared.
Claim: rejection. Violation: accept/truncate65-byte key. Observation: exact KeyTooLong result;
constructor consumes an immutable slice and publishes no result on rejection.
Claim: generated model agreement. Violation: wrong residual subtraction/finality/duplicate rule.
Observation: full ordered rows and every member's every prefix compared for64 fixed seeds.
