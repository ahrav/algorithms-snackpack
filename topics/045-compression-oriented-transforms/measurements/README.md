# Exact-source measurements

Frozen executable commit: `842b5aca4c4a3e6c729f1577ec36863aa2f3a11d`. Library, example and runner match this commit and the final published artifact.

Both hosts: 1020 invocations, including864 measured main,144 warmup and12 A/A control processes. Independent Python checks all72 frames and reconstructions; Rust runs3 contract groups plus a roundtrip in each timing process. Separate workspace integration tests cover state reset and permissive short packets.

Six measured processes per candidate/cell. Medians and full min/max ranges are retained in the external measurements.json. Each candidate occupies each schedule position once. Timings include allocating output destruction. Frames include the5-byte header.

| Words | Shape | Smallest frame | Representations |
|---:|---|---:|---|
| 16 | ramp | 16 B | delta2 |
| 16 | constant | 10 B | delta, xor |
| 16 | random | 69 B | raw |
| 16 | alternating | 28 B | shuffle |
| 1024 | ramp | 78 B | delta2 |
| 1024 | constant | 69 B | shuffle |
| 1024 | random | 4101 B | raw |
| 1024 | alternating | 1085 B | shuffle |
| 65536 | ramp | 4110 B | delta2 |
| 65536 | constant | 4101 B | shuffle |
| 65536 | random | 262149 B | raw |
| 65536 | alternating | 69125 B | shuffle |

For16 constant words, delta and XOR both use10 bytes and their speed contrast is unresolved on both hosts. All other size cells have a unique smallest representation. Storage results agree across hosts.

## Timing screens

A speed screen requires every paired process mean to be at least5percent faster than every competitor. Arm: raw wins12encode cells; all12decode cells unresolved. x86 nominal screens: raw10encode cells, XOR one1024constant decode cell,13unresolved.

A/A ratios (A/B raw random65536 encode): Arm0.965..1.017; x860.788..7.673. The x86 control shows severe time variation. Its medians and nominal screens remain descriptive; a stable x86 speed recommendation requires a new predeclared experiment. Original observations remain retained.

| Host | Shape,65536words | Candidate | Bytes | Encode median[min,max] us | Decode median[min,max] us |
|---|---|---|---:|---|---|
| arm | ramp | delta2 | 4110 | 235.886 [234.279,253.957] | 231.701 [226.803,232.625] |
| arm | constant | shuffle | 4101 | 365.555 [363.518,473.240] | 259.679 [258.207,267.916] |
| arm | random | raw | 262149 | 199.074 [196.868,279.166] | 228.372 [227.102,233.570] |
| arm | alternating | shuffle | 69125 | 538.404 [535.041,546.925] | 261.702 [258.127,268.954] |
| x86 | ramp | delta2 | 4110 | 173.446 [170.592,190.501] | 163.568 [161.157,207.212] |
| x86 | constant | shuffle | 4101 | 292.155 [285.734,324.756] | 225.588 [222.356,329.348] |
| x86 | random | raw | 262149 | 207.945 [174.124,214.774] | 199.920 [170.356,266.090] |
| x86 | alternating | shuffle | 69125 | 606.997 [429.257,666.625] | 224.093 [221.756,258.195] |

## Work, generated code and boundaries

Per word, delta computes one subtraction, delta2 two, and XOR one XOR; shuffle copies four byte positions. Each transform materializes4n bytes. Raw also materializes then copies its byte buffer; it is this implemented baseline. Full allocation/allocator-overhead counters were not measured. Native assembly contains memcpy/memset calls on both hosts. This establishes generated calls, not a cache or hardware-bottleneck diagnosis.

Arm: Neoverse V1 identified by rustc native CPU and CPU part0xd40,64available CPUs,rustc1.98.1. x86: Intel Xeon Platinum8488C,192available CPUs,rustc1.99.0. Both Linux6.12.113-138.209.amzn2023, CPU0 affinity, opt-level3,target-cpu=native. Toolchains differ. Full uname/features/compiler/environment and assembly retained externally.

Warm synthetic block measurements. Startup, cold-cache behavior, I/O, production codecs, peak RSS, service contention and SIMD comparisons remain unmeasured. Timing distributions and A/A limit inference. Byte savings are exact observations for this format and corpus.

Replay: python3 measurements/run.py from the topic directory. The runner prints a task-owned evidence path. Raw archives and member manifests are retained at the receipt paths.
