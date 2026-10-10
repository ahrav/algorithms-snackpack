# Measurement evidence

Initial360-process campaign completed locally on Apple M1 Pro, arm64 Darwin25.6.0,
Rust1.93.1, LLVM21.1.8 with native optimization. Required Linux Arm and x86 hosts each
failed SSH plus one retry before executing commands due expired Midway authentication.
There are no remote identities or successful transfers to report. Exact committed-input
replay remains required on both hosts once authentication works.

The initial word mask was selected in tiny/small_miss/wide_miss/large87/cluster/deleted;
small_hit/bad_tag/build_probe/first_batch remained unresolved. Final committed-input results
will be attached after replay. See BENCHMARK.md for frozen ordering, boundary and selection.
Steady-state medians exclude hashing/build/setup. build_probe includes lifecycle work.
First-batch oracle access prevents a cold-cache interpretation. No A/A or PMU attribution.

Raw/source/failure/replay assets are retained externally under:
/Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-014/20261010

Run `python3 measurements/runner.py` from this topic in a fresh working copy.
It compiles library/external tests/example and optimized executable with exact frozen source
hashes, then records360 candidate processes. The query oracle passes before every timed batch.
`python3 measurements/analyze.py raw` checks complete block coverage and reduces paired samples.
No raw evidence or binaries belong in Git. The compact receipt names exact identities,
retained hashes and missing Linux replay actions. Source changes require a fresh campaign.
