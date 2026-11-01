# Topic 036 plugin audit: no change

Inspected `ahrav/ahrav-claude-plugins` origin/main at
`80a196cb99fd50947efe79a66183b9ebd2bfdc9c` on 2026-11-01.
No plugin files changed. No new plugin or evaluator PR is warranted by this audit.

The scoped question was whether the lesson exposes incorrect reusable guidance
about retained Merkle summaries, content identity, cache publication or selection
of an incremental representation. The audit found existing owners for each:

- `systems-design/skills/quorum-replication-mechanics/references/anti-entropy-and-conflicts.md`,
  “Merkle trees localize differences within a defined scope”, requires compatible
  range/encoding/version/deletion definitions, valid summary-update rules and
  separate reading/hashing, summary-comparison and transfer costs. It explicitly
  distinguishes tree height from aggregate repair work and equality from freshness.
- `systems-design/skills/cache-coherence-and-invalidation/SKILL.md` and
  `references/invalidation-contracts.md` require versioned or immutable-key
  publication, authority identity, proof-failure policy and correct fill admission.
  The immutable-key strategy includes publication order, mutable-pointer freshness
  and retention obligations.
- `performance/skills/hashing-prng-and-checksum-speed/SKILL.md` separates table
  hashing, persisted formats, accidental-corruption checksums and cryptographic
  content identities. It requires exact stable bytes and independent vector checks.
- `low-level-systems/skills/systems-data-structures/SKILL.md` requires workload,
  mutation/result semantics, full-operation cost, a simple baseline, exact oracles
  and snapshot/publication constraints before admitting a specialized structure.
- `performance/skills/compile-time-optimization/SKILL.md` and its caching reference
  already own compiler-cache regime selection and rebuild diagnosis. This audit
  makes no change to those implementation-specific compiler claims.

The fixed-shape array, per-level dirty-frontier deduplication and hash-call counts
are lesson-specific executable choices. The audit found no observed skill miss or
contradictory rule requiring those choices to be mandated in a general owner.
Existing coverage is sufficient for the reusable correctness and selection
contracts examined here. This is a source audit, not a model-based quality result.
No skill inputs changed, so no new development evaluation is required.

Primary cross-checks: RFC 9162 section 2.1; NIST FIPS 180-4; Bazel remote caching
(the lesson README links the exact pages). Prior open skill work remains separate:
PR398 is OPEN; evaluator PR21 is MERGED. Topic035 postmerge synchronization remains
pending until the source PR merges. This run leaves historical merged PRs intact.
