# Exact-source results

All five inputs match committed source a38ff79. Every process oracle, checksum, schedule, linked binary hash and native contract passed. Ranges are observed min/max over eight fresh processes, not confidence intervals.

## arm

Screened wins: {'unresolved': 21, 'tw': 12, 'direct': 7}. A/A paired A/B ratio range: [0.98174912939999, 1.0007586842247873].

| n | m | shape | endpoint | screened winner | smallest-median ns/query [min,max] |
|---:|---:|---|---|---|---|
|64|4|wide|prepared|unresolved|adaptive: 59.2 [58.9,60.6]|
|64|4|wide|oneshot|unresolved|tw: 111.3 [110.8,167.1]|
|64|4|small|prepared|unresolved|adaptive: 88.8 [88.5,89.0]|
|64|4|small|oneshot|unresolved|tw: 113.6 [113.1,115.4]|
|64|4|prefix|prepared|tw|tw: 89.3 [88.7,90.4]|
|64|4|prefix|oneshot|tw|tw: 107.1 [106.7,108.5]|
|64|4|suffix|prepared|tw|tw: 73.4 [73.3,75.0]|
|64|4|suffix|oneshot|tw|tw: 92.4 [92.3,93.9]|
|64|4|early|prepared|unresolved|horspool: 6.3 [6.3,6.4]|
|64|4|early|oneshot|direct|direct: 10.2 [10.2,10.3]|
|64|32|wide|prepared|unresolved|adaptive: 30.1 [30.1,31.7]|
|64|32|wide|oneshot|direct|direct: 80.1 [79.9,82.9]|
|64|32|small|prepared|unresolved|adaptive: 42.0 [41.9,44.0]|
|64|32|small|oneshot|direct|direct: 91.5 [90.1,93.7]|
|64|32|prefix|prepared|tw|tw: 53.3 [47.9,54.9]|
|64|32|prefix|oneshot|unresolved|adaptive: 227.8 [226.1,228.5]|
|64|32|suffix|prepared|unresolved|tw: 56.6 [56.4,57.9]|
|64|32|suffix|oneshot|direct|direct: 65.1 [64.2,67.8]|
|64|32|early|prepared|unresolved|direct: 27.8 [27.7,30.1]|
|64|32|early|oneshot|direct|direct: 32.4 [32.2,32.8]|
|32768|4|wide|prepared|unresolved|adaptive: 35013.3 [34880.9,35725.0]|
|32768|4|wide|oneshot|unresolved|adaptive: 35031.8 [34867.1,36004.5]|
|32768|4|small|prepared|unresolved|tw: 42735.9 [41820.4,43642.9]|
|32768|4|small|oneshot|unresolved|adaptive: 42688.3 [42437.1,42997.7]|
|32768|4|prefix|prepared|tw|tw: 43041.1 [42748.9,44437.1]|
|32768|4|prefix|oneshot|tw|tw: 43533.6 [42500.6,45225.0]|
|32768|4|suffix|prepared|tw|tw: 35645.6 [35466.0,36351.1]|
|32768|4|suffix|oneshot|tw|tw: 35896.6 [35603.9,37056.2]|
|32768|4|early|prepared|unresolved|horspool: 6.3 [6.3,6.5]|
|32768|4|early|oneshot|direct|direct: 10.3 [10.2,10.4]|
|32768|32|wide|prepared|unresolved|adaptive: 4675.0 [4655.9,4736.2]|
|32768|32|wide|oneshot|unresolved|adaptive: 4791.0 [4753.9,4805.2]|
|32768|32|small|prepared|unresolved|adaptive: 29928.8 [29794.7,30715.5]|
|32768|32|small|oneshot|unresolved|adaptive: 30049.2 [29864.6,30118.5]|
|32768|32|prefix|prepared|tw|tw: 43392.8 [42631.5,44657.9]|
|32768|32|prefix|oneshot|tw|tw: 43454.8 [42983.2,44400.3]|
|32768|32|suffix|prepared|unresolved|tw: 26511.5 [26491.6,37190.5]|
|32768|32|suffix|oneshot|tw|tw: 26663.7 [26636.1,27420.6]|
|32768|32|early|prepared|unresolved|direct: 27.8 [27.8,28.0]|
|32768|32|early|oneshot|direct|direct: 32.5 [32.2,34.7]|
## x86

Screened wins: {'horspool': 1, 'direct': 6, 'unresolved': 19, 'tw': 14}. A/A paired A/B ratio range: [0.7636982804168202, 1.2426936606371204].

| n | m | shape | endpoint | screened winner | smallest-median ns/query [min,max] |
|---:|---:|---|---|---|---|
|64|4|wide|prepared|horspool|horspool: 33.8 [33.6,35.1]|
|64|4|wide|oneshot|direct|direct: 67.9 [67.8,69.3]|
|64|4|small|prepared|unresolved|horspool: 52.9 [52.2,60.9]|
|64|4|small|oneshot|tw|tw: 83.3 [75.4,95.0]|
|64|4|prefix|prepared|tw|tw: 71.3 [62.0,80.9]|
|64|4|prefix|oneshot|tw|tw: 76.0 [74.3,90.4]|
|64|4|suffix|prepared|unresolved|tw: 49.7 [49.3,88.4]|
|64|4|suffix|oneshot|unresolved|tw: 60.7 [60.2,70.4]|
|64|4|early|prepared|unresolved|horspool: 6.5 [6.4,10.0]|
|64|4|early|oneshot|direct|direct: 10.1 [9.9,15.1]|
|64|32|wide|prepared|unresolved|horspool: 24.1 [19.7,26.6]|
|64|32|wide|oneshot|unresolved|direct: 65.3 [65.3,72.1]|
|64|32|small|prepared|unresolved|horspool: 23.6 [23.6,33.3]|
|64|32|small|oneshot|unresolved|horspool: 71.5 [66.7,108.0]|
|64|32|prefix|prepared|tw|tw: 45.8 [36.3,46.0]|
|64|32|prefix|oneshot|tw|tw: 118.6 [114.8,142.4]|
|64|32|suffix|prepared|unresolved|direct: 35.7 [34.7,44.4]|
|64|32|suffix|oneshot|direct|direct: 38.5 [37.0,39.5]|
|64|32|early|prepared|unresolved|horspool: 27.2 [19.3,34.6]|
|64|32|early|oneshot|direct|direct: 27.4 [27.2,45.0]|
|32768|4|wide|prepared|unresolved|horspool: 31930.5 [31703.3,34445.9]|
|32768|4|wide|oneshot|unresolved|horspool: 32024.0 [31804.2,32748.2]|
|32768|4|small|prepared|unresolved|tw: 30787.0 [30779.3,60263.7]|
|32768|4|small|oneshot|tw|tw: 30829.0 [30810.4,31752.7]|
|32768|4|prefix|prepared|tw|tw: 30784.9 [30770.9,35839.6]|
|32768|4|prefix|oneshot|tw|tw: 30823.7 [30814.9,30833.5]|
|32768|4|suffix|prepared|tw|tw: 23011.2 [22053.1,25643.8]|
|32768|4|suffix|oneshot|tw|tw: 23109.7 [22019.9,39133.9]|
|32768|4|early|prepared|unresolved|horspool: 6.5 [6.4,6.7]|
|32768|4|early|oneshot|direct|direct: 10.0 [9.7,10.2]|
|32768|32|wide|prepared|unresolved|horspool: 4260.3 [4254.6,4278.4]|
|32768|32|wide|oneshot|unresolved|horspool: 4342.8 [4321.5,4508.9]|
|32768|32|small|prepared|unresolved|horspool: 27103.9 [26919.9,27941.8]|
|32768|32|small|oneshot|unresolved|horspool: 27265.7 [27180.4,27362.8]|
|32768|32|prefix|prepared|tw|tw: 30759.5 [30746.4,62216.1]|
|32768|32|prefix|oneshot|tw|tw: 30858.5 [30840.9,31528.8]|
|32768|32|suffix|prepared|tw|tw: 16917.8 [16909.0,17626.6]|
|32768|32|suffix|oneshot|tw|tw: 16992.5 [16988.8,18005.2]|
|32768|32|early|prepared|unresolved|horspool: 19.7 [18.7,22.3]|
|32768|32|early|oneshot|direct|direct: 27.2 [27.1,27.4]|

## Evidence boundaries

Frozen executable source: a38ff79. Both hosts ran1816processes:1600measured,
200warmup,16A/A. All three native contract groups pass. Arm rustc1.98.1 and
x86 rustc1.99.0 use opt-level3 and target-cpu=native, pinned to CPU0. Toolchains
differ, so cross-host differences include compiler effects. Full CPU/kernel and
target-feature details accompany the external receipts. Timing is warm,
resident, synthetic, and single-core. Ranges are descriptive. The x86 A/A range
is wide; retain unresolved comparisons. No cache, branch or SIMD cause is claimed.

A first campaign failed an input-shape assertion: the four-byte background
contained an earlier match at147. Both failed campaigns are retained externally;
the corrected fixture uses a terminal255 sentinel outside its background alphabet.
Corrected results alone enter selection. The original source remains in history.

At n32768,m32, the repeated-suffix diagnostic counts direct32737,
Horspool1047584, Boyer-Moore32768, Two-Way32768 and adaptive96256 comparisons.
The adaptive candidate falls back once. Counts exclude preparation and explain
algorithmic work; instrumented elapsed time does not enter primary timings.

See BENCHMARK.md for the frozen selection rule, exact commands and exclusions.
All five candidates and all40cells per host are retained in the task delivery's
measurements.json. The table reports the smallest observed median even when the
screening rule leaves selection unresolved.
