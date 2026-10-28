# Measured selection

Frozen executable source: `d409e26a8c995b61b0bc1644fa55313d2c8e6df4`.
Both declared Linux hosts passed7 test groups and every candidate preflight/final oracle.
Each host retained259 process records:222 measured,37 warmups. Six processes/cell,3:3 relative pair order.
Raw archives and exact input/binary hashes are in EVIDENCE_RECEIPT.json.

Numbers below are nanoseconds per complete order or per cached update. Brackets show observed min/max across six processes.
Selection requires5% observed-range separation from all competitors. Unresolved cells retain their lowest-median candidate descriptively.

## arm: dev-dsk-ahrav-2b-7dc7bd93.us-west-2.amazon.com

rustc 1.98.1 (48a229cea 2026-09-01)

- primary/order/tiny: fifo 128.9 [128.4,133.8]; dfs 129.3 [125.3,150.5]; min 123.2 [122.1,128.2]. Selection: unresolved.
- primary/order/chain_root: fifo 30989.4 [30719.9,31330.8]; dfs 22104.7 [21289.9,22422.6]; min 31094.5 [30875.6,31435.1]. Selection: dfs.
- primary/order/islands: fifo 140331.2 [127796.2,144155.0]; dfs 88972.2 [88325.8,92399.4]; min 273177.7 [268926.3,283138.7]. Selection: dfs.
- primary/order/dense_root: fifo 28272.3 [28156.9,29796.9]; dfs 66933.8 [64742.0,76248.5]; min 27851.3 [27747.2,27946.1]. Selection: unresolved.
- primary/update/tiny: full 29.4 [29.2,30.9]; dirty 20.0 [19.7,20.2]; heap 15.6 [15.3,16.0]. Selection: heap.
- primary/update/chain_root: full 10472.0 [10278.0,10689.1]; dirty 40587.4 [40258.4,40760.9]; heap 38877.3 [38142.0,39002.7]. Selection: full.
- primary/update/chain_leaf: full 10381.0 [10286.6,10518.8]; dirty 2423.2 [2396.5,2440.1]; heap 15.7 [15.5,16.1]. Selection: heap.
- primary/update/islands: full 96796.2 [89048.9,101755.5]; dirty 10639.7 [10558.2,10907.7]; heap 579.4 [577.7,583.7]. Selection: heap.
- primary/update/layered_sink: full 27598.1 [27539.0,27671.9]; dirty 2424.0 [2415.5,2431.9]; heap 21.6 [21.2,23.1]. Selection: heap.
- primary/update/dense_root: full 21641.6 [21515.0,22073.6]; dirty 48575.5 [48387.3,48664.8]; heap 75622.0 [74926.0,76054.5]. Selection: full.
- primary/update/noop: full 27616.2 [27550.2,27663.5]; dirty 64381.2 [64087.5,64739.8]; heap 14.3 [14.1,14.6]. Selection: heap.
- aa/order/tiny: fifo 129.1 [124.2,136.7]; fifo_aa 129.2 [128.3,130.1]. Selection: unresolved.
- aa/update/tiny: full 29.3 [29.3,30.3]; full_aa 29.4 [29.1,31.1]. Selection: unresolved.

## x86: dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com

rustc 1.99.0 (b940084d7 2026-09-28)

- primary/order/tiny: fifo 85.0 [83.2,86.8]; dfs 86.4 [84.5,90.8]; min 86.5 [82.5,137.2]. Selection: unresolved.
- primary/order/chain_root: fifo 27617.4 [27222.6,28031.1]; dfs 16475.2 [16427.0,20541.9]; min 28256.2 [27926.4,29079.2]. Selection: dfs.
- primary/order/islands: fifo 156383.5 [146549.6,172644.4]; dfs 74872.4 [70895.4,90003.2]; min 313415.8 [311481.9,346172.7]. Selection: dfs.
- primary/order/dense_root: fifo 26042.6 [25670.0,26471.2]; dfs 64546.6 [64378.5,64718.0]; min 25637.0 [24942.5,26575.6]. Selection: unresolved.
- primary/update/tiny: full 24.8 [24.5,25.0]; dirty 19.5 [18.9,21.7]; heap 16.4 [16.3,18.9]. Selection: unresolved.
- primary/update/chain_root: full 8844.5 [8808.8,9039.8]; dirty 35145.2 [34670.9,35277.3]; heap 23830.8 [23698.8,24973.1]. Selection: full.
- primary/update/chain_leaf: full 8820.7 [8794.4,8914.6]; dirty 2616.8 [2591.2,2629.2]; heap 16.3 [16.0,16.7]. Selection: heap.
- primary/update/islands: full 81040.2 [78330.9,83660.5]; dirty 11585.9 [11479.0,12007.5]; heap 378.2 [370.4,385.9]. Selection: heap.
- primary/update/layered_sink: full 19311.4 [19239.2,19352.1]; dirty 2591.1 [2589.9,2601.7]; heap 18.9 [18.2,20.6]. Selection: heap.
- primary/update/dense_root: full 12892.7 [12838.8,13082.3]; dirty 42867.9 [38965.6,43904.3]; heap 51077.0 [50914.8,51358.0]. Selection: full.
- primary/update/noop: full 19310.5 [19286.1,20416.3]; dirty 53812.9 [53684.6,55342.4]; heap 16.2 [15.9,18.4]. Selection: heap.
- aa/order/tiny: fifo 84.9 [83.1,89.0]; fifo_aa 84.9 [83.3,86.5]. Selection: unresolved.
- aa/update/tiny: full 24.9 [24.8,26.4]; full_aa 24.9 [24.7,30.5]. Selection: unresolved.

## Scope and interpretation

Seventeen of22 primary cells select a candidate; five remain unresolved. DFS wins chain and island ordering on both hosts. Full scans win chain-root and dense-root updates on both. Heap wins localized leaf/island/sink/no-op updates on both, and tiny updates on Arm. Tiny x86 update remains unresolved. Dirty scanning wins no declared cell.

A/A median alias/baseline ratios are close to1; observed individual ratios and ranges remain descriptive. Work counters agree with structure: island heap evaluates64 vertices/update versus16384 for full; chain-root affects all4096. A/A control coverage is limited to tiny workloads. Initial state is process-fresh, but preflight warms the graph before timing. Cold-cache latency and graph-edit performance are unmeasured.

Host compilers differ (Arm1.98.1,x861.99.0); CPU affinity is unbound. Graph admission/initialization, repeated-update work counters and output-copy exclusions limit generalization. Timing ranges supply no confidence bound. No allocation counter or cache/branch mechanism was measured. Whole-process perf cycles/instructions include oracle/setup and cannot support candidate-only causal claims. See retained environment/disassembly receipts.
