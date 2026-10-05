# Exact committed-source campaign

| Workload | Arm best estimate | x86 best estimate |
|---|---|---|
| u-cold | x 0.541 us; unresolved r,heap | r 0.691 us; unresolved x,heap |
| u-tiny | x 0.132 us | x 0.150 us; unresolved r,heap |
| u-sparse | heap 8.098 us; unresolved x | heap 8.349 us; unresolved x |
| u-half | x 35.388 us | r 41.045 us; unresolved x |
| u-large | heap 107.999 us | heap 105.543 us; unresolved x |
| u-full | x 3.435 us; unresolved r | r 2.244 us; unresolved x |
| w-tiny | wheap 0.559 us; unresolved wsort | wheap 0.580 us; unresolved wsort |
| w-sparse | wheap 86.313 us | wheap 88.094 us |
| w-half | wsort 220.914 us | wsort 244.970 us |
| w-skew | wheap 1356.662 us | wheap 1375.937 us |
| w-ascending | wheap 101.803 us | wheap 106.350 us |
| w-full | wheap 247.877 us; unresolved wsort | wheap 281.268 us; unresolved wsort |

A unique winner requires all competitor/fastest lower simultaneous95% bounds >1.05. Other entries are point estimates with unresolved rivals, not equivalence claims. Full method geometric means, median/min/max and all paired ratio intervals are in summary.json. 84-contrast family,12 blocks,11df,t=4.754604. Conditional on one fixed generic build per host and observed conditions; no5%noise calibration claim. No A/A campaign or cross-build generalization.
