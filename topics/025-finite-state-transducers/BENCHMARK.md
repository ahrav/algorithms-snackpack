benchmark_required: true

Predeclared before timing: four correct candidates, eight cases, eight mirrored cyclic process
blocks. Candidate order rotates flat/trie/final/pushed and reverses in blocks 4..7; workload
order alternates. Each block also runs identical flat A/B processes in alternating order,
through the same parser and source build. Each case/candidate therefore has eight independent
process observations. Fixed stopping, no significance or family-wide confidence claim. Seedless
deterministic key generation and hashed exact runner. A resolved winner must have lowest median
and every paired rival/winner ratio >1.03 across all eight blocks. Otherwise retain unresolved
rivals. A/A ratios describe noise, never establish significance.

Timing is warm batch elapsed/operations, excluding process startup, input generation,
independent oracle and validation, index build for query cases, and index teardown. Three full
warmup batches;250 timed point-query sweeps,400 narrow prefix calls,40 broad output calls,4
builds. Build includes normalization, full trie, registry, result allocation, one lookup, and
drop. Query prefix includes owned output allocation/consumption/drop. Small case 8 keys;
others 2048. Shared-tail shape, high entropy keys, random outputs, early miss,8-result
prefix,2048-result broad output, and complete build. Same fixed datasets per
host/candidate/process. Miss byte 0xff absent at root. Values additive correlated versus
unrelated wrapping 64 bit. Controls are unpinned with recorded affinity, no host changes; cross-
host compiler versions differ. Warm results do not describe cold/page fault, tails, production,
or ISA causality. No PMU mechanism claim. Structural counts and retained vector-capacity bytes
exclude stack, index struct, allocator metadata, RSS, retained input and peak builder state.
Postorder registry is complete, no bounded-cache minimality assertion. Replays rerun frozen
source/runner with hashes, preserving all previous samples.
