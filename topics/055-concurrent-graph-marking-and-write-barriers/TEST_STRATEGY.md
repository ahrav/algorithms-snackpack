# Test strategy

The graph oracle computes transitive closure from the edge matrix. It shares
neither colors nor work queues with the collector. Capture the initial
reachable set once and calculate the current one for each reached state.
When a candidate would terminate, compare both sets with scanned nodes.

Exercise a disappearing last edge, cycles and disconnected components.
Compare the paused exact baseline, the two barrier contracts, an unguarded
mutation baseline and a termination rule that checks only shared work.
The moved-pointer witness discriminates the unguarded mutation rule. The
shared-only negative control isolates work accounting from pointer barriers.
Assert that every safe candidate actually reaches a termination point.
