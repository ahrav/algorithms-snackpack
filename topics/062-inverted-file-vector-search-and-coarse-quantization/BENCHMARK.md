# Benchmark decision

Benchmark: true. Test how many rows and how much time each correct search path
requires when mean exact ID recall@5 must reach 95%.

| Cell | Rows | Geometry and query distribution |
| --- | ---: | --- |
| tiny | 256 | 16 spaced centroids, uniform group seeds, continuous integer jitter |
| balanced | 16,384 | 16 spaced centroids, uniform group seeds, residuals exactly on the PQ grid |
| boundary | 16,384 | overlapping groups and queries on adjacent centroid bisectors |
| skew | 16,384 | about 90% corpus and 75% query preference for group 0, with jitter |

The seven timed paths are flat, raw one/four probes, PQ one/four probes, and
PQ one/four probes with a 16-ID exact rerank. Untimed controls scan all
lists with raw and compressed scores. The corpus layout interleaves IDs by
group. Raw list scoring loads the original coordinates through an ID.
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

```bash
cargo bench -p inverted-file-vector-search-and-coarse-quantization --bench search --no-run
# Use the executable printed by Cargo as the first Python argument.
python3 topics/062-inverted-file-vector-search-and-coarse-quantization/benches/run.py "$binary" "$output_dir"
# Native exact-source replay, from a scratch copy of this topic directory:
bash benches/native.sh
```
