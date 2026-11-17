# Benchmark decision

benchmark = true

Compare flat, boxed left-leaning red-black, arena left-leaning red-black and
BTreeSet at32/4096 insertion attempts. Shapes:ascending, descending and an
irregular duplicate-heavy sequence (one quarter distinct keys). Queries:8192
membership tests over dispersed keys or a repeated eight-key hot set; hits and
misses are explicit in retained full traces.

Unit:one fresh process per candidate/workload/block. One warmup, six measured
blocks; each adjacent block pair reverses order, rotating the starting candidate
between pairs. Stop after the declared schedule. Each host has48 correctness,
48 warmup,288 measured and12 A/A processes. A/A repeats the same arena binary.
Pin to the first allowed CPU. Native optimization; synthetic, warm shared hosts.

Build includes initialization, reservation, insertion, node allocation and
insertion-success accumulation. Query includes immutable lookup and a wrapping
checksum. Drop includes freeing the index. Input generation, independent oracle,
full outputs and process startup are outside those intervals. The timed process
uses black_box at input/result boundaries. Check mode validates each prefix and
is excluded from selection. Peak memory and cold-cache behavior are unmeasured.

Report median and min/max per phase/cell/host. A winner must be >5% faster than
every other candidate in all six paired blocks. Otherwise mark unresolved.
A/A ranges qualify stable selection; timing is exploratory with six process
samples. Hosts use different compilers, so cross-host differences carry no
hardware-causal interpretation. Source-level allocation counts and vector shift
counts explain work; timing alone does not establish cache mechanisms.
