//! Reversible u32 transforms followed by a bounded byte run/literal codec.
#![forbid(unsafe_code)]

/// Stored representation. Numeric values are the frame tags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mode {
    /// Little-endian words without packet coding.
    Raw = 0,
    /// Little-endian words with packet coding.
    Identity = 1,
    /// Wrapping first differences, reset to zero per frame.
    Delta = 2,
    /// Wrapping second differences, both states reset per frame.
    Delta2 = 3,
    /// XOR with the previous word, reset to zero per frame.
    Xor = 4,
    /// Four byte planes, low byte first.
    Shuffle = 5,
}

/// Every supported representation in tag order.
pub const MODES: [Mode; 6] = [
    Mode::Raw,
    Mode::Identity,
    Mode::Delta,
    Mode::Delta2,
    Mode::Xor,
    Mode::Shuffle,
];

/// Decoder failure. Failed decoding publishes no output.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Header shorter than five bytes or unknown tag.
    Header,
    /// Declared word count exceeds the caller's cap or addressable size.
    Limit,
    /// Packet/input length disagrees with the declared output size.
    Payload,
}

fn transformed(mode: Mode, values: &[u32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(values.len() * 4);
    if mode == Mode::Shuffle {
        for b in 0..4 {
            for x in values {
                out.push(x.to_le_bytes()[b]);
            }
        }
    } else {
        let (mut prev, mut pd) = (0u32, 0u32);
        for &x in values {
            let d = x.wrapping_sub(prev);
            let y = match mode {
                Mode::Delta => d,
                Mode::Delta2 => d.wrapping_sub(pd),
                Mode::Xor => x ^ prev,
                _ => x,
            };
            out.extend_from_slice(&y.to_le_bytes());
            prev = x;
            pd = d;
        }
    }
    out
}

fn run(data: &[u8], i: usize) -> usize {
    let mut n = 1;
    while n < 128 && i + n < data.len() && data[i + n] == data[i] {
        n += 1;
    }
    n
}

fn packets(data: &[u8], out: &mut Vec<u8>) {
    let mut i = 0;
    while i < data.len() {
        let n = run(data, i);
        if n >= 3 {
            out.push(0x80 | u8::try_from(n - 1).unwrap());
            out.push(data[i]);
            i += n;
        } else {
            let start = i;
            i += n;
            while i < data.len() && i - start < 128 {
                let next = run(data, i);
                if next >= 3 {
                    break;
                }
                i += next.min(128 - (i - start));
            }
            out.push(u8::try_from(i - start - 1).unwrap());
            out.extend_from_slice(&data[start..i]);
        }
    }
}

/// Encode one independent frame: tag:u8, count:u32 little endian, payload.
///
/// The encoder accepts at most `u32::MAX` words and requires addressable storage
/// for four bytes per word. Allocation failure follows Rust's allocator policy.
///
/// # Panics
/// Panics when the count exceeds `u32::MAX` or four times the count overflows usize.
#[must_use]
pub fn encode(mode: Mode, values: &[u32]) -> Vec<u8> {
    let count = u32::try_from(values.len()).expect("u32 count");
    values.len().checked_mul(4).expect("addressable byte count");
    let bytes = transformed(mode, values);
    let mut out = vec![mode as u8];
    out.extend_from_slice(&count.to_le_bytes());
    if mode == Mode::Raw {
        out.extend_from_slice(&bytes);
    } else {
        packets(&bytes, &mut out);
    }
    out
}

/// Decode one entire frame with a caller-supplied word-count cap.
///
/// Arithmetic wraps modulo 2^32. Packet forms are permissive, including
/// short runs. Trailing bytes, truncated packets and over/under-expansion fail.
/// Integrity/authentication belongs to the enclosing format.
///
/// # Errors
/// Returns [`Error::Header`], [`Error::Limit`] or [`Error::Payload`].
pub fn decode(frame: &[u8], max_values: usize) -> Result<Vec<u32>, Error> {
    if frame.len() < 5 {
        return Err(Error::Header);
    }
    let mode = *MODES.get(usize::from(frame[0])).ok_or(Error::Header)?;
    let n = usize::try_from(u32::from_le_bytes([frame[1], frame[2], frame[3], frame[4]]))
        .map_err(|_| Error::Limit)?;
    if n > max_values {
        return Err(Error::Limit);
    }
    let size = n.checked_mul(4).ok_or(Error::Limit)?;
    let payload = &frame[5..];
    let bytes = if mode == Mode::Raw {
        if payload.len() != size {
            return Err(Error::Payload);
        }
        payload.to_vec()
    } else {
        let mut result = Vec::with_capacity(size);
        let mut i = 0;
        while i < payload.len() {
            let tag = payload[i];
            i += 1;
            let len = usize::from(tag & 127) + 1;
            if len > size - result.len() {
                return Err(Error::Payload);
            }
            if tag & 128 != 0 {
                let byte = *payload.get(i).ok_or(Error::Payload)?;
                i += 1;
                result.resize(result.len() + len, byte);
            } else {
                let end = i.checked_add(len).ok_or(Error::Payload)?;
                result.extend_from_slice(payload.get(i..end).ok_or(Error::Payload)?);
                i = end;
            }
        }
        if result.len() != size {
            return Err(Error::Payload);
        }
        result
    };
    let mut out = Vec::with_capacity(n);
    let (mut prev, mut pd) = (0u32, 0u32);
    for i in 0..n {
        let y = if mode == Mode::Shuffle {
            u32::from_le_bytes([bytes[i], bytes[n + i], bytes[2 * n + i], bytes[3 * n + i]])
        } else {
            u32::from_le_bytes([
                bytes[4 * i],
                bytes[4 * i + 1],
                bytes[4 * i + 2],
                bytes[4 * i + 3],
            ])
        };
        let x = match mode {
            Mode::Delta => prev.wrapping_add(y),
            Mode::Delta2 => {
                pd = pd.wrapping_add(y);
                prev.wrapping_add(pd)
            }
            Mode::Xor => prev ^ y,
            _ => y,
        };
        out.push(x);
        prev = x;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_boundaries() {
        for n in [0, 1, 2, 3, 31, 32, 33, 127, 128, 129, 1024] {
            let v: Vec<u32> = (0..n)
                .map(|i| [0, u32::MAX, 1, 256, 0x8000_0000][i % 5])
                .collect();
            for mode in MODES {
                assert_eq!(decode(&encode(mode, &v), n), Ok(v.clone()));
            }
        }
    }

    #[test]
    fn malformed_and_limits() {
        assert_eq!(decode(&[], 1), Err(Error::Header));
        assert_eq!(decode(&[6, 0, 0, 0, 0], 1), Err(Error::Header));
        assert_eq!(decode(&[1, 2, 0, 0, 0], 1), Err(Error::Limit));
        for bytes in [
            vec![0, 1, 0, 0, 0],
            vec![1, 1, 0, 0, 0, 131],
            vec![1, 1, 0, 0, 0, 132, 0],
            vec![1, 1, 0, 0, 0, 3, 0],
            vec![1, 0, 0, 0, 0, 128, 0],
        ] {
            assert_eq!(decode(&bytes, 1), Err(Error::Payload));
        }
        let mut frame = encode(Mode::Delta2, &[1000, 1001, 1002, 1003]);
        frame.push(0);
        assert_eq!(decode(&frame, 4), Err(Error::Payload));
    }

    #[test]
    fn independent_known_transform_bytes() {
        let v = [1000, 1001, 1002, 1003];
        assert_eq!(
            transformed(Mode::Delta, &v),
            [232, 3, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0]
        );
        assert_eq!(
            transformed(Mode::Delta2, &v),
            [232, 3, 0, 0, 25, 252, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(
            transformed(Mode::Shuffle, &v),
            [232, 233, 234, 235, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(encoded_packets(&[7; 129]), vec![255, 7, 0, 7]);
    }

    fn encoded_packets(v: &[u8]) -> Vec<u8> {
        let mut out = vec![];
        packets(v, &mut out);
        out
    }
}
