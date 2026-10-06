//! A presorted-prefix running example.
use sorting_lab::{METHODS, Record, agrees, is_stable, sort};
fn main() {
    let input: Vec<_> = [(1, 7), (1, 2), (1, 2), (2, 8), (2, 1), (2, 4)]
        .into_iter()
        .enumerate()
        .map(|(i, (p, s))| Record {
            key: (p << 32) | s,
            id: u32::try_from(i).unwrap(),
        })
        .collect();
    for method in METHODS {
        let mut a = input.clone();
        sort(&mut a, method).unwrap();
        assert!(agrees(&input, &a, is_stable(method)));
        println!(
            "{method}: {:?}",
            a.iter()
                .map(|r| (r.key >> 32, r.key & 0xffff_ffff, r.id))
                .collect::<Vec<_>>()
        );
    }
}
