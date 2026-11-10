//! Public API running example.
use topic044::{Error, Kind, fixed, reference, scalar, unrolled};

type Decoder = fn(&[u8], usize, &mut Vec<u32>) -> Result<(), Error>;

#[test]
fn running_example() {
    let values = [1, 127, 128, 300, u32::MAX];
    let bytes = topic044::encode(&values);
    assert_eq!(bytes, [1, 127, 128, 1, 172, 2, 255, 255, 255, 255, 15]);
    for decoder in [topic044::scalar, topic044::unrolled] {
        let mut out = vec![];
        decoder(&bytes, 5, &mut out).unwrap();
        assert_eq!(out, values);
    }
}

#[test]
fn varint_rejections_report_exact_error_and_prefix() {
    let check = |bytes: &[u8], limit: usize, offset: usize, kind: Kind, prefix: &[u32]| {
        for decode in [scalar as Decoder, unrolled, reference] {
            let mut out = vec![77];
            let result = decode(bytes, limit, &mut out);
            assert_eq!(result, Err(Error { offset, kind }), "{bytes:02x?}");
            assert_eq!(out, prefix, "{bytes:02x?}");
        }
    };
    check(&[0x80, 0x80, 0x00], 9, 0, Kind::Noncanonical, &[]);
    check(&[0x80, 0x80, 0x80, 0x00], 9, 0, Kind::Noncanonical, &[]);
    check(
        &[0x80, 0x80, 0x80, 0x80, 0x00],
        9,
        0,
        Kind::Noncanonical,
        &[],
    );
    check(
        &[0xac, 0x02, 0x80, 0x80, 0x00],
        9,
        2,
        Kind::Noncanonical,
        &[300],
    );
    check(&[0x80, 0x80, 0x80, 0x80, 0x81], 9, 0, Kind::Overflow, &[]);
    check(&[0x80, 0x80, 0x80, 0x80, 0x8f], 9, 0, Kind::Overflow, &[]);
    check(&[0xff, 0xff, 0xff, 0xff, 0x10], 9, 0, Kind::Overflow, &[]);
    check(
        &[1, 0xff, 0xff, 0xff, 0xff, 0x10],
        9,
        1,
        Kind::Overflow,
        &[1],
    );
    check(&[1, 127, 0x80], 9, 2, Kind::Truncated, &[1, 127]);
    check(&[1, 0x80, 0x80, 0x80, 0x80], 9, 1, Kind::Truncated, &[1]);
    check(&[1], 0, 0, Kind::Limit, &[]);
    check(&[1, 2], 1, 1, Kind::Limit, &[1]);
}

#[test]
fn fixed_replaces_stale_output() {
    let check = |bytes: &[u8], limit: usize, result: Result<(), Error>, want: &[u32]| {
        let mut out = vec![77, 78, 79];
        assert_eq!(fixed(bytes, limit, &mut out), result, "{bytes:?}");
        assert_eq!(out, want, "{bytes:?}");
    };
    let err = |offset, kind| Err(Error { offset, kind });
    check(&[1, 0, 0, 0, 0, 1, 0, 0], 2, Ok(()), &[1, 256]);
    check(&[1, 0, 0, 0, 2], 9, err(4, Kind::Truncated), &[1]);
    check(&[1, 0, 0, 0], 0, err(0, Kind::Limit), &[]);
    check(&[1, 0, 0, 0, 2, 0, 0, 0], 1, err(4, Kind::Limit), &[1]);
}

/// The cap equals the input length, so decoding runs to the end or to the first invalid integer.
#[test]
fn random_bytes_agree_with_reference_at_a_slack_cap() {
    let mut seed = 0x9e37_79b9_u32;
    for len in 1..=16 {
        for _ in 0..1000 {
            let bytes: Vec<u8> = (0..len)
                .map(|_| {
                    seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    seed.to_be_bytes()[0]
                })
                .collect();
            let mut expected = vec![99];
            let result = reference(&bytes, len, &mut expected);
            for decode in [scalar as Decoder, unrolled] {
                let mut actual = vec![77];
                assert_eq!(decode(&bytes, len, &mut actual), result, "{bytes:02x?}");
                assert_eq!(actual, expected, "{bytes:02x?}");
            }
        }
    }
}
