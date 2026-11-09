//! Public API running example.
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
