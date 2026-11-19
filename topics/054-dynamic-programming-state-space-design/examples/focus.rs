//! Print the example rows and verify the score by subset enumeration.
fn main() {
    let items = [(2, 4), (3, 5), (4, 8)];
    let mut row = [0; 7];
    for (weight, value) in items {
        for capacity in (weight..row.len()).rev() {
            row[capacity] = row[capacity].max(row[capacity - weight] + value);
        }
        println!("{row:?}");
    }
    let exhaustive = (0..1 << items.len())
        .filter_map(|mask| {
            let (weight, value) = items
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .fold((0, 0), |(w, v), (_, &(wi, vi))| (w + wi, v + vi));
            (weight <= 6).then_some(value)
        })
        .max()
        .unwrap();
    assert_eq!(row[6], exhaustive);
    assert_eq!(exhaustive, 12);
}
