# Test strategy and invariant audit

An independent i64 full-sort oracle checks exact results. A sorted-vector
frontier and BTreeSet visited reference checks heap search for sizes 1..29,
three queries, k 1/4/40, entries 0 and N/2, and narrow/exhaustive/oversized
widths. Scored, expanded and maximum-pending counters must match the reference. Models share the public
comparator contract and input, with separate scoring/queue implementations.
Connected width=N must equal exact. Disconnected width=N stays in its component.

Width 1/2 rejects greedy-only replacement. Cycles and duplicate/self links check
score-once behavior. Equal distances check stable IDs. Empty/k>N/disconnection
check fill. Invalid shapes, links, entries, widths and coordinates check admission.
Extreme allowed coordinates check wide accumulation. Python recomputes every
benchmark answer and checks IDs, distances, order, duplicates and checksum.
Finite sampled tests establish bounded evidence rather than a universal proof.
