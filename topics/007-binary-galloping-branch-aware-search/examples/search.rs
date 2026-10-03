//! Runnable duplicate and miss example.
use lower_bound_portfolio::{CANDIDATES, oracle};

fn main() {
    let a = [2, 4, 4, 4, 9, 13, 18, 21];
    for key in [0, 4, 10, 22] {
        let expected = oracle(&a, key);
        for (name, f) in CANDIDATES {
            assert_eq!(f(&a, key), expected);
            println!("{name}: key={key} index={expected}");
        }
    }
}
