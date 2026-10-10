//! A matching tag does not establish key equality.
use swisstable_metadata_filtering::{Filter, HashShape, Key, Table, hash};
fn main() {
    let mut table = Table::new(32, true);
    for id in [11, 22, 33] {
        table
            .insert(Key::new(id), hash(id, HashShape::ConstantTag), id * 10)
            .unwrap();
    }
    for filter in [Filter::Unfiltered, Filter::Scalar, Filter::Word] {
        assert_eq!(
            table
                .get(Key::new(22), hash(22, HashShape::ConstantTag), filter)
                .value,
            Some(220)
        );
        assert_eq!(
            table
                .get(Key::new(99), hash(99, HashShape::ConstantTag), filter)
                .value,
            None
        );
    }
    println!("all tags collide; exact lookup still returns 220 and rejects 99");
}
