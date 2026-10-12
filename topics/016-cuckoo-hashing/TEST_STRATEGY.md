# Correctness strategy

Independent authority: BTreeMap keyed by exact u64, without table/hash/relocation
helpers. Each successful update/removal must agree with old values and sorted
entries. Each rejected insert must return the incoming key/value and preserve
all prior entries. Bounded tests do not constitute proof over all possible keys.

Six library groups: mixed/colliding traces across8/32/128 slots (18 traces of2,000
steps); actual relocation and excess-demand with global slack; duplicate update
at a full pair/deletion/reuse; frozen feasible-but-small-budget rejection witness;
zero/max keys and8-slot query bound; full linear deletion and reinsertion.
Two external groups:729 six-operation three-key traces across all three candidates,
and lookup/update of a second-bank key after its first-bank position becomes empty.

Invariant discrimination:
- Returning Err after destructive changes is rejected by snapshot/model comparison.
- Discarding a pending resident is rejected by preserved-key/value comparison.
- Terminating lookup on an empty first bucket misses the second-bank witness.
- Treating duplicate insertion as new capacity demand fails the full-pair update.
- Equating a finite budget with infeasibility fails the two-budget witness.
- Rejecting all insertions fails the concrete four-key relocation example.
- Missing backward cyclic eligibility fails full linear deletion/model traces.

Every measured process runs a bounded differential trace and independently
validates the entire prepared map and expected checksum before warmup. Timed
batch checksum must agree exactly. The standalone runner runs both library and
external contracts before the campaign and retains their output. Admission
rejections are preserved and excluded from throughput selection.

Allocation failure, panics/crash consistency, generic destructors, reference
stability and concurrency are outside this fixed integer single-owner contract.
