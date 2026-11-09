# Test strategy and invariant audit

Risk: hostile bytes can cause overflow, excess work, ambiguous bytes or partial output.
Independent oracle locates the terminal group, reconstructs with u64 multiplication,
checks conversion, and derives minimal length by division. Candidates use checked
byte-at-a-time u32 shifts. Oracle and candidates share no parser/encoder helper.

Coverage: all65793 byte strings of length0..2 at output caps0/1/2, targeted
five-byte overflows/redundancy/truncations, prefix/error offsets, fixed-width tails,
10000 random four-byte strings and13000 variable-length strings, and10011-value
roundtrip including every u32 encoded-length threshold. A public API test pins
the exact11-byte running example. Python independently validates all workload encodings.

Invariant-discriminating checks: byte5=0x10 rejects overflow; 80 00 rejects
redundancy; 80 rejects truncation; [1,80,00] reports offset1 with prefix[1]; limit0
rejects before output. These distinguish missing checks rather than test names alone.
Safe slice access and finite five-step paths enforce read bounds. Full-buffer
decoding requires callers to assemble framing before this API. SIMD/streaming
implementations would require independent tail and every-split tests.
