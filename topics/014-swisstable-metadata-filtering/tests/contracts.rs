//! Independent differential contract for deletion, overwrite and finite absence.
use swisstable_metadata_filtering::{Filter, Key, Table};
#[test]
fn every_erasure_subset_preserves_survivors() {
    for removed in 0..256_u32 {
        let mut table = Table::new(32, true);
        for id in 0..24 {
            table.insert(Key::new(id), 1, id).unwrap();
        }
        for id in 0..8 {
            if removed & (1 << id) != 0 {
                assert_eq!(table.remove(Key::new(id), 1), Some(id));
            }
        }
        for filter in [Filter::Unfiltered, Filter::Scalar, Filter::Word] {
            for id in 0..32 {
                let expected = (id < 24 && (id >= 8 || removed & (1 << id) == 0)).then_some(id);
                assert_eq!(table.get(Key::new(id), 1, filter).value, expected);
            }
        }
    }
}
