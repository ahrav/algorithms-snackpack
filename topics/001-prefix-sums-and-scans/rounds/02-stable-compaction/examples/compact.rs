use stable_flag_compaction::{cursor, ranks, reference, runs, tiles};
use std::num::NonZeroUsize;

fn main() {
    let values = [10, 20, 30, 40, 50, 60, 70, 80];
    let keep = [false, true, true, false, true, false, true, true];
    let expected = reference(&values, &keep);
    assert_eq!(expected, [20, 30, 50, 70, 80]);
    let mut output = [0; 8];
    let mut rank = [0; 8];
    let mut offsets = [0; 2];
    let tile_len = NonZeroUsize::new(4).unwrap();

    assert_eq!(cursor(&values, &keep, &mut output), 5);
    assert_eq!(output[..5], expected);
    assert_eq!(ranks(&values, &keep, &mut output, &mut rank), 5);
    assert_eq!(output[..5], expected);
    assert_eq!(rank, [0, 0, 1, 2, 2, 3, 3, 4]);
    assert_eq!(
        tiles(&values, &keep, &mut output, tile_len, &mut offsets),
        5
    );
    assert_eq!(output[..5], expected);
    assert_eq!(offsets, [0, 2]);
    assert_eq!(runs(&values, &keep, &mut output), 5);
    assert_eq!(output[..5], expected);
    println!("ranks={rank:?} tile_offsets={offsets:?} packed={expected:?}");
}
