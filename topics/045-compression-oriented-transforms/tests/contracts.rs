//! Public frame contract checks.
use topic045::{Error, MODES, Mode, decode, encode};
#[test]
fn independent_frames_reset_predictors() {
    for mode in MODES {
        let a = encode(mode, &[u32::MAX, 0, 1]);
        let b = encode(mode, &[1, 0, u32::MAX]);
        assert_eq!(decode(&a, 3).unwrap(), [u32::MAX, 0, 1]);
        assert_eq!(decode(&b, 3).unwrap(), [1, 0, u32::MAX]);
    }
}
#[test]
fn permissive_short_run_and_exact_consumption() {
    assert_eq!(decode(&[1, 1, 0, 0, 0, 131, 0], 1), Ok(vec![0]));
    assert_eq!(decode(&[1, 1, 0, 0, 0, 128, 0, 130, 0], 1), Ok(vec![0]));
    let mut frame = encode(Mode::Raw, &[0]);
    frame.push(0);
    assert_eq!(decode(&frame, 1), Err(Error::Payload));
}
