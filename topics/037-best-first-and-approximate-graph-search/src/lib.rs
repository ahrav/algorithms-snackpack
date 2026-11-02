//! Exact and bounded best-first search over a validated integer-vector graph.
#![forbid(unsafe_code)]
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// A squared distance and stable corpus ID, ordered lexicographically.
pub type Neighbor = (i64, usize);

/// Work counters for one query. Frontier occupancy can exceed retained width.
#[derive(Debug, Default, Clone, Copy)]
pub struct Work {
    /// Distinct vectors scored.
    pub distances: usize,
    /// Adjacency lists expanded.
    pub expanded: usize,
    /// Maximum pending heap length.
    pub frontier: usize,
}

/// An immutable graph with equal-length, bounded integer coordinates.
#[derive(Debug)]
pub struct Graph {
    points: Vec<Vec<i32>>,
    edges: Vec<Vec<usize>>,
    dim: usize,
}

impl Graph {
    /// Validate vectors and links. An empty graph has dimension zero.
    ///
    /// # Errors
    /// Returns an error for mismatched shapes, invalid links, dimension above
    /// 1024, or coordinates outside -10000..=10000.
    pub fn new(points: Vec<Vec<i32>>, edges: Vec<Vec<usize>>) -> Result<Self, &'static str> {
        let dim = points.first().map_or(0, Vec::len);
        if points.len() != edges.len() || dim > 1024 || (!points.is_empty() && dim == 0) {
            return Err("invalid graph shape");
        }
        if points
            .iter()
            .any(|p| p.len() != dim || p.iter().any(|x| !(-10000..=10000).contains(x)))
        {
            return Err("invalid vector");
        }
        if edges.iter().flatten().any(|&id| id >= points.len()) {
            return Err("invalid link");
        }
        Ok(Self { points, edges, dim })
    }

    /// Borrow corpus vectors for independent checking and accounting.
    #[must_use]
    pub fn points(&self) -> &[Vec<i32>] {
        &self.points
    }

    /// Borrow directed adjacency lists. Duplicate and self links are permitted.
    #[must_use]
    pub fn edges(&self) -> &[Vec<usize>] {
        &self.edges
    }

    fn query(&self, q: &[i32]) -> Result<(), &'static str> {
        if q.len() != self.dim || q.iter().any(|x| !(-10000..=10000).contains(x)) {
            return Err("invalid query");
        }
        Ok(())
    }

    fn distance(&self, id: usize, q: &[i32]) -> i64 {
        self.points[id]
            .iter()
            .zip(q)
            .map(|(&x, &y)| {
                let v = i64::from(x) - i64::from(y);
                v * v
            })
            .sum()
    }

    /// Exhaustively score all vectors and retain a sorted buffer of at most k.
    /// Returns min(k, N) results, including self matches, ordered by distance/ID.
    ///
    /// # Errors
    /// Returns an error for an invalid query shape or coordinate.
    pub fn exact(&self, q: &[i32], k: usize) -> Result<(Vec<Neighbor>, Work), &'static str> {
        self.query(q)?;
        let cap = k.min(self.points.len());
        let mut best = Vec::with_capacity(cap);
        if cap == 0 {
            return Ok((best, Work::default()));
        }
        for id in 0..self.points.len() {
            let item = (self.distance(id, q), id);
            if best.len() < cap || item < best[cap - 1] {
                let at = best.binary_search(&item).unwrap_or_else(|p| p);
                best.insert(at, item);
                best.truncate(cap);
            }
        }
        Ok((
            best,
            Work {
                distances: self.points.len(),
                ..Work::default()
            },
        ))
    }

    /// Single-layer bounded best-first search. Width bounds the retained heap.
    /// Pending nodes survive retained-set eviction. A visited bit scores each
    /// node once. Disconnected or poorly routed graphs can underfill or lose recall.
    ///
    /// # Errors
    /// Returns an error for invalid query, entry, or width below min(k, N).
    pub fn search(
        &self,
        q: &[i32],
        k: usize,
        ef: usize,
        entry: usize,
    ) -> Result<(Vec<Neighbor>, Work), &'static str> {
        self.query(q)?;
        let cap = k.min(self.points.len());
        if cap == 0 {
            return Ok((Vec::new(), Work::default()));
        }
        if entry >= self.points.len() || ef < cap {
            return Err("invalid entry or width");
        }
        let width = ef.min(self.points.len());
        let first = (self.distance(entry, q), entry);
        let mut pending = BinaryHeap::from([Reverse(first)]);
        let mut best = BinaryHeap::from([first]);
        let mut seen = vec![false; self.points.len()];
        seen[entry] = true;
        let mut work = Work {
            distances: 1,
            frontier: 1,
            expanded: 0,
        };
        while let Some(Reverse(current)) = pending.pop() {
            if best.len() == width && best.peek().is_some_and(|worst| current > *worst) {
                break;
            }
            work.expanded += 1;
            for &id in &self.edges[current.1] {
                if seen[id] {
                    continue;
                }
                seen[id] = true;
                let item = (self.distance(id, q), id);
                work.distances += 1;
                if best.len() < width || best.peek().is_some_and(|worst| item < *worst) {
                    best.push(item);
                    pending.push(Reverse(item));
                    if best.len() > width {
                        best.pop();
                    }
                    work.frontier = work.frontier.max(pending.len());
                }
            }
        }
        let mut answer = best.into_sorted_vec();
        answer.truncate(cap);
        Ok((answer, work))
    }
}
