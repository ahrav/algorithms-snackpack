# Benchmark decision

Benchmark decision: true. Compare vector shifting, adjacent pooling and the lower hull under the same output contract.

Fit one array of ordered signed 32-bit observations with equal weights. Return the same run-length block encoding from each candidate. Test increasing, falling, stepped plateau and locally noisy rising inputs. Sizes are 16, 512 and 4,096 where listed in the runner. The lab accepts up to 32,768 observations. The correctness campaign also includes a 32,768-element all-equal case and a high-to-low extreme case.

File transfer is verified by SHA-256 before execution. Compile Rust 2024 with `-D warnings -C opt-level=3 -C target-cpu=native`. Record compiler, features, kernel, CPU model and allowed CPUs. Pin benchmark child processes to the first allowed CPU. Run the independent exact oracle and certificate against the native binary before timing.

Each workload starts with three discarded processes, then uses all six permutations of the three candidates, once each. Insert two separate stack processes after each block for the A/A control. Each process performs 32 untimed fits. Time repeated complete fits, including candidate allocation, compressed output construction and drop. Exclude input generation, checking, digesting, process startup and output formatting. Treat one process as one observation. Keep every sample and report median and min–max. This measures serial warm batches with fixed input on each host.

Call a winner only when the same candidate has the lowest median and is at least 5% faster than each rival in all six paired blocks. Here “5% faster” means rival time / candidate time ≥ 1.05. All six A/A ratios must also stay in [1/1.05, 1.05]. Report unresolved measurements with raw medians and control ranges. The thresholds are descriptive decision rules.
