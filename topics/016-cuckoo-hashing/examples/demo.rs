//! Relocation and unchanged-rejection demonstration.
use cuckoo_hashing::{Cuckoo, Layout, Table};
fn main() {
    let mut map = Cuckoo::<1>::new(8, Layout::Example, 64);
    for key in [10, 20, 30, 40] {
        map.try_insert(key, key + 1).unwrap();
    }
    let before = map.entries();
    assert!(map.try_insert(50, 51).is_err());
    assert_eq!(before, map.entries());
    println!("entries={before:?}; fifth same-pair key rejected with four spare slots");
}
