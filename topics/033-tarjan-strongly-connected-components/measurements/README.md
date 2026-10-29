# Measured SCC selection

Frozen executable source: `e415812a27dca023f0e8e991fe100bbfb470bc79`.
Both exact-source campaigns passed six contract groups,66,067 exhaustive graphs,
the100k chain/cycle,example and every timed process certificate. Each retained
350 process records:300 measured and50 warmup. All pair orders are3:3.

Arm: declared dev-dsk-ahrav-2b-7dc7bd93,aarch64,64 available CPUs,pinned32,
Arm implementer0x41/part0xd40,rustc1.98.1,Amazon Linux kernel6.12.111.
CPU marketing-model text is unavailable; recorded identifiers are authoritative.
x86: xxl resolved dev-dsk-ahrav-2c-32182091,x86_64,192 CPUs,pinned96,
Intel Xeon Platinum8488C,rustc1.99.0,kernel6.12.113. Full uname,target features,
flags,CPU records and binary hashes are in the external verified receipts.

Medians below use microseconds per complete partition. Parentheses give the
outgoing-only decision among Tarjan/Gabow/one-shot Kosaraju. Prepared Kosaraju
retains transpose construction/storage outside its timed boundary.

## arm

- tiny-first: tarjan 0.892, gabow 0.958, kosaraju 2.666, prepared 0.868 (unresolved; median leader tarjan).
- tiny: tarjan 0.218, gabow 0.160, kosaraju 0.378, prepared 0.185 (gabow; median leader gabow).
- isolated: tarjan 249.785, gabow 151.682, kosaraju 378.560, prepared 207.885 (gabow; median leader gabow).
- chain: tarjan 546.995, gabow 389.588, kosaraju 1681.095, prepared 438.186 (gabow; median leader gabow).
- cycle: tarjan 550.147, gabow 403.063, kosaraju 1179.484, prepared 512.513 (gabow; median leader gabow).
- groups: tarjan 549.812, gabow 358.375, kosaraju 1660.512, prepared 508.165 (gabow; median leader gabow).
- permuted: tarjan 663.167, gabow 577.972, kosaraju 2290.085, prepared 978.557 (gabow; median leader gabow).
- duplicates: tarjan 978.053, gabow 888.431, kosaraju 3326.167, prepared 952.385 (gabow; median leader gabow).
- dense: tarjan 227.619, gabow 231.137, kosaraju 535.010, prepared 233.779 (unresolved; median leader tarjan).
- random: tarjan 2033.912, gabow 1759.627, kosaraju 5117.710, prepared 2442.003 (gabow; median leader gabow).

## x86

- tiny-first: tarjan 1.446, gabow 1.782, kosaraju 4.545, prepared 1.633 (unresolved; median leader tarjan).
- tiny: tarjan 0.149, gabow 0.122, kosaraju 0.294, prepared 0.143 (gabow; median leader gabow).
- isolated: tarjan 244.096, gabow 138.229, kosaraju 344.958, prepared 187.457 (gabow; median leader gabow).
- chain: tarjan 503.895, gabow 395.870, kosaraju 1613.911, prepared 533.366 (unresolved; median leader gabow).
- cycle: tarjan 511.363, gabow 404.698, kosaraju 1040.868, prepared 581.747 (gabow; median leader gabow).
- groups: tarjan 602.490, gabow 346.309, kosaraju 1568.741, prepared 596.319 (gabow; median leader gabow).
- permuted: tarjan 450.791, gabow 382.193, kosaraju 2171.338, prepared 1044.875 (gabow; median leader gabow).
- duplicates: tarjan 911.226, gabow 899.813, kosaraju 3250.349, prepared 1322.763 (unresolved; median leader gabow).
- dense: tarjan 167.217, gabow 168.485, kosaraju 479.180, prepared 237.013 (unresolved; median leader tarjan).
- random: tarjan 1762.512, gabow 1302.200, kosaraju 6564.109, prepared 3073.444 (gabow; median leader gabow).

Gabow meets the predeclared5%/all-six-pairs rule in14 of20 outgoing-only
cells; six remain unresolved. Prepared mode never decisively beats the outgoing
median leader here. These results apply to the fixed graphs,representation,
compiled code and whole-operation boundary. Allocation counts,hardware counters,
cold-cache behavior,dynamic update costs and population confidence are unmeasured.

A/A mechanical checks pass. Observed identical-code ratio ranges are
Arm0.936..1.219(all),0.936..1.167(warmed); x860.567..2.158(all/warmed).
These wide controls limit fine-grained speed claims. Dense medians and x86
duplicate-heavy results remain unresolved. First-call microsecond measurements
also remain unresolved. Keep every attempt; no favorable reruns or pooled hosts.

Generated code: objdump output and linked SCC symbols are retained. The two
Tarjan labels dispatch to the same function in source. This supports a code-path
control,not a cache/branch causal explanation. The source uses reserved heap
continuations; exact allocation counts and peak RSS remain unmeasured.

Reproduction: run measurements/run.py with a fresh output directory on each
host; place each results folder under arm/results and x86/results,then run
`python3 measurements/analyze.py PARENT summary.json`. The reducer verifies
counts,certification and order and reproduces medians,full ranges and decisions.
EVIDENCE_RECEIPT.json binds the raw archives,source,binary and environment.
The detailed compact results.json retains all candidate medians and ranges.
