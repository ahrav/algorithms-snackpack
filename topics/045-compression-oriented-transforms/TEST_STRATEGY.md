Scope: bounded independent u32 frames and reversible block transforms.
Contract and failure classes: header, output cap and exact payload/consumption.
Primary technique: finite boundary tables and exact known-byte fixtures.
Secondary techniques: independent Python full-workload arithmetic model.
Oracle: Python modulo arithmetic, byte extraction and prefix reconstruction;
it invokes no Rust codec helper. Hard-coded Rust fixture bytes are separately derived.
Input/model/schedule domain and bounds: lengths0/1/2/3/31/32/33/127/128/129/1024,
wraparound values; 12 synthetic workload cells through65536 words and six modes.
Replay assets: library, example, measurements/run.py, retained raw archives;
measurements/test_run.py checks runner attempt retention with stub processes.
What a green run establishes: agreement on the declared finite/synthetic domain.
Known gaps and delegated skills: arbitrary-byte fuzzing, allocator failure, SIMD,
integrity/authentication and production-format compatibility remain outside scope.
Concrete tests to add: fuzz frames under a small cap before production use.

## Invariant audit

Claim and scope: inverse transform matches all original words for every mode.
Plausible violation: reset or wrapping arithmetic differs; wrong plane stride.
Observation: exact ordered values and independently derived transformed bytes.
Oracle and independence: arithmetic Python model and hand-derived small fixture.
Comparator: complete byte arrays and ordered u32 arrays.
Smallest discriminating case: [MAX,0,1], four-word ramp, separate consecutive frames.
Mutation backstop: fixture distinguishes identity/delta/second-difference/shuffle;
wraparound and independent frame tests distinguish state leakage.

Claim and scope: bounded decoder consumes exactly one frame and returns errors.
Plausible violation: ignore trailing bytes, accept missing run value, overrun count.
Observation: exact Error variant; API returns no partial vector on failure.
Oracle and independence: explicit malformed frame tables, independent expected errors.
Comparator: Result equality.
Smallest discriminating case: empty header, tag6, cap1/count2, truncated run,
run5/count1, literal4 with one byte, nonempty payload/count0, raw trailing byte.
Mutation backstop: each listed acceptance change contradicts its explicit assertion.
