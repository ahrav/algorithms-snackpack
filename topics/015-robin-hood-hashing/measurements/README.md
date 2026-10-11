# Measurements and replay

Initial local396-process campaign passes all oracles/checksums. Exact committed
replay is recorded in the following evidence-only commit. Both required Linux
hosts fail expired Midway authentication before execution, including bounded
refresh/retries. No Linux measurements or cross-host winner is claimed.

External evidence root:
`/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-015/20261011`.
Source snapshots, process observations, host attempts, mutation failures and
assembly stay there. Git retains compact identity and result summaries only.

Replay: from this topic directory, `python3 measurements/runner.py` builds exact
source with rustc edition2024,opt-level3,target-cpu=native, runs five library and
two external contract groups plus example, then396 candidate processes. Transfer
only the frozen five paths, verify SHA256, verify required hostname/architecture,
and retain raw before cleanup. Runner refuses to overwrite raw/.
