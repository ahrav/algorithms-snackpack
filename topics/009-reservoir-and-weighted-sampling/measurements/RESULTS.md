# Final exact committed-source campaign

| Workload | Arm best estimate | x86 best estimate |
|---|---|---|
| u-cold | x 0.528 us; unresolved r,heap | r 0.660 us; unresolved x,heap |
| u-tiny | x 0.134 us | x 0.149 us; unresolved r,heap |
| u-sparse | heap 7.980 us; unresolved x | heap 7.411 us; unresolved x |
| u-half | x 35.476 us | x 39.591 us; unresolved r |
| u-large | heap 107.854 us | heap 95.017 us |
| u-full | r 3.424 us; unresolved x | r 2.384 us; unresolved x |
| w-tiny | wheap 0.559 us; unresolved wsort | wheap 0.579 us; unresolved wsort |
| w-sparse | wheap 86.570 us | wheap 89.855 us |
| w-half | wsort 221.025 us | wsort 244.114 us |
| w-skew | wheap 1356.394 us | wheap 1369.964 us |
| w-ascending | wheap 101.806 us | wheap 101.901 us |
| w-full | wheap 247.924 us; unresolved wsort | wheap 281.282 us; unresolved wsort |

A unique winner requires all competitor/fastest lower simultaneous 95% bounds >1.05. Other entries are point estimates with unresolved rivals, not equivalence claims. Full method geometric means, median/min/max and all paired ratio intervals are in summary.json. 84-contrast family, 12 blocks, 11df, t=4.754624. Conditional on one fixed generic build per host and observed conditions; no 5% noise calibration claim. No A/A campaign or cross-build generalization.

The final replay follows a documentation-only correction to the local Top-K merge precondition. Source, tests, example and runner hashes are bound in EVIDENCE_RECEIPT.json. Previous campaigns remain retained and are not pooled as independent samples.

Two later facts qualify this table. First, commit 36ddffd changed src/lib.rs (combine clamps k to the retained key count) and tests/contracts.rs (two combine assertions) after the replay; the receipt lists both hash sets, and the measured bench and example binaries contain no combine call. Second, the frozen runner's rotate-then-reverse assignment cancels for two-method cells, so every weighted cell ran wsort before wheap in all 12 blocks. Weighted winners and intervals are therefore conditional on that fixed within-block order and any temporal or thermal drift inside a block; uniform cells rotated through four orders and are unaffected. Removing the weighted confound requires a corrected assignment and a new weighted campaign on both hosts.
