benchmark_required: true

Predeclared fixed campaign: compare three separate exact/approximate contract
pairs, never frequency versus cardinality versus quantiles. Length/shape cells:
128 uniform; 4096 uniform; 16384 uniform, 90%-zero skew, sorted and reverse unique.
All values are deterministic; input seed123, sketch seed7. Frequency queries cover
keys0..4095. Rank queries cover101 percentiles. Sketches must satisfy independent
mechanical contracts before timings enter selection. Approximate semantic equality
means the specified query contract and admitted error, with exact baselines as
reference. Accuracy is reported separately, including16 HLL seeds per cell.

Boundary: fresh batch allocation, all updates, queries, compatible two-half merge
for Count-Min/HLL, state/report construction, and internal structure destruction.
Exclude input/query generation, process startup, compilation, JSON output and
returned report destruction. Inner repeats32 for N128,4 otherwise are timer
subsamples; the fresh process is the analysis unit. Each cell has one warmup per
candidate and six paired blocks. Rotate contract-pair order; reverse adjacent
candidate order in odd blocks, yielding3:3 order balance. This fixed deterministic
schedule controls order; it is not a randomized population experiment.

Select a candidate only when all six paired timing ratios exceed a5% advantage.
Otherwise unresolved. Report median and observed min/max; these are descriptive
ranges, with no nominal confidence interval or multiple-comparison significance
claim. No pilot-driven allocation, repeated looks, optional stopping or winner
reruns. This is a finite synthetic screen, not a population estimate. All failures
and partial processes are retained. Correctness failure stops the campaign;
missing pairs cannot produce a selected winner.

A/A: six fresh paired exact-frequency processes at N16384 uniform, distinct A/B
labels, alternating order, same path/controls/parser. Six more HLL seeds per cell
would change the experiment; retain the declared16 rather than rerunning outliers.
Single minimum available CPU affinity; native optimized rustc; warm in-process
repeats. Input is warmed by generation. Cold-cache and service effects require
another experiment. Capture environment, source/runner/binary hashes, compiler,
features, CPUs, raw samples and assembly. Inspect generated code for actual calls;
whole-process counters would also contain correctness checks and are insufficient
to attribute a candidate-only mechanism. Logical storage is measured from retained
entries, not process RSS, peak allocation or container capacity.
