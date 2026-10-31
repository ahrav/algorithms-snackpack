//! Exact reflexive reachability on immutable directed acyclic graphs.
use std::collections::VecDeque;

/// Validated immutable graph with arbitrary vertex numbering.
pub struct Dag {
    edges: Vec<Vec<usize>>,
    order: Vec<usize>,
    rank: Vec<usize>,
}
impl Dag {
    /// Validates endpoints and rejects cycles, including self edges.
    ///
    /// # Errors
    /// Returns `endpoint` or `cycle` for invalid input.
    pub fn new(edges: Vec<Vec<usize>>) -> Result<Self, &'static str> {
        let n = edges.len();
        let mut degree = vec![0; n];
        for row in &edges {
            for &v in row {
                if v >= n {
                    return Err("endpoint");
                }
                degree[v] += 1;
            }
        }
        let mut queue: VecDeque<_> = (0..n).filter(|&v| degree[v] == 0).collect();
        let mut order = Vec::with_capacity(n);
        while let Some(u) = queue.pop_front() {
            order.push(u);
            for &v in &edges[u] {
                degree[v] -= 1;
                if degree[v] == 0 {
                    queue.push_back(v);
                }
            }
        }
        if order.len() != n {
            return Err("cycle");
        }
        let mut rank = vec![0; n];
        for (i, &v) in order.iter().enumerate() {
            rank[v] = i;
        }
        Ok(Self { edges, order, rank })
    }
    /// Number of vertices.
    #[must_use]
    pub fn len(&self) -> usize {
        self.edges.len()
    }
    /// Whether the graph has no vertices.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }
    /// Independent per-source breadth-first oracle, including zero-edge paths.
    ///
    /// # Panics
    /// Panics when `source` is outside the graph.
    #[must_use]
    pub fn oracle(&self, source: usize) -> Vec<bool> {
        let mut reached = vec![false; self.len()];
        let mut queue = VecDeque::from([source]);
        reached[source] = true;
        while let Some(u) = queue.pop_front() {
            for &v in &self.edges[u] {
                if !reached[v] {
                    reached[v] = true;
                    queue.push_back(v);
                }
            }
        }
        reached
    }
}

/// Reusable traversal state. A fresh query clears the visit map in O(n).
pub struct Scratch {
    seen: Vec<bool>,
    stack: Vec<usize>,
}
impl Scratch {
    /// Allocates state for a graph with `n` vertices.
    #[must_use]
    pub fn new(n: usize) -> Self {
        Self {
            seen: vec![false; n],
            stack: Vec::with_capacity(n),
        }
    }
}

/// Optional three-dimensional min-post interval rejection index.
pub struct Intervals {
    labels: Vec<Vec<(usize, usize)>>,
}
impl Intervals {
    /// Builds three deterministic traversal orders, using reversal and rotation.
    #[must_use]
    pub fn new(g: &Dag) -> Self {
        let n = g.len();
        let mut labels = Vec::new();
        for d in 0..3 {
            let mut seen = vec![false; n];
            let mut post = vec![0; n];
            let mut next = 0;
            let roots: Vec<_> = if d == 1 {
                (0..n).rev().collect()
            } else {
                (0..n).collect()
            };
            for root in roots {
                if seen[root] {
                    continue;
                }
                seen[root] = true;
                let mut stack = vec![(root, 0)];
                while let Some((u, i)) = stack.last_mut() {
                    let len = g.edges[*u].len();
                    if *i == len {
                        post[*u] = next;
                        next += 1;
                        stack.pop();
                    } else {
                        let j = if d == 1 {
                            len - 1 - *i
                        } else if d == 2 {
                            (*i + len / 2) % len
                        } else {
                            *i
                        };
                        *i += 1;
                        let v = g.edges[*u][j];
                        if !seen[v] {
                            seen[v] = true;
                            stack.push((v, 0));
                        }
                    }
                }
            }
            let mut low = post.clone();
            for &u in g.order.iter().rev() {
                for &v in &g.edges[u] {
                    low[u] = low[u].min(low[v]);
                }
            }
            labels.push(low.into_iter().zip(post).collect());
        }
        Self { labels }
    }
    /// Necessary containment condition. A passing condition requires exact search.
    #[must_use]
    pub fn may_reach(&self, u: usize, v: usize) -> bool {
        self.labels
            .iter()
            .all(|l| l[u].0 <= l[v].0 && l[v].1 <= l[u].1)
    }
    /// Logical retained interval payload bytes, excluding vector metadata.
    #[must_use]
    pub fn bytes(&self) -> usize {
        self.labels
            .iter()
            .map(|x| x.len() * 2 * size_of::<usize>())
            .sum()
    }
}

/// Exact traversal with topological rejection and optional interval pruning.
///
/// # Panics
/// Panics for endpoints outside the graph or scratch belonging to another size.
pub fn search(g: &Dag, s: &mut Scratch, u: usize, v: usize, filter: Option<&Intervals>) -> bool {
    assert!(u < g.len() && v < g.len() && s.seen.len() == g.len());
    if u == v {
        return true;
    }
    if g.rank[u] >= g.rank[v] {
        return false;
    }
    s.seen.fill(false);
    s.stack.clear();
    s.seen[u] = true;
    s.stack.push(u);
    while let Some(x) = s.stack.pop() {
        if filter.is_some_and(|f| !f.may_reach(x, v)) {
            continue;
        }
        for &y in &g.edges[x] {
            if y == v {
                return true;
            }
            if g.rank[y] < g.rank[v] && !s.seen[y] {
                s.seen[y] = true;
                s.stack.push(y);
            }
        }
    }
    false
}

/// Exact row-major bitset transitive closure, including self reachability.
pub struct Closure {
    words: usize,
    rows: Vec<u64>,
    n: usize,
}
impl Closure {
    /// Builds within a caller-supplied payload budget.
    ///
    /// # Errors
    /// Returns `budget` for overflow or excess payload and `allocation` on reserve failure.
    pub fn new(g: &Dag, budget: usize) -> Result<Self, &'static str> {
        let words = g.len().div_ceil(64);
        let count = g.len().checked_mul(words).ok_or("budget")?;
        if count.checked_mul(8).ok_or("budget")? > budget {
            return Err("budget");
        }
        let mut rows = Vec::new();
        rows.try_reserve_exact(count).map_err(|_| "allocation")?;
        rows.resize(count, 0);
        for &u in g.order.iter().rev() {
            rows[u * words + u / 64] |= 1 << (u % 64);
            for &v in &g.edges[u] {
                for j in 0..words {
                    rows[u * words + j] |= rows[v * words + j];
                }
            }
        }
        Ok(Self {
            words,
            rows,
            n: g.len(),
        })
    }
    /// Queries an exact bit.
    ///
    /// # Panics
    /// Panics for endpoints outside the indexed graph.
    #[must_use]
    pub fn reaches(&self, u: usize, v: usize) -> bool {
        assert!(u < self.n && v < self.n);
        self.rows[u * self.words + v / 64] & (1 << (v % 64)) != 0
    }
    /// Logical retained bitset payload bytes.
    #[must_use]
    pub fn bytes(&self) -> usize {
        self.rows.len() * 8
    }
}
