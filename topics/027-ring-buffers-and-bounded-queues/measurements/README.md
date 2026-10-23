# Exact committed-source measurements

Source commit `fbebe6a17c6946742954d0cc9468a6441f78dadf`. Both required hosts independently ran all 490 fresh
processes, four library groups, one public integration test and the pictured example. Every
warm/timed checksum and final state agrees with the ordered-map oracle. Initial and committed
campaigns total 1960 process observations and are reported separately, without pooling.

## Results

Rates are ns per attempted push or pop. Ranges are observed process min/max, not confidence
intervals. Selection requires every paired rival/best ratio >1.03 in all ten blocks.
The selected name is descriptive under this protocol; close rivals remain unresolved.

| Host / case | Lowest median [min,max] | Selection |
|---|---|---|
| arm / tiny | modulo: 2.245 [2.223, 2.302] | unresolved: modulo / mask |
| arm / steady_power2 | modulo: 2.185 [2.166, 2.202] | unresolved: modulo / mask |
| arm / steady_odd | mask: 2.193 [2.187, 2.203] | unresolved: mask / modulo |
| arm / burst | mask: 2.379 [2.373, 2.875] | unresolved: mask / modulo,branch |
| arm / large | modulo: 2.223 [2.202, 2.265] | unresolved: modulo / mask |
| arm / full | branch: 0.942 [0.940, 1.116] | unresolved: branch / modulo,mask |
| arm / lifecycle | modulo: 2.482 [2.421, 2.574] | unresolved: modulo / mask |
| x86 / tiny | mask: 1.632 [1.630, 1.646] | mask |
| x86 / steady_power2 | mask: 1.621 [1.618, 1.717] | mask |
| x86 / steady_odd | mask: 1.621 [1.618, 1.629] | mask |
| x86 / burst | mask: 1.715 [1.632, 1.908] | unresolved: mask / deque,modulo,branch |
| x86 / large | mask: 1.720 [1.675, 1.794] | unresolved: mask / branch |
| x86 / full | branch: 1.251 [1.251, 1.254] | unresolved: branch / modulo,mask |
| x86 / lifecycle | mask: 1.843 [1.827, 2.148] | unresolved: mask / modulo,branch |

All candidate samples, min/max, medians, paired ratios, checksums and controls are in
[summary.json](summary.json). Mask is selected only for x86 tiny/steady_power2/steady_odd.
All seven Arm cells and the other four x86 cells retain unresolved rivals. The large shifting
baseline has median 2584.199 on Arm and 41057.894 on x86, compared with about 2 for rings.

## Controls and limits

- arm: identical-deque A/A ratio range 0.406..2.631; retained without exclusion.
- x86: identical-deque A/A ratio range 0.922..1.200; retained without exclusion.

Timing includes dispatch, admission, moves, metadata and checksum. Steady setup/preload,
oracle and final verification/destruction are excluded. Lifecycle includes each construction
and destruction. Results are warm and unpinned. Arm rustc1.98.1 has 64 available CPUs; x86
rustc1.99.0 has 192 on Intel Xeon Platinum8488C. Both use -O -C target-cpu=native. The exact
kernel, affinity, target cfg/features and binary/source hashes are retained in each identity.
The Arm CPU reports implementer0x41/part0xd40/variant1/revision1 (MIDR0x411fd401), rather than
a marketing model string. Do not infer an ISA-only cause from these different systems.

On both builds size_of(u64)=8 and size_of(Option<u64>)=16. Ring backing storage therefore
models P*16 bytes: C4 uses64, C1000 remainder/conditional use16000, mask P1024 uses16384.
This excludes the container header, allocator overhead and nested payloads. Steady operations
request no growth by source contract: Vec/VecDeque have reservation and checked admission;
the ring never resizes. Allocator call counts, peak RSS and actual traffic were not measured.
For the shifting baseline, successful pops model (n-1)*8 relocated bytes at pre-pop occupancy n.

Five specialized execute bodies were inspected in each exact linked binary. Division
instructions occur in the remainder bodies and none in mask bodies. The Arm conditional
body also contains a division; function-wide counts are not per-operation counts. No hardware-counter or candidate-only causal speedup claim is made. Raw disassembly
and inspected symbol/body hashes remain external.

## Replay and retained evidence

Run `python3 run.py` in this topic directory on Linux. The script records identities, tests,
example, raw samples and linked disassembly under raw/. A compiler/toolchain mismatch is a
new experiment; use the recorded environment when replaying a result. Raw data and frozen
source archives live at:

/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-027/20261023

[EVIDENCE_RECEIPT.json](EVIDENCE_RECEIPT.json) records each verified archive and source hash.
The raw bundles are outside Git. Four compiled semantic mutants were rejected. Workspace
format, library/example tests, doc tests, Clippy, bench build, rustdoc, all-target tests and
whitespace gates pass. [Plugin audit](../PLUGIN_AUDIT.md) records justified existing coverage.
