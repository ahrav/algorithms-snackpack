# Correctness strategy and invariant audit

The Rust unit groups cover n=1,2,4,8,64 and 64 ordered update rounds each, all
three candidates, all leaf proofs, duplicate writes, no-ops and rejected batches.
The separate recursive oracle uses a different tree traversal and encoding code;
it shares only SHA-256. The Python oracle uses hashlib and its own recursive
shape and workload generator. It independently checks every measured final root.

| Invariant | Discriminating regression |
|---|---|
| All indices admitted before mutation | Valid first write plus invalid second leaves old root intact |
| Last write wins | Repeated index with differing final value matches recursive oracle |
| Child order follows index | Proof presented at wrong position is rejected |
| Proof belongs to bytes and root | Changed bytes, old root and tampered sibling reject |
| Exact tree shape | Extra/short proofs and empty/non-power-of-two inputs reject |
| Dirty ancestors recompute bottom-up once | Two adjacent updates report four hash calls |
| Equality shortcut preserves root | Repeated identical write reports zero calls |
| Hash padding and length are correct | Independent hashlib checks across 55/56/63/64/65 and multi-block inputs |

Inputs are bounded evidence. The implementation has no concurrent API and no
persistent-storage contract. Portable SHA-256 is educational code with independent
vector agreement; that evidence is separate from security certification.
