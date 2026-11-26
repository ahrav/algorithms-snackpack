//! Compare exact sorted output against replay and fixed history witnesses.

use k_way_merge_tombstones_and_snapshots::{
    Point, RangeDelete, Selector, measure_work, read, replay,
};

const SELECTORS: [Selector; 3] = [Selector::Flat, Selector::Heap, Selector::Loser];

fn point(key: u32, seq: u64, value: Option<u32>) -> Point {
    Point { key, seq, value }
}

fn range(start: u32, end: u32, seq: u64) -> RangeDelete {
    RangeDelete { start, end, seq }
}

fn check(runs: &[Vec<Point>], ranges: &[RangeDelete], snapshot: u64, expected: &[(u32, u32)]) {
    assert_eq!(replay(runs, ranges, snapshot), expected);
    for selector in SELECTORS {
        assert_eq!(
            read(runs, ranges, snapshot, selector),
            expected,
            "{selector:?} S={snapshot}"
        );
        assert_eq!(measure_work(runs, ranges, snapshot, selector).0, expected);
    }
}

#[test]
fn overlapping_ranges_preserve_snapshot_boundaries_and_reinsertion() {
    let runs = [
        vec![
            point(8, 2, Some(65)),
            point(10, 3, Some(66)),
            point(14, 12, None),
        ],
        vec![
            point(10, 9, Some(70)),
            point(11, 4, Some(67)),
            point(12, 5, Some(68)),
        ],
        vec![point(12, 11, Some(71)), point(14, 6, Some(69))],
    ];
    let ranges = [range(11, 14, 10), range(10, 13, 7)];
    check(
        &runs,
        &ranges,
        6,
        &[(8, 65), (10, 66), (11, 67), (12, 68), (14, 69)],
    );
    check(&runs, &ranges, 8, &[(8, 65), (14, 69)]);
    check(&runs, &ranges, 10, &[(8, 65), (10, 70), (14, 69)]);
    check(&runs, &ranges, 12, &[(8, 65), (10, 70), (12, 71)]);
}

#[test]
fn bounds_and_absent_heads_are_represented_explicitly() {
    check(&[], &[range(0, u32::MAX, 1)], 1, &[]);
    check(&[vec![], vec![], vec![]], &[], 0, &[]);
    let runs = [
        vec![],
        vec![
            point(0, 0, Some(7)),
            point(1, 1, Some(8)),
            point(u32::MAX, u64::MAX, Some(9)),
        ],
        vec![],
    ];
    let ranges = [range(0, 1, 2), range(1, 1, 3), range(4, 0, 4)];
    check(&runs, &ranges, 0, &[(0, 7)]);
    check(&runs, &ranges, u64::MAX, &[(1, 8), (u32::MAX, 9)]);
}

#[test]
fn lazy_expiration_exposes_older_cover_and_disarms_buried_stale_ranges() {
    let runs = [
        vec![point(5, 3, Some(5)), point(6, 4, Some(6))],
        vec![point(3, 1, Some(3)), point(4, 2, Some(4))],
        vec![point(7, 5, Some(7)), point(8, 6, Some(8))],
    ];
    let ranges = [range(3, 4, 7), range(3, 7, 8), range(4, 6, 9)];
    check(&runs, &ranges, 9, &[(7, 7), (8, 8)]);
    check(&runs, &ranges, 7, &[(4, 4), (5, 5), (6, 6), (7, 7), (8, 8)]);
}

fn word(seed: &mut u64) -> u32 {
    *seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    u32::try_from(*seed >> 32).unwrap()
}

#[test]
fn generated_histories_match_replay_at_every_sequence() {
    for seed in 0..256 {
        let mut state = seed;
        let count = usize::try_from(seed % 11 + 1).unwrap();
        let mut runs = vec![Vec::new(); count];
        let mut ranges = Vec::new();
        for seq in 0..48 {
            let key = word(&mut state) % 12;
            let tag = word(&mut state) % 5;
            if tag < 2 {
                ranges.push(range(key, key + word(&mut state) % 8, seq));
            } else {
                let run = usize::try_from(word(&mut state)).unwrap() % count;
                runs[run].push(point(key, seq, (tag != 2).then_some(word(&mut state))));
            }
        }
        for run in &mut runs {
            run.sort_unstable_by_key(|p| p.order());
        }
        ranges.reverse();
        for snapshot in 0..48 {
            check(&runs, &ranges, snapshot, &replay(&runs, &ranges, snapshot));
        }
    }
}
