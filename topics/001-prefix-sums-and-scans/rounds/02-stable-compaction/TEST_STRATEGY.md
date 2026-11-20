# Test strategy

The backward-walk oracle uses neither destination arithmetic nor tile boundaries.
Each candidate must return the full oracle sequence, the selected count, and
preserve the sentinel suffix. Every mask through length ten exercises 2,047
inputs at four tile sizes. Separate assertions recalculate each retained rank
and tile offset by counting its preceding flags.

An inclusive rank used as a destination writes one position late. A tile whose
cursor starts at zero overwrites its predecessors. Reversing the copy order
breaks stability. The examples and exhaustive cases distinguish these failures.
The partial tail and empty middle tiles test zero-count carry and the final
partial tile. Repeated values test multiplicity. Rejected dimensions must leave
the output intact. The benchmark independently checks the entire output and
untouched suffix for each process.
