# Test strategy

Exhaust all 512 directed graphs on three tasks, including self-waits. The
oracle enumerates possible completions by checking prerequisite membership.
Replay the candidate's ordering and verify every edge of its cycle witness.
Use a duplicate-edge trace to exercise edge multiplicity.

Enumerate capacities 0 through 2 independently over two resource classes.
For two tasks, enumerate every allocation and request consistent with those
capacities. The oracle tries each allowed grant and explicitly returns both
new and held units. Check the stalled set, each trace, the final free vector,
and three single-victim policies for each snapshot. Compare each policy with
simulation of every possible victim, including zero-allocation waiters.

Test the alternate holder, blocked outsider, two disjoint cycles, empty input,
mismatched dimensions, over-allocation and counts at the integer limit.
A premature dependency decrement can emit a task before a required task.
A skipped allocation release can strand tasks that could finish. Selecting by
cost alone can choose a zero-allocation waiter and preserve the deadlock.
