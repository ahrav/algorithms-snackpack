//! Exact prefix statistics over a fixed set of retained snapshot boundaries.

/// Exact intermediate state for a mean. The empty state has no mean.
pub type Stats = (i64, u64);

/// Measurement storage and reduction policy.
#[derive(Clone, Copy, Debug)]
pub enum Policy {
    /// Copy the raw input and sum the visible prefix on each read.
    Raw,
    /// Keep one summary per chunk of up to 32 measurements within each interval.
    Chunked,
    /// Keep one summary per retained interval.
    Compact,
}

enum Storage {
    Raw {
        values: Vec<i32>,
        ends: Vec<usize>,
    },
    Chunked {
        states: Vec<Stats>,
        ends: Vec<usize>,
    },
    Compact(Vec<Stats>),
}

/// A history whose snapshot IDs are ordinals in the declared boundary list.
pub struct History(Storage);

fn summarize(values: &[i32]) -> Stats {
    (
        values.iter().map(|&value| i64::from(value)).sum(),
        values.len() as u64,
    )
}

fn combine(states: &[Stats]) -> Stats {
    states
        .iter()
        .fold((0, 0), |(sum, count), &(s, n)| (sum + s, count + n))
}

impl History {
    /// Build a history from at most `u32::MAX` signed 32-bit measurements.
    ///
    /// The strictly increasing positive prefix lengths must end at the input
    /// length. Empty input requires an empty boundary list. Invalid input returns
    /// `None`. The size bound keeps every partial sum representable in `i64`.
    #[must_use]
    pub fn build(values: &[i32], ends: &[usize], policy: Policy) -> Option<Self> {
        if u32::try_from(values.len()).is_err()
            || ends.last().copied().unwrap_or(0) != values.len()
            || ends.first() == Some(&0)
            || ends.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return None;
        }
        let intervals = || {
            std::iter::once(0)
                .chain(ends.iter().copied())
                .zip(ends.iter().copied())
                .map(|(start, end)| &values[start..end])
        };
        let storage = match policy {
            Policy::Raw => Storage::Raw {
                values: values.to_vec(),
                ends: ends.to_vec(),
            },
            Policy::Chunked => {
                let mut states =
                    Vec::with_capacity(intervals().map(|xs| xs.len().div_ceil(32)).sum());
                let mut chunk_ends = Vec::with_capacity(ends.len());
                for interval in intervals() {
                    states.extend(interval.chunks(32).map(summarize));
                    chunk_ends.push(states.len());
                }
                Storage::Chunked {
                    states,
                    ends: chunk_ends,
                }
            }
            Policy::Compact => Storage::Compact(intervals().map(summarize).collect()),
        };
        Some(Self(storage))
    }

    /// Return exact statistics for a retained snapshot. Zero denotes the empty prefix.
    #[must_use]
    pub fn read(&self, snapshot: usize) -> Option<Stats> {
        let Some(index) = snapshot.checked_sub(1) else {
            return Some((0, 0));
        };
        match &self.0 {
            Storage::Raw { values, ends } => Some(summarize(&values[..*ends.get(index)?])),
            Storage::Chunked { states, ends } => Some(combine(&states[..*ends.get(index)?])),
            Storage::Compact(states) => Some(combine(states.get(..snapshot)?)),
        }
    }

    /// Return the number of raw or summary records visited by a snapshot read.
    #[must_use]
    pub fn visits(&self, snapshot: usize) -> Option<usize> {
        let Some(index) = snapshot.checked_sub(1) else {
            return Some(0);
        };
        match &self.0 {
            Storage::Raw { ends, .. } | Storage::Chunked { ends, .. } => ends.get(index).copied(),
            Storage::Compact(states) => (snapshot <= states.len()).then_some(snapshot),
        }
    }

    /// Return the number of nonempty vector buffers and their reserved bytes.
    #[must_use]
    pub fn reserved(&self) -> (usize, usize) {
        fn buffer<T>(values: &Vec<T>) -> (usize, usize) {
            let capacity = values.capacity();
            (usize::from(capacity != 0), capacity * size_of::<T>())
        }
        fn both(a: (usize, usize), b: (usize, usize)) -> (usize, usize) {
            (a.0 + b.0, a.1 + b.1)
        }
        match &self.0 {
            Storage::Raw { values, ends } => both(buffer(values), buffer(ends)),
            Storage::Chunked { states, ends } => both(buffer(states), buffer(ends)),
            Storage::Compact(states) => buffer(states),
        }
    }
}

/// Compute a reference answer directly from a raw prefix in wide integers.
///
/// Returns `None` if the prefix is outside the input or the exact sum cannot fit.
#[must_use]
pub fn reference(values: &[i32], end: usize) -> Option<Stats> {
    let prefix = values.get(..end)?;
    let total = prefix.iter().fold(0_i128, |acc, &x| acc + i128::from(x));
    Some((i64::try_from(total).ok()?, end as u64))
}

/// An affine operation on a wrapping 64-bit unsigned integer state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Affine {
    /// Coefficient multiplying the input.
    pub multiplier: u64,
    /// Increment added after multiplication.
    pub increment: u64,
}

impl Affine {
    /// Apply the operation with reduction modulo 2^64.
    #[must_use]
    pub fn apply(self, value: u64) -> u64 {
        self.multiplier
            .wrapping_mul(value)
            .wrapping_add(self.increment)
    }

    /// Combine two consecutive operations with the earlier one on the left.
    #[must_use]
    pub fn then(self, later: Self) -> Self {
        Self {
            multiplier: later.multiplier.wrapping_mul(self.multiplier),
            increment: later.apply(self.increment),
        }
    }
}
