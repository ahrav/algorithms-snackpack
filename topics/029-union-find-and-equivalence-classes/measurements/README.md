# Exact committed-source results

Frozen executable source: `5eebe81dae94a91ec55978cc43268f88804bf00b`. Four source/runner/test hashes match
both hosts and the Git commit. Per host: 288 decision processes, 16 A/A processes,
36 discarded warmups, six library and two external test groups. Every accepted
process passes the independent graph-search oracle. Final pair ordering is 4:4.
Initial imbalanced campaigns remain exploratory and are excluded from selection.

| Workload | Arm fastest median, us (range) | x86 fastest median, us (range) |
|---|---|---|
| tiny | labels 0.709 (0.560..0.899); selected | labels 0.773 (0.729..0.805); selected |
| small | half 17.105 (16.846..21.905); selected | half 20.678 (20.367..21.066); selected |
| mixed | size 88.272 (87.561..92.742); unresolved | size 99.919 (99.201..100.698); selected |
| reads | size 30.023 (29.833..31.945); selected | size 35.703 (35.464..35.906); selected |
| unions | half 100.479 (99.855..101.386); selected | half 110.829 (110.649..112.108); selected |
| local | half 42.920 (42.221..45.313); selected | full 49.846 (48.881..56.270); unresolved |
| duplicates | labels 21.947 (21.268..22.209); selected | labels 14.661 (14.610..14.766); selected |
| first | labels 12.759 (11.822..13.385); selected | labels 17.432 (11.493..36.337); selected |
| repeat | labels 10.664 (10.341..13.630); selected | labels 10.384 (10.360..10.474); selected |

Eight process observations per candidate/cell. Ranges describe dispersion, rather
than confidence intervals. A selection requires every paired rival/fastest ratio
to exceed 1.03. See `summary.json` for every candidate, setup and teardown medians.

The label first-query preparation medians are 6584.801 us on Arm and 3383.303 us on x86.
Halving preparation medians are 25.765 us and 22.686 us. Query-only winners therefore
need an amortization decision. The experiment makes no end-to-end build winner
claim from query-only timing.

Arm host: `dev-dsk-ahrav-2b-7dc7bd93.us-west-2.amazon.com`, aarch64,
Linux 6.12.111-136.204.amzn2023.aarch64, ARM model 1 stepping r1p1, 64 online CPUs.
x86 alias `xxl` resolved to `dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com`,
x86_64, Linux 6.12.111-136.204.amzn2023.x86_64,
Intel Xeon Platinum 8488C, 192 online CPUs. Rust 1.98.1 / LLVM 22.1.8 on Arm;
Rust 1.99.0 / LLVM 23.1.1 on x86. Build: `--edition 2024 -O -C target-cpu=generic`.
Exact feature/cpuset/CPU/kernel/toolchain reports and binaries remain in archives.
The unpinned hosts and different toolchains limit cross-host interpretation.

Halving A/A paired ratio range: Arm 0.939010..1.025330, x86 0.995717..1.011655.
Tiny timing includes clock/call/output overhead. First traversal is prepared,
with uncontrolled cache state. Working sets stop at 4096 items. No host-wide,
cache/branch-counter, allocation-count, RSS, latency-tail or universal claim.

Inspected the three linked forest find symbols on each host. Size has a load/branch
traversal. Full has a traversal followed by a parent-store loop. Halving loads a
grandparent and stores it during traversal. This establishes generated code shape;
it does not establish which hardware event caused a timing difference.

`EVIDENCE_RECEIPT.json` records archive path/hash/bytes/member count,
manifest/binary/environment hashes and source identity. Raw evidence is outside
Git at `/Users/ahrav/.codex-amazon/automations/algorithms-daily-curriculum/evidence/topic-029/20261025`. The retained runner and scoped sources reproduce the experiment.
