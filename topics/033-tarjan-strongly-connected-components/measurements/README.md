# Measured SCC selection

Frozen executable source: `7e1069cf60e56091a404c9816f4d9f33d85776af`.
Later commits on this branch change only the benchmark's candidate dispatch
(resolved to one function pointer before the timer starts), its argument filter
(Cargo's `--bench` flag is dropped before positional lookup), `canonical`'s
dense-label assertion, `# Panics` documentation, `certify`'s component-count
expression, the crate manifest (workspace edition, MSRV and lint inheritance),
the contract tests, the runner's argument check, timeout and unparsed-output
records, configurable bound and early `binary_sha256` flush, and the reducer's
cross-host source-hash, schedule and graph-metadata checks; every timed SCC
function, the timed loop body's work and the workload generator are the frozen
commit's. The results below are that commit's. In the frozen binary the candidate
string match ran inside the timed closure on every repetition (about twelve
instructions per call, inside the observed A/A range); the two Tarjan labels took
equal-length but distinct comparison paths there.
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

- tiny-first: tarjan 1.002, gabow 1.079, kosaraju 2.640, prepared 0.876 (unresolved; median leader tarjan).
- tiny: tarjan 0.219, gabow 0.160, kosaraju 0.379, prepared 0.185 (gabow; median leader gabow).
- isolated: tarjan 253.819, gabow 155.066, kosaraju 388.476, prepared 212.362 (gabow; median leader gabow).
- chain: tarjan 555.809, gabow 398.164, kosaraju 1700.948, prepared 447.266 (gabow; median leader gabow).
- cycle: tarjan 561.589, gabow 406.272, kosaraju 1185.651, prepared 525.492 (gabow; median leader gabow).
- groups: tarjan 555.291, gabow 390.833, kosaraju 1692.766, prepared 521.462 (gabow; median leader gabow).
- permuted: tarjan 669.372, gabow 574.646, kosaraju 2319.100, prepared 978.118 (gabow; median leader gabow).
- duplicates: tarjan 989.536, gabow 891.791, kosaraju 3332.680, prepared 960.678 (gabow; median leader gabow).
- dense: tarjan 222.021, gabow 231.630, kosaraju 544.298, prepared 236.485 (unresolved; median leader tarjan).
- random: tarjan 2063.268, gabow 1776.313, kosaraju 5131.558, prepared 2474.908 (gabow; median leader gabow).

## x86

- tiny-first: tarjan 1.419, gabow 1.290, kosaraju 3.604, prepared 1.345 (gabow; median leader gabow).
- tiny: tarjan 0.150, gabow 0.122, kosaraju 0.295, prepared 0.143 (gabow; median leader gabow).
- isolated: tarjan 238.410, gabow 134.759, kosaraju 341.018, prepared 188.294 (gabow; median leader gabow).
- chain: tarjan 498.832, gabow 392.332, kosaraju 1573.699, prepared 525.527 (gabow; median leader gabow).
- cycle: tarjan 504.466, gabow 395.437, kosaraju 1033.324, prepared 570.558 (gabow; median leader gabow).
- groups: tarjan 496.413, gabow 343.890, kosaraju 1553.453, prepared 591.621 (gabow; median leader gabow).
- permuted: tarjan 466.034, gabow 380.707, kosaraju 2129.643, prepared 1037.146 (gabow; median leader gabow).
- duplicates: tarjan 899.837, gabow 752.902, kosaraju 2883.477, prepared 986.228 (gabow; median leader gabow).
- dense: tarjan 166.805, gabow 168.440, kosaraju 466.494, prepared 218.951 (unresolved; median leader tarjan).
- random: tarjan 1480.095, gabow 1219.316, kosaraju 4704.200, prepared 2618.611 (gabow; median leader gabow).

Gabow meets the predeclared5%/all-six-pairs rule in17 of20 outgoing-only
cells; three remain unresolved. Prepared mode never decisively beats the outgoing
median leader here. These results apply to the fixed graphs,representation,
compiled code and whole-operation boundary. Allocation counts,hardware counters,
cold-cache behavior,dynamic update costs and population confidence are unmeasured.

A/A mechanical checks pass. Observed identical-code ratio ranges are
Arm0.832..1.318(all),0.832..1.126(warmed); x860.837..1.051(all/warmed).
These wide controls limit fine-grained speed claims. Dense medians remain unresolved. The earlier x86
chain/duplicate cases change from unresolved to Gabow in this replay. First-call microsecond measurements
need caution; x86 meets the descriptive rule,Arm remains unresolved. Keep both campaigns separate.

Generated code: objdump output and linked SCC symbols are retained. In the frozen
binary the two Tarjan labels dispatch to the same function through distinct string
comparisons inside the timed closure; this supports a code-path control,not a
cache/branch causal explanation. The current benchmark resolves the candidate
before the timer so a future campaign times one identical indirect call per label. The source uses reserved heap
continuations; exact allocation counts and peak RSS remain unmeasured.

Reproduction: run measurements/run.py with a fresh output directory on each
host; place each results folder under arm/results and x86/results,then run
`python3 measurements/analyze.py PARENT summary.json`. The reducer verifies
counts,certification,order,the declared workload list,identical graph metadata
across a workload's processes and identical executable-source hashes across hosts,
and reproduces medians,full ranges and decisions. A process that exceeds the runner's
bound (`TOPIC_033_PROCESS_TIMEOUT_S`,default90 seconds) is retained as an
`exit: "timeout"` record before the campaign stops. `measurements/test_run.py` and
`measurements/test_analyze.py` exercise both checks against stub processes.
EVIDENCE_RECEIPT.json binds the raw archives,source,binary and environment.
The detailed compact results.json retains all candidate medians and ranges.
