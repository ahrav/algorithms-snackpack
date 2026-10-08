//! The same sorted smallest-three output from every candidate.
use top_k_portfolio::CANDIDATES;
fn main() {
    for (name, f) in CANDIDATES {
        let result = f(&[9, 1, 7, 3, 3, 8, 2, 6], 3).unwrap();
        assert_eq!(result, [1, 2, 3]);
        println!("{name}: {result:?}");
    }
}
