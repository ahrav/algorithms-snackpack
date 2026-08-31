//! Contract, boundary, differential, representation, and algebra tests.

use std::collections::BTreeSet;

use topic_003_density_adaptive_bitmaps::{
    AdaptiveBitmap, BitmapError, ContainerKind, ContainerSummary, DenseBitSet, ReferenceSet,
    SortedSet,
};

const TWO_TO_32: u64 = 1_u64 << 32;

struct Sets {
    reference: ReferenceSet,
    sorted: SortedSet,
    dense: DenseBitSet,
    adaptive: AdaptiveBitmap,
}

impl Sets {
    fn new(universe: u64, values: &[u32]) -> Self {
        Self {
            reference: ReferenceSet::try_new(universe, values).expect("valid reference fixture"),
            sorted: SortedSet::try_new(universe, values).expect("valid sorted fixture"),
            dense: DenseBitSet::try_new(universe, values).expect("valid dense fixture"),
            adaptive: AdaptiveBitmap::try_new(universe, values).expect("valid adaptive fixture"),
        }
    }

    fn assert_views(&self, universe: u64, expected: &[u32]) {
        let membership_probes = membership_probes(universe, expected);
        let views: [(&str, &dyn SetView); 4] = [
            ("reference", &self.reference),
            ("sorted", &self.sorted),
            ("dense", &self.dense),
            ("adaptive", &self.adaptive),
        ];
        for (name, set) in views {
            assert_eq!(set.universe_exclusive(), universe, "{name}");
            assert_eq!(set.len(), expected.len(), "{name}");
            assert_eq!(set.is_empty(), expected.is_empty(), "{name}");
            assert_eq!(set.values(), expected, "{name}");
            for &value in &membership_probes {
                let expected_membership = expected.binary_search(&value).is_ok();
                assert_eq!(
                    set.contains(value),
                    expected_membership,
                    "{name} membership mismatch at {value}"
                );
            }
            if let Ok(outside) = u32::try_from(universe) {
                assert!(!set.contains(outside), "{name} admitted {outside}");
                if outside != u32::MAX {
                    assert!(!set.contains(u32::MAX), "{name} admitted {}", u32::MAX);
                }
            }
        }
    }
}

fn membership_probes(universe: u64, expected: &[u32]) -> Vec<u32> {
    let mut probes = BTreeSet::new();
    if universe <= 256 {
        probes.extend(0..u32::try_from(universe).expect("small universe fits u32"));
    } else {
        probes.insert(0);
        probes.insert(u32::try_from(universe / 2).expect("valid universe midpoint fits u32"));
        probes.insert(u32::try_from(universe - 1).expect("valid universe endpoint fits u32"));
        for &value in expected {
            probes.insert(value);
            if value > 0 {
                probes.insert(value - 1);
            }
            if u64::from(value) + 1 < universe {
                probes.insert(value + 1);
            }
        }
    }
    probes.into_iter().collect()
}

trait SetView {
    fn universe_exclusive(&self) -> u64;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn contains(&self, value: u32) -> bool;
    fn values(&self) -> Vec<u32>;
}

macro_rules! impl_set_view {
    ($set:ty) => {
        impl SetView for $set {
            fn universe_exclusive(&self) -> u64 {
                self.universe_exclusive()
            }

            fn len(&self) -> usize {
                self.len()
            }

            fn is_empty(&self) -> bool {
                self.is_empty()
            }

            fn contains(&self, value: u32) -> bool {
                self.contains(value)
            }

            fn values(&self) -> Vec<u32> {
                self.iter().collect()
            }
        }
    };
}

impl_set_view!(ReferenceSet);
impl_set_view!(SortedSet);
impl_set_view!(DenseBitSet);
impl_set_view!(AdaptiveBitmap);

#[test]
fn running_example_matches_all_candidates() {
    let universe = 16;
    let left_values = [1, 2, 3, 4, 10, 12, 14, 15];
    let right_values = [2, 3, 5, 10, 11, 14];
    let expected_intersection = [2, 3, 10, 14];
    let left = Sets::new(universe, &left_values);
    let right = Sets::new(universe, &right_values);

    left.assert_views(universe, &left_values);
    right.assert_views(universe, &right_values);
    assert_eq!(left.reference.intersection_len(&right.reference), Ok(4));
    assert_eq!(left.sorted.intersection_len(&right.sorted), Ok(4));
    assert_eq!(left.dense.intersection_len(&right.dense), Ok(4));
    assert_eq!(left.adaptive.intersection_len(&right.adaptive), Ok(4));

    let observed = left
        .reference
        .iter()
        .filter(|value| right.reference.contains(*value))
        .collect::<Vec<_>>();
    assert_eq!(observed, expected_intersection);
}

#[test]
fn zero_universe_and_out_of_universe_contract() {
    let empty = Sets::new(0, &[]);
    empty.assert_views(0, &[]);

    let too_large = BitmapError::UniverseTooLarge {
        universe_exclusive: TWO_TO_32 + 1,
    };
    let first_invalid = BitmapError::ValueOutOfUniverse {
        input_index: 1,
        value: 4,
        universe_exclusive: 4,
    };
    let zero_invalid = BitmapError::ValueOutOfUniverse {
        input_index: 0,
        value: 0,
        universe_exclusive: 0,
    };

    assert_constructor_errors(TWO_TO_32 + 1, &[0, u32::MAX], too_large);
    assert_constructor_errors(4, &[3, 4, 9], first_invalid);
    assert_constructor_errors(0, &[0], zero_invalid);

    let bounded = Sets::new(4, &[0, 3]);
    assert!(!bounded.reference.contains(4));
    assert!(!bounded.sorted.contains(4));
    assert!(!bounded.dense.contains(4));
    assert!(!bounded.adaptive.contains(4));
    assert!(!bounded.reference.contains(u32::MAX));
    assert!(!bounded.sorted.contains(u32::MAX));
    assert!(!bounded.dense.contains(u32::MAX));
    assert!(!bounded.adaptive.contains(u32::MAX));

    let reference =
        ReferenceSet::try_new(TWO_TO_32, &[0, u32::MAX]).expect("full u32 reference universe");
    let sorted = SortedSet::try_new(TWO_TO_32, &[0, u32::MAX]).expect("full u32 sorted universe");
    let adaptive =
        AdaptiveBitmap::try_new(TWO_TO_32, &[0, u32::MAX]).expect("full u32 adaptive universe");
    assert_eq!(reference.iter().collect::<Vec<_>>(), [0, u32::MAX]);
    assert_eq!(sorted.iter().collect::<Vec<_>>(), [0, u32::MAX]);
    assert_eq!(adaptive.iter().collect::<Vec<_>>(), [0, u32::MAX]);
    assert!(reference.contains(u32::MAX));
    assert!(sorted.contains(u32::MAX));
    assert!(adaptive.contains(u32::MAX));
}

#[test]
fn iteration_is_sorted_unique_and_duplicates_are_idempotent() {
    let input = [9, 1, 9, 4, 1, 7, 4, 0, 9];
    let expected = [0, 1, 4, 7, 9];
    let sets = Sets::new(10, &input);

    sets.assert_views(10, &expected);
    assert_eq!(sets.reference.payload_bytes(), 20);
    assert_eq!(sets.sorted.payload_bytes(), 20);
    assert_eq!(sets.dense.payload_bytes(), 8);
}

#[test]
fn word_and_chunk_boundaries_match_reference() {
    let input = [131_072, 65_536, 65, 64, 63, 0, 65_535, 65_537, 131_071, 64];
    let expected = [0, 63, 64, 65, 65_535, 65_536, 65_537, 131_071, 131_072];
    let sets = Sets::new(131_073, &input);

    sets.assert_views(131_073, &expected);
    assert_eq!(sets.dense.payload_bytes(), 16_392);
    assert_eq!(
        sets.adaptive
            .container_summaries()
            .map(|summary| summary.key)
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );

    let sparse_directory =
        AdaptiveBitmap::try_new(131_073, &[1, 131_072]).expect("valid sparse directory");
    assert_eq!(
        sparse_directory
            .container_summaries()
            .map(|summary| summary.key)
            .collect::<Vec<_>>(),
        vec![0, 2],
        "the empty high-16 key 1 container must be omitted"
    );
}

#[test]
fn adaptive_thresholds_and_strict_run_selection_are_exact() {
    let tie = AdaptiveBitmap::try_new(4, &[0, 1, 2]).expect("valid tie fixture");
    assert_eq!(
        tie.container_summaries().collect::<Vec<_>>(),
        vec![ContainerSummary {
            key: 0,
            kind: ContainerKind::Array,
            cardinality: 3,
            run_count: 1,
            payload_bytes: 6,
        }]
    );

    let consecutive = (0_u32..4).collect::<Vec<_>>();
    let run = AdaptiveBitmap::try_new(4, &consecutive).expect("valid run fixture");
    assert_eq!(
        run.container_summaries().collect::<Vec<_>>(),
        vec![ContainerSummary {
            key: 0,
            kind: ContainerKind::Run,
            cardinality: 4,
            run_count: 1,
            payload_bytes: 6,
        }]
    );

    let array_values = (0_u32..4_096).map(|value| value * 2).collect::<Vec<_>>();
    let array = AdaptiveBitmap::try_new(8_191, &array_values).expect("valid array fixture");
    assert_eq!(
        array.container_summaries().collect::<Vec<_>>(),
        vec![ContainerSummary {
            key: 0,
            kind: ContainerKind::Array,
            cardinality: 4_096,
            run_count: 4_096,
            payload_bytes: 8_192,
        }]
    );

    let below_threshold_values = (0_u32..4_095).map(|value| value * 2).collect::<Vec<_>>();
    let below_threshold = AdaptiveBitmap::try_new(8_189, &below_threshold_values)
        .expect("valid below-threshold fixture");
    assert_eq!(
        below_threshold.container_summaries().collect::<Vec<_>>(),
        vec![ContainerSummary {
            key: 0,
            kind: ContainerKind::Array,
            cardinality: 4_095,
            run_count: 4_095,
            payload_bytes: 8_190,
        }]
    );

    let bitmap_values = (0_u32..4_097).map(|value| value * 2).collect::<Vec<_>>();
    let bitmap = AdaptiveBitmap::try_new(8_193, &bitmap_values).expect("valid bitmap fixture");
    assert_eq!(
        bitmap.container_summaries().collect::<Vec<_>>(),
        vec![ContainerSummary {
            key: 0,
            kind: ContainerKind::Bitmap,
            cardinality: 4_097,
            run_count: 4_097,
            payload_bytes: 8_192,
        }]
    );

    let long_run_values = (0_u32..4_097).collect::<Vec<_>>();
    let long_run =
        AdaptiveBitmap::try_new(4_097, &long_run_values).expect("valid long-run fixture");
    assert_eq!(
        long_run.container_summaries().collect::<Vec<_>>(),
        vec![ContainerSummary {
            key: 0,
            kind: ContainerKind::Run,
            cardinality: 4_097,
            run_count: 1,
            payload_bytes: 6,
        }]
    );

    let terminal_values = (65_532_u32..=65_535).collect::<Vec<_>>();
    let terminal_run =
        AdaptiveBitmap::try_new(65_536, &terminal_values).expect("valid terminal run fixture");
    assert_eq!(terminal_run.iter().collect::<Vec<_>>(), terminal_values);
    assert_eq!(
        terminal_run.container_summaries().collect::<Vec<_>>(),
        vec![ContainerSummary {
            key: 0,
            kind: ContainerKind::Run,
            cardinality: 4,
            run_count: 1,
            payload_bytes: 6,
        }]
    );

    assert_eq!(tie.payload_bytes(), 6);
    assert_eq!(run.payload_bytes(), 6);
    assert_eq!(below_threshold.payload_bytes(), 8_190);
    assert_eq!(array.payload_bytes(), 8_192);
    assert_eq!(bitmap.payload_bytes(), 8_192);
    assert_eq!(long_run.payload_bytes(), 6);
    assert_eq!(terminal_run.payload_bytes(), 6);
}

#[test]
fn intersection_rejects_universe_mismatch() {
    let expected = BitmapError::UniverseMismatch {
        left_universe_exclusive: 8,
        right_universe_exclusive: 9,
    };
    let expected_reverse = BitmapError::UniverseMismatch {
        left_universe_exclusive: 9,
        right_universe_exclusive: 8,
    };
    let left = Sets::new(8, &[1, 3, 7]);
    let right = Sets::new(9, &[1, 3, 8]);

    assert_eq!(
        left.reference.intersection_len(&right.reference),
        Err(expected)
    );
    assert_eq!(left.sorted.intersection_len(&right.sorted), Err(expected));
    assert_eq!(left.dense.intersection_len(&right.dense), Err(expected));
    assert_eq!(
        left.adaptive.intersection_len(&right.adaptive),
        Err(expected)
    );
    assert_eq!(
        right.reference.intersection_len(&left.reference),
        Err(expected_reverse)
    );
    assert_eq!(
        right.sorted.intersection_len(&left.sorted),
        Err(expected_reverse)
    );
    assert_eq!(
        right.dense.intersection_len(&left.dense),
        Err(expected_reverse)
    );
    assert_eq!(
        right.adaptive.intersection_len(&left.adaptive),
        Err(expected_reverse)
    );
}

#[test]
fn intersection_len_matches_independent_reference() {
    let cases = mixed_container_intersection_cases();
    for (case_index, (universe, left_values, right_values)) in cases.iter().enumerate() {
        let left = Sets::new(*universe, left_values);
        let right = Sets::new(*universe, right_values);
        let expected_left = left.reference.iter().collect::<Vec<_>>();
        let expected_right = right.reference.iter().collect::<Vec<_>>();
        left.assert_views(*universe, &expected_left);
        right.assert_views(*universe, &expected_right);
        let expected = left
            .reference
            .intersection_len(&right.reference)
            .expect("matching reference universes");

        assert_eq!(
            left.sorted.intersection_len(&right.sorted),
            Ok(expected),
            "sorted case {case_index}"
        );
        assert_eq!(
            left.dense.intersection_len(&right.dense),
            Ok(expected),
            "dense case {case_index}"
        );
        assert_eq!(
            left.adaptive.intersection_len(&right.adaptive),
            Ok(expected),
            "adaptive case {case_index}"
        );
        assert_eq!(
            left.adaptive.intersection_len(&right.adaptive),
            right.adaptive.intersection_len(&left.adaptive),
            "adaptive symmetry case {case_index}"
        );
    }
}

#[test]
fn exhaustive_small_domain_matches_reference() {
    let mut checked_pairs = 0_u64;
    for universe in 0_u32..=8 {
        let subset_count = 1_u32 << universe;
        for left_mask in 0..subset_count {
            let left_values = values_from_mask(universe, left_mask);
            let left = Sets::new(u64::from(universe), &left_values);
            let expected_left = left.reference.iter().collect::<Vec<_>>();
            assert_eq!(
                expected_left.len(),
                usize::try_from(left_mask.count_ones()).expect("small cardinality fits usize")
            );
            for value in 0..universe {
                assert_eq!(
                    left.reference.contains(value),
                    left_mask & (1_u32 << value) != 0,
                    "universe={universe} mask={left_mask} value={value}"
                );
            }
            left.assert_views(u64::from(universe), &expected_left);

            for right_mask in 0..subset_count {
                let right_values = values_from_mask(universe, right_mask);
                let right = Sets::new(u64::from(universe), &right_values);
                let expected = left
                    .reference
                    .intersection_len(&right.reference)
                    .expect("matching reference universes");
                assert_eq!(left.sorted.intersection_len(&right.sorted), Ok(expected));
                assert_eq!(left.dense.intersection_len(&right.dense), Ok(expected));
                assert_eq!(
                    left.adaptive.intersection_len(&right.adaptive),
                    Ok(expected)
                );
                checked_pairs += 1;
            }
        }
    }
    assert_eq!(checked_pairs, 87_381);
}

#[test]
fn seeded_immutable_cases_match_reference() {
    const SEED: u64 = 2_026_083_103;
    const CASES: usize = 10_000;
    const UNIVERSES: [u32; 10] = [0, 1, 63, 64, 65, 255, 65_535, 65_536, 65_537, 131_072];

    let mut rng = Lcg::new(SEED);
    let mut seen_universes = BTreeSet::new();
    let mut saw_duplicate = false;
    let mut saw_empty = false;
    let mut saw_high_chunk = false;

    for case_index in 0..CASES {
        let universe = UNIVERSES[rng.below(UNIVERSES.len())];
        seen_universes.insert(universe);
        let left_values = generated_values(universe, rng.below(97), &mut rng);
        let right_values = generated_values(universe, rng.below(97), &mut rng);
        saw_duplicate |= has_duplicate(&left_values) || has_duplicate(&right_values);
        saw_empty |= left_values.is_empty() || right_values.is_empty();
        saw_high_chunk |= left_values
            .iter()
            .chain(&right_values)
            .any(|value| *value >= 65_536);

        let left = Sets::new(u64::from(universe), &left_values);
        let right = Sets::new(u64::from(universe), &right_values);
        let expected_left = left.reference.iter().collect::<Vec<_>>();
        let expected_right = right.reference.iter().collect::<Vec<_>>();
        left.assert_views(u64::from(universe), &expected_left);
        right.assert_views(u64::from(universe), &expected_right);
        let expected_intersection = left
            .reference
            .intersection_len(&right.reference)
            .expect("matching reference universes");
        assert_eq!(
            left.sorted.intersection_len(&right.sorted),
            Ok(expected_intersection),
            "sorted case {case_index} seed {SEED}"
        );
        assert_eq!(
            left.dense.intersection_len(&right.dense),
            Ok(expected_intersection),
            "dense case {case_index} seed {SEED}"
        );
        assert_eq!(
            left.adaptive.intersection_len(&right.adaptive),
            Ok(expected_intersection),
            "adaptive case {case_index} seed {SEED}"
        );
    }

    assert_eq!(seen_universes.len(), UNIVERSES.len());
    assert!(saw_duplicate);
    assert!(saw_empty);
    assert!(saw_high_chunk);
}

#[test]
fn intersection_laws_hold_for_all_candidates() {
    let universe = 128;
    let left_values = [1, 2, 3, 64, 65, 90, 127, 3];
    let right_values = [0, 2, 3, 63, 64, 90, 126, 2];
    let empty_values = [];

    assert_reference_laws(universe, &left_values, &right_values, &empty_values);
    assert_sorted_laws(universe, &left_values, &right_values, &empty_values);
    assert_dense_laws(universe, &left_values, &right_values, &empty_values);
    assert_adaptive_laws(universe, &left_values, &right_values, &empty_values);
}

fn assert_constructor_errors(universe: u64, values: &[u32], expected: BitmapError) {
    assert_eq!(ReferenceSet::try_new(universe, values), Err(expected));
    assert_eq!(SortedSet::try_new(universe, values), Err(expected));
    assert_eq!(DenseBitSet::try_new(universe, values), Err(expected));
    assert_eq!(AdaptiveBitmap::try_new(universe, values), Err(expected));
}

fn mixed_container_intersection_cases() -> Vec<(u64, Vec<u32>, Vec<u32>)> {
    let array = (0_u32..100).map(|value| value * 2).collect::<Vec<_>>();
    let run = (50_u32..250).collect::<Vec<_>>();
    let bitmap = (0_u32..4_097).map(|value| value * 2).collect::<Vec<_>>();
    let shifted_bitmap = (2_000_u32..6_097)
        .map(|value| value * 2)
        .collect::<Vec<_>>();
    vec![
        (13_000, array.clone(), run.clone()),
        (13_000, array.clone(), bitmap.clone()),
        (13_000, bitmap.clone(), shifted_bitmap),
        (13_000, bitmap.clone(), run.clone()),
        (13_000, run.clone(), (150_u32..350).collect()),
        (13_000, array.clone(), array.clone()),
        (
            131_074,
            vec![1, 64, 65_536, 131_072],
            vec![0, 64, 65_536, 65_537],
        ),
        (13_000, array, (1_u32..200).step_by(2).collect()),
    ]
}

fn values_from_mask(universe: u32, mask: u32) -> Vec<u32> {
    let mut values = (0..universe)
        .filter(|value| mask & (1_u32 << value) != 0)
        .collect::<Vec<_>>();
    if let Some(&first) = values.first() {
        values.push(first);
    }
    values.reverse();
    values
}

fn generated_values(universe: u32, count: usize, rng: &mut Lcg) -> Vec<u32> {
    if universe == 0 {
        return Vec::new();
    }
    let mut values = Vec::with_capacity(count + usize::from(count > 0));
    for _ in 0..count {
        let value = u32::try_from(rng.next_u64() % u64::from(universe))
            .expect("generated remainder fits u32");
        values.push(value);
    }
    if let Some(&first) = values.first() {
        values.push(first);
    }
    values
}

fn has_duplicate(values: &[u32]) -> bool {
    let unique = values.iter().copied().collect::<BTreeSet<_>>();
    unique.len() != values.len()
}

fn assert_reference_laws(universe: u64, left: &[u32], right: &[u32], empty: &[u32]) {
    let left = ReferenceSet::try_new(universe, left).expect("valid left");
    let right = ReferenceSet::try_new(universe, right).expect("valid right");
    let empty = ReferenceSet::try_new(universe, empty).expect("valid empty");
    assert_laws(
        left.len(),
        right.len(),
        left.intersection_len(&left),
        left.intersection_len(&right),
        right.intersection_len(&left),
        left.intersection_len(&empty),
    );
}

fn assert_sorted_laws(universe: u64, left: &[u32], right: &[u32], empty: &[u32]) {
    let left = SortedSet::try_new(universe, left).expect("valid left");
    let right = SortedSet::try_new(universe, right).expect("valid right");
    let empty = SortedSet::try_new(universe, empty).expect("valid empty");
    assert_laws(
        left.len(),
        right.len(),
        left.intersection_len(&left),
        left.intersection_len(&right),
        right.intersection_len(&left),
        left.intersection_len(&empty),
    );
}

fn assert_dense_laws(universe: u64, left: &[u32], right: &[u32], empty: &[u32]) {
    let left = DenseBitSet::try_new(universe, left).expect("valid left");
    let right = DenseBitSet::try_new(universe, right).expect("valid right");
    let empty = DenseBitSet::try_new(universe, empty).expect("valid empty");
    assert_laws(
        left.len(),
        right.len(),
        left.intersection_len(&left),
        left.intersection_len(&right),
        right.intersection_len(&left),
        left.intersection_len(&empty),
    );
}

fn assert_adaptive_laws(universe: u64, left: &[u32], right: &[u32], empty: &[u32]) {
    let left = AdaptiveBitmap::try_new(universe, left).expect("valid left");
    let right = AdaptiveBitmap::try_new(universe, right).expect("valid right");
    let empty = AdaptiveBitmap::try_new(universe, empty).expect("valid empty");
    assert_laws(
        left.len(),
        right.len(),
        left.intersection_len(&left),
        left.intersection_len(&right),
        right.intersection_len(&left),
        left.intersection_len(&empty),
    );
}

fn assert_laws(
    left_len: usize,
    right_len: usize,
    self_intersection: Result<usize, BitmapError>,
    left_right: Result<usize, BitmapError>,
    right_left: Result<usize, BitmapError>,
    empty_intersection: Result<usize, BitmapError>,
) {
    assert_eq!(self_intersection, Ok(left_len));
    assert_eq!(left_right, right_left);
    assert!(left_right.expect("matching universes") <= left_len.min(right_len));
    assert_eq!(empty_intersection, Ok(0));
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
        let upper = u64::try_from(upper).expect("bound fits u64");
        usize::try_from(self.next_u64() % upper).expect("remainder fits usize")
    }
}
