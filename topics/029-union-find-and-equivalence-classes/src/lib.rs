//! Incremental undirected equivalence classes over dense integer IDs.
//! IDs must be smaller than the constructor's length. Invalid IDs panic.
#![forbid(unsafe_code)]

/// Common connectivity contract. A union reports whether two classes merged.
pub trait Partition: Sized {
    /// Creates singleton classes for `0..n`.
    fn new(n: usize) -> Self;
    /// Returns the structural representative, which may change after a union.
    fn find(&mut self, x: usize) -> usize;
    /// Merges the classes containing the endpoints.
    fn union(&mut self, a: usize, b: usize) -> bool;
    /// Tests equivalence.
    fn same(&mut self, a: usize, b: usize) -> bool {
        self.find(a) == self.find(b)
    }
}

/// Eager labels: constant-time queries and a full scan on a successful merge.
pub struct QuickFind {
    labels: Vec<usize>,
}
impl Partition for QuickFind {
    fn new(n: usize) -> Self {
        Self {
            labels: (0..n).collect(),
        }
    }
    fn find(&mut self, x: usize) -> usize {
        self.labels[x]
    }
    fn union(&mut self, a: usize, b: usize) -> bool {
        let from = self.labels[a];
        let to = self.labels[b];
        if from == to {
            return false;
        }
        for label in &mut self.labels {
            if *label == from {
                *label = to;
            }
        }
        true
    }
}

/// Size-balanced forest. Mode 0 reads paths, 1 compresses fully, 2 halves paths.
/// All modes use separate parent and size arrays with `usize` elements.
pub struct Forest<const MODE: u8> {
    parent: Vec<usize>,
    size: Vec<usize>,
}
/// Size balancing with read-only root traversal.
pub type Size = Forest<0>;
/// Size balancing with two-pass full path compression.
pub type Full = Forest<1>;
/// Size balancing with one-pass path halving.
pub type Half = Forest<2>;
impl<const MODE: u8> Partition for Forest<MODE> {
    fn new(n: usize) -> Self {
        assert!(MODE <= 2);
        Self {
            parent: (0..n).collect(),
            size: vec![1; n],
        }
    }
    #[inline(never)]
    fn find(&mut self, mut x: usize) -> usize {
        if MODE == 2 {
            while self.parent[x] != x {
                let grandparent = self.parent[self.parent[x]];
                self.parent[x] = grandparent;
                x = grandparent;
            }
            return x;
        }
        let mut root = x;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        if MODE == 1 {
            while x != root {
                let next = self.parent[x];
                self.parent[x] = root;
                x = next;
            }
        }
        root
    }
    fn union(&mut self, a: usize, b: usize) -> bool {
        let mut a = self.find(a);
        let mut b = self.find(b);
        if a == b {
            return false;
        }
        if self.size[a] < self.size[b] {
            std::mem::swap(&mut a, &mut b);
        }
        self.parent[b] = a;
        self.size[a] += self.size[b];
        true
    }
}
impl<const MODE: u8> Forest<MODE> {
    /// Returns the current class size. Compression may mutate parent pointers.
    pub fn class_size(&mut self, x: usize) -> usize {
        let r = self.find(x);
        self.size[r]
    }
    /// Enumerates every class in O(n log n) worst case for read-only traversal.
    /// Compressed modes have O(n alpha(n)) amortized traversal plus output costs.
    pub fn groups(&mut self) -> Vec<Vec<usize>> {
        let mut groups = vec![Vec::new(); self.parent.len()];
        for x in 0..self.parent.len() {
            let r = self.find(x);
            groups[r].push(x);
        }
        groups.into_iter().filter(|g| !g.is_empty()).collect()
    }
}

/// A stack of reversible successful size-balanced merges, with no compression.
/// A snapshot is a history length valid only in this object's current lineage.
/// Rolling back and branching invalidates snapshots into the discarded future.
pub struct Rollback {
    forest: Size,
    history: Vec<(usize, usize, usize)>,
}
impl Rollback {
    /// Creates singleton classes.
    #[must_use]
    pub fn new(n: usize) -> Self {
        Self {
            forest: Size::new(n),
            history: Vec::new(),
        }
    }
    /// Captures the current history length.
    #[must_use]
    pub fn snapshot(&self) -> usize {
        self.history.len()
    }
    /// Merges classes and records exactly the successful structural change.
    pub fn union(&mut self, a: usize, b: usize) -> bool {
        let mut a = self.forest.find(a);
        let mut b = self.forest.find(b);
        if a == b {
            return false;
        }
        if self.forest.size[a] < self.forest.size[b] {
            std::mem::swap(&mut a, &mut b);
        }
        self.history.push((b, a, self.forest.size[a]));
        self.forest.parent[b] = a;
        self.forest.size[a] += self.forest.size[b];
        true
    }
    /// Tests equivalence without recording changes.
    pub fn same(&mut self, a: usize, b: usize) -> bool {
        self.forest.same(a, b)
    }
    /// Restores a snapshot in the current lineage; future history lengths panic.
    ///
    /// # Panics
    /// Panics when the snapshot is longer than the current history.
    pub fn rollback(&mut self, snapshot: usize) {
        assert!(snapshot <= self.history.len());
        while self.history.len() > snapshot {
            let (child, parent, old_size) = self.history.pop().unwrap();
            self.forest.parent[child] = child;
            self.forest.size[parent] = old_size;
        }
    }
}

/// An edge addition (`true`) or connectivity query (`false`).
#[derive(Clone, Copy, Debug)]
pub struct Op {
    /// Whether to merge the endpoints.
    pub merge: bool,
    /// First endpoint.
    pub a: usize,
    /// Second endpoint.
    pub b: usize,
}
/// Executes a trace, preserving every union/query response.
pub fn apply<P: Partition>(p: &mut P, ops: &[Op]) -> Vec<bool> {
    ops.iter()
        .map(|op| {
            if op.merge {
                p.union(op.a, op.b)
            } else {
                p.same(op.a, op.b)
            }
        })
        .collect()
}
/// Independent graph-search oracle. Each response is obtained by reachability.
/// Traversal and adjacency storage share no candidate implementation helpers.
#[must_use]
pub fn oracle(n: usize, ops: &[Op]) -> Vec<bool> {
    let mut graph: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut out = Vec::new();
    for op in ops {
        let mut seen = vec![false; n];
        let mut stack = vec![op.a];
        seen[op.a] = true;
        while let Some(x) = stack.pop() {
            for &y in &graph[x] {
                if !seen[y] {
                    seen[y] = true;
                    stack.push(y);
                }
            }
        }
        out.push(if op.merge { !seen[op.b] } else { seen[op.b] });
        if op.merge && !seen[op.b] {
            graph[op.a].push(op.b);
            graph[op.b].push(op.a);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn compare(n: usize, ops: &[Op]) {
        let expected = oracle(n, ops);
        assert_eq!(apply(&mut QuickFind::new(n), ops), expected);
        assert_eq!(apply(&mut Size::new(n), ops), expected);
        assert_eq!(apply(&mut Full::new(n), ops), expected);
        assert_eq!(apply(&mut Half::new(n), ops), expected);
    }
    #[test]
    fn exhaustive_three_element_histories() {
        // 18 possible operations, all histories of length four: 104,976 traces.
        for mut code in 0..18_usize.pow(4) {
            let mut ops = Vec::new();
            for _ in 0..4 {
                let digit = code % 18;
                code /= 18;
                ops.push(Op {
                    merge: digit < 9,
                    a: digit % 9 / 3,
                    b: digit % 3,
                });
            }
            compare(3, &ops);
        }
    }
    #[test]
    fn seeded_larger_histories() {
        let mut seed = 91_u64;
        for n in [1, 2, 8, 31, 64] {
            let mut ops = Vec::new();
            for _ in 0..300 {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                let a = usize::try_from(seed % u64::try_from(n).unwrap()).unwrap();
                seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let b = usize::try_from(seed % u64::try_from(n).unwrap()).unwrap();
                ops.push(Op {
                    merge: seed.trailing_zeros() >= 2,
                    a,
                    b,
                });
            }
            compare(n, &ops);
        }
    }
    #[test]
    fn balanced_paths_and_enumeration() {
        let mut f = Full::new(8);
        for (a, b) in [(0, 1), (2, 3), (4, 5), (6, 7), (0, 2), (4, 6), (0, 4)] {
            assert!(f.union(a, b));
        }
        assert_eq!(f.parent[7], 6);
        assert_eq!(f.find(7), 0);
        assert_eq!(f.parent[7], 0);
        assert_eq!(f.class_size(7), 8);
        assert_eq!(f.groups(), vec![(0..8).collect::<Vec<_>>()]);
        assert!(!f.union(0, 7));
    }
    #[test]
    fn rollback_nested_duplicate_and_branch() {
        let mut f = Rollback::new(5);
        let start = f.snapshot();
        assert!(f.union(0, 1));
        let one = f.snapshot();
        assert!(!f.union(1, 0));
        assert_eq!(one, f.snapshot());
        assert!(f.union(2, 3));
        assert!(f.union(0, 2));
        assert!(f.same(1, 3));
        f.rollback(one);
        assert!(f.same(0, 1));
        assert!(!f.same(1, 3));
        assert!(f.union(1, 4));
        assert!(f.same(0, 4));
        f.rollback(start);
        for a in 0..5 {
            for b in 0..5 {
                assert_eq!(f.same(a, b), a == b);
            }
        }
    }
    #[test]
    fn empty_and_singleton() {
        assert!(Full::new(0).groups().is_empty());
        compare(0, &[]);
        compare(
            1,
            &[
                Op {
                    merge: true,
                    a: 0,
                    b: 0,
                },
                Op {
                    merge: false,
                    a: 0,
                    b: 0,
                },
            ],
        );
    }
    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn invalid_id_panics() {
        Full::new(2).union(0, 2);
    }
}
