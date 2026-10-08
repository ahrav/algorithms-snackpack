# Replay and evidence

Frozen source commit c91029179f5c4b6c59a0f490a528f5c63af9b914 and unchanged exact runner passed eight checks/example and432 processes on EACH Linux host. Initial scratch also ran432 each. EVIDENCE_RECEIPT.json binds hashes/archives/environments; raw data and exact replay archive remain at /Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-009/20261005. Source uses standalone rustc opt-level3, compiler-default target features, first available CPU affinity. Workspace Cargo gates pass locally. See RESULTS.md and summary.json for every winner, unresolved rival and interval.

Post-replay source changes: commit 36ddffd clamps combine's heap capacity to the retained key count and adds two combine regression assertions. src/lib.rs and tests/contracts.rs no longer match the frozen manifest hashes; EVIDENCE_RECEIPT.json records both the frozen and current hashes under post_replay_changes. combine is absent from the measured bench and example binaries. Replaying this tree against the archived source-manifest.json fails campaign.py's hash assertion by design.

Known limitation: with two methods per weighted cell, the rotate-then-reverse block assignment in campaign.py yields the same wsort-then-wheap order in all 12 blocks. Weighted contrasts are confounded with fixed within-block process order. Uniform cells rotate through four orders. See RESULTS.md.

To replay: extract final-replay.tar.gz in a new declared Linux scratch directory and run python3 campaign.py. To regenerate summaries, run python3 summarize.py DIRECTORY containing arm-attempts.jsonl and xxl-attempts.jsonl. Weight setup is excluded, sampler/math/allocation/ID output/checksum/destruction included; process wall separate. No source decoding or communication claim.
