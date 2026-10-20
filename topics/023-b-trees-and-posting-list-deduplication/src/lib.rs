//! Ordered pair indexes and lossless key deduplication.
use std::collections::{BTreeMap, BTreeSet};

/// A key and a stable row identifier. Repeated identical pairs are idempotent.
pub type Pair = (u64, u64);

/// Independent scan oracle. It does not use candidate grouping or tree navigation.
#[must_use]
pub fn oracle(input: &[Pair], low: u64, high: u64) -> Vec<Pair> {
    let mut result = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for &pair in input {
        if pair.0 >= low && pair.0 < high && seen.insert(pair) {
            result.push(pair);
        }
    }
    result.sort_unstable();
    result
}

/// A representation choice for an immutable benchmark snapshot.
#[derive(Clone, Copy, Debug)]
pub enum Kind {
    /// Sorted unique pairs.
    Flat,
    /// Standard-library ordered set of pairs.
    Tree,
    /// Standard-library B-tree of sorted row-ID vectors.
    Posting,
    /// Bulk-built B+ directory with eight slots per leaf/internal group.
    Packed8,
    /// Bulk-built B+ directory with 32 slots per leaf/internal group.
    Packed32,
}

/// Mutable posting map: one key, all distinct sorted row IDs.
#[derive(Default, Debug)]
pub struct PostingMap(BTreeMap<u64, Vec<u64>>);
impl PostingMap {
    /// Insert one pair. Returns false for an already present pair.
    pub fn insert(&mut self, (key, row): Pair) -> bool {
        let rows = self.0.entry(key).or_default();
        match rows.binary_search(&row) {
            Ok(_) => false,
            Err(index) => {
                rows.insert(index, row);
                true
            }
        }
    }
    /// Remove just one row ID, deleting the key only after its last row disappears.
    pub fn remove(&mut self, (key, row): Pair) -> bool {
        let Some(rows) = self.0.get_mut(&key) else {
            return false;
        };
        let Ok(index) = rows.binary_search(&row) else {
            return false;
        };
        rows.remove(index);
        if rows.is_empty() {
            self.0.remove(&key);
        }
        true
    }
    /// Return all pairs in key-major, row-minor order from the half-open range.
    #[must_use]
    pub fn range(&self, low: u64, high: u64) -> Vec<Pair> {
        if low >= high {
            return Vec::new();
        }
        self.0
            .range(low..high)
            .flat_map(|(&key, rows)| rows.iter().map(move |&row| (key, row)))
            .collect()
    }
}

#[derive(Debug)]
struct Group {
    key: u64,
    start: usize,
    end: usize,
}
#[derive(Debug)]
struct Fence {
    max: u64,
    child: usize,
}

/// Immutable bulk-built B+ layout. Leaves index groups; internal fences index
/// groups of children. Nodes have bounded slot counts, not bounded byte sizes.
/// Every row ID remains in the contiguous payload. Updates require rebuilding.
#[derive(Debug)]
pub struct Packed {
    fanout: usize,
    groups: Vec<Group>,
    rows: Vec<u64>,
    // Bottom level: one fence per leaf. Higher levels group previous fences.
    levels: Vec<Vec<Fence>>,
}
impl Packed {
    fn build(input: &[Pair], fanout: usize) -> Self {
        let mut pairs = input.to_vec();
        pairs.sort_unstable();
        pairs.dedup();
        let mut result = Self {
            fanout,
            groups: Vec::new(),
            rows: Vec::new(),
            levels: Vec::new(),
        };
        for (key, row) in pairs {
            if result.groups.last().is_none_or(|g| g.key != key) {
                result.groups.push(Group {
                    key,
                    start: result.rows.len(),
                    end: result.rows.len(),
                });
            }
            result.rows.push(row);
            if let Some(group) = result.groups.last_mut() {
                group.end += 1;
            }
        }
        if result.groups.is_empty() {
            return result;
        }
        let mut fences: Vec<Fence> = result
            .groups
            .chunks(fanout)
            .enumerate()
            .map(|(index, chunk)| Fence {
                max: chunk[chunk.len() - 1].key,
                child: index * fanout,
            })
            .collect();
        loop {
            let count = fences.len();
            let next: Vec<Fence> = fences
                .chunks(fanout)
                .enumerate()
                .map(|(index, chunk)| Fence {
                    max: chunk[chunk.len() - 1].max,
                    child: index * fanout,
                })
                .collect();
            result.levels.push(fences);
            if count <= fanout {
                break;
            }
            fences = next;
        }
        result
    }
    fn lower_group(&self, key: u64) -> usize {
        if self.groups.is_empty() {
            return 0;
        }
        let mut begin = 0;
        for level in self.levels.iter().rev() {
            let end = (begin + self.fanout).min(level.len());
            let offset = level[begin..end].partition_point(|f| f.max < key);
            if begin + offset == end {
                return self.groups.len();
            }
            begin = level[begin + offset].child;
        }
        let end = (begin + self.fanout).min(self.groups.len());
        begin + self.groups[begin..end].partition_point(|g| g.key < key)
    }
    fn range(&self, low: u64, high: u64) -> Vec<Pair> {
        if low >= high {
            return Vec::new();
        }
        let mut output = Vec::new();
        for group in &self.groups[self.lower_group(low)..] {
            if group.key >= high {
                break;
            }
            output.extend(
                self.rows[group.start..group.end]
                    .iter()
                    .map(|&row| (group.key, row)),
            );
        }
        output
    }
    /// Check payload, fence, grouping and ordering invariants.
    #[must_use]
    pub fn valid(&self) -> bool {
        if self.groups.is_empty() {
            return self.rows.is_empty() && self.levels.is_empty();
        }
        if !self
            .groups
            .windows(2)
            .all(|g| g[0].key < g[1].key && g[0].end == g[1].start)
        {
            return false;
        }
        if self.groups[0].start != 0 || self.groups.last().is_none_or(|g| g.end != self.rows.len())
        {
            return false;
        }
        for g in &self.groups {
            if g.start >= g.end
                || g.end > self.rows.len()
                || !self.rows[g.start..g.end].windows(2).all(|r| r[0] < r[1])
            {
                return false;
            }
        }
        for (depth, level) in self.levels.iter().enumerate() {
            let child_count = if depth == 0 {
                self.groups.len()
            } else {
                self.levels[depth - 1].len()
            };
            if level.len() != child_count.div_ceil(self.fanout) {
                return false;
            }
            for (index, fence) in level.iter().enumerate() {
                let end = ((index + 1) * self.fanout).min(child_count) - 1;
                let max = if depth == 0 {
                    self.groups[end].key
                } else {
                    self.levels[depth - 1][end].max
                };
                if fence.child != index * self.fanout || fence.max != max {
                    return false;
                }
            }
        }
        self.levels.last().is_some_and(|l| l.len() <= self.fanout)
    }
}

/// Executable alternatives with identical snapshot query contracts.
#[derive(Debug)]
pub enum Index {
    /// Sorted unique pairs.
    Flat(Vec<Pair>),
    /// Ordered pair set.
    Tree(BTreeSet<Pair>),
    /// Mutable posting B-tree exposed here as a snapshot.
    Posting(PostingMap),
    /// Immutable B+ snapshot.
    Packed(Packed),
}
impl Index {
    /// Build from arbitrary pair order. Duplicate identical pairs are discarded.
    #[must_use]
    pub fn build(kind: Kind, input: &[Pair]) -> Self {
        match kind {
            Kind::Flat => {
                let mut v = input.to_vec();
                v.sort_unstable();
                v.dedup();
                Self::Flat(v)
            }
            Kind::Tree => Self::Tree(input.iter().copied().collect()),
            Kind::Posting => {
                let mut map = PostingMap::default();
                for &pair in input {
                    map.insert(pair);
                }
                Self::Posting(map)
            }
            Kind::Packed8 => Self::Packed(Packed::build(input, 8)),
            Kind::Packed32 => Self::Packed(Packed::build(input, 32)),
        }
    }
    /// Return the exact ordered pair set in `[low, high)`. Reversed ranges are empty.
    #[must_use]
    pub fn range(&self, low: u64, high: u64) -> Vec<Pair> {
        if low >= high {
            return Vec::new();
        }
        match self {
            Self::Flat(v) => {
                v[v.partition_point(|p| p.0 < low)..v.partition_point(|p| p.0 < high)].to_vec()
            }
            Self::Tree(t) => t.range((low, 0)..(high, 0)).copied().collect(),
            Self::Posting(p) => p.range(low, high),
            Self::Packed(p) => p.range(low, high),
        }
    }
    /// Logical used storage model: excludes allocator slack, tree headers and
    /// implementation-dependent B-tree metadata. Not resident memory.
    #[must_use]
    pub fn logical_bytes(&self) -> usize {
        match self {
            Self::Flat(v) => v.len() * 16,
            Self::Tree(t) => t.len() * 16,
            Self::Posting(p) => p.0.len() * 32 + p.0.values().map(|v| v.len() * 8).sum::<usize>(),
            Self::Packed(p) => {
                p.groups.len() * 24
                    + p.rows.len() * 8
                    + p.levels.iter().map(|l| l.len() * 16).sum::<usize>()
            }
        }
    }
    /// Check representation invariants exposed by this implementation.
    #[must_use]
    pub fn valid(&self) -> bool {
        match self {
            Self::Packed(p) => p.valid(),
            Self::Flat(v) => v.windows(2).all(|p| p[0] < p[1]),
            Self::Tree(_) => true,
            Self::Posting(p) => {
                p.0.values()
                    .all(|v| !v.is_empty() && v.windows(2).all(|p| p[0] < p[1]))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const KINDS: [Kind; 5] = [
        Kind::Flat,
        Kind::Tree,
        Kind::Posting,
        Kind::Packed8,
        Kind::Packed32,
    ];
    #[test]
    fn every_short_input() {
        for length in 0_u32..=6 {
            for mut code in 0..4_usize.pow(length) {
                let mut input = Vec::new();
                for _ in 0..length {
                    input.push((
                        u64::try_from(code % 2).unwrap(),
                        u64::try_from((code / 2) % 2).unwrap(),
                    ));
                    code /= 4;
                }
                for kind in KINDS {
                    let index = Index::build(kind, &input);
                    assert!(index.valid());
                    for low in 0..=3 {
                        for high in 0..=3 {
                            assert_eq!(index.range(low, high), oracle(&input, low, high));
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn boundaries_and_deep_directories() {
        for length in [0, 1, 7, 8, 9, 31, 32, 33, 63, 64, 65, 255, 256, 257, 1025] {
            let mut input: Vec<Pair> = (0..length).rev().map(|n| (n * 2, n)).collect();
            input.extend([(u64::MAX - 1, 0), (u64::MAX - 1, u64::MAX)]);
            for kind in KINDS {
                let i = Index::build(kind, &input);
                assert!(i.valid());
                for low in [
                    0,
                    1,
                    15,
                    16,
                    17,
                    63,
                    64,
                    65,
                    127,
                    128,
                    129,
                    511,
                    512,
                    513,
                    2048,
                    u64::MAX - 1,
                    u64::MAX,
                ] {
                    let high = low.saturating_add(3);
                    assert_eq!(i.range(low, high), oracle(&input, low, high));
                }
            }
        }
    }
    #[test]
    fn row_identity_and_last_delete() {
        let mut map = PostingMap::default();
        assert!(map.insert((7, 10)));
        assert!(map.insert((7, 12)));
        assert!(!map.insert((7, 10)));
        assert!(!map.remove((7, 11)));
        assert!(map.remove((7, 10)));
        assert_eq!(map.range(7, 8), [(7, 12)]);
        assert!(map.remove((7, 12)));
        assert!(map.0.is_empty());
        assert!(!map.remove((7, 12)));
    }
    #[test]
    fn mutation_histories() {
        for mut history in 0..6_usize.pow(6) {
            let mut map = PostingMap::default();
            let mut reference = Vec::new();
            for _ in 0..6 {
                let op = history % 6;
                history /= 6;
                let pair = (
                    u64::try_from((op % 3) % 2).unwrap(),
                    u64::try_from(op % 3).unwrap(),
                );
                if op < 3 {
                    let fresh = !reference.contains(&pair);
                    assert_eq!(map.insert(pair), fresh);
                    if fresh {
                        reference.push(pair);
                    }
                } else {
                    let old = reference.len();
                    reference.retain(|&p| p != pair);
                    assert_eq!(map.remove(pair), old != reference.len());
                }
                assert_eq!(map.range(0, 3), oracle(&reference, 0, 3));
            }
        }
    }
    #[test]
    fn reject_bad_fence_and_lost_row() {
        let input: Vec<_> = (0..100).map(|n| (n, n)).collect();
        let mut p = Packed::build(&input, 8);
        assert!(p.valid());
        p.levels[0][0].max += 1;
        assert!(!p.valid());
        let mut p = Packed::build(&[(7, 10), (7, 12)], 8);
        p.groups[0].end -= 1;
        assert!(!p.valid());
    }
}
