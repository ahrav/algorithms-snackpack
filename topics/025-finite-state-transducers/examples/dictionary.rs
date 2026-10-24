//! Residual outputs preserve mapped values across shared continuations.
use algorithms_fst::{Index, Kind};
fn main() {
    let rows = [("bat", 10), ("bats", 12), ("cat", 20), ("cats", 22)]
        .map(|(k, v)| (k.as_bytes().to_vec(), v));
    let index = Index::build(&rows, Kind::Pushed).unwrap();
    assert_eq!(index.get(b"cats"), Some(22));
    assert_eq!(index.get(b"ca"), None);
    assert_eq!(index.stats().0, 5);
    println!("cats=22, ca=absent, states=5");
}
