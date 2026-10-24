# Committed-source results

Source `3f8125cc809aacd68d6368212eef5ea7495bd107`. Library, benchmark and exact runner hashes match both hosts. All384processes/host completed; four library groups and every pre-timing oracle check pass. Preliminary384processes/host are retained separately and never pooled.

Times below are process medians in nanoseconds; point cases are per lookup, prefix/broad per complete owned-output call, build per complete lifecycle. All 8 process ranges, paired ranges and A/A ranges are in summary.json.

| Workload | Arm: flat / trie / final / pushed | Arm selection | x 86: flat / trie / final / pushed | x 86 selection |
|---|---|---|---|---|
| tiny | 19.2 / 71.9 / 79.8 / 71.0 | flat | 34.1 / 51.8 / 51.8 / 51.8 | flat |
| shared | 103.7 / 136.6 / 139.1 / 123.3 | flat | 136.8 / 87.2 / 87.7 / 81.4 | pushed |
| entropy | 97.0 / 313.1 / 301.8 / 297.2 | flat | 87.3 / 337.4 / 361.6 / 355.4 | flat |
| random-values | 101.8 / 137.2 / 140.3 / 123.0 | flat | 137.0 / 87.2 / 87.7 / 81.7 | unresolved: pushed / trie,final |
| miss | 41.1 / 5.4 / 5.5 / 5.2 | unresolved: pushed / trie,final | 127.9 / 3.0 / 4.0 / 3.3 | unresolved: trie / final,pushed |
| prefix | 339.5 / 488.3 / 483.4 / 482.4 | flat | 323.3 / 441.5 / 443.9 / 444.1 | flat |
| broad | 84270.9 / 124893.0 / 125544.6 / 112698.6 | flat | 234293.4 / 104574.2 / 104406.3 / 98662.6 | pushed |
| build | 350113.5 / 1259373.2 / 2367305.6 / 1180080.4 | flat | 290865.9 / 1144834.0 / 2026166.9 / 1009408.5 | flat |

Arm vector wins every resolved hit/output/build case; x 86 pushed wins shared-tail hits and broad output, vector wins tiny/entropy/narrow/build. x 86 random-values and graph early misses remain unresolved. No candidate is universally best.

Shared/correlated: trie 5907 states/5906 arcs versus pushed 20 states/56 arcs. Random outputs yield 140 pushed states; high-entropy keys yield 57890 versus 63761 trie states. Retained capacity is not RSS, actual serialized bytes or peak build memory. All candidates keep caller-owned input outside this count.

A/A ratios span0.918..1.163 on Arm and0.929..1.057 on x86. Results are descriptive and unpinned;3% is a predeclared practical rule, not a significance threshold. Different Rust/LLVM versions prevent ISA-only attribution. Source has no dependency on fst0.4.7 and does not test its performance.

Codegen: actual linked get bodies retain byte comparisons, fanout selection, target loads, checked accumulation and acceptance. Flat branch calls memcmp. The first symbol extractor incorrectly assumed a demangled spelling; the discovered label is `<algorithms_fst::Index>::get`. Two failed extraction attempts retained in external notes; corrected extraction does not rerun or alter timings.

Raw archives, identity, complete samples, assembly, prior campaign, mutations, validation and audit: `/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-025/20261021`. EVIDENCE_RECEIPT.json records archive/source/binary hashes.

Reproduce on Linux from topic root: `python3 run.py`; standalone smoke: build/run portfolio with no arguments. Existing raw output should be retained or run in a fresh exported source directory. Target flags are `-O -C target-cpu=native`, edition2024. Tests/oracle/warmup precede timing. Startup and first-touch/cold requests are not sampled. Build includes drop; point teardown is outside the query boundary; prefix output allocation/drop is inside.

No extra timing is required to complete this round. Revisit cold/late-miss/tail profiles, entropy/size crossover sweeps and packed formats rather than inferring them from these cells.
