//! Public batch boundary checks.
use topic_046_streaming_sketches::evaluate;
#[test]
fn running_example_answers() {
    let data = [4, 7, 4, 9, 4];
    assert_eq!(
        evaluate(&data, &[4, 8], "exact-frequency", 7)
            .unwrap()
            .values,
        [3, 0]
    );
    assert_eq!(
        evaluate(&data, &[], "exact-distinct", 7).unwrap().estimate,
        3.0
    );
    assert_eq!(
        evaluate(&data, &[], "rank-summary", 7).unwrap().values[50],
        4
    );
}
