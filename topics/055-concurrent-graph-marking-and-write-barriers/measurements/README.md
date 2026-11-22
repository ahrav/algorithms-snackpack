# Bounded correctness replay

Source commit `1032dcac11abdf15c7c3cbb570c9d9ed5e126652`. Both native Linux hosts pass all three Rust
contracts and produce byte-identical reports for all 15 configurations.
Inputs, executables and reports are hash-verified in the external archives.

The runner uses Rust 2024, warnings as errors, optimization and native target
features. It compiles the library, CLI and test harness directly with rustc.
Run the Cargo commands in the [lesson](../README.md) to reproduce the counts.

| Heap | Policy | Distinct states | Termination points | Live misses | Snapshot misses | Extra retained vs. current live |
|---|---|---:|---:|---:|---:|---:|
| Fork | paused | 19 | 1 | 0 | 0 | 0 |
| Fork | unguarded | 11940 | 836 | 11 | 330 | 322 |
| Fork | insertion | 12495 | 825 | 0 | 319 | 319 |
| Fork | snapshot | 18163 | 506 | 0 | 0 | 270 |
| Fork | shared-only | 12495 | 5341 | 3911 | 4835 | 1165 |
| Cycle | paused | 12 | 1 | 0 | 0 | 0 |
| Cycle | unguarded | 7950 | 800 | 19 | 299 | 280 |
| Cycle | insertion | 8459 | 781 | 0 | 280 | 280 |
| Cycle | snapshot | 19681 | 501 | 0 | 0 | 246 |
| Cycle | shared-only | 8459 | 5005 | 3331 | 4504 | 863 |
| Disconnected | paused | 6 | 1 | 0 | 0 | 0 |
| Disconnected | unguarded | 266 | 56 | 0 | 14 | 14 |
| Disconnected | insertion | 266 | 56 | 0 | 14 | 14 |
| Disconnected | snapshot | 354 | 42 | 0 | 0 | 14 |
| Disconnected | shared-only | 266 | 182 | 98 | 140 | 14 |

Live and snapshot misses count termination points with at least one missing
object, not object totals. The paused baseline is exact. Insertion with
accounted termination covers current reachability. Deletion with accounted
termination covers the initial snapshot and retains some expired objects.
Shared-only termination fails even with the insertion barrier. The disconnected
case constrains the available mutations, so the unguarded failure mainly
appears in the fork and cycle cases. These are model results for a bounded heap.

See [the evidence receipt](EVIDENCE_RECEIPT.json) for source and archive identity.
