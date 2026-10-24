//! Safe growth-policy experiments over fixed-width records.
//! Capacity is storage, not initialized length or an admission limit.
use std::collections::TryReserveError;

/// How an exhausted buffer requests more storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Growth {
    /// Standard library amortized reservation; factor is unspecified.
    Standard,
    /// Request just one additional record each time capacity is exhausted.
    Exact,
    /// Request current capacity plus its ceiling half.
    OneHalf,
    /// Request twice current capacity.
    Double,
}

/// Policy applied after a successful pop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shrink {
    /// Retain capacity for future reuse.
    Retain,
    /// Request tight capacity after every pop.
    Tight,
    /// At quarter occupancy, request half capacity, with a floor of four.
    Quarter,
}

/// Request-level observations, not allocator internals or resident memory.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    /// Explicit grow/reserve requests, including an initial reservation.
    pub grows: usize,
    /// Explicit shrink requests.
    pub shrinks: usize,
    /// Observed backing-address changes after nonempty growth requests.
    pub address_changes: usize,
    /// Sum of initialized lengths at growth requests; hypothetical copy work.
    pub live_at_growth: u128,
    /// Largest reported capacity, in records.
    pub peak_capacity: usize,
}

/// Checked capacity target. Only `OneHalf` and `Double` apply the four-record
/// startup floor. Returns `None` when the policy arithmetic overflows.
#[must_use]
pub fn growth_target(capacity: usize, growth: Growth) -> Option<usize> {
    match growth {
        Growth::Standard | Growth::Exact => capacity.checked_add(1),
        Growth::OneHalf => capacity
            .checked_add(capacity / 2 + capacity % 2)
            .map(|n| n.max(4)),
        Growth::Double => capacity.checked_mul(2).map(|n| n.max(4)),
    }
}

/// A sequence of nonzero-width, inline `u64` records. No unsafe code.
///
/// ```
/// use algorithms_growth::{Array, Growth, Shrink};
/// let mut a = Array::<1>::new(Growth::Double, Shrink::Retain, 0).unwrap();
/// a.try_push([42]).unwrap();
/// assert_eq!(a.as_slice(), &[[42]]);
/// assert_eq!(a.pop(), Some([42]));
/// ```
#[derive(Debug)]
pub struct Array<const W: usize> {
    data: Vec<[u64; W]>,
    growth: Growth,
    shrink: Shrink,
    stats: Stats,
}

impl<const W: usize> Array<W> {
    /// Creates an empty sequence with a caller-provided capacity hint.
    ///
    /// # Errors
    /// Returns allocation or capacity errors without constructing a sequence.
    /// # Panics
    /// Panics when `W == 0`; zero-sized records are outside this experiment.
    pub fn new(growth: Growth, shrink: Shrink, hint: usize) -> Result<Self, TryReserveError> {
        assert!(W > 0, "nonzero record width required");
        let mut data = Vec::new();
        data.try_reserve_exact(hint)?;
        let stats = Stats {
            grows: usize::from(hint > 0),
            peak_capacity: data.capacity(),
            ..Stats::default()
        };
        Ok(Self {
            data,
            growth,
            shrink,
            stats,
        })
    }

    /// Appends one initialized record.
    /// # Errors
    /// Returns allocation/capacity errors with the old contents intact.
    pub fn try_push(&mut self, value: [u64; W]) -> Result<(), TryReserveError> {
        if self.data.len() == self.data.capacity() {
            let old_len = self.data.len();
            let old_ptr = self.data.as_ptr();
            if self.growth == Growth::Standard {
                self.data.try_reserve(1)?;
            } else {
                let target = growth_target(self.data.capacity(), self.growth).unwrap_or(usize::MAX);
                // reserve_exact takes ADDITIONAL relative to length, not capacity.
                self.data.try_reserve_exact(target - old_len)?;
            }
            self.stats.grows += 1;
            self.stats.live_at_growth += old_len as u128;
            self.stats.address_changes += usize::from(old_len > 0 && old_ptr != self.data.as_ptr());
            self.stats.peak_capacity = self.stats.peak_capacity.max(self.data.capacity());
        }
        self.data.push(value);
        Ok(())
    }

    /// Removes the last record and applies the chosen shrink request policy.
    /// Shrinking does not promise an exact resulting capacity or physical release.
    pub fn pop(&mut self) -> Option<[u64; W]> {
        let value = self.data.pop()?;
        let cap = self.data.capacity();
        match self.shrink {
            Shrink::Tight => {
                self.stats.shrinks += 1;
                self.data.shrink_to_fit();
            }
            Shrink::Quarter if cap > 4 && self.data.len() <= cap / 4 => {
                self.stats.shrinks += 1;
                self.data.shrink_to((cap / 2).max(4));
            }
            Shrink::Retain | Shrink::Quarter => {}
        }
        Some(value)
    }

    /// Drops logical contents but retains the allocation for reuse.
    pub fn clear(&mut self) {
        self.data.clear();
    }
    /// Returns only initialized records.
    #[must_use]
    pub fn as_slice(&self) -> &[[u64; W]] {
        &self.data
    }
    /// Returns reported storage capacity in records.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }
    /// Returns request-level counters.
    #[must_use]
    pub fn stats(&self) -> Stats {
        self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn exhaustive_histories_against_ordered_map() {
        for growth in [
            Growth::Standard,
            Growth::Exact,
            Growth::OneHalf,
            Growth::Double,
        ] {
            for shrink in [Shrink::Retain, Shrink::Tight, Shrink::Quarter] {
                for code in 0..6561_u32 {
                    // Every length-eight push/pop/clear history.
                    let mut a = Array::<1>::new(growth, shrink, 0).unwrap();
                    let mut model = BTreeMap::new();
                    let mut digits = code;
                    for step in 0..8_u64 {
                        match digits % 3 {
                            0 => {
                                a.try_push([step]).unwrap();
                                model.insert(model.len(), [step]);
                            }
                            1 => {
                                let expected = model.pop_last().map(|(_, v)| v);
                                assert_eq!(a.pop(), expected);
                            }
                            _ => {
                                a.clear();
                                model.clear();
                            }
                        }
                        digits /= 3;
                        assert_eq!(a.as_slice(), model.values().copied().collect::<Vec<_>>());
                        assert!(a.capacity() >= a.as_slice().len());
                    }
                }
            }
        }
    }

    #[test]
    fn target_rounding_overflow_and_reservation_failure() {
        assert_eq!(growth_target(5, Growth::OneHalf), Some(8));
        assert_eq!(growth_target(0, Growth::Double), Some(4));
        assert_eq!(growth_target(usize::MAX, Growth::Double), None);
        assert_eq!(growth_target(usize::MAX, Growth::OneHalf), None);
        assert_eq!(growth_target(usize::MAX, Growth::Exact), None);
        assert!(Array::<1>::new(Growth::Double, Shrink::Retain, usize::MAX).is_err());
    }

    #[test]
    fn shrink_hysteresis_reuse_and_wide_records() {
        let mut a = Array::<32>::new(Growth::Double, Shrink::Quarter, 16).unwrap();
        for i in 0..16 {
            a.try_push([i; 32]).unwrap();
        }
        for i in (4..16).rev() {
            assert_eq!(a.pop(), Some([i; 32]));
        }
        assert_eq!(a.stats().shrinks, 1);
        assert!(a.capacity() >= 8);
        let cap = a.capacity();
        a.clear();
        assert_eq!(a.capacity(), cap);
        a.try_push([99; 32]).unwrap();
        assert_eq!(a.as_slice(), &[[99; 32]]);
    }
}
