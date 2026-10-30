# Benchmark decision

benchmark_decision: true

Claim: compare correct immediate-parent builders and insertion-stream maintainers
under the explicit output/state contract, graph families and compiler controls.
Static candidates: set fixed point plus parent extraction, CHK, identical CHK
control. Dynamic: full-set rebuild, cached-set worklist, CHK rebuild plus set
expansion. Dynamic candidates clone the prepared graph/cache, apply8 insertions
and extract final parents. Each edit retains a complete exact set matrix.

Static workloads: tiny7; chain512; permuted chain256 (adversarial ID scan order);
joins512; cyclic512; dense128; disconnected512. Dynamic512 workloads:8 redundant
self-loop insertions;8 bypass edges from root to vertices256..263; newly reachable
tail with0->256 followed by7 duplicates. Fixed input/output representation.

Freeze source and runner before each campaign. Rust release,-C opt-level=3,
-C target-cpu=native, one CPU chosen from the allowed affinity mask. Six paired
fresh-process blocks alternate forward/reverse candidate order,3:3 each pair.
One discarded process per candidate/cell precedes them. Repetition counts are
fixed in run.py, not adaptive. Independent oracle and one candidate preflight
warm the input before timing. Cold-cache and first-call claims are unmeasured.

Timing includes graph preparation, allocation, scans, output and drop for the
candidate's complete operation. Input generation, oracle and initial cached sets
are outside. Process wall time is also retained, separately from kernel time.
Inner iterations amortize clock overhead; the process is the independent unit.

Stop after6 blocks. Report median and full observed process range. Select a
candidate only if it is at least5% faster than every distinct rival in every
paired block. Otherwise report unresolved plus the median leader. CHK's identical
control is excluded from selection and retained as a descriptive noise check.
All candidates must pass the oracle before performance selection.

Counters count vertex visits, incoming edges and word/chain steps for one
operation outside the timer. Assembly is retained for static inspection.
Allocation counts, RSS, hardware events, population confidence and real compiler
CFG distributions are unmeasured. Measured comparisons establish no mechanism
from whole-process counters that contain the oracle. No universal optimum claim.
