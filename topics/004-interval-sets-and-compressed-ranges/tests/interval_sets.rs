//! Contract, boundary, differential, metamorphic, and algebra tests.

use topic_004_interval_sets_and_compressed_ranges::{
    BTreeIntervalSet, BoundaryEventSet, DOMAIN_END, FlatIntervalSet, Interval, IntervalError,
    IntervalSet, PackedIntervalSet, PointOracle, difference, intersection, union,
};

fn interval(start: u64, end: u64) -> Interval {
    Interval::new(start, end)
}

struct Candidates {
    oracle: PointOracle,
    flat: FlatIntervalSet,
    packed: PackedIntervalSet,
    events: BoundaryEventSet,
    btree: BTreeIntervalSet,
}

impl Candidates {
    fn build(input: &[Interval]) -> Self {
        Self {
            oracle: PointOracle::try_from_intervals(input).expect("valid oracle fixture"),
            flat: FlatIntervalSet::try_from_intervals(input).expect("valid flat fixture"),
            packed: PackedIntervalSet::try_from_intervals(input).expect("valid packed fixture"),
            events: BoundaryEventSet::try_from_intervals(input).expect("valid event fixture"),
            btree: BTreeIntervalSet::try_from_intervals(input).expect("valid tree fixture"),
        }
    }

    fn views(&self) -> [(&'static str, &dyn IntervalSet); 5] {
        [
            ("oracle", &self.oracle),
            ("flat", &self.flat),
            ("packed", &self.packed),
            ("events", &self.events),
            ("btree", &self.btree),
        ]
    }

    fn assert_exact(&self, expected: &[Interval], probes: &[u32]) {
        let expected_cardinality = expected.iter().map(|item| item.len()).sum::<u64>();
        assert_canonical(expected);
        for (name, candidate) in self.views() {
            let observed = candidate.canonical_intervals();
            assert_eq!(observed, expected, "{name} canonical intervals");
            assert_canonical(&observed);
            assert_eq!(
                candidate.cardinality(),
                expected_cardinality,
                "{name} cardinality"
            );
            for &point in probes {
                let expected_membership = expected
                    .iter()
                    .any(|item| item.start() <= u64::from(point) && u64::from(point) < item.end());
                assert_eq!(
                    candidate.contains(point),
                    expected_membership,
                    "{name} membership at {point}"
                );
            }
        }
    }
}

fn assert_canonical(intervals: &[Interval]) {
    for item in intervals {
        assert!(item.start() < item.end(), "empty or reversed run: {item:?}");
        assert!(
            item.end() <= DOMAIN_END,
            "run endpoint escaped domain: {item:?}"
        );
    }
    for pair in intervals.windows(2) {
        assert!(
            pair[0].end() < pair[1].start(),
            "runs overlap, touch, or are unsorted: {pair:?}"
        );
    }
}

fn assert_all_constructor_errors(input: &[Interval], expected: IntervalError) {
    assert_eq!(
        PointOracle::try_from_intervals(input).map(|set| set.canonical_intervals()),
        Err(expected),
        "oracle"
    );
    assert_eq!(
        FlatIntervalSet::try_from_intervals(input).map(|set| set.canonical_intervals()),
        Err(expected),
        "flat"
    );
    assert_eq!(
        PackedIntervalSet::try_from_intervals(input).map(|set| set.canonical_intervals()),
        Err(expected),
        "packed"
    );
    assert_eq!(
        BoundaryEventSet::try_from_intervals(input).map(|set| set.canonical_intervals()),
        Err(expected),
        "events"
    );
    assert_eq!(
        BTreeIntervalSet::try_from_intervals(input).map(|set| set.canonical_intervals()),
        Err(expected),
        "btree"
    );
}

#[test]
fn running_example_canonicalizes_to_two_exact_runs_in_every_representation() {
    let input = [
        interval(5, 9),
        interval(1, 4),
        interval(3, 7),
        interval(9, 12),
        interval(6, 6),
        interval(14, 16),
    ];
    let expected = [interval(1, 12), interval(14, 16)];
    let probes = [0, 1, 3, 6, 11, 12, 13, 14, 15, 16, u32::MAX];

    Candidates::build(&input).assert_exact(&expected, &probes);
}

#[test]
fn error_precedence_reports_first_out_of_domain_endpoint_before_reversed_order() {
    let start_out_of_domain_and_reversed = [interval(DOMAIN_END + 7, 3)];
    let both_endpoints_out_of_domain = [interval(DOMAIN_END + 7, DOMAIN_END + 9)];
    let end_out_of_domain = [interval(0, 1), interval(4, DOMAIN_END + 9)];
    let reversed_in_domain = [interval(0, 1), interval(9, 4)];
    let earlier_reversed_before_later_domain_error =
        [interval(9, 4), interval(DOMAIN_END + 1, DOMAIN_END + 2)];

    assert_all_constructor_errors(
        &start_out_of_domain_and_reversed,
        IntervalError::OutOfDomain {
            index: 0,
            endpoint: DOMAIN_END + 7,
        },
    );
    assert_all_constructor_errors(
        &both_endpoints_out_of_domain,
        IntervalError::OutOfDomain {
            index: 0,
            endpoint: DOMAIN_END + 7,
        },
    );
    assert_all_constructor_errors(
        &end_out_of_domain,
        IntervalError::OutOfDomain {
            index: 1,
            endpoint: DOMAIN_END + 9,
        },
    );
    assert_all_constructor_errors(
        &reversed_in_domain,
        IntervalError::Reversed {
            index: 1,
            start: 9,
            end: 4,
        },
    );
    assert_all_constructor_errors(
        &earlier_reversed_before_later_domain_error,
        IntervalError::Reversed {
            index: 0,
            start: 9,
            end: 4,
        },
    );
}

#[test]
fn empty_intervals_at_zero_and_domain_end_are_no_ops() {
    let input = [
        interval(0, 0),
        interval(DOMAIN_END, DOMAIN_END),
        interval(5, 5),
    ];

    Candidates::build(&input).assert_exact(&[], &[0, 5, u32::MAX]);
}

#[test]
fn packed_full_domain_run_decodes_endpoint_with_widened_arithmetic() {
    let packed = PackedIntervalSet::try_from_intervals(&[interval(0, DOMAIN_END)])
        .expect("full-domain packed run");
    let run = packed.packed_runs()[0];

    assert_eq!(run.start(), 0);
    assert_eq!(run.length_minus_one(), u32::MAX);
    assert_eq!(run.decode(), interval(0, DOMAIN_END));
    assert_eq!(packed.cardinality(), DOMAIN_END);
    assert!(packed.contains(0));
    assert!(packed.contains(u32::MAX));

    let terminal =
        PackedIntervalSet::try_from_intervals(&[interval(u64::from(u32::MAX), DOMAIN_END)])
            .expect("terminal singleton packed run");
    assert_eq!(
        terminal.packed_runs()[0].decode(),
        interval(u64::from(u32::MAX), DOMAIN_END)
    );
}

#[test]
fn bridge_interval_coalesces_predecessor_and_all_adjacent_successors() {
    let input = [
        interval(1, 3),
        interval(9, 11),
        interval(5, 7),
        interval(3, 10),
    ];
    let tree = BTreeIntervalSet::try_from_intervals(&input).expect("valid bridge fixture");

    assert_eq!(tree.run_count(), 1);
    assert_eq!(tree.canonical_intervals(), [interval(1, 11)]);
    assert_eq!(tree.cardinality(), 10);
}

#[test]
fn grouped_boundary_events_preserve_coverage_across_canceling_adjacent_events() {
    let input = [
        interval(1, 3),
        interval(3, 5),
        interval(1, 3),
        interval(5, 7),
        interval(3, 5),
    ];
    let events = BoundaryEventSet::try_from_intervals(&input).expect("valid event fixture");

    assert_eq!(events.canonical_intervals(), [interval(1, 7)]);
    assert_eq!(events.cardinality(), 6);
    assert!(events.contains(3));
    assert!(events.contains(5));
}

#[test]
fn exhaustive_sequences_of_up_to_three_intervals_match_independent_point_model_on_domain_six() {
    let mut choices = vec![None];
    for start in 0..=6 {
        for end in start..=6 {
            choices.push(Some(interval(start, end)));
        }
    }

    for first in &choices {
        for second in &choices {
            for third in &choices {
                let input = [*first, *second, *third]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>();
                assert_optimized_match_oracle(
                    &input,
                    &(0..=7).collect::<Vec<_>>(),
                    "exhaustive domain-six sequence",
                );
            }
        }
    }
}

#[test]
fn seeded_differential_cases_match_independent_point_model() {
    for seed in [0_u64, 1, 0x05ee_d004, 0x9e37_79b9_7f4a_7c15, u64::MAX] {
        let mut random = Lcg::new(seed);
        for case_index in 0..64 {
            let mut input = Vec::with_capacity(48);
            for item_index in 0..48 {
                let left = random.bounded(257);
                let right = random.bounded(257);
                let start = left.min(right);
                let end = left.max(right);
                input.push(interval(start, end));
                if item_index % 11 == 0 {
                    input.push(interval(start, end));
                }
                if item_index % 13 == 0 {
                    input.push(interval(end, end));
                }
            }
            let replay = format!("seed {seed:#018x}, case {case_index}");
            assert_optimized_match_oracle(&input, &(0..=257).collect::<Vec<_>>(), &replay);
        }
    }
}

#[test]
fn permutation_duplicates_and_empty_intervals_preserve_anchored_set() {
    let anchor = [
        interval(2, 5),
        interval(8, 10),
        interval(4, 9),
        interval(20, 21),
    ];
    let expected = [interval(2, 10), interval(20, 21)];
    let mut transformed = anchor.to_vec();
    transformed.reverse();
    transformed.rotate_left(1);
    transformed.extend(anchor);
    transformed.push(interval(0, 0));
    transformed.push(interval(DOMAIN_END, DOMAIN_END));

    Candidates::build(&anchor).assert_exact(&expected, &[1, 2, 9, 10, 20, 21]);
    Candidates::build(&transformed).assert_exact(&expected, &[1, 2, 9, 10, 20, 21]);
}

#[test]
fn union_intersection_and_difference_match_independent_point_algebra() {
    let left_input = [interval(1, 5), interval(8, 14), interval(20, 22)];
    let right_input = [interval(3, 10), interval(12, 13), interval(18, 21)];
    let left = Candidates::build(&left_input);
    let right = Candidates::build(&right_input);
    let expected_union = left.oracle.union(&right.oracle).canonical_intervals();
    let expected_intersection = left
        .oracle
        .intersection(&right.oracle)
        .canonical_intervals();
    let expected_difference = left.oracle.difference(&right.oracle).canonical_intervals();

    left.assert_exact(
        &left.oracle.canonical_intervals(),
        &(0..=23).collect::<Vec<_>>(),
    );
    right.assert_exact(
        &right.oracle.canonical_intervals(),
        &(0..=23).collect::<Vec<_>>(),
    );

    for (left_name, left_view) in left.views() {
        for (right_name, right_view) in right.views() {
            assert_eq!(
                union(left_view, right_view).canonical_intervals(),
                expected_union,
                "union {left_name}/{right_name}"
            );
            assert_eq!(
                intersection(left_view, right_view).canonical_intervals(),
                expected_intersection,
                "intersection {left_name}/{right_name}"
            );
            assert_eq!(
                difference(left_view, right_view).canonical_intervals(),
                expected_difference,
                "difference {left_name}/{right_name}"
            );
        }
    }
}

#[test]
fn set_algebra_identities_hold_for_canonical_projection() {
    let a =
        FlatIntervalSet::try_from_intervals(&[interval(0, 4), interval(9, 13)]).expect("valid A");
    let b = PackedIntervalSet::try_from_intervals(&[interval(2, 10), interval(14, 17)])
        .expect("valid B");
    let c =
        BoundaryEventSet::try_from_intervals(&[interval(1, 3), interval(12, 16)]).expect("valid C");

    assert_eq!(
        union(&a, &b).canonical_intervals(),
        union(&b, &a).canonical_intervals(),
        "union commutativity"
    );
    assert_eq!(
        intersection(&a, &b).canonical_intervals(),
        intersection(&b, &a).canonical_intervals(),
        "intersection commutativity"
    );
    assert_eq!(
        union(&union(&a, &b), &c).canonical_intervals(),
        union(&a, &union(&b, &c)).canonical_intervals(),
        "union associativity"
    );
    assert_eq!(
        intersection(&intersection(&a, &b), &c).canonical_intervals(),
        intersection(&a, &intersection(&b, &c)).canonical_intervals(),
        "intersection associativity"
    );
    assert!(difference(&a, &a).canonical_intervals().is_empty());
    assert_eq!(
        union(&difference(&a, &b), &intersection(&a, &b)).canonical_intervals(),
        a.canonical_intervals(),
        "difference/intersection partition A"
    );
}

#[test]
fn set_algebra_identities_hold_for_the_empty_set_and_for_subsets() {
    let a =
        FlatIntervalSet::try_from_intervals(&[interval(0, 4), interval(9, 13)]).expect("valid A");
    let empty = FlatIntervalSet::try_from_intervals(&[]).expect("valid empty set");
    let subset = PackedIntervalSet::try_from_intervals(&[interval(1, 3), interval(10, 12)])
        .expect("valid subset of A");

    assert!(empty.canonical_intervals().is_empty(), "empty projection");
    assert_eq!(empty.cardinality(), 0, "empty cardinality");

    assert_eq!(
        union(&a, &empty).canonical_intervals(),
        a.canonical_intervals(),
        "union right identity"
    );
    assert_eq!(
        union(&empty, &a).canonical_intervals(),
        a.canonical_intervals(),
        "union left identity"
    );

    assert!(
        intersection(&a, &empty).canonical_intervals().is_empty(),
        "intersection right annihilator"
    );
    assert!(
        intersection(&empty, &a).canonical_intervals().is_empty(),
        "intersection left annihilator"
    );

    assert_eq!(
        difference(&a, &empty).canonical_intervals(),
        a.canonical_intervals(),
        "difference by the empty set"
    );
    assert!(
        difference(&empty, &a).canonical_intervals().is_empty(),
        "difference of the empty set"
    );

    assert_eq!(
        intersection(&subset, &a).canonical_intervals(),
        subset.canonical_intervals(),
        "intersection absorbs a subset"
    );
    assert!(
        difference(&subset, &a).canonical_intervals().is_empty(),
        "subset difference empties"
    );
    assert_eq!(
        difference(&a, &subset).canonical_intervals(),
        [
            interval(0, 1),
            interval(3, 4),
            interval(9, 10),
            interval(12, 13)
        ],
        "superset difference retains the relative complement"
    );
}

#[test]
fn operations_preserve_domain_end_without_narrowing() {
    let left = FlatIntervalSet::try_from_intervals(&[interval(DOMAIN_END - 10, DOMAIN_END)])
        .expect("valid terminal left run");
    let right = PackedIntervalSet::try_from_intervals(&[interval(DOMAIN_END - 4, DOMAIN_END)])
        .expect("valid terminal right run");

    assert_eq!(
        union(&left, &right).canonical_intervals(),
        [interval(DOMAIN_END - 10, DOMAIN_END)]
    );
    assert_eq!(
        intersection(&left, &right).canonical_intervals(),
        [interval(DOMAIN_END - 4, DOMAIN_END)]
    );
    assert_eq!(
        difference(&left, &right).canonical_intervals(),
        [interval(DOMAIN_END - 10, DOMAIN_END - 4)]
    );
}

#[test]
fn membership_observes_inclusive_start_and_exclusive_end() {
    let input = [
        interval(0, 1),
        interval(5, 8),
        interval(DOMAIN_END - 1, DOMAIN_END),
    ];
    let candidates = Candidates::build(&input);

    for (name, candidate) in candidates.views() {
        assert!(candidate.contains(0), "{name} omitted inclusive zero start");
        assert!(!candidate.contains(1), "{name} included exclusive end 1");
        assert!(!candidate.contains(4), "{name} included gap point 4");
        assert!(candidate.contains(5), "{name} omitted inclusive start 5");
        assert!(
            candidate.contains(7),
            "{name} omitted final covered point 7"
        );
        assert!(!candidate.contains(8), "{name} included exclusive end 8");
        assert!(
            candidate.contains(u32::MAX),
            "{name} omitted terminal u32 point"
        );
    }
}

fn assert_optimized_match_oracle(input: &[Interval], probes: &[u32], replay: &str) {
    let oracle = PointOracle::try_from_intervals(input).expect("bounded oracle fixture");
    let expected = oracle.canonical_intervals();
    let expected_cardinality = oracle.cardinality();
    let flat = FlatIntervalSet::try_from_intervals(input).expect("bounded flat fixture");
    let packed = PackedIntervalSet::try_from_intervals(input).expect("bounded packed fixture");
    let events = BoundaryEventSet::try_from_intervals(input).expect("bounded event fixture");
    let btree = BTreeIntervalSet::try_from_intervals(input).expect("bounded tree fixture");
    let optimized: [(&str, &dyn IntervalSet); 4] = [
        ("flat", &flat),
        ("packed", &packed),
        ("events", &events),
        ("btree", &btree),
    ];

    for (name, candidate) in optimized {
        let observed = candidate.canonical_intervals();
        assert_eq!(
            observed, expected,
            "{name} intervals for {input:?}; replay: {replay}"
        );
        assert_canonical(&observed);
        assert_eq!(
            candidate.cardinality(),
            expected_cardinality,
            "{name} cardinality for {input:?}; replay: {replay}"
        );
        for &point in probes {
            assert_eq!(
                candidate.contains(point),
                oracle.contains(point),
                "{name} membership at {point} for {input:?}; replay: {replay}"
            );
        }
    }
}

struct Lcg(u64);

impl Lcg {
    const fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }

    fn bounded(&mut self, exclusive: u64) -> u64 {
        self.next() % exclusive
    }
}
