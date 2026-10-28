//! Topological order and pure, wrapping-u64 dependency evaluation.
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

/// Rejected graph or update input. Returned errors preserve published state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// An endpoint or edited vertex is outside the graph.
    Vertex,
    /// A repeated edge violates the simple-graph contract.
    Duplicate,
    /// The directed graph contains a cycle.
    Cycle,
    /// The base vector has a different length from the graph.
    Length,
}

/// Ordering strategy. Each returns a complete valid order or a cycle error.
#[derive(Clone, Copy, Debug)]
pub enum Ordering {
    /// Kahn's algorithm with a first-in, first-out ready queue.
    Fifo,
    /// Kahn's algorithm with smallest-ID-first readiness.
    Minimum,
    /// Iterative depth-first search and reverse finish order.
    Dfs,
}

/// A simple directed acyclic graph with dense vertex IDs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Graph {
    outgoing: Vec<Vec<usize>>,
    incoming: Vec<Vec<usize>>,
    order: Vec<usize>,
    rank: Vec<usize>,
}
impl Graph {
    /// Validate endpoints, duplicate edges and acyclicity.
    ///
    /// # Errors
    /// Returns [`Error::Vertex`], [`Error::Duplicate`] or [`Error::Cycle`].
    pub fn new(n: usize, edges: &[(usize, usize)]) -> Result<Self, Error> {
        let mut g = Self {
            outgoing: vec![vec![]; n],
            incoming: vec![vec![]; n],
            order: vec![],
            rank: vec![0; n],
        };
        for &(u, v) in edges {
            if u >= n || v >= n {
                return Err(Error::Vertex);
            }
            if g.outgoing[u].contains(&v) {
                return Err(Error::Duplicate);
            }
            g.outgoing[u].push(v);
            g.incoming[v].push(u);
        }
        g.order = g.topological(Ordering::Fifo)?;
        g.reindex();
        Ok(g)
    }
    /// Number of vertices.
    #[must_use]
    pub fn len(&self) -> usize {
        self.outgoing.len()
    }
    /// Whether the graph has no vertices.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Stored valid dependency order.
    #[must_use]
    pub fn order(&self) -> &[usize] {
        &self.order
    }
    fn reindex(&mut self) {
        for (i, &u) in self.order.iter().enumerate() {
            self.rank[u] = i;
        }
    }
    /// Allocate and compute an order using the selected strategy.
    ///
    /// # Errors
    /// Returns [`Error::Cycle`] when an order cannot include every vertex.
    pub fn topological(&self, strategy: Ordering) -> Result<Vec<usize>, Error> {
        let n = self.len();
        let mut result = Vec::with_capacity(n);
        if matches!(strategy, Ordering::Dfs) {
            let mut color = vec![0_u8; n];
            let mut stack = Vec::new();
            for root in 0..n {
                if color[root] != 0 {
                    continue;
                }
                color[root] = 1;
                stack.push((root, 0));
                while let Some((u, next)) = stack.last_mut() {
                    if *next == self.outgoing[*u].len() {
                        color[*u] = 2;
                        result.push(*u);
                        stack.pop();
                    } else {
                        let v = self.outgoing[*u][*next];
                        *next += 1;
                        if color[v] == 1 {
                            return Err(Error::Cycle);
                        }
                        if color[v] == 0 {
                            color[v] = 1;
                            stack.push((v, 0));
                        }
                    }
                }
            }
            result.reverse();
        } else {
            let mut degree: Vec<_> = self.incoming.iter().map(Vec::len).collect();
            let mut fifo = VecDeque::new();
            let mut minimum = BinaryHeap::new();
            for (u, &d) in degree.iter().enumerate() {
                if d == 0 {
                    if matches!(strategy, Ordering::Minimum) {
                        minimum.push(Reverse(u));
                    } else {
                        fifo.push_back(u);
                    }
                }
            }
            loop {
                let next = if matches!(strategy, Ordering::Minimum) {
                    minimum.pop().map(|Reverse(u)| u)
                } else {
                    fifo.pop_front()
                };
                let Some(u) = next else {
                    break;
                };
                result.push(u);
                for &v in &self.outgoing[u] {
                    degree[v] -= 1;
                    if degree[v] == 0 {
                        if matches!(strategy, Ordering::Minimum) {
                            minimum.push(Reverse(v));
                        } else {
                            fifo.push_back(v);
                        }
                    }
                }
            }
        }
        if result.len() == n {
            Ok(result)
        } else {
            Err(Error::Cycle)
        }
    }
    /// Insert one edge atomically. Forward edges preserve order; backward edges rebuild it.
    ///
    /// # Errors
    /// Returns an endpoint, duplicate or cycle error while preserving the graph.
    pub fn insert(&mut self, u: usize, v: usize) -> Result<(), Error> {
        if u >= self.len() || v >= self.len() {
            return Err(Error::Vertex);
        }
        if self.outgoing[u].contains(&v) {
            return Err(Error::Duplicate);
        }
        if self.rank[u] < self.rank[v] {
            self.outgoing[u].push(v);
            self.incoming[v].push(u);
        } else {
            let mut candidate = self.clone();
            candidate.outgoing[u].push(v);
            candidate.incoming[v].push(u);
            candidate.order = candidate.topological(Ordering::Fifo)?;
            candidate.reindex();
            *self = candidate;
        }
        Ok(())
    }
    fn value(&self, u: usize, base: &[u64], values: &[u64]) -> u64 {
        self.incoming[u]
            .iter()
            .fold(base[u], |sum, &p| sum.wrapping_add(values[p]))
    }
}

/// Fixed-graph value-update algorithm.
#[derive(Clone, Copy, Debug)]
pub enum Update {
    /// Recompute every value in stored order.
    Full,
    /// Mark every descendant and scan the complete order.
    Dirty,
    /// Recompute by topological position and propagate changed values.
    Heap,
}

/// Algorithmic work for one update batch. Timing samples keep these counts separately.
#[derive(Default, Clone, Copy, Debug)]
pub struct Work {
    /// Vertices evaluated.
    pub evaluated: usize,
    /// Incoming contributions read during evaluation.
    pub parents: usize,
    /// Outgoing edges inspected for discovery or propagation.
    pub scheduled_edges: usize,
    /// Entries inspected in the stored order.
    pub order_entries: usize,
}

/// Cached evaluator. All scheduling buffers are retained across updates.
#[derive(Clone, Debug)]
pub struct Evaluator {
    graph: Graph,
    base: Vec<u64>,
    values: Vec<u64>,
    marked: Vec<bool>,
    pending: BinaryHeap<Reverse<(usize, usize)>>,
    discovery: Vec<usize>,
}
impl Evaluator {
    /// Initialize cached values with a complete dependency pass.
    ///
    /// # Errors
    /// Returns [`Error::Length`] for a base-vector length mismatch.
    pub fn new(graph: Graph, base: Vec<u64>) -> Result<Self, Error> {
        let n = graph.len();
        if base.len() != n {
            return Err(Error::Length);
        }
        let mut values = vec![0; n];
        for &u in graph.order() {
            values[u] = graph.value(u, &base, &values);
        }
        Ok(Self {
            graph,
            base,
            values,
            marked: vec![false; n],
            pending: BinaryHeap::with_capacity(n),
            discovery: Vec::with_capacity(n),
        })
    }
    /// Current complete value vector.
    #[must_use]
    pub fn values(&self) -> &[u64] {
        &self.values
    }
    /// Current complete base vector.
    #[must_use]
    pub fn bases(&self) -> &[u64] {
        &self.base
    }
    /// Apply a batch atomically with respect to returned input errors.
    /// Repeated vertex assignments use the last value in the batch.
    /// An empty batch performs zero work. Allocation failure follows Rust's allocator policy.
    ///
    /// # Errors
    /// Returns [`Error::Vertex`] before changing any base or value.
    pub fn update(&mut self, edits: &[(usize, u64)], strategy: Update) -> Result<Work, Error> {
        if edits.iter().any(|&(u, _)| u >= self.graph.len()) {
            return Err(Error::Vertex);
        }
        let mut work = Work::default();
        if edits.is_empty() {
            return Ok(work);
        }
        for &(u, x) in edits {
            self.base[u] = x;
        }
        match strategy {
            Update::Full => {
                for &u in &self.graph.order {
                    self.values[u] = self.graph.value(u, &self.base, &self.values);
                    work.evaluated += 1;
                    work.parents += self.graph.incoming[u].len();
                    work.order_entries += 1;
                }
            }
            Update::Dirty => {
                self.discovery.clear();
                for &(u, _) in edits {
                    if !self.marked[u] {
                        self.marked[u] = true;
                        self.discovery.push(u);
                    }
                }
                let mut at = 0;
                while at < self.discovery.len() {
                    let u = self.discovery[at];
                    at += 1;
                    for &v in &self.graph.outgoing[u] {
                        work.scheduled_edges += 1;
                        if !self.marked[v] {
                            self.marked[v] = true;
                            self.discovery.push(v);
                        }
                    }
                }
                for &u in &self.graph.order {
                    work.order_entries += 1;
                    if self.marked[u] {
                        self.values[u] = self.graph.value(u, &self.base, &self.values);
                        work.evaluated += 1;
                        work.parents += self.graph.incoming[u].len();
                        self.marked[u] = false;
                    }
                }
            }
            Update::Heap => {
                for &(u, _) in edits {
                    if !self.marked[u] {
                        self.marked[u] = true;
                        self.pending.push(Reverse((self.graph.rank[u], u)));
                    }
                }
                while let Some(Reverse((_, u))) = self.pending.pop() {
                    self.marked[u] = false;
                    work.evaluated += 1;
                    work.parents += self.graph.incoming[u].len();
                    let next = self.graph.value(u, &self.base, &self.values);
                    if next != self.values[u] {
                        self.values[u] = next;
                        for &v in &self.graph.outgoing[u] {
                            work.scheduled_edges += 1;
                            if !self.marked[v] {
                                self.marked[v] = true;
                                self.pending.push(Reverse((self.graph.rank[v], v)));
                            }
                        }
                    }
                }
            }
        }
        Ok(work)
    }
    /// Insert an edge and refresh values after successful graph validation.
    ///
    /// # Errors
    /// Returns the graph admission error without changing graph, bases or values.
    pub fn insert(&mut self, u: usize, v: usize) -> Result<(), Error> {
        self.graph.insert(u, v)?;
        for &x in &self.graph.order {
            self.values[x] = self.graph.value(x, &self.base, &self.values);
        }
        Ok(())
    }
}

/// Independently evaluate a raw DAG using memoized predecessor traversal.
/// Uses its own dependency lists, traversal state and u128 modular accumulation.
///
/// # Panics
/// Panics for malformed endpoints or a cycle. Intended as a test/experiment oracle.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn oracle(n: usize, edges: &[(usize, usize)], bases: &[u64]) -> Vec<u64> {
    assert_eq!(n, bases.len());
    let mut dependencies = vec![vec![]; n];
    for &(from, to) in edges {
        assert!(from < n && to < n);
        dependencies[to].push(from);
    }
    let mut cached = vec![None; n];
    let mut active = vec![false; n];
    for root in 0..n {
        if cached[root].is_some() {
            continue;
        }
        let mut frames = vec![(root, 0)];
        active[root] = true;
        while let Some((node, offset)) = frames.last_mut() {
            if *offset < dependencies[*node].len() {
                let dependency = dependencies[*node][*offset];
                *offset += 1;
                if cached[dependency].is_none() {
                    assert!(!active[dependency], "oracle cycle");
                    active[dependency] = true;
                    frames.push((dependency, 0));
                }
            } else {
                let mut result = u128::from(bases[*node]);
                for &dependency in &dependencies[*node] {
                    result += u128::from(cached[dependency].unwrap());
                }
                cached[*node] = Some(result as u64);
                active[*node] = false;
                frames.pop();
            }
        }
    }
    cached.into_iter().map(Option::unwrap).collect()
}
