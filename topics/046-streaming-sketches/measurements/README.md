# Measurements

Frozen source: `71f970442a3a9d3a04bc57c067df9d01183c7fde`. The receipt hashes bind
src/lib.rs, main.rs and run.py at this commit. src/lib.rs and main.rs are
byte-identical at HEAD. run.py at HEAD keeps the same schedule, build commands and
binary inputs, and differs from the archived runner in five ways. It exits at
startup when assertions are disabled, and when raw/, results.json or manifest.json
of an earlier campaign exist. It builds inside the topic directory from any working
directory. It writes one record with a status for every attempt, including a
timeout (TOPIC_046_PROCESS_TIMEOUT_S, default 120), a nonzero exit and rejected
output, before halting. Its oracle also requires printed labels equal to the
invocation, a finite positive time, logical bytes equal to the retained entries, and
quantile values and summary entries that occur in the input. The archived runs used
the 71f9704 checks. test_run.py checks the HEAD behavior with stub processes.
Python reconstructed and checked all
360 outputs per host:216 main,36 warmup,96 HLL seed sensitivity,12 A/A processes.
Four contract groups also pass on both hosts. The receipt identifies complete
archives, source/runner/binary/manifest identities and local independent rechecks.

Arm: hostname dev-dsk-ahrav-2b-7dc7bd93.us-west-2.amazon.com, aarch64,
CPU implementer0x41/part0xd40 (Neoverse V1),64 available CPUs, rustc1.98.1.
x86: resolved xxl to dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com, x86_64,
Intel Xeon Platinum8488C,192 available CPUs, rustc1.99.0. Both use Linux6.12.113,
CPU0 affinity, edition2024, opt-level3, target-cpu=native. Exact uname, compiler,
features, CPUinfo and allowed CPUs are retained in each environment.json.
These compiler differences preclude attributing timing differences to hardware.

On both hosts: Count-Min wins6/6 frequency cells; HLL wins5/6 cardinality cells,
with exact set winning N128; copy/sort wins6/6 quantile cells. Each selection means
all six paired contrasts exceed5%, under admitted approximation, at this boundary.
No population interval or universal performance optimum is claimed.

Count-Min's N16384 uniform queries have median excess43 and maximum79 counts.
On unique sorted/reverse data true frequency1 can be overestimated by71.
HLL at N16384 uniform has observed relative error+3.632%..+5.621% across16 seeds.
Those additive seeds yield correlated sets of hash inputs for consecutive keys;
this is a deterministic sensitivity sweep, not independent statistical coverage.
At N16384 uniform the rank summary's maximum queried rank distance is323 of327
allowed; its retained entries occupy912 bytes versus131072 sorted-data bytes.
Approximation accuracy and timing are separate outputs. Choose exact counts for
rare-key accuracy unless a validated sketch error budget admits these excesses.

Assembly contains counter-update calls, memmove for vector maintenance, and
power/log calls in estimation. These are observed code shapes; profiling has not
established their share of runtime. Logical bytes count retained entries only,
excluding capacity, allocation overhead, temporary halves and report/input buffers.
Both native campaigns are warm synthetic batch measurements. Peak resident memory,
cold-cache behavior, isolated update/query costs and service latency remain open.

The initial d21dd36 campaign is retained separately. A pinned-toolchain float-assertion
lint repair changed only test assertions, prompting the final exact-source replay.
Both campaigns remain distinct; final tables use only71f9704. All seven required
Rust gates plus workspace all-targets pass. Commands and predeclared controls are
in README.md and BENCHMARK.md. Complete raw data remain external to Git.

### arm

| N | Shape | Candidate | Median us | Observed range us | Logical bytes | Selection |
|---:|---|---|---:|---|---:|---|
| 128 | uniform | exact-frequency | 52.086 | 51.835..52.862 | 1984 | count-min |
| 128 | uniform | count-min | 30.464 | 30.418..30.775 | 10240 | count-min |
| 128 | uniform | exact-distinct | 2.035 | 2.023..2.051 | 992 | exact-distinct |
| 128 | uniform | hll | 2.750 | 2.715..2.845 | 1024 | exact-distinct |
| 128 | uniform | exact-quantile | 0.976 | 0.959..0.990 | 1024 | exact-quantile |
| 128 | uniform | rank-summary | 10.432 | 10.407..10.471 | 936 | exact-quantile |
| 4096 | uniform | exact-frequency | 213.716 | 210.681..216.447 | 42000 | count-min |
| 4096 | uniform | count-min | 58.876 | 58.669..61.148 | 10240 | count-min |
| 4096 | uniform | exact-distinct | 76.970 | 76.597..79.230 | 21000 | hll |
| 4096 | uniform | hll | 13.607 | 13.377..14.108 | 1024 | hll |
| 4096 | uniform | exact-quantile | 48.833 | 48.357..50.510 | 32768 | exact-quantile |
| 4096 | uniform | rank-summary | 432.938 | 427.842..443.024 | 912 | exact-quantile |
| 16384 | uniform | exact-frequency | 455.652 | 451.600..458.417 | 64400 | count-min |
| 16384 | uniform | count-min | 128.533 | 128.392..128.665 | 10240 | count-min |
| 16384 | uniform | exact-distinct | 297.267 | 295.154..300.031 | 32200 | hll |
| 16384 | uniform | hll | 35.575 | 35.393..36.398 | 1024 | hll |
| 16384 | uniform | exact-quantile | 233.587 | 232.308..237.835 | 131072 | exact-quantile |
| 16384 | uniform | rank-summary | 1532.658 | 1528.746..1548.099 | 912 | exact-quantile |
| 16384 | skew | exact-frequency | 297.427 | 294.533..301.096 | 20992 | count-min |
| 16384 | skew | count-min | 129.577 | 129.437..129.861 | 10240 | count-min |
| 16384 | skew | exact-distinct | 249.832 | 247.738..252.200 | 10496 | hll |
| 16384 | skew | hll | 41.735 | 41.624..42.525 | 1024 | hll |
| 16384 | skew | exact-quantile | 54.593 | 53.978..55.051 | 131072 | exact-quantile |
| 16384 | skew | rank-summary | 3659.318 | 3651.881..3693.585 | 4656 | exact-quantile |
| 16384 | sorted | exact-frequency | 1043.390 | 1032.355..1061.032 | 262144 | count-min |
| 16384 | sorted | count-min | 128.440 | 128.319..131.528 | 10240 | count-min |
| 16384 | sorted | exact-distinct | 277.123 | 274.505..281.184 | 131072 | hll |
| 16384 | sorted | hll | 35.047 | 34.769..46.170 | 1024 | hll |
| 16384 | sorted | exact-quantile | 17.816 | 17.626..18.451 | 131072 | exact-quantile |
| 16384 | sorted | rank-summary | 918.407 | 910.282..950.705 | 864 | exact-quantile |
| 16384 | reverse | exact-frequency | 1035.876 | 1031.228..1062.941 | 262144 | count-min |
| 16384 | reverse | count-min | 128.701 | 128.478..132.954 | 10240 | count-min |
| 16384 | reverse | exact-distinct | 275.793 | 273.235..278.590 | 131072 | hll |
| 16384 | reverse | hll | 35.151 | 35.071..35.268 | 1024 | hll |
| 16384 | reverse | exact-quantile | 20.677 | 20.300..21.041 | 131072 | exact-quantile |
| 16384 | reverse | rank-summary | 1210.688 | 1208.780..1211.557 | 864 | exact-quantile |

A/A B/A observed range 0.957..1.006.

### x86

| N | Shape | Candidate | Median us | Observed range us | Logical bytes | Selection |
|---:|---|---|---:|---|---:|---|
| 128 | uniform | exact-frequency | 49.081 | 48.899..49.769 | 1984 | count-min |
| 128 | uniform | count-min | 28.225 | 28.076..31.177 | 10240 | count-min |
| 128 | uniform | exact-distinct | 1.709 | 1.672..2.361 | 992 | exact-distinct |
| 128 | uniform | hll | 4.419 | 4.257..4.626 | 1024 | exact-distinct |
| 128 | uniform | exact-quantile | 0.904 | 0.886..0.936 | 1024 | exact-quantile |
| 128 | uniform | rank-summary | 9.238 | 9.194..9.289 | 936 | exact-quantile |
| 4096 | uniform | exact-frequency | 207.220 | 204.273..220.046 | 42000 | count-min |
| 4096 | uniform | count-min | 58.300 | 57.453..66.096 | 10240 | count-min |
| 4096 | uniform | exact-distinct | 72.559 | 71.928..81.519 | 21000 | hll |
| 4096 | uniform | hll | 19.489 | 18.337..23.200 | 1024 | hll |
| 4096 | uniform | exact-quantile | 45.620 | 44.704..46.302 | 32768 | exact-quantile |
| 4096 | uniform | rank-summary | 390.519 | 382.666..438.435 | 912 | exact-quantile |
| 16384 | uniform | exact-frequency | 435.321 | 429.284..443.368 | 64400 | count-min |
| 16384 | uniform | count-min | 131.287 | 129.533..255.535 | 10240 | count-min |
| 16384 | uniform | exact-distinct | 269.194 | 265.782..314.515 | 32200 | hll |
| 16384 | uniform | hll | 42.514 | 41.177..46.368 | 1024 | hll |
| 16384 | uniform | exact-quantile | 236.751 | 212.508..321.827 | 131072 | exact-quantile |
| 16384 | uniform | rank-summary | 1396.158 | 1374.932..1476.396 | 912 | exact-quantile |
| 16384 | skew | exact-frequency | 282.332 | 277.856..285.836 | 20992 | count-min |
| 16384 | skew | count-min | 130.205 | 129.678..130.605 | 10240 | count-min |
| 16384 | skew | exact-distinct | 214.377 | 208.272..229.488 | 10496 | hll |
| 16384 | skew | hll | 50.795 | 50.234..51.711 | 1024 | hll |
| 16384 | skew | exact-quantile | 53.433 | 51.883..54.736 | 131072 | exact-quantile |
| 16384 | skew | rank-summary | 2817.780 | 2730.024..3032.892 | 4656 | exact-quantile |
| 16384 | sorted | exact-frequency | 911.990 | 904.045..950.361 | 262144 | count-min |
| 16384 | sorted | count-min | 130.137 | 129.669..132.886 | 10240 | count-min |
| 16384 | sorted | exact-distinct | 245.436 | 238.953..246.269 | 131072 | hll |
| 16384 | sorted | hll | 42.501 | 41.567..43.360 | 1024 | hll |
| 16384 | sorted | exact-quantile | 20.115 | 19.072..21.649 | 131072 | exact-quantile |
| 16384 | sorted | rank-summary | 751.357 | 745.865..766.590 | 864 | exact-quantile |
| 16384 | reverse | exact-frequency | 915.611 | 901.461..930.668 | 262144 | count-min |
| 16384 | reverse | count-min | 131.015 | 129.520..139.907 | 10240 | count-min |
| 16384 | reverse | exact-distinct | 244.553 | 240.423..245.837 | 131072 | hll |
| 16384 | reverse | hll | 43.203 | 42.191..44.404 | 1024 | hll |
| 16384 | reverse | exact-quantile | 23.238 | 21.645..25.666 | 131072 | exact-quantile |
| 16384 | reverse | rank-summary | 1146.010 | 1123.522..1169.838 | 864 | exact-quantile |

A/A B/A observed range 0.813..1.053. This identical-artifact control left the 5%
band on both sides: below 1/1.05 = 0.952 and above 1.05. The 5% rule is therefore
uncalibrated against x86 null variation. The x86 medians and selections remain
nominal descriptive screens; a stable x86 speed recommendation requires a new
predeclared experiment. The Arm control stayed inside the band.
