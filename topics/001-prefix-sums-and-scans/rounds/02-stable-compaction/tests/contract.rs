use stable_flag_compaction::{cursor, ranks, reference, runs, tiles};
use std::num::NonZeroUsize;

fn compare(values: &[u32], flags: &[bool], tile_len: usize) {
    let expected = reference(values, flags);
    let sentinel = u32::MAX;
    let mut output = vec![sentinel; values.len() + 3];
    let verify = |count, output: &mut [u32]| {
        assert_eq!(count, expected.len());
        assert_eq!(&output[..count], expected);
        assert!(output[count..].iter().all(|&x| x == sentinel));
        output.fill(sentinel);
    };

    verify(cursor(values, flags, &mut output), &mut output);
    let mut rank = vec![usize::MAX; values.len()];
    verify(ranks(values, flags, &mut output, &mut rank), &mut output);
    for (index, rank) in rank.into_iter().enumerate() {
        assert_eq!(
            rank,
            flags[..index].iter().filter(|&&selected| selected).count()
        );
    }
    let mut offsets = vec![usize::MAX; flags.len().div_ceil(tile_len)];
    verify(
        tiles(
            values,
            flags,
            &mut output,
            NonZeroUsize::new(tile_len).unwrap(),
            &mut offsets,
        ),
        &mut output,
    );
    for (tile, offset) in offsets.into_iter().enumerate() {
        assert_eq!(
            offset,
            flags[..tile * tile_len]
                .iter()
                .filter(|&&selected| selected)
                .count()
        );
    }
    verify(runs(values, flags, &mut output), &mut output);
}

#[test]
fn exhaustive_masks_through_ten() {
    for n in 0..=10 {
        let values: Vec<u32> = (0..n as u32)
            .map(|x| x.wrapping_mul(2_654_435_761))
            .collect();
        for mask in 0..1 << n {
            let flags: Vec<_> = (0..n).map(|i| (mask & (1 << i)) != 0).collect();
            for tile_len in [1, 3, 4, 16] {
                compare(&values, &flags, tile_len);
            }
        }
    }
}

#[test]
fn empty_tiles_repeated_values_and_partial_tail() {
    let values: Vec<_> = (0..29).map(|x| x % 5).collect();
    let flags: Vec<_> = (0..29).map(|i| matches!(i, 0 | 3 | 20..=23 | 28)).collect();
    compare(&values, &flags, 4);
    compare(&[8, 8, 7, 8], &[true, true, false, true], usize::MAX);
}

#[test]
fn invalid_shape_panics_before_writes() {
    let mut output = [9; 2];
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        ranks(&[1, 2], &[true, true], &mut output, &mut []);
    }));
    assert!(panic.is_err());
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tiles(
            &[1, 2],
            &[true, true],
            &mut output,
            NonZeroUsize::MIN,
            &mut [0],
        );
    }));
    assert!(panic.is_err());
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        cursor(&[1, 2], &[true], &mut output);
    }));
    assert!(panic.is_err());
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runs(&[1, 2], &[true, true], &mut output[..1]);
    }));
    assert!(panic.is_err());
    assert_eq!(output, [9; 2]);
}
