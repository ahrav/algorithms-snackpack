Scope: exact forward snapshot reads over finite sorted histories.
Contract and failure classes: run order, unique sequences, point and range
visibility, half-open boundaries, no fallback after a point delete.
Primary technique: independent chronological replay and exact row comparison.
Secondary techniques: fixed boundary witnesses and deterministic generated histories.
Oracle: sequence sort followed by mutations of an ordered map, plus hand-derived
expected vectors. The selectors use key ordering and a range priority queue.
Input/model/schedule domain and bounds: 256 histories, 48 operations and
48 snapshots each, 12 generated keys and up to 11 runs, plus fixed extrema.
Replay assets: tests/contract.rs and examples/snapshot.rs.
What a green run establishes: equality of complete ordered vectors on these inputs.
Known gaps and delegated skills: reverse iteration, persistence, duplicate
sequence arbitration and multi-snapshot compaction are outside the contract.
Concrete tests to add with scope: reverse starts/ends, seeks into an active
span and multi-snapshot compaction equivalence against untouched runs.

| Claim and scope | Plausible violation | Observation and comparator | Independent oracle and discriminating input |
| --- | --- | --- | --- |
| Snapshot ordering, reinsertion and point deletes | Future points mark a key seen, or the older put survives a point delete | Full ordered rows at S 6, 8, 10 and 12 | Fixed vectors. 12@11 stays invisible at S=10, then survives at S=12. 14@12 suppresses 14@6. |
| Half-open bounds and absence | Treat end as inclusive, or encode exhaustion with a valid key | Full output with empty runs, zero and maximal keys/sequences | Fixed vectors. Key 1 survives [0,1) and the maximal key remains real. |
| Lazy heap expiration | Pop once, or keep only the newest range | Rows when top range expires and an older still-live range takes over | Fixed vectors. At key 6 the old [3,7) still covers after [4,6) expires. At 7 buried ranges expire too. |
| Exact merge agreement across run layouts and snapshots | Replay a winner against the wrong opponents or fail to exhaust a run | Full sequence of ordered key/value rows across all generated snapshots | Chronological BTreeMap replay, no shared selection/sweep logic. Three runs plus padding in the fixed examples. |

Mutation backstop: change the end comparison to strict less-than, change
repeated expiration to a single pop, or mark the key seen before checking
snapshot visibility. Each error must fail the boundary/history tests.
