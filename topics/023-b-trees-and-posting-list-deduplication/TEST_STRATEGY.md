# Correctness and invariant audit

Independent oracle scans arbitrary original pairs, filters range, deduplicates
with a HashSet and sorts output. It shares no candidate grouping, lower-bound,
maximum fence or tree navigation helper. Benchmark compares every query and
complete checksum before accepting timings. Tests are bounded evidence.

All5461inputs length0..6 over4possible pairs, all16bounds combinations, five
candidates. 46656six-operation posting histories compare every state and
insert/remove boolean with an unsorted-vector authority. Distinct insert and
remove opcodes target the same three pairs. Other tests cover fanout-adjacent
sizes through1025keys, multi-level directories, integer extremes, absent and
empty/reversed ranges, duplicates, running example and last-row deletion.

Invariant: expanding postings reconstructs exactly the unique ordered pair
set. Tests reject dropping repeated-key rows, duplicate IDs and empty groups.
Packed.valid checks strict keys/rows, contiguous payload boundaries and every
level's child index/max-key fence. Corrupted fence and lost-row witnesses are
rejected even before query comparison. Test regressions must also reject
< versus <= boundary mistakes and deletion of a whole key for one row.
Standard-library B-tree balancing is delegated to Rust, not independently
proved here. No exhaustive arbitrary-u64, allocation-failure, concurrency or
durable-recovery claim.

Four compiled mutants are rejected by existing tests: fence equality routed to
the wrong child, leaf equality skipped, repeated-key row loss, and whole-key
deletion on any row removal. External raw mutant outputs are retained.

The campaign runner truncates attempts.jsonl once per campaign, so a rerun into
an existing output directory retains exactly the current schedule's attempts
(measurements/test_runner.py runs the campaign twice and compares).
