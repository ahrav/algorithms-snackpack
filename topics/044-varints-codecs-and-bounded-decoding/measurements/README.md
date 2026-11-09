# Obtained measurements

Frozen source: 7c35370523cefbc65aec9a46f4d5e47b409da73b.
Both required Linux hosts ran the same three committed inputs. The receipt records
their input, binary, environment, manifest and raw-archive identities.
Each host produced 144 measured main samples, 24 warmup samples and 12 A/A samples.
Three Rust contract groups and all eight independent Python encoding checks passed.

Values below are median microseconds per complete decode, followed by observed
minimum..maximum. Six fresh processes per candidate/cell. These ranges describe
dispersion; they are not population confidence intervals. Choices use the frozen
all-six-pairs 5% rule. Fixed width changes the representation.

## arm

| N / distribution | Scalar us | Unrolled us | Fixed32 us | Selection | Varint / fixed bytes |
|---|---:|---:|---:|---|---:|
| 32 / small | 0.0425 (0.0344..0.0434) | 0.0325 (0.0324..0.0336) | 0.0407 (0.0406..0.0427) | unrolled | 32 / 128 |
| 32 / threshold | 0.0627 (0.0624..0.0628) | 0.0618 (0.0616..0.0620) | 0.0406 (0.0406..0.0426) | fixed | 96 / 128 |
| 32 / full32 | 0.0940 (0.0933..0.0952) | 0.0917 (0.0916..0.0918) | 0.0406 (0.0406..0.0423) | fixed | 157 / 128 |
| 32 / alternating | 0.0729 (0.0720..0.0749) | 0.0687 (0.0677..0.0710) | 0.0406 (0.0406..0.0408) | fixed | 96 / 128 |
| 65536 / small | 77.1987 (75.8697..80.6323) | 66.2588 (66.2045..66.6707) | 85.2860 (85.1619..85.6447) | unrolled | 65536 / 262144 |
| 65536 / threshold | 128.0613 (127.9167..130.7228) | 125.9020 (125.6477..126.1961) | 85.2752 (85.1577..88.3116) | fixed | 196608 / 262144 |
| 65536 / full32 | 200.9862 (200.6717..203.0674) | 202.3647 (201.9289..203.5582) | 85.3109 (85.1707..85.4407) | fixed | 323623 / 262144 |
| 65536 / alternating | 122.8966 (122.7540..123.1166) | 119.1722 (118.8625..121.4962) | 85.4464 (85.2212..88.3951) | fixed | 196608 / 262144 |

A/A scalar time ratio range: 0.982..1.022.

## x86

| N / distribution | Scalar us | Unrolled us | Fixed32 us | Selection | Varint / fixed bytes |
|---|---:|---:|---:|---|---:|
| 32 / small | 0.0435 (0.0432..0.0442) | 0.0326 (0.0325..0.0328) | 0.0338 (0.0337..0.0340) | unresolved | 32 / 128 |
| 32 / threshold | 0.0613 (0.0598..0.0629) | 0.0696 (0.0663..0.0697) | 0.0338 (0.0337..0.0340) | fixed | 96 / 128 |
| 32 / full32 | 0.0870 (0.0854..0.0898) | 0.0847 (0.0847..0.0848) | 0.0339 (0.0337..0.0352) | fixed | 157 / 128 |
| 32 / alternating | 0.0579 (0.0552..0.0598) | 0.0627 (0.0609..0.0632) | 0.0337 (0.0336..0.0342) | fixed | 96 / 128 |
| 65536 / small | 82.2730 (82.2085..82.4251) | 61.9557 (61.5365..62.3644) | 72.9228 (72.4935..76.1351) | unrolled | 65536 / 262144 |
| 65536 / threshold | 117.1674 (115.4885..121.2013) | 130.8987 (128.4133..137.1789) | 72.4845 (72.4142..73.2320) | fixed | 196608 / 262144 |
| 65536 / full32 | 212.1676 (201.2492..231.0746) | 224.7093 (212.4357..245.7747) | 72.9588 (72.4611..82.6137) | fixed | 323623 / 262144 |
| 65536 / alternating | 115.1509 (108.5111..126.6187) | 116.8670 (112.7997..122.8502) | 72.9511 (72.3447..74.2449) | fixed | 196608 / 262144 |

A/A scalar time ratio range: 1.003..1.072.

## Selection and limits

Unrolled wins the large small-value cell on both hosts and the tiny small-value
cell on Arm. Tiny small-value x86 is unresolved under the paired rule. Fixed32
wins the other six cells on each host. Scalar remains the readable reference.
For 65536 small values, varints use 65536 bytes versus 262144 fixed bytes.
For full-width pseudorandom values, read results.json for the exact encoded size.
The results show a workload-dependent time/size decision among tested candidates.

Warm, repeated, single-threaded synthetic buffers pinned to CPU 0. Arm rustc 1.98.1
and x86 rustc 1.99.0 differ. CPU, features, kernel and availability are in the
external environment.json. Allocation, input encoding, setup, startup, teardown,
storage and network are outside timing. SIMD/block codecs, actual cold-cache
behavior, real corpora, parser integration and encode performance require revisits.
Elapsed times support the ranking. They do not attribute branches, caches or ISA causes.
Six-process findings are exploratory; A/A variation remains visible.

## Replay and external evidence

Run `python3 run.py` from this directory on Linux with rustc, python3, lscpu and taskset.
The runner supports both repository-relative and standalone layouts. It verifies
all candidates before each timed process and prints its raw evidence directory.
Raw bundles are retained outside Git at
`/Users/ahrav/.codex-amazon/automations/algorithms-daily-curriculum/evidence/topic-044/20261109/`.
The EVIDENCE_RECEIPT.json records local/remote archive and every-member verification.
Compact results live in results.json. Contracts live in src/lib.rs and tests/contract.rs.

## Plugin audit

Source origin/main 304b6c583d73106b4bcaddbd298ec906253cadeb already covers this scope.
The simd-optimize integer-decoding reference specifies format differences, width,
canonicality, input/output capacity, tails, streaming, partial writes, scalar
oracles, malformed cases and measurement dimensions. Compression and wire-schema
owners cover resource bounds and signed mapping compatibility. No material gap
was found. No skill inputs changed; no model evaluation or plugin PR is needed.
Five existing Python asset/native tests passed, including five embedded Rust tests.
These are contract probes, not a model quality result.
