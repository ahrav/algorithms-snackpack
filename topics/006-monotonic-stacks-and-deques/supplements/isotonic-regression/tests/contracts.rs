//! Endpoint and arithmetic contracts for each fit.

use isotonic_regression::{Block, MAX_LEN, hull, shifting, stack};

#[test]
fn all_candidates_pool_backwards_and_preserve_empty_input() {
    for fit in [shifting, stack, hull] {
        assert_eq!(fit(&[]), Vec::<Block>::new());
        assert_eq!(
            fit(&[6, 2, 3, 8]),
            [Block { sum: 11, len: 3 }, Block { sum: 8, len: 1 }]
        );
        assert_eq!(fit(&[4, 4, 4]), [Block { sum: 12, len: 3 }]);
        assert_eq!(fit(&[i32::MAX, i32::MIN]), [Block { sum: -1, len: 2 }]);
    }
}

#[test]
fn all_candidates_reject_oversized_input() {
    let input = vec![0; MAX_LEN + 1];
    for fit in [shifting, stack, hull] {
        assert!(std::panic::catch_unwind(|| fit(&input)).is_err());
    }
}
