//! Contract, exhaustive differential, seeded differential, and metamorphic tests.

use topic_005_two_pointers_and_sliding_windows::{
    count_bounded_subarrays, direct_sliding, oversized_reset_sliding, prefix_binary_search,
    quadratic_early_exit, recompute_reference,
};

type Candidate = fn(&[u64], u128) -> u128;

const IMPLEMENTATIONS: [(&str, Candidate); 6] = [
    ("default", count_bounded_subarrays),
    ("recompute reference", recompute_reference),
    ("quadratic early exit", quadratic_early_exit),
    ("direct sliding", direct_sliding),
    ("oversized reset sliding", oversized_reset_sliding),
    ("prefix binary search", prefix_binary_search),
];

const OPTIMIZED_IMPLEMENTATIONS: [(&str, Candidate); 5] = [
    ("default", count_bounded_subarrays),
    ("quadratic early exit", quadratic_early_exit),
    ("direct sliding", direct_sliding),
    ("oversized reset sliding", oversized_reset_sliding),
    ("prefix binary search", prefix_binary_search),
];

fn assert_all_equal(values: &[u64], budget: u128, expected: u128, context: &str) {
    for (name, candidate) in IMPLEMENTATIONS {
        assert_eq!(
            candidate(values, budget),
            expected,
            "{name} failed {context}; values={values:?}, budget={budget}"
        );
    }
}

fn triangular(length: usize) -> u128 {
    let length = u128::try_from(length).expect("usize fits u128");
    length * (length + 1) / 2
}

#[test]
fn running_example_counts_eleven_half_open_subarrays() {
    assert_all_equal(&[2, 0, 3, 2, 1], 5, 11, "running example");
}

#[test]
fn empty_zero_only_and_duplicate_prefix_boundaries_follow_the_contract() {
    assert_all_equal(&[], 0, 0, "empty input at zero budget");
    assert_all_equal(&[], u128::MAX, 0, "empty input at maximum budget");
    assert_all_equal(&[0], 0, 1, "one zero");
    assert_all_equal(&[0, 0, 0, 0, 0, 0], 0, 21, "six zeros");
    assert_all_equal(&[0, 0, 1, 0], 0, 4, "duplicate prefix sums");
}

#[test]
fn oversized_values_split_independent_valid_runs() {
    assert_all_equal(&[2, 100, 0, 3], 3, 4, "one oversized separator");
    assert_all_equal(
        &[u64::MAX, 0, u64::MAX, 0],
        0,
        2,
        "alternating oversized values and zeros",
    );
    assert_all_equal(
        &[u64::MAX, u64::MAX, u64::MAX],
        u128::from(u64::MAX) - 1,
        0,
        "every value is oversized",
    );
}

#[test]
fn maximum_value_separator_after_positive_prefix_resets_state() {
    let mut values = vec![1; 11];
    values.extend([u64::MAX, 0]);

    assert_all_equal(
        &values,
        11,
        67,
        "eleven-one run plus a deep maximum-value separator and trailing zero",
    );
}

#[test]
fn u128_accumulation_preserves_mathematical_sums_above_u64_max() {
    let values = [u64::MAX, u64::MAX];
    assert_all_equal(
        &values,
        u128::from(u64::MAX),
        2,
        "two singleton ranges fit but their pair does not",
    );
    assert_all_equal(
        &values,
        2 * u128::from(u64::MAX),
        3,
        "the pair fits in a wide budget",
    );
}

#[test]
fn exhaustive_lengths_zero_through_six_match_on_frozen_small_domain() {
    let mut checked = 0_u64;

    for length in 0_usize..=6 {
        let exponent = u32::try_from(length).expect("small exhaustive length fits u32");
        for encoded in 0..4_usize.pow(exponent) {
            let mut remainder = encoded;
            let mut values = Vec::with_capacity(length);
            for _ in 0..length {
                values.push(u64::try_from(remainder % 4).expect("base-four digit fits u64"));
                remainder /= 4;
            }

            for budget in 0_u128..20 {
                let expected = recompute_reference(&values, budget);
                for (name, candidate) in OPTIMIZED_IMPLEMENTATIONS {
                    assert_eq!(
                        candidate(&values, budget),
                        expected,
                        "{name}; values={values:?}, budget={budget}"
                    );
                }
                checked += 1;
            }
        }
    }

    // (1 + 4 + 4^2 + ... + 4^6) arrays times 20 budgets.
    assert_eq!(checked, 109_220);
}

#[test]
fn seeded_differential_cases_match_the_independent_reference() {
    for seed in [
        0_u64,
        1,
        0x0000_0005_5A17_D0C5,
        0x9E37_79B9_7F4A_7C15,
        u64::MAX,
    ] {
        let mut random = Lcg::new(seed);
        for case_index in 0..128 {
            let length = usize::try_from(random.bounded(25)).expect("bounded length fits usize");
            let mut values = Vec::with_capacity(length);
            for index in 0..length {
                let value = if index % 11 == 0 && random.bounded(4) == 0 {
                    u64::MAX
                } else {
                    random.bounded(33)
                };
                values.push(value);
            }
            let budget = match case_index % 5 {
                0 => 0,
                1 => u128::from(random.bounded(33)),
                2 => u128::from(random.bounded(513)),
                3 => u128::from(u64::MAX),
                _ => u128::MAX,
            };
            let expected = recompute_reference(&values, budget);
            for (name, candidate) in OPTIMIZED_IMPLEMENTATIONS {
                assert_eq!(
                    candidate(&values, budget),
                    expected,
                    "{name}; seed={seed:#018x}, case={case_index}, values={values:?}, budget={budget}"
                );
            }
        }
    }
}

#[test]
fn reversing_values_preserves_the_number_of_valid_ranges() {
    let fixtures = [
        (vec![], 0_u128),
        (vec![2, 0, 3, 2, 1], 5),
        (vec![7, 0, 0, 2, 9, 1], 9),
        (vec![u64::MAX, 0, 1], u128::from(u64::MAX)),
    ];

    for (values, budget) in fixtures {
        let expected = recompute_reference(&values, budget);
        let mut reversed = values.clone();
        reversed.reverse();
        assert_eq!(recompute_reference(&reversed, budget), expected);
        for (name, candidate) in OPTIMIZED_IMPLEMENTATIONS {
            assert_eq!(
                candidate(&reversed, budget),
                expected,
                "{name}; reversed={reversed:?}, budget={budget}"
            );
        }
    }
}

#[test]
fn increasing_the_budget_never_decreases_the_count() {
    let fixtures = [
        vec![],
        vec![0],
        vec![2, 0, 3, 2, 1],
        vec![u64::MAX, 0, 1, u64::MAX],
    ];
    let budgets = [
        0_u128,
        1,
        2,
        5,
        u128::from(u64::MAX),
        2 * u128::from(u64::MAX),
        u128::MAX,
    ];

    for values in fixtures {
        for (name, candidate) in IMPLEMENTATIONS {
            let mut previous = 0;
            for budget in budgets {
                let observed = candidate(&values, budget);
                assert!(
                    observed >= previous,
                    "{name}; values={values:?}, budget={budget}, previous={previous}, observed={observed}"
                );
                previous = observed;
            }
        }
    }
}

#[test]
fn scaling_values_and_budget_by_a_positive_factor_preserves_the_count() {
    let values = [0_u64, 1, 4, 2, 0, 6, 3];
    let budget = 7_u128;
    let factor = 9_u64;
    let scaled = values
        .iter()
        .map(|&value| value * factor)
        .collect::<Vec<_>>();
    let expected = recompute_reference(&values, budget);

    for (name, candidate) in IMPLEMENTATIONS {
        assert_eq!(
            candidate(&scaled, budget * u128::from(factor)),
            expected,
            "{name}; scaled positive-factor fixture"
        );
    }
}

#[test]
fn a_budget_covering_the_total_sum_counts_the_triangular_number() {
    let fixtures = [
        vec![],
        vec![0],
        vec![1, 2, 3, 4],
        vec![u64::MAX, u64::MAX, 0, 1],
    ];

    for values in fixtures {
        let total = values.iter().map(|&value| u128::from(value)).sum();
        assert_all_equal(
            &values,
            total,
            triangular(values.len()),
            "all-fit triangular formula",
        );
    }
}

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0xD1B5_4A32_D192_ED03,
        }
    }

    fn next(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }

    fn bounded(&mut self, upper: u64) -> u64 {
        assert!(upper > 0);
        self.next() % upper
    }
}
