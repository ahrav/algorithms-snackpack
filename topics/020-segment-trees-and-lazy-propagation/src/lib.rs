//! Half-open range updates and sums modulo 2^64, plus ordered folds.
#![forbid(unsafe_code)]

/// An affine element update `x -> mul*x + add`, modulo 2^64.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tag {
    /// Multiplier. Zero assigns; one adds.
    pub mul: u64,
    /// Added value.
    pub add: u64,
}
impl Tag {
    /// The identity update.
    pub const ID: Self = Self { mul: 1, add: 0 };
    /// Add a value to each element.
    #[must_use]
    pub const fn adding(add: u64) -> Self {
        Self { mul: 1, add }
    }
    /// Assign the same value to each element.
    #[must_use]
    pub const fn assigning(add: u64) -> Self {
        Self { mul: 0, add }
    }
    /// Compose this newer update after `old`.
    #[must_use]
    pub const fn after(self, old: Self) -> Self {
        Self {
            mul: self.mul.wrapping_mul(old.mul),
            add: self.mul.wrapping_mul(old.add).wrapping_add(self.add),
        }
    }
    fn value(self, value: u64) -> u64 {
        self.mul.wrapping_mul(value).wrapping_add(self.add)
    }
    fn sum(self, sum: u64, len: usize) -> u64 {
        self.mul.wrapping_mul(sum).wrapping_add(
            self.add
                .wrapping_mul(u64::try_from(len).expect("length fits u64")),
        )
    }
}

/// Implemented benchmark representations.
#[derive(Clone, Copy, Debug)]
pub enum Kind {
    /// Eager array with direct range scans.
    Scan,
    /// Rebuild all prefixes after each nonempty update.
    Prefix,
    /// Eager tree descends to affected leaves.
    Eager,
    /// Lazy tree retains composed updates at covered nodes.
    Lazy,
}

/// A fixed-size mutable array with explicit wrapping arithmetic.
pub struct RangeSum {
    n: usize,
    values: Vec<u64>,
    sums: Vec<u64>,
    tags: Vec<Tag>,
    kind: Kind,
}
impl RangeSum {
    /// Build in O(n) time and space. Allocation failures follow `Vec` policy.
    ///
    /// # Panics
    /// Panics if the required tree capacity cannot fit usize.
    #[must_use]
    pub fn new(input: &[u64], kind: Kind) -> Self {
        let n = input.len();
        let capacity = n.max(1).checked_mul(4).expect("tree capacity overflow");
        let mut result = Self {
            n,
            values: Vec::new(),
            sums: Vec::new(),
            tags: Vec::new(),
            kind,
        };
        match kind {
            Kind::Scan => result.values = input.to_vec(),
            Kind::Prefix => {
                result.values = input.to_vec();
                result.sums = vec![0; n + 1];
                result.rebuild();
            }
            Kind::Eager | Kind::Lazy => {
                result.sums = vec![0; capacity];
                if matches!(kind, Kind::Lazy) {
                    result.tags = vec![Tag::ID; capacity];
                }
                if n > 0 {
                    result.build(1, 0, n, input);
                }
            }
        }
        result
    }
    fn rebuild(&mut self) {
        for (i, &v) in self.values.iter().enumerate() {
            self.sums[i + 1] = self.sums[i].wrapping_add(v);
        }
    }
    fn build(&mut self, p: usize, lo: usize, hi: usize, a: &[u64]) {
        if hi - lo == 1 {
            self.sums[p] = a[lo];
            return;
        }
        let mid = lo + (hi - lo) / 2;
        self.build(2 * p, lo, mid, a);
        self.build(2 * p + 1, mid, hi, a);
        self.pull(p);
    }
    fn pull(&mut self, p: usize) {
        self.sums[p] = self.sums[2 * p].wrapping_add(self.sums[2 * p + 1]);
    }
    fn apply(&mut self, p: usize, len: usize, tag: Tag) {
        self.sums[p] = tag.sum(self.sums[p], len);
        if len > 1 {
            self.tags[p] = tag.after(self.tags[p]);
        }
    }
    fn push(&mut self, p: usize, lo: usize, hi: usize) {
        if self.tags[p] == Tag::ID || hi - lo == 1 {
            return;
        }
        let mid = lo + (hi - lo) / 2;
        let tag = self.tags[p];
        self.apply(2 * p, mid - lo, tag);
        self.apply(2 * p + 1, hi - mid, tag);
        self.tags[p] = Tag::ID;
    }
    /// Apply an affine update to `[l,r)`. Empty updates do nothing.
    ///
    /// # Panics
    /// Panics unless `l <= r <= n`, before changing state.
    pub fn update(&mut self, l: usize, r: usize, tag: Tag) {
        assert!(l <= r && r <= self.n);
        if l == r {
            return;
        }
        match self.kind {
            Kind::Scan | Kind::Prefix => {
                for v in &mut self.values[l..r] {
                    *v = tag.value(*v);
                }
                if matches!(self.kind, Kind::Prefix) {
                    self.rebuild();
                }
            }
            Kind::Eager | Kind::Lazy => self.change(1, 0, self.n, l, r, tag),
        }
    }
    #[allow(clippy::too_many_arguments)] // Explicit node and query intervals keep the recursion visible.
    fn change(&mut self, p: usize, lo: usize, hi: usize, l: usize, r: usize, tag: Tag) {
        if r <= lo || hi <= l {
            return;
        }
        if matches!(self.kind, Kind::Lazy) && l <= lo && hi <= r {
            self.apply(p, hi - lo, tag);
            return;
        }
        if hi - lo == 1 {
            self.sums[p] = tag.value(self.sums[p]);
            return;
        }
        if matches!(self.kind, Kind::Lazy) {
            self.push(p, lo, hi);
        }
        let mid = lo + (hi - lo) / 2;
        self.change(2 * p, lo, mid, l, r, tag);
        self.change(2 * p + 1, mid, hi, l, r, tag);
        self.pull(p);
    }
    /// Sum `[l,r)` modulo 2^64. Queries may push lazy tags.
    ///
    /// # Panics
    /// Panics unless `l <= r <= n`.
    pub fn query(&mut self, l: usize, r: usize) -> u64 {
        assert!(l <= r && r <= self.n);
        if l == r {
            return 0;
        }
        match self.kind {
            Kind::Scan => self.values[l..r]
                .iter()
                .fold(0_u64, |s, x| s.wrapping_add(*x)),
            Kind::Prefix => self.sums[r].wrapping_sub(self.sums[l]),
            Kind::Eager | Kind::Lazy => self.fold(1, 0, self.n, l, r),
        }
    }
    fn fold(&mut self, p: usize, lo: usize, hi: usize, l: usize, r: usize) -> u64 {
        if r <= lo || hi <= l {
            return 0;
        }
        if l <= lo && hi <= r {
            return self.sums[p];
        }
        if matches!(self.kind, Kind::Lazy) {
            self.push(p, lo, hi);
        }
        let mid = lo + (hi - lo) / 2;
        let left = self.fold(2 * p, lo, mid, l, r);
        let right = self.fold(2 * p + 1, mid, hi, l, r);
        left.wrapping_add(right)
    }
    /// Retained vector payload bytes, excluding allocation metadata and capacity slack.
    #[must_use]
    pub fn payload_bytes(&self) -> usize {
        self.values.len() * size_of::<u64>()
            + self.sums.len() * size_of::<u64>()
            + self.tags.len() * size_of::<Tag>()
    }
}

/// A static segment tree preserving operand order for an associative operation.
pub struct OrderedTree<T> {
    n: usize,
    base: usize,
    data: Vec<T>,
    identity: T,
    combine: fn(&T, &T) -> T,
}
impl<T: Clone> OrderedTree<T> {
    /// Build from an identity and associative combine. Neither commutativity nor inverses are required.
    ///
    /// # Panics
    /// Panics if the required padded tree capacity cannot fit usize.
    #[must_use]
    pub fn new(a: &[T], identity: T, combine: fn(&T, &T) -> T) -> Self {
        let base = a
            .len()
            .max(1)
            .checked_next_power_of_two()
            .expect("capacity overflow");
        let mut data = vec![identity.clone(); base.checked_mul(2).expect("capacity overflow")];
        data[base..base + a.len()].clone_from_slice(a);
        for p in (1..base).rev() {
            data[p] = combine(&data[2 * p], &data[2 * p + 1]);
        }
        Self {
            n: a.len(),
            base,
            data,
            identity,
            combine,
        }
    }
    /// Fold `[l,r)` left to right; empty ranges return the identity.
    ///
    /// # Panics
    /// Panics unless `l <= r <= n`.
    #[must_use]
    pub fn query(&self, mut l: usize, mut r: usize) -> T {
        assert!(l <= r && r <= self.n);
        l += self.base;
        r += self.base;
        let mut left = self.identity.clone();
        let mut right = self.identity.clone();
        while l < r {
            if l & 1 == 1 {
                left = (self.combine)(&left, &self.data[l]);
                l += 1;
            }
            if r & 1 == 1 {
                r -= 1;
                right = (self.combine)(&self.data[r], &right);
            }
            l /= 2;
            r /= 2;
        }
        (self.combine)(&left, &right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const KINDS: [Kind; 4] = [Kind::Scan, Kind::Prefix, Kind::Eager, Kind::Lazy];
    // Oracle never uses Tag helpers or tree decomposition.
    fn oracle_update(a: &mut [u64], l: usize, r: usize, t: Tag) {
        for x in &mut a[l..r] {
            *x = t.mul.wrapping_mul(*x).wrapping_add(t.add);
        }
    }
    fn oracle_sum(a: &[u64], l: usize, r: usize) -> u64 {
        a[l..r].iter().copied().fold(0, u64::wrapping_add)
    }
    #[test]
    fn exhaustive_small_ranges_and_tag_order() {
        for n in 0..=8 {
            let initial: Vec<u64> = (0..n).map(|x| u64::try_from(x).unwrap()).collect();
            for kind in KINDS {
                for l in 0..=n {
                    for r in l..=n {
                        for first in [
                            Tag::adding(3),
                            Tag::assigning(0),
                            Tag::assigning(7),
                            Tag {
                                mul: 2,
                                add: u64::MAX,
                            },
                        ] {
                            let mut a = initial.clone();
                            let mut t = RangeSum::new(&a, kind);
                            oracle_update(&mut a, l, r, first);
                            t.update(l, r, first);
                            oracle_update(&mut a, 0, n, Tag::adding(5));
                            t.update(0, n, Tag::adding(5));
                            for ql in 0..=n {
                                for qr in ql..=n {
                                    assert_eq!(t.query(ql, qr), oracle_sum(&a, ql, qr));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn seeded_differential_overlaps_wrapping_and_nonpowers() {
        for n in [1, 3, 17, 65] {
            for kind in KINDS {
                let mut a = vec![u64::MAX; n];
                let mut tree = RangeSum::new(&a, kind);
                let mut seed = 19_u64;
                for i in 0..2048 {
                    seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                    let l = usize::try_from(seed % u64::try_from(n + 1).unwrap()).unwrap();
                    let r = l + usize::try_from((seed >> 32) % u64::try_from(n - l + 1).unwrap())
                        .unwrap();
                    let tag = if i % 3 == 0 {
                        Tag::assigning(seed)
                    } else {
                        Tag::adding(seed)
                    };
                    tree.update(l, r, tag);
                    oracle_update(&mut a, l, r, tag);
                    assert_eq!(tree.query(0, n), oracle_sum(&a, 0, n));
                    assert_eq!(tree.query(l, r), oracle_sum(&a, l, r));
                }
            }
        }
    }
    #[test]
    fn newer_tag_after_older_is_not_commutative() {
        let assign = Tag::assigning(4);
        let add = Tag::adding(3);
        assert_eq!(add.after(assign), Tag::assigning(7));
        assert_eq!(assign.after(add), assign);
        let mut t = RangeSum::new(&[2, 1, 3, 4, 0, 5, 2, 1], Kind::Lazy);
        t.update(2, 6, Tag::adding(3));
        assert_eq!(t.query(0, 8), 30);
        assert_eq!(t.query(3, 5), 10);
        t.update(0, 8, Tag::assigning(4));
        t.update(0, 8, Tag::adding(3));
        assert_eq!(t.query(1, 7), 42);
        t.update(0, 8, Tag::assigning(4));
        assert_eq!(t.query(1, 7), 24);
    }
    #[test]
    fn ordered_fold_rejects_reversed_right_accumulator() {
        fn join(a: &String, b: &String) -> String {
            format!("{a}{b}")
        }
        let a: Vec<String> = (0..13).map(|x| format!("<{x}>")).collect();
        let t = OrderedTree::new(&a, String::new(), join);
        for l in 0..=a.len() {
            for r in l..=a.len() {
                assert_eq!(t.query(l, r), a[l..r].concat());
            }
        }
    }
    #[test]
    fn invalid_update_panics_before_mutation() {
        for kind in KINDS {
            let mut t = RangeSum::new(&[1, 2, 3], kind);
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| t.update(
                    0,
                    4,
                    Tag::assigning(9)
                )))
                .is_err()
            );
            assert_eq!(t.query(0, 3), 6);
        }
    }
}
