//! Mutable half-open prefix sums. Aggregates use arithmetic modulo 2^64.
#![forbid(unsafe_code)]

/// A fixed-length array index for point additions and prefix/range sums.
#[derive(Clone, Debug)]
pub struct Fenwick {
    tree: Vec<u64>,
}
impl Fenwick {
    /// Build in linear time; slot zero is an unused sentinel.
    ///
    /// # Panics
    /// Panics if the sentinel length cannot be represented.
    #[must_use]
    pub fn new(values: &[u64]) -> Self {
        let mut tree = vec![0_u64; values.len().checked_add(1).expect("length overflow")];
        for (p, &value) in values.iter().enumerate() {
            let i = p + 1;
            tree[i] = tree[i].wrapping_add(value);
            if let Some(j) = i
                .checked_add(i & i.wrapping_neg())
                .filter(|&j| j < tree.len())
            {
                tree[j] = tree[j].wrapping_add(tree[i]);
            }
        }
        Self { tree }
    }
    /// Number of logical elements.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tree.len() - 1
    }
    /// Whether there are no elements.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Add modulo 2^64.
    ///
    /// # Panics
    /// Panics for an index outside the array before mutation.
    pub fn add(&mut self, p: usize, delta: u64) {
        assert!(p < self.len());
        let mut i = p + 1;
        while i < self.tree.len() {
            self.tree[i] = self.tree[i].wrapping_add(delta);
            let Some(j) = i.checked_add(i & i.wrapping_neg()) else {
                break;
            };
            i = j;
        }
    }
    /// Sum `[0, end)` modulo 2^64. `end == len` is valid.
    ///
    /// # Panics
    /// Panics if end exceeds the logical length.
    #[must_use]
    pub fn prefix(&self, mut end: usize) -> u64 {
        assert!(end <= self.len());
        let mut sum = 0_u64;
        while end != 0 {
            sum = sum.wrapping_add(self.tree[end]);
            end &= end - 1;
        }
        sum
    }
    /// Sum `[start, end)`.
    ///
    /// # Panics
    /// Panics for reversed or out-of-bounds ranges.
    #[must_use]
    pub fn range(&self, start: usize, end: usize) -> u64 {
        assert!(start <= end && end <= self.len());
        self.prefix(end).wrapping_sub(self.prefix(start))
    }
    // Precondition: all leaves nonnegative and total fits u64, enforced by Counts.
    fn select(&self, mut rank: u64) -> usize {
        let mut index = 0;
        let mut step = if self.is_empty() {
            0
        } else {
            1 << (usize::BITS - 1 - self.len().leading_zeros())
        };
        while step != 0 {
            let next = index + step;
            if next <= self.len() && self.tree[next] <= rank {
                rank -= self.tree[next];
                index = next;
            }
            step >>= 1;
        }
        index
    }
}

/// Exact nonnegative frequencies with a checked total. No modular selection.
#[derive(Clone, Debug)]
pub struct Counts {
    values: Vec<u64>,
    sums: Fenwick,
    total: u64,
}
impl Counts {
    /// Reject a total exceeding u64 before constructing the index.
    #[must_use]
    pub fn new(values: &[u64]) -> Option<Self> {
        let total = values.iter().try_fold(0_u64, |a, &b| a.checked_add(b))?;
        Some(Self {
            values: values.to_vec(),
            sums: Fenwick::new(values),
            total,
        })
    }
    /// Replace a count. Invalid indices or overflow leave the object unchanged.
    pub fn set(&mut self, p: usize, value: u64) -> bool {
        let Some(&old) = self.values.get(p) else {
            return false;
        };
        let Some(total) = (self.total - old).checked_add(value) else {
            return false;
        };
        self.sums.add(p, value.wrapping_sub(old));
        self.values[p] = value;
        self.total = total;
        true
    }
    /// Index containing the zero-based count rank, or None for rank >= total.
    #[must_use]
    pub fn select(&self, rank: u64) -> Option<usize> {
        (rank < self.total).then(|| self.sums.select(rank))
    }
}

/// Range additions and range sums, modulo 2^64, using two difference trees.
#[derive(Clone, Debug)]
pub struct RangeAdd {
    d: Fenwick,
    weighted: Fenwick,
}
impl RangeAdd {
    /// Initially zero, with a fixed logical length.
    #[must_use]
    pub fn new(n: usize) -> Self {
        Self {
            d: Fenwick::new(&vec![0; n]),
            weighted: Fenwick::new(&vec![0; n]),
        }
    }
    fn boundary(&mut self, p: usize, delta: u64) {
        if p < self.d.len() {
            self.d.add(p, delta);
            self.weighted.add(
                p,
                delta.wrapping_mul(u64::try_from(p).expect("index fits u64")),
            );
        }
    }
    /// Add delta to `[start, end)`; validates both endpoints before any mutation.
    ///
    /// # Panics
    /// Panics for reversed or out-of-bounds ranges.
    pub fn add(&mut self, start: usize, end: usize, delta: u64) {
        assert!(start <= end && end <= self.d.len());
        if start == end {
            return;
        }
        self.boundary(start, delta);
        self.boundary(end, delta.wrapping_neg());
    }
    /// Sum `[0, end)` modulo 2^64.
    ///
    /// # Panics
    /// Panics for an endpoint beyond the logical length or u64 index domain.
    #[must_use]
    pub fn prefix(&self, end: usize) -> u64 {
        u64::try_from(end)
            .expect("index fits u64")
            .wrapping_mul(self.d.prefix(end))
            .wrapping_sub(self.weighted.prefix(end))
    }
    /// Sum `[start, end)` modulo 2^64.
    ///
    /// # Panics
    /// Panics for reversed or out-of-bounds ranges.
    #[must_use]
    pub fn range(&self, start: usize, end: usize) -> u64 {
        assert!(start <= end && end <= self.d.len());
        self.prefix(end).wrapping_sub(self.prefix(start))
    }
}

/// Common benchmark contract: fixed length, wrapping point additions and ranges.
pub trait Index: Sized {
    /// Build from logical values.
    fn build(values: &[u64]) -> Self;
    /// Point addition; caller supplies a valid position.
    fn add(&mut self, p: usize, delta: u64);
    /// Half-open range sum; caller supplies valid endpoints.
    fn range(&self, start: usize, end: usize) -> u64;
}
impl Index for Fenwick {
    fn build(v: &[u64]) -> Self {
        Self::new(v)
    }
    fn add(&mut self, p: usize, d: u64) {
        Self::add(self, p, d);
    }
    fn range(&self, l: usize, r: usize) -> u64 {
        Self::range(self, l, r)
    }
}
/// Independent eager-array reference; queries scan original values.
#[derive(Clone)]
pub struct Scan(Vec<u64>);
impl Index for Scan {
    fn build(v: &[u64]) -> Self {
        Self(v.to_vec())
    }
    fn add(&mut self, p: usize, d: u64) {
        self.0[p] = self.0[p].wrapping_add(d);
    }
    fn range(&self, l: usize, r: usize) -> u64 {
        self.0[l..r].iter().fold(0_u64, |a, &b| a.wrapping_add(b))
    }
}
/// Materialized prefix array; a point update repairs every following prefix.
pub struct Prefix(Vec<u64>);
impl Index for Prefix {
    fn build(v: &[u64]) -> Self {
        let mut p = vec![0_u64];
        for &x in v {
            p.push(p.last().unwrap().wrapping_add(x));
        }
        Self(p)
    }
    fn add(&mut self, p: usize, d: u64) {
        for x in &mut self.0[p + 1..] {
            *x = x.wrapping_add(d);
        }
    }
    fn range(&self, l: usize, r: usize) -> u64 {
        self.0[r].wrapping_sub(self.0[l])
    }
}
/// Iterative segment tree padded to a power of two; same sum contract.
pub struct Segment {
    base: usize,
    tree: Vec<u64>,
}
impl Index for Segment {
    fn build(v: &[u64]) -> Self {
        let base = v.len().max(1).next_power_of_two();
        let mut tree = vec![0_u64; 2 * base];
        tree[base..base + v.len()].copy_from_slice(v);
        for i in (1..base).rev() {
            tree[i] = tree[2 * i].wrapping_add(tree[2 * i + 1]);
        }
        Self { base, tree }
    }
    fn add(&mut self, p: usize, d: u64) {
        let mut i = self.base + p;
        self.tree[i] = self.tree[i].wrapping_add(d);
        while i > 1 {
            i /= 2;
            self.tree[i] = self.tree[2 * i].wrapping_add(self.tree[2 * i + 1]);
        }
    }
    fn range(&self, l: usize, r: usize) -> u64 {
        let (mut l, mut r, mut sum) = (l + self.base, r + self.base, 0_u64);
        while l < r {
            if l & 1 != 0 {
                sum = sum.wrapping_add(self.tree[l]);
                l += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                sum = sum.wrapping_add(self.tree[r]);
            }
            l /= 2;
            r /= 2;
        }
        sum
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exhaustive_prefix_ranges_and_updates() {
        for n in 0_u32..=7 {
            for code in 0..3_usize.pow(n) {
                let mut x = code;
                let mut a: Vec<u64> = (0..n)
                    .map(|_| {
                        let y = u64::try_from(x % 3).unwrap();
                        x /= 3;
                        y
                    })
                    .collect();
                let mut f = Fenwick::new(&a);
                for pass in 0..=a.len() {
                    for l in 0..=a.len() {
                        for r in l..=a.len() {
                            assert_eq!(
                                f.range(l, r),
                                a[l..r].iter().fold(0_u64, |x, &y| x.wrapping_add(y))
                            );
                        }
                    }
                    if pass < a.len() {
                        a[pass] = a[pass].wrapping_sub(3);
                        f.add(pass, 3_u64.wrapping_neg());
                    }
                }
            }
        }
    }
    #[test]
    fn selection_zeros_and_atomic_rejections() {
        for n in 0_u32..=6 {
            for code in 0..3_usize.pow(n) {
                let mut x = code;
                let a: Vec<u64> = (0..n)
                    .map(|_| {
                        let y = u64::try_from(x % 3).unwrap();
                        x /= 3;
                        y
                    })
                    .collect();
                let c = Counts::new(&a).unwrap();
                for rank in 0..=a.iter().sum() {
                    let mut total = 0;
                    let oracle = a.iter().position(|&v| {
                        total += v;
                        total > rank
                    });
                    assert_eq!(c.select(rank), oracle);
                }
            }
        }
        let mut c = Counts::new(&[u64::MAX, 0]).unwrap();
        assert!(!c.set(1, 1));
        assert!(!c.set(2, 0));
        assert_eq!(c.select(u64::MAX - 1), Some(0));
        assert!(c.set(0, 0));
        assert_eq!(c.select(0), None);
        assert!(Counts::new(&[u64::MAX, 1]).is_none());
    }
    #[test]
    fn range_add_differential() {
        for n in 0..=17 {
            let mut f = RangeAdd::new(n);
            let mut a = vec![0_u64; n];
            for l in 0..=n {
                for r in l..=n {
                    let d = if (l + r) % 2 == 0 {
                        7
                    } else {
                        3_u64.wrapping_neg()
                    };
                    f.add(l, r, d);
                    for x in &mut a[l..r] {
                        *x = x.wrapping_add(d);
                    }
                    for s in 0..=n {
                        for e in s..=n {
                            assert_eq!(
                                f.range(s, e),
                                a[s..e].iter().fold(0_u64, |x, &y| x.wrapping_add(y))
                            );
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn invalid_bounds_and_wrap() {
        let mut f = Fenwick::new(&[u64::MAX, 1, 2]);
        assert_eq!(f.prefix(2), 0);
        assert_eq!(f.range(1, 3), 3);
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f.add(3, 7))).is_err());
        assert_eq!(f.range(0, 3), 2);
        assert!(std::panic::catch_unwind(|| f.range(2, 1)).is_err());
        assert!(std::panic::catch_unwind(|| f.prefix(4)).is_err());
    }
}
