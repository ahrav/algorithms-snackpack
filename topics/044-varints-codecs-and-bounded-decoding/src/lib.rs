//! Canonical bounded u32 varints and a fixed-width comparison.
#![forbid(unsafe_code)]

/// A decoding failure. The offset starts the affected integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error {
    /// Byte offset in the input.
    pub offset: usize,
    /// Failure classification.
    pub kind: Kind,
}

/// The four independent failure conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Input ends before the integer does.
    Truncated,
    /// Payload exceeds u32 or continuation exceeds five bytes.
    Overflow,
    /// Multibyte integer has a redundant final zero payload.
    Noncanonical,
    /// Caller output count has been reached.
    Limit,
}

/// Decode canonical varints, clearing output first and retaining a valid prefix on error.
///
/// # Errors
/// Returns an [`Error`] for truncation, overflow, redundancy, or output count exhaustion.
/// Each integer inspects at most five bytes; count is checked before each integer.
pub fn scalar(input: &[u8], limit: usize, out: &mut Vec<u32>) -> Result<(), Error> {
    out.clear();
    let mut pos = 0;
    while pos < input.len() {
        let start = pos;
        if out.len() == limit {
            return Err(Error {
                offset: start,
                kind: Kind::Limit,
            });
        }
        let mut value = 0;
        for i in 0..5 {
            let byte = *input.get(pos).ok_or(Error {
                offset: start,
                kind: Kind::Truncated,
            })?;
            pos += 1;
            if i == 4 && byte > 15 {
                return Err(Error {
                    offset: start,
                    kind: Kind::Overflow,
                });
            }
            value |= u32::from(byte & 127) << (7 * i);
            if byte < 128 {
                if i > 0 && byte == 0 {
                    return Err(Error {
                        offset: start,
                        kind: Kind::Noncanonical,
                    });
                }
                out.push(value);
                break;
            }
        }
    }
    Ok(())
}

/// Decode the same contract with explicitly expanded byte positions.
///
/// # Errors
/// Returns the same offset, kind, and output prefix as [`scalar`].
pub fn unrolled(input: &[u8], limit: usize, out: &mut Vec<u32>) -> Result<(), Error> {
    out.clear();
    let mut pos = 0;
    'values: while pos < input.len() {
        let start = pos;
        if out.len() == limit {
            return Err(Error {
                offset: start,
                kind: Kind::Limit,
            });
        }
        let mut value = 0u32;
        // Expansion keeps shifts constant. All loads retain slice bounds checks.
        macro_rules! step {
            ($i:literal) => {{
                let byte = *input.get(pos).ok_or(Error {
                    offset: start,
                    kind: Kind::Truncated,
                })?;
                pos += 1;
                if $i == 4 && byte > 15 {
                    return Err(Error {
                        offset: start,
                        kind: Kind::Overflow,
                    });
                }
                value |= u32::from(byte & 127) << (7 * $i);
                if byte < 128 {
                    if $i > 0 && byte == 0 {
                        return Err(Error {
                            offset: start,
                            kind: Kind::Noncanonical,
                        });
                    }
                    out.push(value);
                    continue 'values;
                }
            }};
        }
        step!(0);
        step!(1);
        step!(2);
        step!(3);
        step!(4);
    }
    Ok(())
}

/// Decode four-byte little-endian integers into reused output.
///
/// # Errors
/// Returns truncation or output count exhaustion, preserving the valid prefix.
pub fn fixed(input: &[u8], limit: usize, out: &mut Vec<u32>) -> Result<(), Error> {
    out.clear();
    for (i, chunk) in input.chunks(4).enumerate() {
        let offset = 4 * i;
        if out.len() == limit {
            return Err(Error {
                offset,
                kind: Kind::Limit,
            });
        }
        let bytes = chunk.try_into().map_err(|_| Error {
            offset,
            kind: Kind::Truncated,
        })?;
        out.push(u32::from_le_bytes(bytes));
    }
    Ok(())
}

/// Encode unsigned integers in their shortest seven-bit representation.
#[must_use]
pub fn encode(values: &[u32]) -> Vec<u8> {
    let mut out = Vec::new();
    for &initial in values {
        let mut x = initial;
        loop {
            let mut byte = x.to_le_bytes()[0] & 127;
            x >>= 7;
            if x != 0 {
                byte |= 128;
            }
            out.push(byte);
            if x == 0 {
                break;
            }
        }
    }
    out
}

/// Independent group-based oracle, using u64 arithmetic and division-based size validation.
///
/// # Errors
/// Returns the strict varint contract error and prefix, just as the candidates do.
pub fn reference(input: &[u8], limit: usize, out: &mut Vec<u32>) -> Result<(), Error> {
    out.clear();
    let mut start = 0;
    while start < input.len() {
        if out.len() == limit {
            return Err(Error {
                offset: start,
                kind: Kind::Limit,
            });
        }
        let tail = &input[start..];
        let terminal = tail.iter().take(5).position(|&b| b < 128);
        let Some(end) = terminal else {
            return Err(Error {
                offset: start,
                kind: if tail.len() < 5 {
                    Kind::Truncated
                } else {
                    Kind::Overflow
                },
            });
        };
        let value: u64 = tail[..=end]
            .iter()
            .zip(0u32..)
            .map(|(b, i)| u64::from(b & 127) * 128u64.pow(i))
            .sum();
        let value = u32::try_from(value).map_err(|_| Error {
            offset: start,
            kind: Kind::Overflow,
        })?;
        let mut quotient = value;
        let mut needed = 1;
        while quotient >= 128 {
            quotient /= 128;
            needed += 1;
        }
        if needed != end + 1 {
            return Err(Error {
                offset: start,
                kind: Kind::Noncanonical,
            });
        }
        out.push(value);
        start += end + 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn agree(bytes: &[u8], limit: usize) {
        let mut expected = vec![99];
        let result = reference(bytes, limit, &mut expected);
        for candidate in [scalar, unrolled] {
            let mut actual = vec![77];
            assert_eq!(candidate(bytes, limit, &mut actual), result, "{bytes:?}");
            assert_eq!(actual, expected);
        }
    }
    #[test]
    fn exhaustive_short_and_limits() {
        for limit in [0, 1, 2] {
            agree(&[], limit);
        }
        for a in 0..=255u8 {
            for limit in [0, 1, 2] {
                agree(&[a], limit);
            }
            for b in 0..=255u8 {
                for limit in [0, 1, 2] {
                    agree(&[a, b], limit);
                }
            }
        }
    }
    #[test]
    fn boundaries_and_errors() {
        for len in 1..5 {
            agree(&vec![128; len], 3);
        }
        for bytes in [
            &[0x80][..],
            &[0x80, 0],
            &[255, 255, 255, 255, 16],
            &[128, 128, 128, 128, 128],
            &[255, 255, 255, 255, 15],
            &[1, 128, 0],
            &[128, 128, 1],
            &[128, 128, 128, 1],
        ] {
            for limit in 0..=3 {
                agree(bytes, limit);
            }
        }
        let mut out = vec![];
        assert_eq!(
            scalar(&[1, 128, 0], 9, &mut out),
            Err(Error {
                offset: 1,
                kind: Kind::Noncanonical
            })
        );
        assert_eq!(out, vec![1]);
        assert_eq!(
            fixed(&[1, 0, 0, 0, 2], 9, &mut out),
            Err(Error {
                offset: 4,
                kind: Kind::Truncated
            })
        );
        assert_eq!(out, vec![1]);
        assert_eq!(
            fixed(&[1, 0, 0, 0], 0, &mut out),
            Err(Error {
                offset: 0,
                kind: Kind::Limit
            })
        );
    }
    #[test]
    fn random_bytes_and_roundtrips() {
        let mut seed = 123u32;
        let mut values = vec![
            0,
            1,
            127,
            128,
            16383,
            16384,
            (1 << 21) - 1,
            1 << 21,
            (1 << 28) - 1,
            1 << 28,
            u32::MAX,
        ];
        for _ in 0..10_000 {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            values.push(seed);
            agree(&seed.to_le_bytes(), 3);
        }
        for len in 0..=12 {
            for _ in 0..1000 {
                let bytes: Vec<_> = (0..len)
                    .map(|_| {
                        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                        seed.to_be_bytes()[0]
                    })
                    .collect();
                agree(&bytes, 3);
            }
        }
        let encoded = encode(&values);
        agree(&encoded, values.len());
        let mut out = vec![];
        scalar(&encoded, values.len(), &mut out).unwrap();
        assert_eq!(out, values);
        let bytes: Vec<_> = values.iter().flat_map(|x| x.to_le_bytes()).collect();
        fixed(&bytes, values.len(), &mut out).unwrap();
        assert_eq!(out, values);
    }
}
