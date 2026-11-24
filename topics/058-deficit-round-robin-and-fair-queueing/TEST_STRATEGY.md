# Test strategy and invariant audit

| Contract | Independent check | Defect the check exposes |
| --- | --- | --- |
| Packet turns and FIFO | Iterate packet positions by active flow | Selecting one flow twice or reordering its packets |
| DRR credit and progress | Recompute cumulative byte ceilings by round, for three quanta | Resetting residual credit or ignoring the next packet's length |
| Weighted service | Advance fluid work by weight in exact twelfths and compare completions | Ordering by packet count, omitting weights, changing ties, or missing the one-maximum-packet lateness bound |
| Byte-cap FIFO and persistent marks | Simulate each wall tick, recompute queued bytes, and scan recent sojourns | Admitting an extra packet, changing arrival/dequeue order, marking one sample too early, or suppressing service after a mark |

The generated tests cover 384 batches and 512 arrival traces. Focused cases
cover the carry-reset mutant, a quantum below the head length, an exact byte
cap plus its predecessor, and marking exactly at the persistence boundary.
The fluid oracle samples weights 1 through 4, which divide its fixed scale.
Exhaustive arbitrary traffic, kernel scheduling, and live sender response are
outside the modeled contract.
