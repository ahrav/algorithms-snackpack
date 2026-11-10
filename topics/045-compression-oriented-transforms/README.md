# Compression-oriented transforms

Store u32 blocks with a reversible representation and a bounded byte packet codec.
Compare raw little-endian words, identity packets, wrapping delta, wrapping second
differences, predecessor XOR and four-plane byte shuffle. Each frame resets state.

## Contract

A frame contains a u8 mode (0 through 5), u32 little-endian word count and payload.
Raw payload has exactly four bytes per word. Other modes use literal/run packets:
high tag bit selects a run, low seven bits store length minus one. Literals carry
1 through 128 bytes; runs carry one byte repeated 1 through 128 times. The encoder
chooses runs of at least three. The decoder accepts equivalent short-run forms.

The caller supplies an output word cap. Unknown headers, truncation, excessive
expansion, trailing bytes and count violations fail without returned partial output.
The cap bounds declared allocation; allocation failure follows Rust allocator policy.
Arithmetic wraps modulo 2^32. An enclosing format supplies integrity and authentication.

## Running example and costs

For [1000,1001,1002,1003], delta gives [1000,1,1,1], second differences
[1000,4294966297,0,0], XOR [1000,1,3,1]. Shuffle groups equal byte positions.
Complete frames are raw21, identity22, delta21, delta2 16, XOR21, shuffle14 bytes.
Transform output stays 16 bytes before packet coding.

For n words all transforms take linear work; predictor state is constant-sized.
This implementation materializes a 4n-byte transform buffer. Decode allocates both
4n transformed bytes and 4n output bytes. Encoder frame allocation grows as needed.
For header H=5, literal bytes L, literal packets p and run packets r,
stored bytes C=H+L+p+2r. Shuffle's example is 5+4+1+4=14.
Raw is H+4n=5+16=21. Allocation capacities and allocator overhead are additional.

For serial encode-once/read-k-times use T=E+k(D+C/B), where E and D are encode
and decode seconds and B is storage bytes/s. Ten reads of the example cost
E+10(D+14/B) for shuffle. Compare with the raw implementation's measured E,D and
21 bytes. Pipelining and cache hits require a different model.

## Run and choose

```sh
cargo test -p topic045
cargo run -p topic045 --release --example experiment -- example
python3 topics/045-compression-oriented-transforms/measurements/run.py
```

The Python runner requires Linux, rustc, lscpu and taskset. It prints its retained
scratch path. It compares sizes and allocating encode/decode times on 12 workloads.
Use second differences for stable increments, XOR for stable bits, shuffle for
stable byte planes. Test identity when bytes already form runs. Keep raw fallback
when framing or CPU costs exceed the benefit. Selection depends on the next codec:
small numeric deltas remain four bytes before this experiment's byte coding.
Read measurements/README.md for observed choices and limits.

Coverage: six scalar block representations and a byte run/literal codec.
Explicit revisit: zigzag, bit packing/shuffling, dictionary coding, Burrows-Wheeler,
move-to-front, SIMD, checkpoints/indexes, full Gorilla/Parquet formats and production
codec comparisons. The catalog has no coverage array or ALGORITHM_INDEX.md at the
inspected main revision; these variants remain explicit here.

## Primary sources

- https://parquet.apache.org/docs/file-format/data-pages/encodings/ : delta and byte-stream split contracts.
- https://www.vldb.org/pvldb/vol8/p1816-teller.pdf : Gorilla timestamp and value predictors.
- https://www.cl.cam.ac.uk/teaching/2223/Bioinfo/papers/burrows_wheeler.pdf : block sorting and move-to-front pipeline.

These implementations isolate transforms with a teaching codec. Interoperability
requires implementing each production format's complete framing and packing rules.
