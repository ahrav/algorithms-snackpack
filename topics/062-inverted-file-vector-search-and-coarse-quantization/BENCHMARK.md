benchmark_required: true

Test how many rows and how much time each correct search path requires when
mean exact ID recall@5 must reach 95%.

| Cell | Rows | Geometry and query distribution |
| --- | ---: | --- |
| tiny | 256 | 16 spaced centroids, uniform group seeds, continuous integer jitter |
| balanced | 16,384 | 16 spaced centroids, uniform group seeds, residuals exactly on the PQ grid |
| boundary | 16,384 | overlapping groups. Queries seeded from groups 0 to 14 sit on the upper adjacent-centroid bisector. Queries seeded from group 15 keep their jittered coordinate (4 of 96 with seed 97) |
| skew | 16,384 | about 90% corpus and 75% query preference for group 0, with jitter |

The seven timed paths are flat, raw one/four probes, PQ one/four probes, and
PQ one/four probes with a 16-ID exact rerank. Untimed controls scan all
lists with raw and compressed scores. The corpus layout interleaves IDs by
group. One linear congruential generator in `benches/search.rs` draws every
cell, seeded with 23 for the corpus and 97 for the queries.
Raw list scoring loads the original coordinates through an ID.
The balanced query cell intentionally gives the fixed codec zero distortion.

Run 16 complete process blocks per host and cell. Rotate eight slots
(including a second flat path) in the first eight blocks and reverse the
rotations for the final eight. Run one warmup block plus one warmup sweep
inside each process. Count process batches as samples. Each timed batch
runs three sweeps of 96 fixed queries, or ten on tiny. Keep the sample count
fixed. Time the full in-memory search path, including allocation, centroid
routing, scanning, heap selection, reranking when enabled, output digest and
dropping results. Record index build separately. Corpus construction, oracle,
process startup, and serialization stay outside the query clock. Each process
builds the same combined index so the build number describes that index.

A winner must meet the recall floor, beat every other eligible path by at least
1.05x median paired ratio, and win at least 13 of 16 paired blocks against each.
Require the same-mode control's median ratio between 0.95 and 1.05 for a
resolved result. Report process median, middle two quartile bounds, scan counts
and low-tail query recall. A sole eligible path meets the performance
comparison by default subject to the control. Measure query time on resident
synthetic data and capture host flags and generated assembly. The experiment
does not attribute time to a specific CPU event or model cold I/O and training.

The count of 16 blocks is fixed with no interim looks. No pilot variance
estimate is recorded, so the block count carries no sensitivity or power claim.
The winner rule is a descriptive engineering criterion with no nominal
confidence level, confidence bound or population inference. Under independent
null signs, 13 or more wins in 16 blocks has one-sided probability 697/65,536
(about 0.011) for one pairwise contrast. No error budget is allocated across
contrasts. Seven paths give up to 21 pairs in each of four cells on two hosts,
and a Bonferroni bound over those 168 contrasts exceeds one, so winner labels
carry no family-wise guarantee. The runner sets no timeout. A process that
exits non-zero or returns a digest different from the untimed run stops the
runner before it writes `summary.json`. A campaign with a failed attempt
therefore yields no winner labels, and no observation is censored or dropped
from a summary. The timed processes are not pinned. Each inherits the allowed
CPU list of the shell that runs `benches/native.sh`, which the script records
in `host.txt` on Linux. Placement within that list is uncontrolled. This
paragraph, the seed sentence and the group-15 clause of the boundary row were
written after the measurements. They describe the rule, the generator and the
placement as run and change no threshold, count, query or label.

```bash
cargo bench -p inverted-file-vector-search-and-coarse-quantization --bench search --no-run
# Use the executable printed by Cargo as the first Python argument.
# The runner creates the output directory and refuses an existing one.
python3 topics/062-inverted-file-vector-search-and-coarse-quantization/benches/run.py "$binary" "$output_dir"
# Native replay, from a scratch copy of this topic directory. It creates
# evidence/ and refuses an existing one. measurements/README.md names the
# commit that reproduces the receipt's source hash.
bash benches/native.sh
```
