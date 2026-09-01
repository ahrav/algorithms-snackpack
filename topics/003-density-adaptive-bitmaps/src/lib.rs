//! Immutable integer sets with sparse, dense, and density-adaptive representations.
//!
//! Every set stores distinct [`u32`] values in the half-open universe
//! `0..universe_exclusive`. Construction rejects invalid input before publishing
//! a set. Iteration always yields owned values in strict ascending order.

use std::collections::BTreeSet;
use std::fmt;

const MAX_UNIVERSE_EXCLUSIVE: u64 = 1_u64 << 32;
const ARRAY_MAX_CARDINALITY: usize = 4_096;
const CONTAINER_BITMAP_WORDS: usize = 1_024;
const BITS_PER_WORD: u64 = 64;

/// A construction or set-operation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BitmapError {
    /// The requested exclusive upper bound exceeds `2^32`.
    UniverseTooLarge {
        /// The rejected exclusive upper bound.
        universe_exclusive: u64,
    },
    /// One input value lies outside the requested universe.
    ValueOutOfUniverse {
        /// The zero-based position of the first invalid input value.
        input_index: usize,
        /// The rejected value.
        value: u32,
        /// The exclusive upper bound used for validation.
        universe_exclusive: u64,
    },
    /// An intersection was requested across different universes.
    UniverseMismatch {
        /// The left operand's exclusive upper bound.
        left_universe_exclusive: u64,
        /// The right operand's exclusive upper bound.
        right_universe_exclusive: u64,
    },
    /// The dense representation could not reserve its word payload.
    DenseAllocation {
        /// The logical number of bytes required for the dense words.
        required_bytes: u64,
    },
}

impl fmt::Display for BitmapError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UniverseTooLarge { universe_exclusive } => write!(
                formatter,
                "universe upper bound {universe_exclusive} exceeds 2^32"
            ),
            Self::ValueOutOfUniverse {
                input_index,
                value,
                universe_exclusive,
            } => write!(
                formatter,
                "input value {value} at index {input_index} is outside 0..{universe_exclusive}"
            ),
            Self::UniverseMismatch {
                left_universe_exclusive,
                right_universe_exclusive,
            } => write!(
                formatter,
                "cannot intersect universes 0..{left_universe_exclusive} and 0..{right_universe_exclusive}"
            ),
            Self::DenseAllocation { required_bytes } => write!(
                formatter,
                "could not reserve {required_bytes} bytes for dense bitmap words"
            ),
        }
    }
}

impl std::error::Error for BitmapError {}

/// An independent reference set backed by [`BTreeSet`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceSet {
    universe_exclusive: u64,
    values: BTreeSet<u32>,
}

impl ReferenceSet {
    /// Builds a reference set from values in `0..universe_exclusive`.
    ///
    /// Duplicate values are ignored.
    ///
    /// # Errors
    ///
    /// Returns [`BitmapError::UniverseTooLarge`] before inspecting `values` when
    /// `universe_exclusive` exceeds `2^32`. Otherwise returns the first
    /// [`BitmapError::ValueOutOfUniverse`] in input order.
    pub fn try_new(universe_exclusive: u64, values: &[u32]) -> Result<Self, BitmapError> {
        if universe_exclusive > MAX_UNIVERSE_EXCLUSIVE {
            return Err(BitmapError::UniverseTooLarge { universe_exclusive });
        }
        for (input_index, &value) in values.iter().enumerate() {
            if u64::from(value) >= universe_exclusive {
                return Err(BitmapError::ValueOutOfUniverse {
                    input_index,
                    value,
                    universe_exclusive,
                });
            }
        }

        Ok(Self {
            universe_exclusive,
            values: values.iter().copied().collect(),
        })
    }

    /// Returns the exclusive universe upper bound.
    #[must_use]
    pub const fn universe_exclusive(&self) -> u64 {
        self.universe_exclusive
    }

    /// Returns the number of distinct values.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns `true` when the set contains no values.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns whether `value` belongs to the set.
    ///
    /// Values outside the universe return `false`.
    #[must_use]
    pub fn contains(&self, value: u32) -> bool {
        u64::from(value) < self.universe_exclusive && self.values.contains(&value)
    }

    /// Iterates over owned values in strict ascending order.
    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.values.iter().copied()
    }

    /// Counts values present in both reference sets.
    ///
    /// # Errors
    ///
    /// Returns [`BitmapError::UniverseMismatch`] when the universes differ.
    pub fn intersection_len(&self, other: &Self) -> Result<usize, BitmapError> {
        if self.universe_exclusive != other.universe_exclusive {
            return Err(BitmapError::UniverseMismatch {
                left_universe_exclusive: self.universe_exclusive,
                right_universe_exclusive: other.universe_exclusive,
            });
        }
        Ok(self.values.intersection(&other.values).count())
    }

    /// Returns the logical value payload size in bytes.
    ///
    /// This count includes four bytes per distinct value. It excludes tree
    /// nodes, object headers, capacity slack, and allocator rounding.
    #[must_use]
    pub fn payload_bytes(&self) -> u64 {
        logical_payload_bytes(self.values.len(), 4)
    }
}

/// An immutable set stored as a sorted unique vector.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SortedSet {
    universe_exclusive: u64,
    values: Vec<u32>,
}

impl SortedSet {
    /// Builds a sorted set from values in `0..universe_exclusive`.
    ///
    /// Duplicate values are ignored.
    ///
    /// # Errors
    ///
    /// Returns [`BitmapError::UniverseTooLarge`] before inspecting `values` when
    /// `universe_exclusive` exceeds `2^32`. Otherwise returns the first
    /// [`BitmapError::ValueOutOfUniverse`] in input order.
    pub fn try_new(universe_exclusive: u64, values: &[u32]) -> Result<Self, BitmapError> {
        if universe_exclusive > MAX_UNIVERSE_EXCLUSIVE {
            return Err(BitmapError::UniverseTooLarge { universe_exclusive });
        }
        for (input_index, &value) in values.iter().enumerate() {
            if u64::from(value) >= universe_exclusive {
                return Err(BitmapError::ValueOutOfUniverse {
                    input_index,
                    value,
                    universe_exclusive,
                });
            }
        }

        let mut sorted = values.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        Ok(Self {
            universe_exclusive,
            values: sorted,
        })
    }

    /// Returns the exclusive universe upper bound.
    #[must_use]
    pub const fn universe_exclusive(&self) -> u64 {
        self.universe_exclusive
    }

    /// Returns the number of distinct values.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns `true` when the set contains no values.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns whether `value` belongs to the set.
    ///
    /// Values outside the universe return `false`.
    #[must_use]
    pub fn contains(&self, value: u32) -> bool {
        u64::from(value) < self.universe_exclusive && self.values.binary_search(&value).is_ok()
    }

    /// Iterates over owned values in strict ascending order.
    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.values.iter().copied()
    }

    /// Counts values present in both sorted sets.
    ///
    /// # Errors
    ///
    /// Returns [`BitmapError::UniverseMismatch`] when the universes differ.
    pub fn intersection_len(&self, other: &Self) -> Result<usize, BitmapError> {
        if self.universe_exclusive != other.universe_exclusive {
            return Err(BitmapError::UniverseMismatch {
                left_universe_exclusive: self.universe_exclusive,
                right_universe_exclusive: other.universe_exclusive,
            });
        }
        Ok(sorted_intersection_len(&self.values, &other.values))
    }

    /// Returns the logical value payload size in bytes.
    ///
    /// This count includes four bytes per distinct value. It excludes the
    /// vector header, capacity slack, and allocator rounding.
    #[must_use]
    pub fn payload_bytes(&self) -> u64 {
        logical_payload_bytes(self.values.len(), 4)
    }
}

/// An immutable dense bitset backed by packed 64-bit words.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DenseBitSet {
    universe_exclusive: u64,
    words: Vec<u64>,
    len: usize,
}

impl DenseBitSet {
    /// Builds a dense bitset from values in `0..universe_exclusive`.
    ///
    /// Duplicate values are ignored. The word vector has exactly
    /// `ceil(universe_exclusive / 64)` logical words.
    ///
    /// # Errors
    ///
    /// Returns [`BitmapError::UniverseTooLarge`] before inspecting `values` when
    /// `universe_exclusive` exceeds `2^32`. Otherwise returns the first
    /// [`BitmapError::ValueOutOfUniverse`] in input order. Returns
    /// [`BitmapError::DenseAllocation`] if the exact word payload cannot be
    /// reserved.
    pub fn try_new(universe_exclusive: u64, values: &[u32]) -> Result<Self, BitmapError> {
        if universe_exclusive > MAX_UNIVERSE_EXCLUSIVE {
            return Err(BitmapError::UniverseTooLarge { universe_exclusive });
        }
        for (input_index, &value) in values.iter().enumerate() {
            if u64::from(value) >= universe_exclusive {
                return Err(BitmapError::ValueOutOfUniverse {
                    input_index,
                    value,
                    universe_exclusive,
                });
            }
        }

        let word_count_u64 = universe_exclusive.div_ceil(BITS_PER_WORD);
        let required_bytes = word_count_u64 * 8;
        let word_count = usize::try_from(word_count_u64)
            .map_err(|_| BitmapError::DenseAllocation { required_bytes })?;
        let mut words = Vec::new();
        words
            .try_reserve_exact(word_count)
            .map_err(|_| BitmapError::DenseAllocation { required_bytes })?;
        words.resize(word_count, 0_u64);

        let mut len = 0_usize;
        for &value in values {
            let value_index = usize::try_from(value).unwrap_or(usize::MAX);
            let word_index = value_index / 64;
            let bit_index = value_index % 64;
            let mask = 1_u64 << bit_index;
            if words[word_index] & mask == 0 {
                words[word_index] |= mask;
                len += 1;
            }
        }

        Ok(Self {
            universe_exclusive,
            words,
            len,
        })
    }

    /// Returns the exclusive universe upper bound.
    #[must_use]
    pub const fn universe_exclusive(&self) -> u64 {
        self.universe_exclusive
    }

    /// Returns the number of distinct values.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` when the set contains no values.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns whether `value` belongs to the set.
    ///
    /// Values outside the universe return `false`.
    #[must_use]
    pub fn contains(&self, value: u32) -> bool {
        if u64::from(value) >= self.universe_exclusive {
            return false;
        }
        let value_index = usize::try_from(value).unwrap_or(usize::MAX);
        let word = self.words[value_index / 64];
        word & (1_u64 << (value_index % 64)) != 0
    }

    /// Iterates over owned values in strict ascending order.
    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        DenseValueIter::new(&self.words)
    }

    /// Counts values present in both dense bitsets.
    ///
    /// # Errors
    ///
    /// Returns [`BitmapError::UniverseMismatch`] when the universes differ.
    pub fn intersection_len(&self, other: &Self) -> Result<usize, BitmapError> {
        if self.universe_exclusive != other.universe_exclusive {
            return Err(BitmapError::UniverseMismatch {
                left_universe_exclusive: self.universe_exclusive,
                right_universe_exclusive: other.universe_exclusive,
            });
        }
        Ok(self
            .words
            .iter()
            .zip(&other.words)
            .map(|(&left, &right)| usize::try_from((left & right).count_ones()).unwrap_or_default())
            .sum())
    }

    /// Returns the logical packed-word payload size in bytes.
    ///
    /// This count excludes the vector header, capacity slack, and allocator
    /// rounding.
    #[must_use]
    pub fn payload_bytes(&self) -> u64 {
        logical_payload_bytes(self.words.len(), 8)
    }
}

/// The selected low-16-bit container representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContainerKind {
    /// A sorted unique array of 16-bit values.
    Array,
    /// A fixed bitmap of 1,024 64-bit words.
    Bitmap,
    /// Sorted maximal inclusive runs.
    Run,
}

/// A portable summary of one adaptive container.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContainerSummary {
    /// The high 16 bits shared by every value in the container.
    pub key: u16,
    /// The selected representation.
    pub kind: ContainerKind,
    /// The number of distinct low-16-bit values.
    pub cardinality: usize,
    /// The number of maximal nonadjacent runs in the represented values.
    pub run_count: usize,
    /// The logical local-container payload size in bytes.
    pub payload_bytes: u64,
}

/// An immutable bitmap whose high-16-bit directory selects a local container.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdaptiveBitmap {
    universe_exclusive: u64,
    containers: Vec<Container>,
    len: usize,
}

impl AdaptiveBitmap {
    /// Builds a density-adaptive bitmap from values in `0..universe_exclusive`.
    ///
    /// Duplicate values are ignored. Empty high-16-bit containers are omitted.
    /// A container with at most 4,096 values uses Array as its baseline; a
    /// larger container uses Bitmap. Run replaces that baseline only when
    /// `2 + 4 * run_count` is strictly smaller than the baseline payload.
    ///
    /// # Errors
    ///
    /// Returns [`BitmapError::UniverseTooLarge`] before inspecting `values` when
    /// `universe_exclusive` exceeds `2^32`. Otherwise returns the first
    /// [`BitmapError::ValueOutOfUniverse`] in input order.
    pub fn try_new(universe_exclusive: u64, values: &[u32]) -> Result<Self, BitmapError> {
        if universe_exclusive > MAX_UNIVERSE_EXCLUSIVE {
            return Err(BitmapError::UniverseTooLarge { universe_exclusive });
        }
        for (input_index, &value) in values.iter().enumerate() {
            if u64::from(value) >= universe_exclusive {
                return Err(BitmapError::ValueOutOfUniverse {
                    input_index,
                    value,
                    universe_exclusive,
                });
            }
        }

        let mut sorted = values.to_vec();
        sorted.sort_unstable();
        sorted.dedup();

        let mut containers = Vec::new();
        let mut group_start = 0_usize;
        while group_start < sorted.len() {
            let key = u16::try_from(sorted[group_start] >> 16).unwrap_or_default();
            let mut group_end = group_start + 1;
            while group_end < sorted.len() && sorted[group_end] >> 16 == u32::from(key) {
                group_end += 1;
            }

            let lows = sorted[group_start..group_end]
                .iter()
                .map(|value| u16::try_from(value & 0xffff).unwrap_or_default())
                .collect::<Vec<_>>();
            let cardinality = lows.len();
            let runs = build_runs(&lows);
            let run_count = runs.len();
            let run_payload_bytes = 2 + logical_payload_bytes(run_count, 4);
            let baseline_payload_bytes = if cardinality <= ARRAY_MAX_CARDINALITY {
                logical_payload_bytes(cardinality, 2)
            } else {
                8_192
            };

            let (kind, payload_bytes, data) = if run_payload_bytes < baseline_payload_bytes {
                (
                    ContainerKind::Run,
                    run_payload_bytes,
                    ContainerData::Run(runs),
                )
            } else if cardinality <= ARRAY_MAX_CARDINALITY {
                (
                    ContainerKind::Array,
                    baseline_payload_bytes,
                    ContainerData::Array(lows),
                )
            } else {
                (
                    ContainerKind::Bitmap,
                    baseline_payload_bytes,
                    ContainerData::Bitmap(build_container_bitmap(&lows)),
                )
            };

            containers.push(Container {
                key,
                kind,
                cardinality,
                run_count,
                payload_bytes,
                data,
            });
            group_start = group_end;
        }

        Ok(Self {
            universe_exclusive,
            containers,
            len: sorted.len(),
        })
    }

    /// Returns the exclusive universe upper bound.
    #[must_use]
    pub const fn universe_exclusive(&self) -> u64 {
        self.universe_exclusive
    }

    /// Returns the number of distinct values.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` when the set contains no values.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns whether `value` belongs to the set.
    ///
    /// Values outside the universe return `false`.
    #[must_use]
    pub fn contains(&self, value: u32) -> bool {
        if u64::from(value) >= self.universe_exclusive {
            return false;
        }
        let key = u16::try_from(value >> 16).unwrap_or_default();
        let low = u16::try_from(value & 0xffff).unwrap_or_default();
        self.containers
            .binary_search_by_key(&key, |container| container.key)
            .is_ok_and(|index| self.containers[index].data.contains(low))
    }

    /// Iterates over owned values in strict ascending order.
    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.containers.iter().flat_map(|container| {
            let high = u32::from(container.key) << 16;
            container.data.iter().map(move |low| high | u32::from(low))
        })
    }

    /// Counts values present in both adaptive bitmaps.
    ///
    /// # Errors
    ///
    /// Returns [`BitmapError::UniverseMismatch`] when the universes differ.
    pub fn intersection_len(&self, other: &Self) -> Result<usize, BitmapError> {
        if self.universe_exclusive != other.universe_exclusive {
            return Err(BitmapError::UniverseMismatch {
                left_universe_exclusive: self.universe_exclusive,
                right_universe_exclusive: other.universe_exclusive,
            });
        }

        let mut left_index = 0_usize;
        let mut right_index = 0_usize;
        let mut count = 0_usize;
        while left_index < self.containers.len() && right_index < other.containers.len() {
            let left = &self.containers[left_index];
            let right = &other.containers[right_index];
            match left.key.cmp(&right.key) {
                std::cmp::Ordering::Less => left_index += 1,
                std::cmp::Ordering::Greater => right_index += 1,
                std::cmp::Ordering::Equal => {
                    count += left.data.intersection_len(&right.data);
                    left_index += 1;
                    right_index += 1;
                }
            }
        }
        Ok(count)
    }

    /// Returns the sum of logical local-container payload sizes in bytes.
    ///
    /// This count excludes directory keys, vector and object headers, capacity
    /// slack, and allocator rounding.
    #[must_use]
    pub fn payload_bytes(&self) -> u64 {
        self.containers
            .iter()
            .map(|container| container.payload_bytes)
            .sum()
    }

    /// Iterates over portable container summaries in ascending key order.
    pub fn container_summaries(&self) -> impl Iterator<Item = ContainerSummary> + '_ {
        self.containers.iter().map(Container::summary)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Container {
    key: u16,
    kind: ContainerKind,
    cardinality: usize,
    run_count: usize,
    payload_bytes: u64,
    data: ContainerData,
}

impl Container {
    const fn summary(&self) -> ContainerSummary {
        ContainerSummary {
            key: self.key,
            kind: self.kind,
            cardinality: self.cardinality,
            run_count: self.run_count,
            payload_bytes: self.payload_bytes,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ContainerData {
    Array(Vec<u16>),
    Bitmap(Box<[u64; CONTAINER_BITMAP_WORDS]>),
    Run(Vec<Run>),
}

impl ContainerData {
    fn contains(&self, value: u16) -> bool {
        match self {
            Self::Array(values) => values.binary_search(&value).is_ok(),
            Self::Bitmap(words) => {
                let index = usize::from(value);
                words[index / 64] & (1_u64 << (index % 64)) != 0
            }
            Self::Run(runs) => {
                let index = runs.partition_point(|run| run.end < value);
                runs.get(index).is_some_and(|run| run.start <= value)
            }
        }
    }

    fn iter(&self) -> LowValueIter<'_> {
        match self {
            Self::Array(values) => LowValueIter::Array(values.iter().copied()),
            Self::Bitmap(words) => LowValueIter::Bitmap(BitmapLowIter::new(words.as_slice())),
            Self::Run(runs) => LowValueIter::Run(RunLowIter::new(runs)),
        }
    }

    fn intersection_len(&self, other: &Self) -> usize {
        match (self, other) {
            (Self::Array(left), Self::Array(right)) => sorted_intersection_len(left, right),
            (Self::Array(array), Self::Bitmap(bitmap))
            | (Self::Bitmap(bitmap), Self::Array(array)) => {
                array_bitmap_intersection_len(array, bitmap)
            }
            (Self::Array(array), Self::Run(runs)) | (Self::Run(runs), Self::Array(array)) => {
                array_run_intersection_len(array, runs)
            }
            (Self::Bitmap(left), Self::Bitmap(right)) => left
                .iter()
                .zip(right.iter())
                .map(|(&left_word, &right_word)| {
                    usize::try_from((left_word & right_word).count_ones())
                        .expect("word count fits usize")
                })
                .sum(),
            (Self::Bitmap(bitmap), Self::Run(runs)) | (Self::Run(runs), Self::Bitmap(bitmap)) => {
                runs.iter()
                    .map(|run| bitmap_range_cardinality(bitmap, run.start, run.end))
                    .sum()
            }
            (Self::Run(left), Self::Run(right)) => run_intersection_len(left, right),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Run {
    start: u16,
    end: u16,
}

fn sorted_intersection_len<T: Ord>(left: &[T], right: &[T]) -> usize {
    let mut left_index = 0_usize;
    let mut right_index = 0_usize;
    let mut count = 0_usize;
    while left_index < left.len() && right_index < right.len() {
        match left[left_index].cmp(&right[right_index]) {
            std::cmp::Ordering::Less => left_index += 1,
            std::cmp::Ordering::Greater => right_index += 1,
            std::cmp::Ordering::Equal => {
                count += 1;
                left_index += 1;
                right_index += 1;
            }
        }
    }
    count
}

fn logical_payload_bytes(element_count: usize, element_bytes: u64) -> u64 {
    u64::try_from(element_count)
        .unwrap_or(u64::MAX)
        .saturating_mul(element_bytes)
}

fn build_runs(values: &[u16]) -> Vec<Run> {
    let Some((&first, remaining)) = values.split_first() else {
        return Vec::new();
    };
    let mut runs = Vec::new();
    let mut start = first;
    let mut end = first;
    for &value in remaining {
        if u32::from(value) == u32::from(end) + 1 {
            end = value;
        } else {
            runs.push(Run { start, end });
            start = value;
            end = value;
        }
    }
    runs.push(Run { start, end });
    runs
}

fn build_container_bitmap(values: &[u16]) -> Box<[u64; CONTAINER_BITMAP_WORDS]> {
    let mut words = Box::new([0_u64; CONTAINER_BITMAP_WORDS]);
    for &value in values {
        let index = usize::from(value);
        words[index / 64] |= 1_u64 << (index % 64);
    }
    words
}

fn array_bitmap_intersection_len(values: &[u16], bitmap: &[u64; CONTAINER_BITMAP_WORDS]) -> usize {
    values
        .iter()
        .filter(|&&value| {
            let index = usize::from(value);
            bitmap[index / 64] & (1_u64 << (index % 64)) != 0
        })
        .count()
}

fn array_run_intersection_len(values: &[u16], runs: &[Run]) -> usize {
    let mut value_index = 0_usize;
    let mut run_index = 0_usize;
    let mut count = 0_usize;
    while value_index < values.len() && run_index < runs.len() {
        let value = values[value_index];
        let run = runs[run_index];
        if value < run.start {
            value_index += 1;
        } else if value > run.end {
            run_index += 1;
        } else {
            count += 1;
            value_index += 1;
        }
    }
    count
}

fn bitmap_range_cardinality(bitmap: &[u64; CONTAINER_BITMAP_WORDS], start: u16, end: u16) -> usize {
    let start_index = usize::from(start);
    let end_index = usize::from(end);
    let start_word = start_index / 64;
    let end_word = end_index / 64;
    let lower_mask = u64::MAX << (start_index % 64);
    let end_bit = end_index % 64;
    let upper_mask = if end_bit == 63 {
        u64::MAX
    } else {
        (1_u64 << (end_bit + 1)) - 1
    };

    if start_word == end_word {
        return usize::try_from((bitmap[start_word] & lower_mask & upper_mask).count_ones())
            .expect("word count fits usize");
    }

    let mut count = usize::try_from((bitmap[start_word] & lower_mask).count_ones())
        .expect("word count fits usize");
    for &word in &bitmap[start_word + 1..end_word] {
        count += usize::try_from(word.count_ones()).expect("word count fits usize");
    }
    count
        + usize::try_from((bitmap[end_word] & upper_mask).count_ones())
            .expect("word count fits usize")
}

fn run_intersection_len(left: &[Run], right: &[Run]) -> usize {
    let mut left_index = 0_usize;
    let mut right_index = 0_usize;
    let mut count = 0_usize;
    while left_index < left.len() && right_index < right.len() {
        let left_run = left[left_index];
        let right_run = right[right_index];
        let overlap_start = left_run.start.max(right_run.start);
        let overlap_end = left_run.end.min(right_run.end);
        if overlap_start <= overlap_end {
            count += usize::from(overlap_end - overlap_start) + 1;
        }
        if left_run.end <= right_run.end {
            left_index += 1;
        }
        if right_run.end <= left_run.end {
            right_index += 1;
        }
    }
    count
}

enum LowValueIter<'a> {
    Array(std::iter::Copied<std::slice::Iter<'a, u16>>),
    Bitmap(BitmapLowIter<'a>),
    Run(RunLowIter<'a>),
}

impl Iterator for LowValueIter<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Array(values) => values.next(),
            Self::Bitmap(values) => values.next(),
            Self::Run(values) => values.next(),
        }
    }
}

struct DenseValueIter<'a> {
    words: &'a [u64],
    next_word_index: usize,
    remaining: u64,
}

impl<'a> DenseValueIter<'a> {
    const fn new(words: &'a [u64]) -> Self {
        Self {
            words,
            next_word_index: 0,
            remaining: 0,
        }
    }
}

impl Iterator for DenseValueIter<'_> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.remaining != 0 {
                let bit = self.remaining.trailing_zeros();
                self.remaining &= self.remaining - 1;
                let word_index = self
                    .next_word_index
                    .checked_sub(1)
                    .expect("a loaded word has an index");
                let base = u64::try_from(word_index).expect("word index fits u64") * 64;
                return Some(
                    u32::try_from(base + u64::from(bit)).expect("set bits represent u32 values"),
                );
            }
            self.remaining = *self.words.get(self.next_word_index)?;
            self.next_word_index += 1;
        }
    }
}

struct BitmapLowIter<'a> {
    words: &'a [u64],
    next_word_index: usize,
    remaining: u64,
}

impl<'a> BitmapLowIter<'a> {
    const fn new(words: &'a [u64]) -> Self {
        Self {
            words,
            next_word_index: 0,
            remaining: 0,
        }
    }
}

impl Iterator for BitmapLowIter<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.remaining != 0 {
                let bit = self.remaining.trailing_zeros();
                self.remaining &= self.remaining - 1;
                let word_index = self
                    .next_word_index
                    .checked_sub(1)
                    .expect("a loaded word has an index");
                let base = u64::try_from(word_index).expect("word index fits u64") * 64;
                return Some(
                    u16::try_from(base + u64::from(bit)).expect("container bit index fits u16"),
                );
            }
            self.remaining = *self.words.get(self.next_word_index)?;
            self.next_word_index += 1;
        }
    }
}

struct RunLowIter<'a> {
    runs: &'a [Run],
    run_index: usize,
    next_value: Option<u32>,
}

impl<'a> RunLowIter<'a> {
    const fn new(runs: &'a [Run]) -> Self {
        Self {
            runs,
            run_index: 0,
            next_value: None,
        }
    }
}

impl Iterator for RunLowIter<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_value.is_none() {
            let run = self.runs.get(self.run_index)?;
            self.next_value = Some(u32::from(run.start));
        }

        let run = self.runs[self.run_index];
        let value = self.next_value.expect("run value was initialized");
        if value == u32::from(run.end) {
            self.run_index += 1;
            self.next_value = None;
        } else {
            self.next_value = Some(value + 1);
        }
        Some(u16::try_from(value).expect("run value fits u16"))
    }
}
