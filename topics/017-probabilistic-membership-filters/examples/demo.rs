//! Exact recheck and binary-equation demonstration.
use membership_filters::{Bloom, solve_binary};
fn main() {
    let keys = [17, 42, 99];
    let mut b = Bloom::new(64, 3, false);
    for k in keys {
        b.insert(k);
    }
    for k in [17, 18, 42, 99] {
        println!(
            "key={k} maybe={} exact={}",
            b.contains(k),
            b.contains(k) && keys.contains(&k)
        );
    }
    let s = solve_binary(&[(3, 5), (6, 9), (12, 12)]).unwrap();
    println!("binary solution={:?}", &s[..4]);
}
