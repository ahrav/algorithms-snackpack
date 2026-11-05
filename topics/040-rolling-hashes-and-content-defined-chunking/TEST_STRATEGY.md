# Test strategy

Scope: complete-slice chunk boundaries and exact byte matching.
Contract and failure classes: exclusive ends, coverage, min/max, reset, rejected
configuration, overlap semantics and collision rejection.
Primary technique: independent Python per-boundary polynomial and closed-form Gear
oracle for every candidate on bothhosts, including the17-byte edited input.
Secondary techniques: Rust parameter table, differential direct/rolling agreement,
exact slice search oracle, constructed collision.
Input/model/schedule domain and bounds: fourwindows,12lengths,3shapes in Rust;
12native cells per candidate plus identical-artifact A/A. Search n<80,m<20.
Replay assets: measurements/run.py, native receipts and external archives.
What a green run establishes: agreement within the declared finite input set.
Known gaps and delegated skills: fragmentation, production Rabin/FastCDC, fuzzing,
cryptographic identities and storage-inclusive costs remain revisit work.
Concrete tests to add: persistent streaming state parity when that API is introduced.

## Invariant audit

Claim and scope: direct/rolling end-vector agreement and bounded chunks.
Plausible violation: wrong outgoing coefficient, wrong reset, missing final tail.
Observation: exact ordered vectors plus allintervalbounds and finalexclusiveend.
Oracle and independence: Rust direct independent update logic; Python explicit
weighted sums independently check both, original and edited. Comparator: exactvectors.
Smallest discriminating cases: window1,min64; n65; n1025 and repeatedzero.
Mutation backstop: the above cases expose exponent/short-tail errors; mutation
execution is unavailable and is not reported as passing.

Claim and scope: exact search, overlaps and empty pattern.
Plausible violation: return hash hits as matches or skip overlaps.
Observation: exactoffsetlist against slice enumeration; a^5/a^3 gives[0,1,2].
Oracle and independence: byte slices without hashing. Comparator: orderedoffsets.
Smallest discriminating case: constructed unequal9-byte polynomial collision,
padded to65bytes, rejected by fullbyteverification.
Mutation backstop: accepting hash hits alone fails the collision assertion.
