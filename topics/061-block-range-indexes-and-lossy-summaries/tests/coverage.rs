//! Compare every candidate with independently filtered rows.

use block_range_indexes_and_lossy_summaries::{Index, Layout, Scan, full_scan};
use std::num::NonZeroUsize;

fn width(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).unwrap()
}

#[test]
fn rechecks_covered_holes_and_preserves_endpoints() {
    let rows = [10, 11, 12, 13, 18, 20, 91, 93, 49, 50, 52, 54];
    let minmax = Index::build(&rows, width(4), Layout::MinMax);
    let two = Index::build(&rows, width(4), Layout::TwoRanges);
    assert_eq!(
        minmax.scan(50, 51),
        Scan {
            matches: 1,
            summaries: 3,
            candidate_blocks: 2,
            checked_rows: 8
        }
    );
    assert_eq!(
        two.scan(50, 51),
        Scan {
            matches: 1,
            summaries: 3,
            candidate_blocks: 1,
            checked_rows: 4
        }
    );
    assert_eq!(two.scan(53, 53).checked_rows, 4);
    assert_eq!(two.scan(53, 53).matches, 0);
    assert_eq!(two.scan(20, 20).matches, 1);
    assert_eq!(two.scan(91, 91).matches, 1);
}

#[test]
fn wide_signed_gap_and_short_tail() {
    let rows = [i32::MAX, i32::MIN, i32::MIN, 7, 7];
    for layout in [Layout::MinMax, Layout::TwoRanges] {
        let index = Index::build(&rows, width(3), layout);
        assert_eq!(index.scan(7, 7).matches, 2);
        assert_eq!(index.scan(i32::MIN, i32::MIN).matches, 2);
        assert_eq!(index.scan(i32::MAX, i32::MAX).matches, 1);
        assert_eq!(index.scan(i32::MAX, i32::MIN), Scan::default());
        assert_eq!(index.scan(i32::MIN, i32::MAX).checked_rows, 5);
        assert_eq!(
            Index::build(&[], width(3), layout).scan(0, 0),
            Scan::default()
        );
    }
    let two = Index::build(&rows, width(3), Layout::TwoRanges);
    assert_eq!(two.scan(0, 0).checked_rows, 0);
    assert_eq!(two.scan(7, 7).checked_rows, 2);
}

#[test]
fn exhaustive_small_sequences_match_independent_filter() {
    let values = [-3, -1, 0, 2, 4];
    for variant in 0..5_usize.pow(5) {
        let mut state = variant;
        let rows = std::array::from_fn::<_, 5, _>(|_| {
            let value = values[state % values.len()];
            state /= values.len();
            value
        });
        for w in [1, 2, 3, 5, 8] {
            let minmax = Index::build(&rows, width(w), Layout::MinMax);
            let two = Index::build(&rows, width(w), Layout::TwoRanges);
            for low in -4..=5 {
                for high in -4..=5 {
                    let expected = rows.iter().filter(|&&v| (low..=high).contains(&v)).count();
                    let a = minmax.scan(low, high);
                    let b = two.scan(low, high);
                    assert_eq!(full_scan(&rows, low, high).matches, expected);
                    assert_eq!(a.matches, expected);
                    assert_eq!(b.matches, expected);
                    assert!(b.checked_rows <= a.checked_rows);
                }
            }
        }
    }
}
