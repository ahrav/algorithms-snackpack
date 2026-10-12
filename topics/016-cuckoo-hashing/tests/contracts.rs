//! External exact-map and reachability checks.
use cuckoo_hashing::{Cuckoo, Layout, Linear, Table};
#[test]
fn exhaustive_three_key_traces() {
    for encoded in 0_u64..729 {
        let mut a = Linear::new(8, Layout::SamePair);
        let mut b = Cuckoo::<1>::new(8, Layout::SamePair, 64);
        let mut c = Cuckoo::<4>::new(8, Layout::SamePair, 64);
        let mut models = [
            std::collections::BTreeMap::new(),
            std::collections::BTreeMap::new(),
            std::collections::BTreeMap::new(),
        ];
        let mut x = encoded;
        for step in 0..6 {
            let key = x % 3;
            x /= 3;
            let tables: [&mut dyn Table; 3] = [&mut a, &mut b, &mut c];
            for (table, model) in tables.into_iter().zip(&mut models) {
                let before = table.entries();
                match table.try_insert(key, step) {
                    Ok(old) => assert_eq!(old, model.insert(key, step)),
                    Err(_) => assert_eq!(table.entries(), before),
                }
                assert_eq!(
                    table.entries(),
                    model.iter().map(|(&k, &v)| (k, v)).collect::<Vec<_>>()
                );
            }
        }
    }
}
#[test]
fn sparse_first_bucket_does_not_hide_second_bucket_key() {
    let mut table = Cuckoo::<1>::new(8, Layout::SamePair, 64);
    table.try_insert(1, 101).unwrap();
    table.try_insert(2, 202).unwrap();
    assert_eq!(table.remove(1), Some(101));
    assert_eq!(table.get(2), Some(202));
    assert_eq!(table.try_insert(2, 999), Ok(Some(202)));
    assert_eq!(table.entries(), vec![(2, 999)]);
}
