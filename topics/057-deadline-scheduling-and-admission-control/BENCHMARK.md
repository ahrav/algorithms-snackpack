# Benchmark decision

`benchmark: false`

The decision is feasibility under a stated ideal processor model. Simulator
wall time cannot establish that property. Compare EDF and fixed priority by
missed deadlines, count required processor ticks in a window, and compare
server policies by the ticks at which they service identical work. The
independent checks and recorded workloads are in [test strategy](TEST_STRATEGY.md)
and [measurements](measurements/README.md).
