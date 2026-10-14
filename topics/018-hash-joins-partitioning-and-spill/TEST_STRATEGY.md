# Correctness strategy and invariant audit

14, 641 ordered pairs of all 121 vectors of length 0..4 over 3 keys compare every
resident candidate and budgets 1/2/5 to independent nested loops. This kills lost
duplicate multiplicity, incorrect routing, run cursor errors and chunk double count.
Extreme 0/MAX keys and million-pair growth cover key/count boundaries. Each timed
process checks its exact dataset against an independent BTreeMap frequency oracle.

File-backed tests cover recursive overflow with 257/131 rows, all-equal 31/29 with
no-progress fallback (899 pairs, one partition), max_build_rows<=budget, empty inputs,
zero budget and truncated records. Demo asserts all six results equal 7. These
checks kill missing recursion progress, lost partition pairs and malformed decoding.
They are bounded correctness evidence, not a proof, whole-memory bound, durability
check, cancellation/fault-injection test or production SQL conformance suite.
