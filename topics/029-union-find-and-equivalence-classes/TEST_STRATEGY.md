# Correctness strategy and invariant audit

The independent oracle stores a graph and traverses reachability before each
operation. It retains an edge only when components merge; this preserves
connectivity while bounding the retained graph to a forest. Candidate parent
arrays and eager-label replacement share no oracle helper.

- Enumerate all 18^4 = 104,976 histories of four operations on three items.
  Compare every response for all four candidates, including redundant/self merges.
- Compare seeded 300-operation histories for N=1,2,8,31,64.
- Inspect a balanced eight-item path before and after full compression; verify
  class size, membership and duplicate merge response.
- Check empty input and invalid IDs. Invalid IDs panic without a transactional
  preservation guarantee.
- Check nested rollback, duplicate history suppression and a branch after undo.
- Enumerate 4096 three-merge extensions of a rollback snapshot and verify the
  restored equivalence relation on all 16 endpoint pairs.
- Check sizes and enumerated groups across size/full/half implementations.
- Every benchmark process compares its entire response vector to graph search
  outside the measured region before its row can be accepted.

These checks discriminate partial label replacement, wrong root attachment,
lost transitivity, duplicate merge success, stale sizes, incomplete compression,
and failed rollback restoration. They are bounded executable evidence. Concurrency,
weighted constraints, allocation failure recovery and persistence are outside
this implementation's contract.
