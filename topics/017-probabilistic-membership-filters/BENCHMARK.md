# Predeclared experiment

benchmark_required: true

Question: When does a correct membership sidecar reduce complete exact-lookup cost?
Candidates: direct sorted exact,bloom,word-blocked,cuckoo,static quotient runs.
Selection: must beat every rival by>5% in each of10 matched process blocks;
otherwise unresolved among candidates not consistently beaten. Descriptive full
ranges retained. No confidence interval or universal optimality claim.

Eight frozen cases: tiny32 misses;4096 misses;4096 90% hits;131072 misses;131072
90% hits;4096 repeated single-key miss;4096 generation build;4096 first measured
10%hit batch.8192 queries/batch. Query cases12 repetitions except first_batch1.
Two untimed warmups except first_batch; all validation runs before timing.
Build4 repetitions,includes exact-key clone allocation/copy/drop and additional
sidecar allocation/build/drop. Authority keys already sorted; input sorting and
oracle construction excluded. Query includes hashes,filter,exact binary search,
result counting/assertion; allocation,build and teardown excluded. Direct baseline
always validates. The runner labels baseline absent admissions as checks,not its
own membership errors.

Balanced assignment: five cyclic candidate orders followed by their reversals,
10 independent fresh processes/candidate/case,400 total/host. Host runs can overlap
across machines; candidate runs on one host are serial. Fixed workload seed/layout,
opt-level3,target-cpu=native,edition2024. No affinity governor or cache-flush controls.
No reruns to manufacture separation. Exact committed replay is a required second
campaign,not a search for favourable data.

Build and query units differ (ns/input key versus ns/query). All candidates must
pass complete admission/member checks and independent BTreeSet exact-answer oracle
before timing. Fingerprint/deletion/admission regressions checked in lib tests.
A generated assembly artifact is retained; no counters were collected,no
candidate-only cache/branch/ISA attribution is made. Keys are trusted and immutable.
