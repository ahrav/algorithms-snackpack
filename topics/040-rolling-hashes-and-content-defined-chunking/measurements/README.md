# Native results

All 556 processes per host passed original and edited boundary-vector oracles: 480 measured, 60 warmup, 16 A/A. Both hosts pass all 3 native contract groups. Raw data and compact receipt are retained separately.

Source commit: d01ae3cc1405096664e93b0c80eef66c47ccb946. All 5 native inputs match that commit; `EVIDENCE_RECEIPT.json` records their hashes. Review fixes after measurement changed `src/lib.rs` (`fixture` rejects unknown shape names before generating bytes), `benches/chunk.rs` (Cargo's `--bench` argument is filtered; `max = target * 4` uses checked arithmetic and the config panic names its constraint), `tests/contracts.rs` (added tests) and `measurements/run.py` (failed attempts are retained, results are validated before they enter a record, a member manifest is written). The timed `boundaries` function and the bench's warmup and timed loops are byte-identical to the measured source.

## arm: dev-dsk-ahrav-2b-7dc7bd93.us-west-2.amazon.com

Architecture aarch64; CPU affinity 0; available CPUs 64; compiler rustc 1.98.1 (48a229cea 2026-09-01). Native opt-level 3, edition 2024. Full uname, CPU identification and target features are retained. Warm state; frequency and interference uncontrolled.

| Bytes | Target | Shape | Clear speed winner | Polynomial choice |
|---:|---:|---|---|---|
| 256 | 256 | random | fixed | rolling |
| 256 | 256 | zero | fixed | unresolved |
| 256 | 256 | periodic | fixed | rolling |
| 256 | 1024 | random | fixed | direct |
| 256 | 1024 | zero | fixed | direct |
| 256 | 1024 | periodic | fixed | direct |
| 65536 | 256 | random | fixed | rolling |
| 65536 | 256 | zero | fixed | unresolved |
| 65536 | 256 | periodic | fixed | rolling |
| 65536 | 1024 | random | fixed | rolling |
| 65536 | 1024 | zero | fixed | direct |
| 65536 | 1024 | periodic | fixed | rolling |

Direct/rolling are output-equivalent; other policies differ. Fixed has the smallest median in all 12 cells on this host. Clear winner requires >5% in every paired block. Unresolved means the screen did not select a winner.

A/A rolling A/B ratio range: 0.916..1.093. This variation limits fine-grained claims. Results are exploratory screens without inferential error control.

| Policy | Random 65,536, target 256 median us | Full 8-process range us |
|---|---:|---:|
| fixed | 67.057 | 66.375..67.265 |
| direct | 982.028 | 981.375..1133.606 |
| rolling | 157.368 | 156.743..158.695 |
| gear | 116.858 | 116.085..117.736 |
| normalized | 116.141 | 115.348..116.829 |
## x86: dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com

Architecture x86_64; CPU affinity 0; available CPUs 192; compiler rustc 1.99.0 (b940084d7 2026-09-28). Native opt-level 3, edition 2024. Full uname, CPU identification and target features are retained. Warm state; frequency and interference uncontrolled.

| Bytes | Target | Shape | Clear speed winner | Polynomial choice |
|---:|---:|---|---|---|
| 256 | 256 | random | fixed | rolling |
| 256 | 256 | zero | fixed | unresolved |
| 256 | 256 | periodic | fixed | rolling |
| 256 | 1024 | random | unresolved | direct |
| 256 | 1024 | zero | unresolved | direct |
| 256 | 1024 | periodic | unresolved | direct |
| 65536 | 256 | random | fixed | rolling |
| 65536 | 256 | zero | unresolved | unresolved |
| 65536 | 256 | periodic | fixed | rolling |
| 65536 | 1024 | random | fixed | rolling |
| 65536 | 1024 | zero | unresolved | direct |
| 65536 | 1024 | periodic | fixed | rolling |

Direct/rolling are output-equivalent; other policies differ. Fixed has the smallest median in all 12 cells on this host. Clear winner requires >5% in every paired block. Unresolved means the screen did not select a winner.

A/A rolling A/B ratio range: 0.934..1.119. This variation limits fine-grained claims. Results are exploratory screens without inferential error control.

| Policy | Random 65,536, target 256 median us | Full 8-process range us |
|---|---:|---:|
| fixed | 85.722 | 83.814..105.704 |
| direct | 702.425 | 697.457..908.077 |
| rolling | 143.222 | 142.460..175.823 |
| gear | 121.700 | 117.294..123.947 |
| normalized | 124.112 | 123.365..125.833 |

## Reuse and metadata

Insert 17 bytes at 21,845 into the 65,536-byte random object at target 256 (min 64, max 1,024). Reuse is exact-byte membership in the original chunk set, the content-addressed model in which identical chunks share one stored copy, divided by 65,553 edited bytes. On this random input every original chunk is distinct, so multiset-bounded reuse gives the same bytes. Metadata is c*r bytes for c original chunks at the illustrative r=48 from README.md. Both hosts agree.

| Policy | Original chunks | Min/max bytes | Reused bytes | Reused percent | Metadata at r=48 |
|---|---:|---|---:|---:|---:|
| fixed | 256 | 256/256 | 21760 | 33.195% | 12,288 |
| direct | 221 | 65/1024 | 65287 | 99.594% | 10,608 |
| rolling | 221 | 65/1024 | 65287 | 99.594% | 10,608 |
| gear | 181 | 65/1024 | 65189 | 99.445% | 8,688 |
| normalized | 191 | 65/1024 | 65227 | 99.503% | 9,168 |

## Work and generated code

Derived counts: rolling performs 65,536 hash appends and 32 power-setup steps at most; an outgoing removal applies after each chunk's first 32 bytes. Direct hashes 32 bytes per eligible position, skipping positions below min and the forced max. Zero input at target 256 cuts every 64 bytes: 1,024 boundary tests require 32,768 direct hash-byte steps versus 65,536 rolling appends. This explains a plausible workload-dependent advantage; timings establish only the measured outcome.

Native linked-code inspection: Arm boundaries symbol has 242 disassembly lines; x86 has 319. Both lower multiplication by 257 to shift/add forms. The x86 outgoing-contribution path includes imul, sub and shift/add; Gear paths load a table entry and add to shifted state. Full linked disassembly and symbol excerpts are retained. These observations support the implemented recurrences; they do not establish a hardware bottleneck or a cross-architecture causal comparison.

## Selection and limits

Use rolling to preserve polynomial boundaries on random or periodic inputs. The all-8-block screen selects rolling in 6 cells per host, direct in 4, and leaves 2 unresolved. Direct wins the small target 1024 cells and the large zero target 1024 cell because few eligible windows need recomputation. Fixed is the measured speed choice where its offset-based contract fits. Gear and normalized Gear have similar reuse on this fixture with different counts and timing; real corpus and storage costs remain required before selecting a production policy.

Both hosts use different toolchains (Arm 1.98.1, x86 1.99.0). Data is synthetic, warm and CPU 0 pinned. Startup, cryptographic digest, disk/network I/O, persisted indexes, streaming fragmentation, production Rabin/FastCDC and cold-state performance remain unmeasured. The normalized Gear format here hashes all bytes and is an educational policy.

Per-cell medians and eight-process ranges for the other 11 cells per host live in the retained `results.jsonl` archives named in `EVIDENCE_RECEIPT.json`; this summary reports one cell per host in full.
