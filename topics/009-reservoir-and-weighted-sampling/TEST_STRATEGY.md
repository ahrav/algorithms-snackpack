# Correctness evidence

Eight checks cover independent priority extraction, direct-clock versus log-clock
ordering, rejection of invalid weights even at k0, min(n,k) and sorted unique IDs,
PRNG known answer/endpoints, exact 60 R paths over ten subsets, X survival product,
unequal-shard combination and deterministic ties. Seeded frequency checks over
20,000 seeds are bounded diagnostics only. The exact R-path enumeration tests
the recurrence independently; the implementation is additionally tested via
subset/frequency and seed checks, not formally proved by enumeration alone.

Priority oracle uses repeated minimum extraction and independent monotone IEEE
bit ordering, not the heap's Ord or sort. A second direct-clock oracle checks
transformation on ordinary weights; extreme finite weights test finite log keys
without claiming precise ideal-law ranks. For weights1,1,1,2 and k2, the heavy
inclusion diagnostic targets.7, not proportional.8. Source ordering/seed are fixed.

Invariant discrimination: endpoint U0/U1 breaks logs; invalid weights cannot
silently enter; oversized count clamps; duplicate output fails strict ordering;
wrong heap polarity fails exact oracle; resetting keys at merge fails global
oracle; full/all-zero-count shortcuts still obey their stated validation policy.
No unsafe code or external dependencies. No native random-quality certification,
proof of universal floating exactness, or overlapping-ID merge claim.
