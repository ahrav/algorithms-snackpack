# Test strategy

The independent oracle executes each task serially and accumulates its count
and wrapping checksum. The candidate checks cover one through six workers,
zero, one, two, seventeen, and 256 tasks, and three leaf-work values. They
exercise empty queues, the final-task state, more workers than tasks, balanced
placement, and forced stealing from worker zero. A zero worker count panics at
the public contract boundary.

The tests establish bounded agreement for the safe locked models. They do not
establish lock-free atomic ordering or retired-array reclamation.
