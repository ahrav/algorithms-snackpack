//! Contract, boundary, differential, and metamorphic tests for range updates.

use std::collections::BTreeSet;

use topic_002_difference_arrays_and_range_updates::{
    RangeError, RangeUpdate, apply_dense_sidecar, apply_in_place_difference, apply_reference,
    apply_sorted_events,
};

type CandidateFn = fn(&[i64], &[RangeUpdate]) -> Result<Vec<i64>, RangeError>;

const CANDIDATES: [(&str, CandidateFn); 3] = [
    ("dense", apply_dense_sidecar),
    ("in_place", apply_in_place_difference),
    ("sorted_events", apply_sorted_events),
];

const ALL_IMPLEMENTATIONS: [(&str, CandidateFn); 4] = [
    ("reference", apply_reference),
    ("dense", apply_dense_sidecar),
    ("in_place", apply_in_place_difference),
    ("sorted_events", apply_sorted_events),
];

#[test]
fn literal_half_open_boundaries_produce_exact_sequence() {
    let input = [10, 20, 30, 40];
    let updates = [
        RangeUpdate {
            start: 0,
            end: 4,
            delta: 5,
        },
        RangeUpdate {
            start: 1,
            end: 3,
            delta: -7,
        },
        RangeUpdate {
            start: 2,
            end: 2,
            delta: 99,
        },
        RangeUpdate {
            start: 0,
            end: 1,
            delta: -15,
        },
    ];
    let expected = vec![0, 18, 28, 45];

    for (name, candidate) in ALL_IMPLEMENTATIONS {
        assert_eq!(candidate(&input, &updates), Ok(expected.clone()), "{name}");
    }
}

#[test]
fn valid_len_boundary_empty_ranges_and_zero_deltas_are_no_ops() {
    let input = [-2, 7];
    let updates = [
        RangeUpdate {
            start: 2,
            end: 2,
            delta: 99,
        },
        RangeUpdate {
            start: 0,
            end: 2,
            delta: 0,
        },
        RangeUpdate {
            start: 0,
            end: 0,
            delta: i64::MIN,
        },
    ];

    for (name, candidate) in ALL_IMPLEMENTATIONS {
        assert_eq!(candidate(&input, &updates), Ok(input.to_vec()), "{name}");
    }

    let empty_updates = [RangeUpdate {
        start: 0,
        end: 0,
        delta: 1,
    }];
    for (name, candidate) in ALL_IMPLEMENTATIONS {
        assert_eq!(candidate(&[], &empty_updates), Ok(Vec::new()), "{name}");
    }
}

#[test]
fn overflow_uses_wrapping_i64_arithmetic() {
    let input = [i64::MAX, i64::MIN, -1, 0];
    let updates = [
        RangeUpdate {
            start: 0,
            end: 2,
            delta: 1,
        },
        RangeUpdate {
            start: 1,
            end: 4,
            delta: i64::MAX,
        },
        RangeUpdate {
            start: 2,
            end: 3,
            delta: 2,
        },
    ];
    let expected = vec![i64::MIN, 0, i64::MIN, i64::MAX];

    for (name, candidate) in ALL_IMPLEMENTATIONS {
        assert_eq!(candidate(&input, &updates), Ok(expected.clone()), "{name}");
    }

    let min_stop_input = [0, 0];
    let min_stop_updates = [RangeUpdate {
        start: 0,
        end: 1,
        delta: i64::MIN,
    }];
    let min_stop_expected = vec![i64::MIN, 0];
    for (name, candidate) in ALL_IMPLEMENTATIONS {
        assert_eq!(
            candidate(&min_stop_input, &min_stop_updates),
            Ok(min_stop_expected.clone()),
            "{name}"
        );
    }
}

#[test]
fn first_invalid_update_is_reported_exactly() {
    let input = vec![11, 12, 13];
    let start_after_end_first = [
        RangeUpdate {
            start: 0,
            end: 3,
            delta: 1,
        },
        RangeUpdate {
            start: 2,
            end: 1,
            delta: 7,
        },
        RangeUpdate {
            start: 0,
            end: 4,
            delta: 9,
        },
    ];
    let expected_start_after_end = RangeError {
        update_index: 1,
        start: 2,
        end: 1,
        len: 3,
    };

    for (name, candidate) in ALL_IMPLEMENTATIONS {
        assert_eq!(
            candidate(&input, &start_after_end_first),
            Err(expected_start_after_end),
            "{name}"
        );
        assert_eq!(input, [11, 12, 13], "{name} changed the caller input");
    }

    let out_of_bounds_later = [
        RangeUpdate {
            start: 3,
            end: 3,
            delta: 5,
        },
        RangeUpdate {
            start: 0,
            end: 2,
            delta: 1,
        },
        RangeUpdate {
            start: 1,
            end: 4,
            delta: 2,
        },
    ];
    let expected_out_of_bounds = RangeError {
        update_index: 2,
        start: 1,
        end: 4,
        len: 3,
    };
    for (name, candidate) in ALL_IMPLEMENTATIONS {
        assert_eq!(
            candidate(&input, &out_of_bounds_later),
            Err(expected_out_of_bounds),
            "{name}"
        );
    }

    let out_of_bounds_before_reversed = [
        RangeUpdate {
            start: 0,
            end: 4,
            delta: 1,
        },
        RangeUpdate {
            start: 2,
            end: 1,
            delta: 2,
        },
    ];
    let expected_first_out_of_bounds = RangeError {
        update_index: 0,
        start: 0,
        end: 4,
        len: 3,
    };
    for (name, candidate) in ALL_IMPLEMENTATIONS {
        assert_eq!(
            candidate(&input, &out_of_bounds_before_reversed),
            Err(expected_first_out_of_bounds),
            "{name}"
        );
    }
}

#[test]
fn duplicate_and_canceling_endpoints_preserve_multiplicity() {
    let input = [5, 6, 7, 8, 9];
    let updates = [
        RangeUpdate {
            start: 1,
            end: 4,
            delta: 7,
        },
        RangeUpdate {
            start: 1,
            end: 4,
            delta: 7,
        },
        RangeUpdate {
            start: 1,
            end: 4,
            delta: -14,
        },
        RangeUpdate {
            start: 0,
            end: 5,
            delta: i64::MIN,
        },
        RangeUpdate {
            start: 0,
            end: 5,
            delta: i64::MIN,
        },
        RangeUpdate {
            start: 2,
            end: 5,
            delta: 3,
        },
    ];
    let expected = vec![5, 6, 10, 11, 12];

    for (name, candidate) in ALL_IMPLEMENTATIONS {
        assert_eq!(candidate(&input, &updates), Ok(expected.clone()), "{name}");
    }
}

#[test]
fn permuting_valid_updates_preserves_exact_output() {
    let input = [0; 5];
    let mut updates = [
        RangeUpdate {
            start: 0,
            end: 3,
            delta: 2,
        },
        RangeUpdate {
            start: 1,
            end: 5,
            delta: -1,
        },
        RangeUpdate {
            start: 2,
            end: 4,
            delta: 5,
        },
        RangeUpdate {
            start: 3,
            end: 3,
            delta: i64::MAX,
        },
    ];
    let expected = [2, 1, 6, 4, -1];

    let mut seen = BTreeSet::new();
    let permutation_count = assert_every_permutation(&input, &mut updates, 0, &expected, &mut seen);
    assert_eq!(permutation_count, 24);
    assert_eq!(seen.len(), 24);
}

#[test]
fn exhaustive_small_domain_matches_independent_reference() {
    const VALUES: [i64; 3] = [-1, 0, 1];
    const DELTAS: [i64; 3] = [-1, 0, 1];
    let mut cases = 0_u64;

    for len in 0..=4 {
        let base_count = 3_usize.pow(u32::try_from(len).expect("small length"));
        let bases: Vec<Vec<i64>> = (0..base_count)
            .map(|base_code| decode_base(base_code, len, &VALUES))
            .collect();
        assert!(bases.iter().all(|input| input.len() == len));
        assert!(bases.iter().flatten().all(|value| VALUES.contains(value)));
        let base_universe: BTreeSet<_> = bases.iter().cloned().collect();
        assert_eq!(base_universe.len(), base_count);
        let update_options = valid_update_options(len, &DELTAS);
        let expected_option_count = (len + 1) * (len + 2) / 2 * DELTAS.len();
        assert_eq!(update_options.len(), expected_option_count);
        let option_universe: BTreeSet<_> = update_options
            .iter()
            .map(|update| {
                assert!(update.start <= update.end);
                assert!(update.end <= len);
                assert!(DELTAS.contains(&update.delta));
                (update.start, update.end, update.delta)
            })
            .collect();
        assert_eq!(option_universe.len(), expected_option_count);
        for input in bases {
            assert_matches_reference(&input, &[]);
            cases += 1;

            for &first in &update_options {
                assert_matches_reference(&input, &[first]);
                cases += 1;
                for &second in &update_options {
                    assert_matches_reference(&input, &[first, second]);
                    cases += 1;
                }
            }
        }
    }

    assert_eq!(cases, 196_261);
}

#[test]
fn seeded_differential_cases_match_independent_reference() {
    const SEED: u64 = 2_026_083_002;
    const CASES: usize = 10_000;
    let mut rng = Lcg::new(SEED);
    let mut lengths = BTreeSet::new();
    let mut update_counts = BTreeSet::new();
    let mut saw_empty_range = false;
    let mut saw_end_at_len = false;
    let mut saw_overlap = false;
    let mut saw_min = false;
    let mut saw_max = false;
    let mut saw_negative_one = false;
    let mut saw_arbitrary_value = false;
    let mut input_value_classes = [0_usize; 6];
    let mut input_values = 0_usize;

    for case_index in 0..CASES {
        let len = rng.below(129);
        let update_count = rng.below(65);
        let input: Vec<i64> = (0..len).map(|_| rng.mixed_i64()).collect();
        let updates: Vec<RangeUpdate> = (0..update_count)
            .map(|_| {
                let start = rng.below(len + 1);
                let end = start + rng.below(len - start + 1);
                RangeUpdate {
                    start,
                    end,
                    delta: rng.mixed_i64(),
                }
            })
            .collect();

        lengths.insert(len);
        update_counts.insert(update_count);
        for &value in &input {
            input_value_classes[value_class(value)] += 1;
            input_values += 1;
        }
        saw_empty_range |= updates.iter().any(|update| update.start == update.end);
        saw_end_at_len |= updates.iter().any(|update| update.end == len);
        saw_min |= input
            .iter()
            .chain(updates.iter().map(|update| &update.delta))
            .any(|&value| value == i64::MIN);
        saw_max |= input
            .iter()
            .chain(updates.iter().map(|update| &update.delta))
            .any(|&value| value == i64::MAX);
        saw_negative_one |= input
            .iter()
            .chain(updates.iter().map(|update| &update.delta))
            .any(|&value| value == -1);
        saw_arbitrary_value |= input
            .iter()
            .chain(updates.iter().map(|update| &update.delta))
            .any(|value| !matches!(*value, i64::MIN | i64::MAX | -1 | 0 | 1));
        if !saw_overlap {
            saw_overlap = updates.iter().enumerate().any(|(left_index, left)| {
                left.start < left.end
                    && updates.iter().skip(left_index + 1).any(|right| {
                        right.start < right.end
                            && left.start.max(right.start) < left.end.min(right.end)
                    })
            });
        }

        let expected = apply_reference(&input, &updates).expect("generated ranges are valid");
        for (name, candidate) in CANDIDATES {
            assert_eq!(
                candidate(&input, &updates),
                Ok(expected.clone()),
                "candidate={name} case={case_index} seed={SEED} input={input:?} updates={updates:?}"
            );
        }
    }

    assert_eq!(lengths.len(), 129);
    assert_eq!(update_counts.len(), 65);
    assert!(saw_empty_range);
    assert!(saw_end_at_len);
    assert!(saw_overlap);
    assert!(saw_min);
    assert!(saw_max);
    assert!(saw_negative_one);
    assert!(saw_arbitrary_value);

    // A state advance that depends on the drawn class can starve classes while presence checks still pass. commentlint: allow(JUDGE)
    let floor = input_values / 20;
    for (class, &count) in input_value_classes.iter().enumerate() {
        assert!(
            count >= floor,
            "input value class {class} appeared {count} times, \
             below the {floor} floor over {input_values} generated elements"
        );
    }
}

/// Classifies values according to [`Lcg::mixed_i64`]'s six output classes.
fn value_class(value: i64) -> usize {
    match value {
        i64::MIN => 0,
        i64::MAX => 1,
        -1 => 2,
        0 => 3,
        1 => 4,
        _ => 5,
    }
}

fn assert_matches_reference(input: &[i64], updates: &[RangeUpdate]) {
    let expected = apply_reference(input, updates).expect("exhaustive ranges are valid");
    for (name, candidate) in CANDIDATES {
        assert_eq!(
            candidate(input, updates),
            Ok(expected.clone()),
            "candidate={name} input={input:?} updates={updates:?}"
        );
    }
}

fn assert_every_permutation(
    input: &[i64],
    updates: &mut [RangeUpdate],
    next: usize,
    expected: &[i64],
    seen: &mut BTreeSet<Vec<(usize, usize, i64)>>,
) -> usize {
    if next == updates.len() {
        let permutation = updates
            .iter()
            .map(|update| (update.start, update.end, update.delta))
            .collect();
        assert!(
            seen.insert(permutation),
            "duplicate permutation: {updates:?}"
        );
        for (name, candidate) in ALL_IMPLEMENTATIONS {
            assert_eq!(
                candidate(input, updates),
                Ok(expected.to_vec()),
                "candidate={name} updates={updates:?}"
            );
        }
        return 1;
    }

    let mut count = 0;
    for index in next..updates.len() {
        updates.swap(next, index);
        count += assert_every_permutation(input, updates, next + 1, expected, seen);
        updates.swap(next, index);
    }
    count
}

fn valid_update_options(len: usize, deltas: &[i64]) -> Vec<RangeUpdate> {
    let mut updates = Vec::new();
    for start in 0..=len {
        for end in start..=len {
            for &delta in deltas {
                updates.push(RangeUpdate { start, end, delta });
            }
        }
    }
    updates
}

fn decode_base(mut code: usize, len: usize, values: &[i64]) -> Vec<i64> {
    let mut input = Vec::with_capacity(len);
    for _ in 0..len {
        input.push(values[code % values.len()]);
        code /= values.len();
    }
    input
}

struct Lcg(u64);

impl Lcg {
    const fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }

    fn below(&mut self, upper: usize) -> usize {
        assert!(upper > 0);
        let upper_u64 = u64::try_from(upper).expect("test bounds fit u64");
        usize::try_from(self.next_u64() % upper_u64).expect("remainder fits usize")
    }

    /// `mixed_i64` draws both words so state advancement does not depend on `selector`. commentlint: allow(JUDGE)
    /// Bit `k` of this modulus-`2^64` LCG has period `2^(k+1)`, so bits 61-63 select far more evenly than bits 0-2. commentlint: allow(JUDGE)
    /// A separate `arbitrary` word keeps the unbiased branch full-range in both signs. commentlint: allow(JUDGE)
    fn mixed_i64(&mut self) -> i64 {
        let selector = self.next_u64() >> 61;
        let arbitrary = self.next_u64();
        match selector {
            0 => i64::MIN,
            1 => i64::MAX,
            2 => -1,
            3 => 0,
            4 => 1,
            _ => i64::from_ne_bytes(arbitrary.to_ne_bytes()),
        }
    }
}
