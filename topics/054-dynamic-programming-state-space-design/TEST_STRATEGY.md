# Test strategy

- Compare all five solvers with direct bitmask subset enumeration over 220
  deterministic instances. This oracle sums selected items and has no DP
  recurrence, row reuse, or dominance pruning.
- Detect a forward in-place transition with a single weight-two item and
  room for three copies. Check that two distinct identical items may both
  be taken.
- Check empty input, zero-weight and zero-value items, equal scores with
  different weights, and maximum item fields. The latter also detects
  truncating a total score to 16 bits.
- Reject scores from unreachable value states and check discard of heavier
  states through independent oracles. The runner enumerates the full fixed
  fixture independently, checks identity and score on every process, and
  computes frontier counts separately with a sorted map.
- Run the same committed source on Arm and x86-64. Workspace tests and native
  tests certify this lab's bounded contract. The transition and dominance
  arguments establish correctness over the admitted inputs.
