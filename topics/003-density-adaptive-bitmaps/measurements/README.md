# Measurements

No final benchmark campaign has been collected yet.

The native harness is `benches/bitmap_sets.rs`. The orchestrator is
`scripts/run_experiment.py`. Collection modes require an absolute output path
that does not exist:

```bash
python3 topics/003-density-adaptive-bitmaps/scripts/run_experiment.py plan
python3 topics/003-density-adaptive-bitmaps/scripts/run_experiment.py self-check
python3 topics/003-density-adaptive-bitmaps/scripts/run_experiment.py quick \
  --output-dir /absolute/external/new-directory
python3 topics/003-density-adaptive-bitmaps/scripts/run_experiment.py all \
  --output-dir /absolute/external/new-directory
```

Raw process attempts, executables, profiler output, stdout, stderr, and run
bundles stay outside Git. After a complete campaign, package all complete,
partial, failed, timed-out, interrupted, and reset-failed records into the
automation-owned external archive. Verify that archive before removing its
source directory. Commit only the compact aggregate and evidence receipt.
