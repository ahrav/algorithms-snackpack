# Benchmark decision: true

Claim: select the fastest correct reservation implementation among scan, buddy
ordered sets and largest-block tree for each fixed trace and host.

Workloads: U=64,4096; tiny (eight fill-half/free rounds), mixed requests
1,3,5,9,17,33 (eight rounds), alternating occupied/free holes with failed
size-two probes, churn with large/small split/merge requests (eight rounds).
Logical arenas start empty; CPUs/caches are warm. No cache-flush policy.

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
claim. Six alternating-label identical-binary A/A pairs check asymmetry but do
not establish statistical calibration or a noise floor. Retain every attempt.

Logical probes, split checks and summary repairs describe algorithms. No hardware
cause is inferred. External raw bundles hold outputs, identities and manifests.
