# Benchmark decision

benchmark: true

Compare correct placement policies across N=4/32/128 owners and K=32/32768
unique uniformly hashed keys, plus N=32,K=4096 targeted keys that all prefer
owner 7 under ring32. Stable IDs are 17i+7. Hash, ordered keys and runner are
frozen. One deterministic map per cell limits generalization across hash seeds.

Each process builds and validates a map outside batch timing. The timed batch
allocates output, assigns every item and drops output. Bounded also allocates,
resets and drops shared owner counts. Build timing includes canonical owner
sorting, token generation, token sorting, allocation and teardown. Bounded
build is byte-identical to ring32 build, so the build contest uses four policies.
Input generation, oracle, process startup and JSON output are outside timing.
One warmup process per candidate/cell precedes six fresh process blocks.
Even blocks run forward candidate order, odd blocks reverse it. Inner repeats
are 512 for tiny/build and four otherwise; the process is the sample unit.
Stop after six blocks. Report median and full min/max range. A winner must
beat each other eligible candidate by >5% in every paired block; otherwise
report unresolved. Compare all stateless policies, the three stable named-owner
policies separately, and the sole implemented count-cap policy as a feasible
option. A singleton group supplies no comparative speed evidence.

The runner pins to the first allowed CPU. Native target flags, toolchain,
CPU/kernel, available CPUs, assembly, binary/source hashes and process schedule
are retained. Six repeated ring32 A/A blocks measure timing variation. Runs
use warm inputs and a shared host. Cold cache, RSS, production hashing, replicas,
byte-rate enforcement and actual migrations are outside the measured scope.

Every record's input and all old/add/remove/replace assignment vectors are
checked in Python. Small Rust tests use a linear clockwise-distance oracle.
Correctness checks run outside timing. Reference token visits and KN score
counts describe model work; whole-process hardware counters would include
correctness/setup and could not isolate a candidate mechanism.

The hot-key calculation assigns 9K work units to the first item and one to
each remaining item. It measures assignment-weight arithmetic, rather than
request execution. Report its maximum work share separately from item counts.
