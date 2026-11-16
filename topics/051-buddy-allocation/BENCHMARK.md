benchmark_required: true

Claim: select the fastest correct reservation implementation among scan, buddy
ordered sets and largest-block tree for each fixed trace and host.

Workloads: U=64,4096; tiny (eight fill-half/free rounds), mixed requests
1,3,5,9,17,33 (eight rounds), alternating occupied/free holes with failed
size-two probes, churn with large/small split/merge requests (eight rounds).
Logical arenas start empty. Each timed interval is the first and only replay in
a fresh process, after that process generated its trace and constructed its pool.
The warmup process of a cell warms host state only (binary page cache, CPU
frequency). Allocator growth, first-touch pages and first execution of the
operation code are inside every timed interval. Steady-state repetition inside
one process is outside this experiment. No cache-flush policy.

Boundary: timed operation loop includes all metadata mutations, ID bookkeeping,
result-vector writes and dynamic ordered-set allocation/deallocation.
Trace generation, initial construction, independent oracle, output and destruction
are excluded. Construction is separately recorded. Results are consumed.

One warmup per candidate/cell; six fresh-process blocks using all six candidate
permutations. No adaptive rerun or sample deletion. One complete block is the
analysis unit; operations are subsamples. Candidate assignment is deterministic
counterbalancing, not randomization. Fixed build per host, CPU0, native target.
Shared host interference persists across processes; generalization is restricted
to these builds/hosts/traces. Different toolchains preclude hardware attribution.

Criterion: a candidate wins only when at least 5% faster than both competitors
in all six paired blocks. Otherwise unresolved. Report median and full range.
Exploratory descriptive screen; no confidence level, power or universal optimum
claim. The A/A control is 12 processes of one candidate (scan, 4096, mixed)
after all treatment cells: six pairs whose A and B labels, recorded in `phase`,
alternate order. The arms share the build and the runner's launch, parse and
acceptance path with treatment processes. They do not pass through the
three-candidate permutation order, the other cells or a `kind` label, so the
control screens order asymmetry between two consecutive identical processes.
It establishes no statistical calibration or noise floor, and its ratios were
computed outside the repository. Retain every attempt.

Each process appends one line to the results file with a status: ok, failed
(nonzero exit), timeout (over 60 seconds; TOPIC_051_PROCESS_TIMEOUT_S replaces
the limit) or rejected (unparsable or wrong output). A line that is not ok keeps
the command, stdout and stderr, and the campaign stops there; later cells stay
unresolved. The runner requires enabled assertions and a new results path.
It hashes the binary before the first launch and before every later launch; a
changed or unreadable binary stops the campaign before that launch.
The runner holds its own definition of the four traces. A check output must be
those operations in order, one state line and a summary with that operation
count. Complete answers and final state are verified in the 24 check processes. A
warmup, measured or A/A process must echo its kind, n and shape and repeat the
digest, operation count and work of its check process. The digest folds result
magnitudes: a failed allocation (-1), address 1 and a successful free add the
same term.

Logical probes, split checks and summary repairs describe algorithms. No hardware
cause is inferred. External raw bundles hold outputs, identities and manifests.
