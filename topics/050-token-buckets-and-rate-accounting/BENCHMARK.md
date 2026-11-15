# Benchmark decision: true

Compare three equivalent serial strict accounting representations. Input:
N=32/32768 attempts, F=1/4096 allocated flows, burst/steady/idle/weighted arrivals.
Capacity 10 and rate 3 are fixed. Burst time=floor(round/1024), steady=round,
idle=100*round, weighted=floor(round/2), round=floor(index/F). Weighted costs
cycle 0,1,6,11,4,10,3; other costs are 6. Flow=index modulo F.

Boundary: allocate and initialize state, consume events, calculate a wrapping
checksum, retain state through black_box, and free state. Exclude process
startup, input generation, oracle, full-output construction, and printing.
256 complete replays per small process or 16 per large process stabilize timing;
only fresh processes are samples. CPU caches are warm; logical meters start full.

One complete-output correctness process and one warmup per candidate/cell.
Six process blocks rotate candidate order and reverse the last three blocks.
Every candidate occupies each position twice. One candidate wins only when all
six paired times beat both alternatives by more than 5 percent. Retain medians,
ranges, and unresolved cells. Six A/A pairs screen same-candidate variation.
Stop after this schedule. This is a descriptive screen, with no confidence or
universal-optimum claim. Record hardware/compiler differences as confounders.

Every process digest matches an arbitrary-precision accepted-cost ledger.
Complete decision/remaining-credit vectors are compared in correctness processes.
The ledger's prefix-envelope computation is cross-checked with direct accepted
suffix scans on the first 128 events. Rust tests independently scan accepted
suffixes across boundary/extreme/random contracts. Incorrect candidates are
excluded. Input/source/runner/binary hashes and machine identity are retained.

State storage is size_of(candidate)*F (Vec metadata/allocation overhead excluded).
Inspect emitted assembly for wide arithmetic/division; generated code establishes
code shape, while hardware causal attribution requires separate measurement.

Replay on Linux. The runner works in the topic directory that holds it:

```sh
python3 benches/run.py --source "$(git rev-parse HEAD)"
```

`--source` takes a 40-digit lowercase commit id. The runner hashes src/lib.rs,
tests/contracts.rs, examples/running.rs, benches/replay.rs and benches/run.py
into environment.json. In a checkout, where Git tracks these five inputs, each
must equal its blob at the source commit (`source_check: git`). In an extracted
frozen-source archive the runner records the id as given (`source_check: label`).
An INPUTS.json in the topic directory, which a frozen-source archive carries,
must equal the five computed hashes. The runner requires enabled assertions and
creates raw/ itself: archive raw/ outside the repository and remove it before
another run. Raw output and host artifacts live outside Git and are linked by
measurements/EVIDENCE_RECEIPT.json.
