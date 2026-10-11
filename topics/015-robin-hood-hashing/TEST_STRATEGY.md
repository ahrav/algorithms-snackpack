# Correctness strategy and invariant audit

Scope: new/get/get_counted/insert/remove/len/is_empty/layout/valid under all three
policies and mixed/identity/constant hash modes. Single-threaded fixed capacity.

Oracle: independent std BTreeMap, sharing no hash/probe/displacement logic.
Primary method: seeded LCG history,4000 transitions per policy/hash/capacity,
capacities2/8/64,97-key domain. Compare mutation results/count and every stored
value after every transition. Benchmark processes compare every query against
BTreeMap; churn also verifies the mutation sequence and final contents. Pair
checksums must agree. Literal expectations backstop topology witnesses.

Invariant-discriminating witnesses:
- keys0,8,2 at capacity8 give DIB0,1,0: rejects globally sorted-DIB assumption;
- keys6,7,14 cross wrap: swap and erase preserve every resident value;
- linear keys0,1,8 then erase0: an at-home resident cannot end deletion scanning;
- full keys0,1,2,3,8,9,10,11 then insert16: rejects mutation before Full;
- full-table update8 returns old value, changes only that value;
- EMPTY/absent/extreme-key/reinsertion and Robin early-exit reachability are
external contracts in tests/contracts.rs.

Green means agreement on bounded tested histories and exact witnesses. `valid`
is an internal diagnostic and not an independent oracle. Tests are not a proof
for all histories. No unsafe/concurrency tools are warranted. Known gaps: exhaustive
operation schedules, arbitrary payload ownership/destructors, compact-DIB overflow,
allocation failure injection, invalid-capacity panic and production resizing.

Applied scratch mutations: stopping at equal DIB fails the0,8,2 test; ending ordinary
linear deletion at DIB0 fails the0,1,8 test; reversing swap priority fails the wrap
example. Each compiled with rustc edition2024 and failed its named exact test.
Mutated sources, exit status and output are retained externally under
evidence/topic-015/20261011/mutation-backstops. Repository source was never mutated.
