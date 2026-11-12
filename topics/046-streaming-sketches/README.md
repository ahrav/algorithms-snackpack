# Streaming sketches

Frequency, distinct count and quantile rank require separate contracts. Compare
Count-Min with a standard hash map, HLL with a hash set, and an eager GK-style
rank summary with copying and sorting. All candidates accept at most one million
unit insertions of u64 observations. Frequency queries are supplied separately;
quantiles use 101 ranks, r = 1 + floor((N-1)q/100). Empty quantiles return no values.

Count-Min uses 5 rows of 256 u64 counters, giving 10,240 retained counter bytes.
Insertion-only estimates are upper bounds. Overflow is excluded by the batch
limit. HLL uses 1,024 byte registers and small-cardinality linear counting.
Duplicate insertions preserve its state. Counter addition and register maxima
merge two equal-seed halves; internal incompatible-seed merges fail unchanged.
The rank summary preserves intervals within floor(N/50) ranks, including duplicate
rank intervals and exact endpoints. It uses eager adjacent compression; the
original GK banded-compression space theorem requires its original algorithm.

## Costs and selection

For N observations, D distinct keys and Q frequency queries: expected hash-map
work O(N+Q), retained logical entries 16D bytes; Count-Min O(5N+5Q), 10,240 bytes;
expected hash-set work O(N), 8D bytes; HLL O(N+1024), 1,024 bytes. Copy/sort costs
O(N log N), retaining 8N bytes. With s rank entries, eager vector maintenance
costs up to O(Ns), queries O(s), and retained logical bytes 24s on the tested hosts.
Container capacity, allocator overhead, temporary halves and report buffers are
outside these logical byte counts. The timing includes temporary halves and
report construction. Actual peak memory remains unmeasured.

For [4,7,4,9,4]: frequency(4)=3, distinct=3, median rank=3 and value=4.
Count-Min performs 25 increments. Exact sorted storage is 40 bytes.

```text
Before [4,7,4,9]   Observe 4              After [4,7,4,9,4]
f(4)=2             increment each row     f(4)=3; Count-Min >=3
3 distinct         register max update   HLL state unchanged
[4,4,7,9]          insert/rank compress   [4,4,4,7,9], median=4
```

## Run

```sh
cargo test -p topic-046-streaming-sketches --all-targets
cargo run -p topic-046-streaming-sketches --release --example sketch-lab -- count-min 16384 skew 7 1
cd topics/046-streaming-sketches
python3 run.py
python3 test_run.py
```

The replay needs rustc, Python 3 with assertions enabled and taskset on Linux. It
starts from any working directory and builds inside the topic directory. It creates
raw/, binaries, assembly, a hash manifest and results.json. Each attempt writes one
raw record with a status; a nonzero exit, a timeout (TOPIC_046_PROCESS_TIMEOUT_S,
default 120) or rejected output stops the campaign after its record is written.
The runner exits at startup while raw/, results.json or manifest.json exist: archive
them outside the repository, remove them, then start again. test_run.py checks this
behavior with stub processes. Read BENCHMARK.md before interpreting
results. Python independently reconstructs counts, registers, cardinality estimates
and quantile rank intervals. Statistical error coverage requires separate evidence.

## Guarantees and revisits

Under independent ideal hashes, Count-Min sizing w≈e/epsilon, d≈ln(1/delta)
controls additive frequency error. At w256/d5 this suggests epsilon≈0.0106 and
per-query delta≈0.00674. Row r hashes key x to the bucket row 0 gives key x+r:
the row number is added to the seed and the mixer adds the seed to the key. The
five rows are shifted copies of one SplitMix64 function, so the ideal-hash bound
is an empirical assumption here; theoretical probability coverage is therefore
conditional. HLL's
ideal-hash typical relative standard error is 1.04/sqrt(1024)=3.25%, rather than
a per-stream bound. Rank tolerance epsilon=.02 gives 327 positions at N16,384;
value error can be large across gaps in the distribution.

Choose exact state when its resource cost fits and exact answers matter. Choose
a sketch after accepting its specific error model and measuring the tradeoff.
Small bounded universes justify an exact dense-counter/bitmap comparison.
Explicit revisits: KLL randomized compaction, relative-error quantiles (REQ),
t-digest, weighted updates, heavy-hitter discovery, deletion/sliding windows,
adversarial keyed hashing, serialization and distributed compatibility. The
catalog currently supplies no coverage array or ALGORITHM_INDEX.md.

## Primary sources

- [Cormode and Muthukrishnan: Count-Min](https://sites.google.com/site/countminsketch/home)
- [Flajolet et al.: HyperLogLog](https://algo.inria.fr/flajolet/Publications/FlFuGaMe07.pdf)
- [Greenwald and Khanna: quantile summaries](https://cs.emory.edu/~cheung/Courses/584/Syllabus/papers/Quantiles/2001-Greenwald-Quantile-paper.pdf)
- [Karnin, Lang and Liberty: KLL](https://arxiv.org/abs/1603.05346)
- [Cormode et al.: relative-error quantiles](https://arxiv.org/abs/2004.01668)
