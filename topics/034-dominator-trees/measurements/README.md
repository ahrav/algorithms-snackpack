# Exact committed-source results

Executable source: `1c247acb7da3e18c625a58301bee1143e6e97b76`. Later commits add analysis, documentation and receipts. The algorithms,benchmark,runner and build manifests remain frozen.

Both declared Linux hosts passed three contract groups,the running example and every process oracle. Each retained210 process records:180 measured and30 warmup. All source/archive/linked-image hashes,graph metadata,completion markers and3:3pair orders verify.

Arm: dev-dsk-ahrav-2b-7dc7bd93,aarch64,64 available CPUs,pinned32,implementer0x41/part0xd40,rustc1.98.1. x86: xxl resolved dev-dsk-ahrav-2c-32182091,x86_64,192 CPUs,pinned96,Intel Xeon Platinum8488C,rustc1.99.0. Both kernel6.12.113. Full uname,CPU,compiler,target features and exact flags are retained. Build:-C target-cpu=native -C opt-level=3.

Times are microseconds per complete operation. Brackets give the full observed process range,not confidence intervals.

## arm

| Workload | Candidate median [min,max] microseconds | Selection |
|---|---|---|
| tiny n7 m6 | sets 0.888 [0.880,0.908]; chk 0.599 [0.595,0.608] | chk |
| chain n512 m511 | sets 339.271 [337.313,363.672]; chk 29.446 [29.121,29.999] | chk |
| permuted n256 m255 | sets 1918.274 [1912.657,1923.931]; chk 16.508 [16.297,19.668] | chk |
| joins n512 m1020 | sets 295.627 [294.780,296.432]; chk 36.124 [35.607,36.358] | chk |
| cycle n512 m1022 | sets 343.838 [340.552,353.542]; chk 33.739 [33.440,34.728] | chk |
| dense n128 m2468 | sets 83.785 [82.365,84.367]; chk 56.381 [55.304,67.715] | chk |
| unreachable n512 m510 | sets 179.604 [179.233,180.482]; chk 25.241 [25.115,25.495] | chk |
| noop n512 m511 | rebuild 903.955 [900.021,906.578]; incremental 490.073 [488.640,510.933]; rebuild-tree 3281.465 [3261.585,3331.703] | incremental |
| bypass n512 m511 | rebuild 877.213 [874.200,877.968]; incremental 540.673 [538.168,542.406]; rebuild-tree 1941.870 [1927.045,1970.627] | incremental |
| newreach n512 m510 | rebuild 877.412 [875.681,896.783]; incremental 523.214 [521.202,530.231]; rebuild-tree 1944.711 [1929.912,1959.943] | incremental |
## x86

| Workload | Candidate median [min,max] microseconds | Selection |
|---|---|---|
| tiny n7 m6 | sets 0.660 [0.634,0.683]; chk 0.394 [0.390,0.441] | chk |
| chain n512 m511 | sets 423.023 [421.540,507.944]; chk 23.359 [23.052,23.839] | chk |
| permuted n256 m255 | sets 1797.950 [1788.284,2220.434]; chk 14.073 [13.568,20.040] | chk |
| joins n512 m1020 | sets 322.348 [320.011,373.525]; chk 29.342 [28.569,31.076] | chk |
| cycle n512 m1022 | sets 426.842 [424.460,427.343]; chk 27.355 [27.277,42.509] | chk |
| dense n128 m2468 | sets 74.962 [74.440,115.849]; chk 46.943 [46.635,47.939] | chk |
| unreachable n512 m510 | sets 203.150 [201.660,205.940]; chk 19.492 [19.002,29.216] | chk |
| noop n512 m511 | rebuild 890.894 [886.745,897.219]; incremental 534.875 [532.814,552.289]; rebuild-tree 3412.320 [3398.079,3536.157] | incremental |
| bypass n512 m511 | rebuild 829.990 [827.320,1006.147]; incremental 557.771 [554.783,682.552]; rebuild-tree 2009.934 [2007.071,2139.214] | incremental |
| newreach n512 m510 | rebuild 892.346 [823.804,1067.709]; incremental 538.090 [532.191,555.212]; rebuild-tree 2012.832 [2007.170,2200.702] | incremental |

CHK meets the5%/all-six-pairs rule in all14 immediate-parent cells. Incremental full sets meet it in all6 insertion-stream cells. No cells are unresolved under this descriptive criterion. Identical CHK control is excluded from selection. Static parent construction and dynamic retained-set maintenance have different output/state contracts. CHK rebuild pays set expansion after every edit.

The newreach stream rebuilds on its first insertion,then propagates7 duplicates. Its complete8-edit win establishes no first-reachability-change advantage. Snapshot cloning,final parent extraction and teardown are included. Oracle/preflight warm input; cold starts,population confidence,real compiler graphs,allocation counts,RSS,hardware causes and Lengauer-Tarjan remain unmeasured.

A/A CHK/control ratios span arm 0.548..1.219, x86 0.649..1.511. Wide controls limit fine-grained claims. Assembly and operation counters are retained; no candidate-only cache/branch explanation follows from oracle-containing whole-process activity.

Reproduce with `python3 measurements/run.py NEW_RESULTS` per host. Place complete source/results folders under arm/ and x86/,then run `python3 measurements/analyze.py PARENT summary.json`. Reducer checks exact source,required architectures,tests,210-record schedule,dimensions,positive timing and order. EVIDENCE_RECEIPT.json binds archives,source,linked images and environments.

Earlier scratch and8a5e815 campaigns remain separate. The first workspace all-targets check rejected the benchmark with no arguments. The final source adds a default smoke workload and strengthens the deletion regression to discriminate stale cyclic support. Both required hosts replayed the corrected committed source. All20 final selection decisions agree with the earlier committed campaign; timings remain separate.
