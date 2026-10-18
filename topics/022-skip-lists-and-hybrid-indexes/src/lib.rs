//! Sequential member-to-score indexes with exact zero-based rank and selection.
//! Integer scores order ascending, then unique integer member IDs ascending.
use std::collections::{BTreeSet, HashMap};

/// Ordered record `(score, member)`.
pub type Key = (i64, u64);
/// Common infallible-allocation teaching contract. No concurrency or panic rollback.
pub trait OrderedIndex {
    /// Insert or replace the score of a member.
    fn put(&mut self, id: u64, score: i64);
    /// Remove a member; return whether it existed.
    fn remove(&mut self, id: u64) -> bool;
    /// Read the authoritative score by member.
    fn get(&self, id: u64) -> Option<i64>;
    /// Return the zero-based position of an existing member.
    fn rank(&self, id: u64) -> Option<usize>;
    /// Return the record at a zero-based rank, or None beyond the end.
    fn select(&self, rank: usize) -> Option<Key>;
    /// Return the first at most `limit` records whose score is >= `low`.
    fn range(&self, low: i64, limit: usize) -> Vec<Key>;
    /// Return all ordered live records for independent validation.
    fn snapshot(&self) -> Vec<Key>;
}

/// Sorted contiguous records plus direct member lookup.
#[derive(Default)]
pub struct Flat {
    rows: Vec<Key>,
    members: HashMap<u64, i64>,
}
impl OrderedIndex for Flat {
    fn put(&mut self, id: u64, score: i64) {
        self.remove(id);
        let k = (score, id);
        let p = self.rows.binary_search(&k).unwrap_err();
        self.rows.insert(p, k);
        self.members.insert(id, score);
    }
    fn remove(&mut self, id: u64) -> bool {
        if let Some(s) = self.members.remove(&id) {
            let p = self.rows.binary_search(&(s, id)).unwrap();
            self.rows.remove(p);
            true
        } else {
            false
        }
    }
    fn get(&self, id: u64) -> Option<i64> {
        self.members.get(&id).copied()
    }
    fn rank(&self, id: u64) -> Option<usize> {
        self.get(id)
            .map(|s| self.rows.binary_search(&(s, id)).unwrap())
    }
    fn select(&self, rank: usize) -> Option<Key> {
        self.rows.get(rank).copied()
    }
    fn range(&self, low: i64, limit: usize) -> Vec<Key> {
        let p = self.rows.partition_point(|k| k.0 < low);
        self.rows[p..].iter().take(limit).copied().collect()
    }
    fn snapshot(&self) -> Vec<Key> {
        self.rows.clone()
    }
}

/// Standard ordered set plus direct member lookup; rank/select walk iterators.
#[derive(Default)]
pub struct Tree {
    rows: BTreeSet<Key>,
    members: HashMap<u64, i64>,
}
impl OrderedIndex for Tree {
    fn put(&mut self, id: u64, score: i64) {
        self.remove(id);
        self.rows.insert((score, id));
        self.members.insert(id, score);
    }
    fn remove(&mut self, id: u64) -> bool {
        if let Some(s) = self.members.remove(&id) {
            assert!(self.rows.remove(&(s, id)));
            true
        } else {
            false
        }
    }
    fn get(&self, id: u64) -> Option<i64> {
        self.members.get(&id).copied()
    }
    fn rank(&self, id: u64) -> Option<usize> {
        self.get(id).map(|s| self.rows.range(..(s, id)).count())
    }
    fn select(&self, rank: usize) -> Option<Key> {
        self.rows.iter().nth(rank).copied()
    }
    fn range(&self, low: i64, limit: usize) -> Vec<Key> {
        self.rows.range((low, 0)..).take(limit).copied().collect()
    }
    fn snapshot(&self) -> Vec<Key> {
        self.rows.iter().copied().collect()
    }
}

const HEIGHT: usize = 24;
#[derive(Clone, Copy, Default)]
struct Link {
    next: usize,
    span: usize,
}
struct Node {
    key: Key,
    links: Vec<Link>,
}

/// Arena-backed skip list, geometric height p=1/4, capped at 24 levels.
/// Slots are reused only after all incoming links are removed. No external handles.
pub struct Skip {
    nodes: Vec<Option<Node>>,
    free: Vec<usize>,
    members: HashMap<u64, i64>,
    level: usize,
    len: usize,
    random: u64,
}
impl Default for Skip {
    fn default() -> Self {
        Self::with_seed(22018)
    }
}
impl Skip {
    /// Construct with a nonzero xorshift64 state. Seed is for reproducibility, not security.
    ///
    /// # Panics
    /// Panics if `seed` is zero.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        assert_ne!(seed, 0);
        Self {
            nodes: vec![Some(Node {
                key: (0, 0),
                links: vec![Link::default(); HEIGHT],
            })],
            free: vec![],
            members: HashMap::new(),
            level: 1,
            len: 0,
            random: seed,
        }
    }
    fn node(&self, i: usize) -> &Node {
        self.nodes[i].as_ref().unwrap()
    }
    fn link(&self, i: usize, l: usize) -> Link {
        self.node(i).links[l]
    }
    fn set(&mut self, i: usize, l: usize, link: Link) {
        self.nodes[i].as_mut().unwrap().links[l] = link;
    }
    fn height(&mut self) -> usize {
        let mut h = 1;
        while h < HEIGHT {
            self.random ^= self.random << 13;
            self.random ^= self.random >> 7;
            self.random ^= self.random << 17;
            if self.random & 3 != 0 {
                break;
            }
            h += 1;
        }
        h
    }
    // Header position 0, first record position 1. Terminal spans reach position len.
    fn path(&self, key: Key) -> ([usize; HEIGHT], [usize; HEIGHT]) {
        let mut update = [0; HEIGHT];
        let mut ranks = [0; HEIGHT];
        let mut x = 0;
        let mut crossed = 0;
        for l in (0..self.level).rev() {
            loop {
                let a = self.link(x, l);
                if a.next == 0 || self.node(a.next).key >= key {
                    break;
                }
                crossed += a.span;
                x = a.next;
            }
            update[l] = x;
            ranks[l] = crossed;
        }
        (update, ranks)
    }
    fn insert_new(&mut self, key: Key) {
        let (mut update, mut ranks) = self.path(key);
        let height = self.height();
        if height > self.level {
            for l in self.level..height {
                ranks[l] = 0;
                update[l] = 0;
                self.set(
                    0,
                    l,
                    Link {
                        next: 0,
                        span: self.len,
                    },
                );
            }
            self.level = height;
        }
        let node = Node {
            key,
            links: vec![Link::default(); height],
        };
        let x = if let Some(i) = self.free.pop() {
            self.nodes[i] = Some(node);
            i
        } else {
            self.nodes.push(Some(node));
            self.nodes.len() - 1
        };
        for l in 0..height {
            let old = self.link(update[l], l);
            let d = ranks[0] - ranks[l];
            self.set(
                x,
                l,
                Link {
                    next: old.next,
                    span: old.span - d,
                },
            );
            self.set(
                update[l],
                l,
                Link {
                    next: x,
                    span: d + 1,
                },
            );
        }
        for (l, &u) in update.iter().enumerate().take(self.level).skip(height) {
            let mut a = self.link(u, l);
            a.span += 1;
            self.set(u, l, a);
        }
        self.len += 1;
    }
    /// Validate every active span, order, live member and recycled arena slot.
    /// This diagnostic is excluded from the timed region.
    ///
    /// # Panics
    /// Panics if an order, membership, span, link or arena invariant is violated.
    pub fn validate(&self) {
        let mut positions = HashMap::new();
        positions.insert(0, 0);
        let mut ordered = vec![];
        let mut x = self.link(0, 0).next;
        while x != 0 {
            assert!(!positions.contains_key(&x), "cycle or duplicate node");
            ordered.push(self.node(x).key);
            positions.insert(x, ordered.len());
            x = self.link(x, 0).next;
        }
        assert_eq!(ordered.len(), self.len);
        assert!(ordered.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(self.members.len(), self.len);
        for &(score, id) in &ordered {
            assert_eq!(self.members.get(&id), Some(&score));
        }
        assert!(
            (1..=HEIGHT).contains(&self.level),
            "level {} out of range",
            self.level
        );
        assert!(
            self.level == 1 || self.link(0, self.level - 1).next != 0,
            "empty top level {} retained",
            self.level - 1
        );
        for (&i, &pos) in &positions {
            assert!(
                i == 0 || self.node(i).links.len() <= self.level,
                "node {i} tower exceeds level {}",
                self.level
            );
            for l in 0..self.node(i).links.len().min(self.level) {
                let a = self.link(i, l);
                let target = if a.next == 0 {
                    self.len
                } else {
                    assert!(self.node(a.next).links.len() > l);
                    positions[&a.next]
                };
                assert_eq!(a.span, target - pos, "span at node {i} level {l}");
            }
        }
        let free: BTreeSet<_> = self.free.iter().copied().collect();
        assert_eq!(free.len(), self.free.len());
        for i in 1..self.nodes.len() {
            assert_eq!(self.nodes[i].is_none(), free.contains(&i));
            assert_eq!(self.nodes[i].is_some(), positions.contains_key(&i));
        }
    }
    /// Arena slots retained at the historical peak, excluding the header.
    #[must_use]
    pub fn retained_slots(&self) -> usize {
        self.nodes.len() - 1
    }
}
impl OrderedIndex for Skip {
    fn put(&mut self, id: u64, score: i64) {
        self.remove(id);
        self.insert_new((score, id));
        self.members.insert(id, score);
    }
    fn remove(&mut self, id: u64) -> bool {
        let Some(score) = self.members.get(&id).copied() else {
            return false;
        };
        let (update, _) = self.path((score, id));
        let x = self.link(update[0], 0).next;
        assert_ne!(x, 0);
        assert_eq!(self.node(x).key, (score, id));
        for (l, &u) in update.iter().enumerate().take(self.level) {
            let mut a = self.link(u, l);
            if a.next == x {
                let b = self.link(x, l);
                a.span = a.span + b.span - 1;
                a.next = b.next;
            } else {
                a.span -= 1;
            }
            self.set(u, l, a);
        }
        self.len -= 1;
        while self.level > 1 && self.link(0, self.level - 1).next == 0 {
            self.level -= 1;
        }
        self.members.remove(&id);
        self.nodes[x] = None;
        self.free.push(x);
        true
    }
    fn get(&self, id: u64) -> Option<i64> {
        self.members.get(&id).copied()
    }
    fn rank(&self, id: u64) -> Option<usize> {
        self.get(id).map(|score| self.path((score, id)).1[0])
    }
    fn select(&self, rank: usize) -> Option<Key> {
        if rank >= self.len {
            return None;
        }
        let target = rank + 1;
        let mut crossed = 0;
        let mut x = 0;
        for l in (0..self.level).rev() {
            loop {
                let a = self.link(x, l);
                if a.next == 0 || a.span > target - crossed {
                    break;
                }
                crossed += a.span;
                x = a.next;
            }
        }
        assert_eq!(crossed, target);
        Some(self.node(x).key)
    }
    fn range(&self, low: i64, limit: usize) -> Vec<Key> {
        let (update, _) = self.path((low, 0));
        let mut x = self.link(update[0], 0).next;
        let mut out = vec![];
        while x != 0 && out.len() < limit {
            out.push(self.node(x).key);
            x = self.link(x, 0).next;
        }
        out
    }
    fn snapshot(&self) -> Vec<Key> {
        self.range(i64::MIN, self.len)
    }
}

/// Independent oracle: unsorted unique members, with fresh sorting for order queries.
#[derive(Default)]
pub struct Oracle {
    rows: Vec<(u64, i64)>,
}
impl OrderedIndex for Oracle {
    fn put(&mut self, id: u64, score: i64) {
        if let Some(r) = self.rows.iter_mut().find(|r| r.0 == id) {
            r.1 = score;
        } else {
            self.rows.push((id, score));
        }
    }
    fn remove(&mut self, id: u64) -> bool {
        if let Some(i) = self.rows.iter().position(|r| r.0 == id) {
            self.rows.swap_remove(i);
            true
        } else {
            false
        }
    }
    fn get(&self, id: u64) -> Option<i64> {
        self.rows.iter().find(|r| r.0 == id).map(|r| r.1)
    }
    fn rank(&self, id: u64) -> Option<usize> {
        self.snapshot().iter().position(|r| r.1 == id)
    }
    fn select(&self, rank: usize) -> Option<Key> {
        self.snapshot().get(rank).copied()
    }
    fn range(&self, low: i64, limit: usize) -> Vec<Key> {
        self.snapshot()
            .into_iter()
            .filter(|r| r.0 >= low)
            .take(limit)
            .collect()
    }
    fn snapshot(&self) -> Vec<Key> {
        let mut out: Vec<_> = self.rows.iter().map(|&(id, s)| (s, id)).collect();
        out.sort_unstable();
        out
    }
}

#[cfg(test)]
#[allow(clippy::many_single_char_names)]
mod tests {
    use super::*;
    fn check(s: &Skip, f: &Flat, t: &Tree, o: &Oracle) {
        s.validate();
        let expected = o.snapshot();
        for c in [s as &dyn OrderedIndex, f, t] {
            assert_eq!(c.snapshot(), expected);
            for id in 0..8 {
                assert_eq!(c.get(id), o.get(id));
                assert_eq!(c.rank(id), o.rank(id));
            }
            for rank in 0..=expected.len() + 1 {
                assert_eq!(c.select(rank), o.select(rank));
            }
            for low in [-2, 0, 2, i64::MIN, i64::MAX] {
                for limit in [0, 1, 8] {
                    assert_eq!(c.range(low, limit), o.range(low, limit));
                }
            }
        }
    }
    #[test]
    fn exhaustive_short_histories() {
        // Nine actions, histories of length five: 59,049 independently replayed histories.
        for mut code in 0..9_usize.pow(5) {
            let (mut s, mut f, mut t, mut o) = (
                Skip::default(),
                Flat::default(),
                Tree::default(),
                Oracle::default(),
            );
            for _ in 0..5 {
                let a = code % 9;
                code /= 9;
                let id = (a / 3) as u64;
                if a % 3 == 2 {
                    let v = o.remove(id);
                    assert_eq!(s.remove(id), v);
                    assert_eq!(f.remove(id), v);
                    assert_eq!(t.remove(id), v);
                } else {
                    let score = i64::try_from(a % 3).unwrap() - 1;
                    for c in [&mut s as &mut dyn OrderedIndex, &mut f, &mut t, &mut o] {
                        c.put(id, score);
                    }
                }
            }
            check(&s, &f, &t, &o);
        }
    }
    #[test]
    fn seeded_churn_checks_every_mutation() {
        let (mut s, mut f, mut t, mut o) = (
            Skip::default(),
            Flat::default(),
            Tree::default(),
            Oracle::default(),
        );
        let mut r = 123_u64;
        for i in 0..4000 {
            r ^= r << 13;
            r ^= r >> 7;
            r ^= r << 17;
            let id = r % 8;
            if i % 4 == 0 {
                let v = o.remove(id);
                assert_eq!(s.remove(id), v);
                assert_eq!(f.remove(id), v);
                assert_eq!(t.remove(id), v);
            } else {
                let score = if i % 37 == 0 {
                    i64::MIN
                } else if i % 41 == 0 {
                    i64::MAX
                } else {
                    i64::try_from(r % 5).unwrap() - 2
                };
                for c in [&mut s as &mut dyn OrderedIndex, &mut f, &mut t, &mut o] {
                    c.put(id, score);
                }
            }
            check(&s, &f, &t, &o);
        }
        assert!(s.retained_slots() <= 8);
    }
    #[test]
    fn tall_paths_and_terminal_spans() {
        for seed in 1..=8 {
            let mut s = Skip::with_seed(seed);
            for id in 0..1024 {
                s.put(id, i64::try_from(id % 7).unwrap());
            }
            s.validate();
            for id in (0..1024).step_by(3) {
                s.remove(id);
                s.validate();
            }
            for id in 0..1024 {
                s.remove(id);
            }
            s.validate();
            assert_eq!(s.select(usize::MAX), None);
        }
    }
}
