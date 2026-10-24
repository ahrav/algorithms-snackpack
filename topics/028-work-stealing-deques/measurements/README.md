# Measurement status

The frozen source is this topic directory. The runner is
`scripts/run_campaign.py`, which invokes the release `compare` example once per
candidate process. It verifies the sequential oracle before emitting a duration.
The process sample is the measurement unit. The local campaign records medians,
ranges, rotating candidate order, worker count, task count, leaf work, and
placement. It supplies a local smoke comparison and no host selection claim.
`LOCAL_SMOKE.md` contains its compact summary. The raw 70-process JSON record
stays outside the repository.

The required Arm and x86 campaigns remain pending. Both initial host probes
failed before authentication because the WSSH proxy DNS lookup failed. The
campaign will record hostname, architecture, kernel, CPU model, Rust toolchain,
available CPUs, source hash, paired order-balanced process samples, and the
separate setup and execution boundary.
