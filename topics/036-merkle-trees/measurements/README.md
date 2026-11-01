# Native results: fixed Merkle update portfolio

Frozen executable-source commit: b2139a801c1c5fd198b367d6e6a7c7cd50b75a83.
The retained data was measured from that commit's five inputs, whose hashes
are in EVIDENCE_RECEIPT.json. Review fixes after measurement changed four of
them: the write payload delta is now `(r+q)%250+1` so every non-noop write
changes its leaf, the bench binary accepts Cargo's `--bench` flag and drops
its construction-timed tree before lifecycle batches (about 4 MiB less resident
memory in large-record lifecycle cells), summarize.py asserts the block sequence, and src/lib.rs encodes the SHA-256 bit length with
`checked_mul` and tests Full and Eager call counts. tests/contracts.rs is
unchanged. The timings were not re-measured. In the retained data the old delta
`r+q+1` wrapped to zero whenever `(r+q+1) % 251 == 0`, so Eager and Batch
skipped 184 of the schedule's writes as byte-equal: 32 of 8192 per batch in
dense/lifecycle (16351 Batch calls instead of 16383), 32 in the first of four
dense/update batches, 8 of 2048 tiny/lifecycle writes and 16 of 4096
small/lifecycle writes, visible as fractional hash-call means in results.json.
Full rebuild counts are unaffected.
Each host passed two unit groups,
one independent recursive contract test and 136 SHA-256/hashlib comparisons.
All 378 process roots per host match the independent Python tree oracle.
There are 324 measured and 54 warmup processes per host, 18 cells each.

Arm: dev-dsk-ahrav-2b-7dc7bd93.us-west-2.amazon.com, aarch64,
Linux 6.12.113-138.209.amzn2023, ARM implementer 0x41 part 0xd40,
64 available CPUs, rustc 1.98.1. x86 alias xxl resolved to
 dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com, x86_64,
Linux 6.12.113-138.209.amzn2023, Intel Xeon Platinum 8488C,
192 available CPUs, rustc 1.99.0. Full uname, CPU identities and target cfg
are in external environment.json. Flags: edition 2024, opt-level=3,
target-cpu=native. Source uses portable allocating SHA-256.

Numbers below are microseconds per batch: median [observed minimum, maximum].
F=Full, E=Eager, B=Batch. The fastest median is reported even when the paired
5% rule leaves selection unresolved. Ranges describe six process samples,
not population confidence intervals. All original medians and hash-call counts
are retained in results.json. Lifecycle includes construction and destruction.

## arm

| Workload / boundary | F | E | B | Fastest median | Selection |
|---|---:|---:|---:|---|---|
| tiny / update | 4.058 [4.047, 4.110] | 1.872 [1.851, 1.875] | 1.923 [1.914, 1.967] | eager | unresolved |
| tiny / lifecycle | 8.735 [8.716, 8.756] | 6.368 [6.351, 6.393] | 6.100 [6.084, 6.104] | batch | unresolved |
| small / update | 67.227 [67.149, 67.353] | 36.399 [36.271, 36.489] | 10.602 [10.561, 10.653] | batch | batch |
| small / lifecycle | 136.916 [136.611, 137.129] | 105.304 [105.226, 105.555] | 82.138 [82.042, 82.338] | batch | batch |
| sparse / update | 8844.773 [8842.002, 8854.981] | 9.394 [9.307, 9.412] | 9.469 [9.429, 9.533] | eager | unresolved |
| sparse / lifecycle | 18377.061 [18366.366, 18406.238] | 9432.446 [9414.722, 9472.341] | 9432.736 [9413.603, 9495.047] | eager | unresolved |
| cluster / update | 8981.275 [8960.649, 9002.165] | 613.363 [609.919, 614.496] | 76.336 [76.187, 76.611] | batch | batch |
| cluster / lifecycle | 18360.756 [18337.991, 18428.100] | 9747.090 [9737.418, 9782.891] | 9411.890 [9393.675, 9557.373] | batch | unresolved |
| scatter / update | 8977.554 [8971.164, 9003.244] | 614.797 [612.880, 615.602] | 366.543 [364.983, 368.678] | batch | batch |
| scatter / lifecycle | 18362.997 [18356.557, 18431.957] | 9740.807 [9739.058, 9760.740] | 9685.959 [9656.346, 9707.726] | batch | unresolved |
| dense / update | 8973.312 [8941.892, 9097.997] | 77530.295 [77420.294, 77625.668] | 9194.699 [9168.448, 9206.564] | full | unresolved |
| dense / lifecycle | 18300.062 [18277.988, 18341.062] | 87597.721 [87478.613, 87810.992] | 18535.266 [18489.527, 18697.763] | full | unresolved |
| duplicate / update | 8713.661 [8693.142, 8744.297] | 9683.709 [9674.319, 9722.260] | 422.338 [421.298, 424.276] | batch | batch |
| duplicate / lifecycle | 18379.021 [18343.252, 18465.215] | 19326.106 [19295.795, 19333.901] | 9820.985 [9798.895, 9854.644] | batch | batch |
| noop / update | 8970.568 [8957.706, 8983.604] | 0.478 [0.432, 0.512] | 0.449 [0.443, 0.665] | batch | unresolved |
| noop / lifecycle | 18364.011 [18336.562, 18388.630] | 9174.908 [9148.215, 9287.148] | 9158.623 [9154.703, 9191.929] | batch | unresolved |
| large-record / update | 19508.111 [19499.809, 19558.429] | 25.242 [25.168, 25.481] | 25.346 [25.320, 25.409] | eager | unresolved |
| large-record / lifecycle | 40360.976 [40350.564, 40462.513] | 20876.958 [20847.424, 20972.350] | 19760.037 [19757.843, 19924.643] | batch | unresolved |

## x86

| Workload / boundary | F | E | B | Fastest median | Selection |
|---|---:|---:|---:|---|---|
| tiny / update | 3.266 [3.230, 3.292] | 1.505 [1.491, 1.534] | 1.537 [1.522, 1.547] | eager | unresolved |
| tiny / lifecycle | 6.911 [6.876, 6.996] | 5.124 [5.106, 5.308] | 4.888 [4.845, 4.937] | batch | unresolved |
| small / update | 56.490 [54.773, 57.360] | 29.949 [29.878, 30.285] | 8.725 [8.699, 8.806] | batch | batch |
| small / lifecycle | 115.538 [113.405, 116.838] | 88.965 [87.481, 96.426] | 67.166 [66.767, 67.869] | batch | batch |
| sparse / update | 7180.402 [7111.949, 7470.210] | 7.918 [7.719, 8.850] | 7.834 [7.713, 19.774] | batch | unresolved |
| sparse / lifecycle | 15138.397 [15028.883, 15245.030] | 7763.120 [7711.497, 7886.972] | 7770.322 [7697.585, 7915.809] | eager | unresolved |
| cluster / update | 7247.005 [7169.265, 7327.969] | 500.492 [495.180, 503.935] | 62.943 [62.360, 65.431] | batch | batch |
| cluster / lifecycle | 15209.191 [15061.158, 15583.401] | 8193.420 [8005.843, 8452.855] | 7840.819 [7666.659, 7923.841] | batch | unresolved |
| scatter / update | 7282.941 [7183.388, 7371.866] | 501.690 [489.625, 503.894] | 301.121 [299.155, 325.891] | batch | batch |
| scatter / lifecycle | 15057.564 [15033.534, 15699.683] | 8156.679 [7980.214, 8249.108] | 8026.958 [7915.924, 8245.445] | batch | unresolved |
| dense / update | 7248.520 [7224.051, 7372.498] | 63374.852 [63200.342, 63780.724] | 7618.517 [7589.677, 7843.439] | full | unresolved |
| dense / lifecycle | 15052.240 [14975.761, 15342.954] | 72023.358 [71784.039, 72227.604] | 15607.128 [15313.890, 15887.317] | full | unresolved |
| duplicate / update | 7359.303 [7235.936, 7541.039] | 7886.778 [7861.800, 7926.895] | 345.001 [338.921, 348.735] | batch | batch |
| duplicate / lifecycle | 15056.143 [14979.628, 15239.148] | 15767.557 [15689.956, 15879.323] | 8080.954 [7993.201, 8270.690] | batch | batch |
| noop / update | 7238.120 [7195.275, 7275.689] | 0.449 [0.348, 0.610] | 0.575 [0.343, 0.750] | eager | unresolved |
| noop / lifecycle | 15080.094 [14989.890, 15154.483] | 7642.233 [7609.489, 7776.878] | 7639.226 [7545.799, 7814.652] | batch | unresolved |
| large-record / update | 16848.638 [16828.830, 16934.016] | 21.499 [21.404, 22.027] | 21.629 [21.550, 21.793] | eager | unresolved |
| large-record / lifecycle | 34909.891 [34892.452, 35006.677] | 18118.821 [18098.726, 18238.809] | 17286.152 [17257.063, 17475.050] | batch | unresolved |

## Selection and limitations

Batch satisfies the paired criterion on small, clustered, scattered and repeated-
index updates on both hosts. It also wins small and repeated-index lifecycle
cells. Every other cell remains unresolved by the predeclared criterion.
Full has the lowest dense-update median on both hosts. Eager has the lowest
single-large-record update median on both. Eager and Batch are close for one
sparse write; construction dominates many lifecycle differences.

For 8192 leaves, one changed leaf needs 14 calls versus 16383 full calls.
Sixty-four adjacent leaves share ancestors: 134 batch calls versus 896 eager
calls. Scattered positions share fewer ancestors. Duplicate writes still hash
each changed payload in Batch before ancestor deduplication.

These timings cover this implementation and declared inputs. A production hash
backend, last-write coalescing, alternate scratch layouts or a different allocator
may change the result. Host load, frequency and affinity were uncontrolled.
No startup/cold-cache/peak-RSS/hardware-counter or universal-optimum claim is made.
The linked disassembly is retained as static evidence; the comparison does not
assign differences to CPU microarchitecture. The hosts also used different rustc
versions. An initial local verification-script list/tuple comparison failed;
correcting that verifier reproduced the unchanged native data without rerunning.

See EVIDENCE_RECEIPT.json for archive hashes and retained raw-evidence paths.
PLUGIN_AUDIT.md records the current scoped no-change finding. The missing catalog
coverage/index metadata remains a separate pending action.
