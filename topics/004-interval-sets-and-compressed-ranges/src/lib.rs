//! Exact sets of [`u32`] points represented as canonical half-open intervals.
//!
//! Input endpoints use [`u64`] so the exclusive endpoint `2^32` is representable.
//! All constructors reject an out-of-domain endpoint before checking whether the
//! same interval is reversed. Empty intervals are valid no-ops.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// The exclusive endpoint of the complete [`u32`] point domain.
pub const DOMAIN_END: u64 = 1_u64 << 32;

/// A half-open interval `[start, end)`.
///
/// [`Interval::new`] only records endpoints. Set constructors validate them.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Interval {
    start: u64,
    end: u64,
}

impl Interval {
    /// Records a half-open interval `[start, end)` for later validation.
    #[must_use]
    pub const fn new(start: u64, end: u64) -> Self {
        Self { start, end }
    }

    /// Returns the inclusive start endpoint.
    #[must_use]
    pub const fn start(self) -> u64 {
        self.start
    }

    /// Returns the exclusive end endpoint.
    #[must_use]
    pub const fn end(self) -> u64 {
        self.end
    }

    /// Returns the number of points when the interval is not reversed.
    ///
    /// A reversed, not-yet-validated descriptor returns zero.
    #[must_use]
    pub const fn len(self) -> u64 {
        self.end.saturating_sub(self.start)
    }

    /// Returns whether both endpoints are equal.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

/// An interval-construction error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntervalError {
    /// At least one endpoint exceeds [`DOMAIN_END`].
    OutOfDomain {
        /// The zero-based position of the first invalid input interval.
        index: usize,
        /// The first rejected endpoint within that interval.
        endpoint: u64,
    },
    /// The start endpoint exceeds the end endpoint.
    Reversed {
        /// The zero-based position of the first reversed input interval.
        index: usize,
        /// The rejected start endpoint.
        start: u64,
        /// The rejected end endpoint.
        end: u64,
    },
}

impl fmt::Display for IntervalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfDomain { index, endpoint } => write!(
                formatter,
                "interval {index} has endpoint {endpoint} above {DOMAIN_END}"
            ),
            Self::Reversed { index, start, end } => write!(
                formatter,
                "interval {index} [{start}, {end}) has start above end"
            ),
        }
    }
}

impl std::error::Error for IntervalError {}

/// The common semantic projection exposed by every representation.
pub trait IntervalSet {
    /// Returns whether `point` belongs to the set.
    #[must_use]
    fn contains(&self, point: u32) -> bool;

    /// Returns the number of distinct points in the set.
    #[must_use]
    fn cardinality(&self) -> u64;

    /// Returns sorted, nonempty, disjoint, and non-adjacent half-open intervals.
    #[must_use]
    fn canonical_intervals(&self) -> Vec<Interval>;
}

/// A deliberately eager and independent point-by-point reference model.
///
/// This model uses a [`BTreeSet`] of individual points. It does not call any
/// production canonicalization, validation, or set-operation helper.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PointOracle {
    points: BTreeSet<u32>,
}

impl PointOracle {
    /// Builds the reference model by inserting every covered point.
    ///
    /// Empty intervals insert nothing. Duplicate coverage has set semantics.
    /// This eager model is intentionally unsuitable for very long intervals.
    ///
    /// # Errors
    ///
    /// Returns [`IntervalError::OutOfDomain`] for the first interval with an
    /// endpoint above [`DOMAIN_END`]. That check precedes the reversed check for
    /// the same input. Otherwise returns [`IntervalError::Reversed`] for the
    /// first interval whose start exceeds its end.
    pub fn try_from_intervals(intervals: &[Interval]) -> Result<Self, IntervalError> {
        let mut points = BTreeSet::new();
        for (index, interval) in intervals.iter().copied().enumerate() {
            if interval.start > DOMAIN_END {
                return Err(IntervalError::OutOfDomain {
                    index,
                    endpoint: interval.start,
                });
            }
            if interval.end > DOMAIN_END {
                return Err(IntervalError::OutOfDomain {
                    index,
                    endpoint: interval.end,
                });
            }
            if interval.start > interval.end {
                return Err(IntervalError::Reversed {
                    index,
                    start: interval.start,
                    end: interval.end,
                });
            }
            for point in interval.start..interval.end {
                let point = u32::try_from(point).map_err(|_| IntervalError::OutOfDomain {
                    index,
                    endpoint: point,
                })?;
                points.insert(point);
            }
        }
        Ok(Self { points })
    }

    /// Returns the set union using the reference representation.
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        Self {
            points: self.points.union(&other.points).copied().collect(),
        }
    }

    /// Returns the set intersection using the reference representation.
    #[must_use]
    pub fn intersection(&self, other: &Self) -> Self {
        Self {
            points: self.points.intersection(&other.points).copied().collect(),
        }
    }

    /// Returns the points in `self` that are absent from `other`.
    #[must_use]
    pub fn difference(&self, other: &Self) -> Self {
        Self {
            points: self.points.difference(&other.points).copied().collect(),
        }
    }

    /// Iterates over individual points in ascending order.
    pub fn points(&self) -> impl Iterator<Item = u32> + '_ {
        self.points.iter().copied()
    }
}

impl IntervalSet for PointOracle {
    fn contains(&self, point: u32) -> bool {
        self.points.contains(&point)
    }

    fn cardinality(&self) -> u64 {
        u64::try_from(self.points.len()).expect("a u32-domain set cardinality fits u64")
    }

    fn canonical_intervals(&self) -> Vec<Interval> {
        let mut points = self.points.iter().copied();
        let Some(first) = points.next() else {
            return Vec::new();
        };

        let mut output = Vec::new();
        let mut start = u64::from(first);
        let mut end = start + 1;
        for point in points {
            let point = u64::from(point);
            if point == end {
                end += 1;
            } else {
                output.push(Interval::new(start, end));
                start = point;
                end = point + 1;
            }
        }
        output.push(Interval::new(start, end));
        output
    }
}

/// A wide-endpoint interval set built by sorting and merging a flat vector.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlatIntervalSet {
    intervals: Vec<Interval>,
    cardinality: u64,
}

impl FlatIntervalSet {
    /// Validates, sorts, and merges the input intervals.
    ///
    /// Empty intervals are ignored. Overlapping and adjacent intervals merge.
    ///
    /// # Errors
    ///
    /// Returns the first error according to [`IntervalError`].
    pub fn try_from_intervals(intervals: &[Interval]) -> Result<Self, IntervalError> {
        validate_intervals(intervals)?;
        let mut sorted = intervals
            .iter()
            .copied()
            .filter(|interval| !interval.is_empty())
            .collect::<Vec<_>>();
        sorted.sort_unstable_by_key(|interval| (interval.start, interval.end));

        let mut canonical = Vec::with_capacity(sorted.len());
        for interval in sorted {
            push_merged(&mut canonical, interval);
        }
        Ok(Self::from_canonical(canonical))
    }

    /// Borrows the canonical interval vector without allocating.
    #[must_use]
    pub fn intervals(&self) -> &[Interval] {
        &self.intervals
    }

    fn from_canonical(intervals: Vec<Interval>) -> Self {
        debug_assert!(is_canonical(&intervals));
        let cardinality = intervals.iter().map(|interval| interval.len()).sum();
        Self {
            intervals,
            cardinality,
        }
    }
}

impl IntervalSet for FlatIntervalSet {
    fn contains(&self, point: u32) -> bool {
        contains_intervals(&self.intervals, point)
    }

    fn cardinality(&self) -> u64 {
        self.cardinality
    }

    fn canonical_intervals(&self) -> Vec<Interval> {
        self.intervals.clone()
    }
}

/// One packed run encoded as a [`u32`] start and `length - 1`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackedRun {
    start: u32,
    length_minus_one: u32,
}

impl PackedRun {
    /// Returns the inclusive start point.
    #[must_use]
    pub const fn start(self) -> u32 {
        self.start
    }

    /// Returns the encoded run length minus one.
    #[must_use]
    pub const fn length_minus_one(self) -> u32 {
        self.length_minus_one
    }

    /// Decodes the run with widened arithmetic.
    ///
    /// Widening before addition preserves an exclusive endpoint of `2^32`.
    #[must_use]
    pub fn decode(self) -> Interval {
        let start = u64::from(self.start);
        let end = start + u64::from(self.length_minus_one) + 1;
        Interval::new(start, end)
    }

    fn encode(interval: Interval) -> Self {
        debug_assert!(!interval.is_empty());
        let start =
            u32::try_from(interval.start).expect("a nonempty validated interval starts below 2^32");
        let length_minus_one = u32::try_from(interval.len() - 1)
            .expect("a u32-domain interval length minus one fits u32");
        Self {
            start,
            length_minus_one,
        }
    }
}

/// A canonical interval set stored in two [`u32`] scalars per run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackedIntervalSet {
    runs: Vec<PackedRun>,
    cardinality: u64,
}

impl PackedIntervalSet {
    /// Builds a packed set after wide-endpoint validation and canonicalization.
    ///
    /// # Errors
    ///
    /// Returns the first error according to [`IntervalError`].
    pub fn try_from_intervals(intervals: &[Interval]) -> Result<Self, IntervalError> {
        let flat = FlatIntervalSet::try_from_intervals(intervals)?;
        let runs = flat
            .intervals
            .iter()
            .copied()
            .map(PackedRun::encode)
            .collect();
        Ok(Self {
            runs,
            cardinality: flat.cardinality,
        })
    }

    /// Borrows the packed run payload.
    #[must_use]
    pub fn packed_runs(&self) -> &[PackedRun] {
        &self.runs
    }
}

impl IntervalSet for PackedIntervalSet {
    fn contains(&self, point: u32) -> bool {
        let index = self.runs.partition_point(|run| run.start <= point);
        index != 0 && u64::from(point) < self.runs[index - 1].decode().end
    }

    fn cardinality(&self) -> u64 {
        self.cardinality
    }

    fn canonical_intervals(&self) -> Vec<Interval> {
        self.runs.iter().copied().map(PackedRun::decode).collect()
    }
}

/// A canonical interval set built by grouping boundary events by endpoint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundaryEventSet {
    intervals: Vec<Interval>,
    cardinality: u64,
}

impl BoundaryEventSet {
    /// Builds the union by sweeping grouped `+1` start and `-1` end events.
    ///
    /// Grouping events at one endpoint before changing coverage is what merges
    /// adjacent intervals without a special adjacency branch.
    ///
    /// # Errors
    ///
    /// Returns the first error according to [`IntervalError`].
    pub fn try_from_intervals(intervals: &[Interval]) -> Result<Self, IntervalError> {
        validate_intervals(intervals)?;
        let mut events = Vec::new();
        for interval in intervals.iter().copied().filter(|item| !item.is_empty()) {
            events.push((interval.start, 1_i128));
            events.push((interval.end, -1_i128));
        }
        events.sort_unstable_by_key(|event| event.0);

        let mut canonical = Vec::new();
        let mut depth = 0_i128;
        let mut active_start = None;
        let mut index = 0;
        while index < events.len() {
            let coordinate = events[index].0;
            let mut delta = 0_i128;
            while index < events.len() && events[index].0 == coordinate {
                delta += events[index].1;
                index += 1;
            }
            let next_depth = depth + delta;
            debug_assert!(next_depth >= 0);
            if depth == 0 && next_depth > 0 {
                active_start = Some(coordinate);
            } else if depth > 0 && next_depth == 0 {
                debug_assert!(active_start.is_some());
                if let Some(start) = active_start.take() {
                    canonical.push(Interval::new(start, coordinate));
                }
            }
            depth = next_depth;
        }
        debug_assert_eq!(depth, 0);
        debug_assert!(active_start.is_none());

        let cardinality = canonical.iter().map(|interval| interval.len()).sum();
        debug_assert!(is_canonical(&canonical));
        Ok(Self {
            intervals: canonical,
            cardinality,
        })
    }

    /// Borrows the canonical interval vector without allocating.
    #[must_use]
    pub fn intervals(&self) -> &[Interval] {
        &self.intervals
    }
}

impl IntervalSet for BoundaryEventSet {
    fn contains(&self, point: u32) -> bool {
        contains_intervals(&self.intervals, point)
    }

    fn cardinality(&self) -> u64 {
        self.cardinality
    }

    fn canonical_intervals(&self) -> Vec<Interval> {
        self.intervals.clone()
    }
}

/// A canonical interval set built by incrementally coalescing a [`BTreeMap`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BTreeIntervalSet {
    intervals: BTreeMap<u64, u64>,
    cardinality: u64,
}

impl BTreeIntervalSet {
    /// Validates all inputs, then inserts and coalesces each nonempty interval.
    ///
    /// # Errors
    ///
    /// Returns the first error according to [`IntervalError`]. Validation of the
    /// complete input precedes any tree mutation.
    pub fn try_from_intervals(intervals: &[Interval]) -> Result<Self, IntervalError> {
        validate_intervals(intervals)?;
        let mut tree = BTreeMap::new();
        for interval in intervals.iter().copied().filter(|item| !item.is_empty()) {
            insert_coalesced(&mut tree, interval);
        }
        let cardinality = tree.iter().map(|(&start, &end)| end - start).sum();
        debug_assert!(is_canonical_map(&tree));
        Ok(Self {
            intervals: tree,
            cardinality,
        })
    }

    /// Returns the number of canonical runs stored in the tree.
    #[must_use]
    pub fn run_count(&self) -> usize {
        self.intervals.len()
    }
}

impl IntervalSet for BTreeIntervalSet {
    fn contains(&self, point: u32) -> bool {
        let point = u64::from(point);
        self.intervals
            .range(..=point)
            .next_back()
            .is_some_and(|(_, &end)| point < end)
    }

    fn cardinality(&self) -> u64 {
        self.cardinality
    }

    fn canonical_intervals(&self) -> Vec<Interval> {
        self.intervals
            .iter()
            .map(|(&start, &end)| Interval::new(start, end))
            .collect()
    }
}

/// Returns the union of any two interval-set representations.
#[must_use]
pub fn union<L: IntervalSet + ?Sized, R: IntervalSet + ?Sized>(
    left: &L,
    right: &R,
) -> FlatIntervalSet {
    let left = left.canonical_intervals();
    let right = right.canonical_intervals();
    let mut output = Vec::with_capacity(left.len() + right.len());
    let mut left_index = 0;
    let mut right_index = 0;

    while left_index < left.len() || right_index < right.len() {
        let take_left = right_index == right.len()
            || (left_index < left.len()
                && (left[left_index].start, left[left_index].end)
                    <= (right[right_index].start, right[right_index].end));
        let interval = if take_left {
            let interval = left[left_index];
            left_index += 1;
            interval
        } else {
            let interval = right[right_index];
            right_index += 1;
            interval
        };
        push_merged(&mut output, interval);
    }
    FlatIntervalSet::from_canonical(output)
}

/// Returns the intersection of any two interval-set representations.
#[must_use]
pub fn intersection<L: IntervalSet + ?Sized, R: IntervalSet + ?Sized>(
    left: &L,
    right: &R,
) -> FlatIntervalSet {
    let left = left.canonical_intervals();
    let right = right.canonical_intervals();
    let mut output = Vec::new();
    let mut left_index = 0;
    let mut right_index = 0;

    while left_index < left.len() && right_index < right.len() {
        let start = left[left_index].start.max(right[right_index].start);
        let end = left[left_index].end.min(right[right_index].end);
        if start < end {
            output.push(Interval::new(start, end));
        }
        if left[left_index].end < right[right_index].end {
            left_index += 1;
        } else {
            right_index += 1;
        }
    }
    FlatIntervalSet::from_canonical(output)
}

/// Returns the points in `left` that are absent from `right`.
#[must_use]
pub fn difference<L: IntervalSet + ?Sized, R: IntervalSet + ?Sized>(
    left: &L,
    right: &R,
) -> FlatIntervalSet {
    let left = left.canonical_intervals();
    let right = right.canonical_intervals();
    let mut output = Vec::new();
    let mut right_index = 0;

    for left_interval in left {
        while right_index < right.len() && right[right_index].end <= left_interval.start {
            right_index += 1;
        }
        let mut cursor = left_interval.start;
        let mut index = right_index;
        while index < right.len() && right[index].start < left_interval.end {
            if right[index].start > cursor {
                output.push(Interval::new(
                    cursor,
                    right[index].start.min(left_interval.end),
                ));
            }
            cursor = cursor.max(right[index].end);
            if cursor >= left_interval.end {
                break;
            }
            index += 1;
        }
        if cursor < left_interval.end {
            output.push(Interval::new(cursor, left_interval.end));
        }
    }
    FlatIntervalSet::from_canonical(output)
}

fn validate_intervals(intervals: &[Interval]) -> Result<(), IntervalError> {
    for (index, interval) in intervals.iter().copied().enumerate() {
        if interval.start > DOMAIN_END {
            return Err(IntervalError::OutOfDomain {
                index,
                endpoint: interval.start,
            });
        }
        if interval.end > DOMAIN_END {
            return Err(IntervalError::OutOfDomain {
                index,
                endpoint: interval.end,
            });
        }
        if interval.start > interval.end {
            return Err(IntervalError::Reversed {
                index,
                start: interval.start,
                end: interval.end,
            });
        }
    }
    Ok(())
}

fn push_merged(output: &mut Vec<Interval>, interval: Interval) {
    if let Some(previous) = output.last_mut()
        && interval.start <= previous.end
    {
        previous.end = previous.end.max(interval.end);
        return;
    }
    output.push(interval);
}

fn contains_intervals(intervals: &[Interval], point: u32) -> bool {
    let point = u64::from(point);
    let index = intervals.partition_point(|interval| interval.start <= point);
    index != 0 && point < intervals[index - 1].end
}

fn insert_coalesced(tree: &mut BTreeMap<u64, u64>, interval: Interval) {
    let mut start = interval.start;
    let mut end = interval.end;

    if let Some((&previous_start, &previous_end)) = tree.range(..=start).next_back()
        && previous_end >= start
    {
        start = previous_start;
        end = end.max(previous_end);
        tree.remove(&previous_start);
    }

    loop {
        let next = tree
            .range(start..=end)
            .next()
            .map(|(&key, &value)| (key, value));
        let Some((next_start, next_end)) = next else {
            break;
        };
        end = end.max(next_end);
        tree.remove(&next_start);
    }
    tree.insert(start, end);
}

fn is_canonical(intervals: &[Interval]) -> bool {
    intervals
        .iter()
        .all(|interval| interval.start < interval.end && interval.end <= DOMAIN_END)
        && intervals.windows(2).all(|pair| pair[0].end < pair[1].start)
}

fn is_canonical_map(intervals: &BTreeMap<u64, u64>) -> bool {
    let as_intervals = intervals
        .iter()
        .map(|(&start, &end)| Interval::new(start, end))
        .collect::<Vec<_>>();
    is_canonical(&as_intervals)
}
