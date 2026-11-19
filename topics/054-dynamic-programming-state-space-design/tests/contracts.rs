//! Compare every candidate with direct subset sums and fixed regressions.

use topic_054_dynamic_programming::{Item, Strategy, solve};

const CANDIDATES: [Strategy; 5] = [
    Strategy::Exhaustive,
    Strategy::Table,
    Strategy::Rolling,
    Strategy::Value,
    Strategy::Frontier,
];

fn check(pairs: &[(u16, u16)], budget: u16, expected: u32) {
    let items: Vec<_> = pairs
        .iter()
        .map(|&(weight, value)| Item { weight, value })
        .collect();
    for candidate in CANDIDATES {
        assert_eq!(solve(candidate, &items, budget), expected, "{candidate:?}");
    }
}

#[test]
fn boundaries_and_overwrite_order() {
    check(&[], 0, 0);
    check(&[(2, 4)], 6, 4);
    check(&[(2, 4), (2, 4)], 6, 8);
    check(&[(0, 9), (0, 0), (1, 11)], 0, 9);
    check(&[(3, 5), (4, 5), (2, 7)], 5, 12);
    check(&[(2, 4), (3, 5), (4, 8)], 6, 12);
    check(&[(0, u16::MAX), (u16::MAX, u16::MAX)], u16::MAX, 131_070);
}

#[test]
fn deterministic_subset_oracle() {
    let mut rng = 19_u32;
    for n in 0..=9 {
        for budget in 0..=21 {
            let mut draw = || {
                rng = rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                u16::try_from(rng >> 24).unwrap()
            };
            let pairs: Vec<_> = (0..n).map(|_| (draw() % 13, draw() % 17)).collect();
            let expected = (0..1 << n)
                .filter_map(|mask| {
                    let (weight, value) = pairs
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| mask & (1 << i) != 0)
                        .fold((0_u32, 0_u32), |(w, v), (_, &(wi, vi))| {
                            (w + u32::from(wi), v + u32::from(vi))
                        });
                    (weight <= u32::from(budget)).then_some(value)
                })
                .max()
                .unwrap();
            check(&pairs, budget, expected);
        }
    }
}

#[test]
#[should_panic(expected = "at most 20 items")]
fn bounded_lab_domain() {
    let _ = solve(
        Strategy::Rolling,
        &[Item {
            weight: 0,
            value: 0,
        }; 21],
        0,
    );
}
