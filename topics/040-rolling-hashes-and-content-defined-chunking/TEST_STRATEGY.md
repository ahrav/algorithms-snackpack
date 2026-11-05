# Test strategy

Scope: complete-slice chunk boundaries and exact byte matching.
Contract and failure classes: exclusive ends, coverage, min/max, reset, rejected
configuration, overlap semantics and collision rejection.
Primary technique: independent Python per-boundary polynomial and closed-form Gear
oracle for every candidate on both hosts, including the 17-byte edited input.
Secondary techniques: Rust parameter table, differential direct/rolling agreement,
Rust closed-form Gear reference, fixed-interval enumeration, exact slice search
oracle, constructed collision, runner behavior against stub tools.
Oracle: Python explicit weighted sums (`measurements/run.py`) and the Rust
`gear_reference` in `tests/contracts.rs`, both written from the recurrence
definitions without calling `boundaries`; byte-slice enumeration for search.
Input/model/schedule domain and bounds: four windows, 12 lengths, 3 shapes in Rust;
12 native cells per candidate plus identical-artifact A/A. Search n<80, m<20.
Replay assets: measurements/run.py, measurements/test_run.py, native receipts and
external archives.
What a green run establishes: agreement within the declared finite input set.
Known gaps and delegated skills: fragmentation, production Rabin/FastCDC, fuzzing,
cryptographic identities and storage-inclusive costs remain revisit work.
Concrete tests to add: persistent streaming state parity when that API is introduced.

## Invariant audit

Claim and scope: direct/rolling end-vector agreement and bounded chunks.
Plausible violation: wrong outgoing coefficient, wrong reset, missing final tail.
Observation: exact ordered vectors plus all interval bounds and final exclusive end.
Oracle and independence: Rust direct independent update logic; Python explicit
weighted sums independently check both, original and edited. Comparator: exact vectors.
Smallest discriminating cases: random n=4096 with windows 8, 32 and 64 (thousands of
eligible positions, so a wrong outgoing coefficient or carried state changes some
cut); n=65 with window 64 reaches the first removal step; n=1025 and n=0 check the
final tail. Zero input keeps the polynomial hash at zero and checks only tail
emission and the forced cut at max.
Mutation backstop: removing the reset at a cut fails this test (observed); mutation
execution for the remaining cases is unavailable and is not reported as passing.

Claim and scope: Gear and normalized Gear agree with a closed-form reference and
reset at cuts.
Plausible violation: carried state across a cut, wrong table, wrong mask region.
Observation: exact end vectors for min=2, target=64, max 64 and 256, 11 lengths,
3 shapes.
Oracle and independence: Rust `gear_reference` sums `GEAR[b] << j` over each
chunk's last 64 bytes from the chunk start; it shares only `gear_table` with the
implementation. Comparator: exact vectors.
Smallest discriminating case: a forced cut at max=64 leaves nonzero state; with
min=2 the 6-bit mask reads bits 2.. of that state at the next eligible position.
A minimum of 8 or more hides carried state above every mask bit, which is why
the campaign configurations (min=64) cannot observe the reset.
Mutation backstop: removing `h = 0` at the cut fails this test (observed).

Claim and scope: fixed policy cuts at every target multiple and emits the tail.
Plausible violation: off-by-one cut length, dropped tail, cut on empty input.
Observation: exact end vectors for 8 lengths including 0, 1, 255, 256, 257.
Oracle and independence: arithmetic range enumeration. Comparator: exact vectors.
Smallest discriminating case: n=257 (one full chunk plus a 1-byte tail).
Mutation backstop: cutting at target+1 fails this test (observed).

Claim and scope: `Config::new` rejects each invalid setting independently.
Plausible violation: a guard missing while the other guards hold.
Observation: `None` for min > target, window == 0, target > usize::MAX/2 with all
other guards satisfied; `Some` for target == 2^(usize::BITS-2).
Mutation backstop: removing the `usize::MAX / 2` guard fails this test (observed).

Claim and scope: exact search, overlaps and empty pattern.
Plausible violation: return hash hits as matches or skip overlaps.
Observation: exact offset list against slice enumeration; a^5/a^3 gives [0,1,2].
Oracle and independence: byte slices without hashing. Comparator: ordered offsets.
Smallest discriminating case: constructed unequal 9-byte polynomial collision,
padded to 65 bytes, rejected by full byte verification; the same colliding window
at offset 3 inside a 23-byte text is rejected while the true match at offset 14
is reported, which exercises the rolling update across the collision.
Mutation backstop: accepting hash hits alone fails the collision assertion.
