//! Single-source shortest paths with exact nonnegative integer distances.
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

/// Unreachable distance. Admission keeps every finite distance below this value.
pub const INF: u64 = u64::MAX;
/// Graph or queue admission failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The graph exceeds the admitted vertex count.
    TooManyVertices,
    /// An edge endpoint or source is outside the graph.
    InvalidVertex,
    /// The specialist queue requires a smaller edge-weight domain.
    WeightDomain,
}
/// Immutable adjacency lists. Parallel edges, zero weights and loops are admitted.
#[derive(Debug)]
pub struct Graph {
    edges: Vec<Vec<(usize, u32)>>,
    max_weight: u32,
}
impl Graph {
    /// Admit at most `u32::MAX` vertices and validate every endpoint once.
    ///
    /// # Errors
    /// Returns an admission error before exposing an invalid graph.
    pub fn new(n: usize, edges: &[(usize, usize, u32)]) -> Result<Self, Error> {
        if n > u32::MAX as usize {
            return Err(Error::TooManyVertices);
        }
        let mut rows = vec![Vec::new(); n];
        let mut max_weight = 0;
        for &(u, v, w) in edges {
            if u >= n || v >= n {
                return Err(Error::InvalidVertex);
            }
            rows[u].push((v, w));
            max_weight = max_weight.max(w);
        }
        Ok(Self {
            edges: rows,
            max_weight,
        })
    }
    /// Number of vertices, including isolated vertices.
    #[must_use]
    pub fn len(&self) -> usize {
        self.edges.len()
    }
    /// Whether the graph has zero vertices.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }
    /// Number of directed edges, counting duplicates.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.iter().map(Vec::len).sum()
    }
}
/// Queue portfolio. Specialist domains are checked before query allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Queue {
    /// Linear scan of all unsettled labels.
    Scan,
    /// Standard binary heap with duplicate entries and stale-pop rejection.
    Lazy,
    /// Binary heap with one live entry per vertex and a position map.
    Indexed,
    /// Monotone 65-bucket radix heap, with duplicate entries.
    Radix,
    /// Circular buckets, admitted only for maximum edge weight at most 64.
    Dial,
    /// Double-ended queue, admitted only for weights zero and one.
    ZeroOne,
}
/// Observable work counts. Bookkeeping is included in timed queries.
#[derive(Debug, Default, Clone, Copy)]
pub struct Stats {
    /// Queue removals, including stale entries; scan counts settled vertices.
    pub pops: usize,
    /// Removed entries whose label had already improved.
    pub stale: usize,
    /// Strictly improving edge relaxations.
    pub improved: usize,
    /// Maximum number of live queue records; scan uses zero queue records.
    pub peak: usize,
}
/// Complete distance vector and queue work counters.
#[derive(Debug)]
pub struct Run {
    /// `INF` marks unreachable vertices. All other values are exact costs.
    pub distances: Vec<u64>,
    /// Work counters for this query.
    pub stats: Stats,
}
/// Compute every source distance on an admitted immutable graph.
///
/// Vertices are finalized on minimum extraction. Only strict improvements enqueue.
/// A simple path uses at most `n-1` edges; one relaxation uses at most `n`
/// `u32` weights. Since `n <= u32::MAX`, this sum is strictly below `u64::MAX`.
/// Negative weights require a different input type and algorithm.
///
/// ```
/// use shortest_path_queues::{Graph, Queue, shortest_paths};
/// let graph = Graph::new(3, &[(0, 1, 10), (0, 2, 1), (2, 1, 1)]).unwrap();
/// assert_eq!(shortest_paths(&graph, 0, Queue::Lazy).unwrap().distances, [0, 2, 1]);
/// ```
///
/// # Errors
/// Returns [`Error::InvalidVertex`] for an invalid source, including an empty graph.
/// Returns [`Error::WeightDomain`] when a specialist's weight precondition fails.
pub fn shortest_paths(g: &Graph, source: usize, queue: Queue) -> Result<Run, Error> {
    if source >= g.len() {
        return Err(Error::InvalidVertex);
    }
    if (queue == Queue::Dial && g.max_weight > 64) || (queue == Queue::ZeroOne && g.max_weight > 1)
    {
        return Err(Error::WeightDomain);
    }
    let mut d = vec![INF; g.len()];
    d[source] = 0;
    let mut stats = Stats::default();
    if queue == Queue::Scan {
        let mut done = vec![false; g.len()];
        for _ in 0..g.len() {
            let Some(u) = (0..g.len())
                .filter(|&i| !done[i] && d[i] != INF)
                .min_by_key(|&i| (d[i], i))
            else {
                break;
            };
            done[u] = true;
            stats.pops += 1;
            for &(v, w) in &g.edges[u] {
                let next = d[u] + u64::from(w);
                if next < d[v] {
                    d[v] = next;
                    stats.improved += 1;
                }
            }
        }
    } else {
        let mut q = Worklist::new(queue, g.len(), g.max_weight);
        q.push(0, source);
        stats.peak = 1;
        while let Some((cost, u)) = q.pop() {
            stats.pops += 1;
            if cost != d[u] {
                stats.stale += 1;
                continue;
            }
            for &(v, w) in &g.edges[u] {
                let next = cost + u64::from(w);
                if next < d[v] {
                    d[v] = next;
                    stats.improved += 1;
                    q.push(next, v);
                    stats.peak = stats.peak.max(q.len());
                }
            }
        }
    }
    Ok(Run {
        distances: d,
        stats,
    })
}

struct Indexed {
    heap: Vec<(u64, usize)>,
    pos: Vec<usize>,
}
impl Indexed {
    fn exchange(&mut self, a: usize, b: usize) {
        self.heap.swap(a, b);
        self.pos[self.heap[a].1] = a;
        self.pos[self.heap[b].1] = b;
    }
    fn push(&mut self, key: u64, v: usize) {
        let mut i = self.pos[v];
        if i == usize::MAX {
            i = self.heap.len();
            self.heap.push((key, v));
            self.pos[v] = i;
        } else {
            self.heap[i].0 = key;
        }
        while i > 0 {
            let p = (i - 1) / 2;
            if self.heap[p] <= self.heap[i] {
                break;
            }
            self.exchange(p, i);
            i = p;
        }
    }
    fn pop(&mut self) -> Option<(u64, usize)> {
        if self.heap.is_empty() {
            return None;
        }
        let last = self.heap.len() - 1;
        self.exchange(0, last);
        let result = self.heap.pop()?;
        self.pos[result.1] = usize::MAX;
        let mut i = 0;
        loop {
            let left = 2 * i + 1;
            if left >= self.heap.len() {
                break;
            }
            let right = left + 1;
            let child = if right < self.heap.len() && self.heap[right] < self.heap[left] {
                right
            } else {
                left
            };
            if self.heap[i] <= self.heap[child] {
                break;
            }
            self.exchange(i, child);
            i = child;
        }
        Some(result)
    }
}
struct Radix {
    buckets: Vec<Vec<(u64, usize)>>,
    last: u64,
    size: usize,
}
impl Radix {
    fn bucket(key: u64, last: u64) -> usize {
        (64 - (key ^ last).leading_zeros()) as usize
    }
    fn push(&mut self, key: u64, v: usize) {
        assert!(key >= self.last);
        self.buckets[Self::bucket(key, self.last)].push((key, v));
        self.size += 1;
    }
    fn pop(&mut self) -> Option<(u64, usize)> {
        if self.size == 0 {
            return None;
        }
        if self.buckets[0].is_empty() {
            let i = (1..65).find(|&i| !self.buckets[i].is_empty())?;
            self.last = self.buckets[i].iter().map(|x| x.0).min()?;
            let old = std::mem::take(&mut self.buckets[i]);
            for item in old {
                self.buckets[Self::bucket(item.0, self.last)].push(item);
            }
        }
        self.size -= 1;
        self.buckets[0].pop()
    }
}
struct Dial {
    buckets: Vec<Vec<(u64, usize)>>,
    current: u64,
    size: usize,
}
impl Dial {
    fn push(&mut self, key: u64, v: usize) {
        let i = usize::try_from(key % u64::try_from(self.buckets.len()).unwrap()).unwrap();
        self.buckets[i].push((key, v));
        self.size += 1;
    }
    fn pop(&mut self) -> Option<(u64, usize)> {
        if self.size == 0 {
            return None;
        }
        loop {
            let i =
                usize::try_from(self.current % u64::try_from(self.buckets.len()).unwrap()).unwrap();
            if let Some(item) = self.buckets[i].pop() {
                assert_eq!(item.0, self.current);
                self.size -= 1;
                return Some(item);
            }
            self.current += 1;
        }
    }
}
enum Worklist {
    Lazy(BinaryHeap<Reverse<(u64, usize)>>),
    Indexed(Indexed),
    Radix(Radix),
    Dial(Dial),
    ZeroOne(VecDeque<(u64, usize)>, u64),
}
impl Worklist {
    fn new(q: Queue, n: usize, max: u32) -> Self {
        match q {
            Queue::Lazy => Self::Lazy(BinaryHeap::new()),
            Queue::Indexed => Self::Indexed(Indexed {
                heap: Vec::new(),
                pos: vec![usize::MAX; n],
            }),
            Queue::Radix => Self::Radix(Radix {
                buckets: vec![Vec::new(); 65],
                last: 0,
                size: 0,
            }),
            Queue::Dial => Self::Dial(Dial {
                buckets: vec![Vec::new(); max as usize + 1],
                current: 0,
                size: 0,
            }),
            Queue::ZeroOne => Self::ZeroOne(VecDeque::new(), 0),
            Queue::Scan => unreachable!(),
        }
    }
    fn push(&mut self, key: u64, v: usize) {
        match self {
            Self::Lazy(q) => q.push(Reverse((key, v))),
            Self::Indexed(q) => q.push(key, v),
            Self::Radix(q) => q.push(key, v),
            Self::Dial(q) => q.push(key, v),
            Self::ZeroOne(q, current) => {
                if key == *current {
                    q.push_front((key, v));
                } else {
                    q.push_back((key, v));
                }
            }
        }
    }
    fn pop(&mut self) -> Option<(u64, usize)> {
        match self {
            Self::Lazy(q) => q.pop().map(|x| x.0),
            Self::Indexed(q) => q.pop(),
            Self::Radix(q) => q.pop(),
            Self::Dial(q) => q.pop(),
            Self::ZeroOne(q, current) => {
                let item = q.pop_front()?;
                *current = item.0;
                Some(item)
            }
        }
    }
    fn len(&self) -> usize {
        match self {
            Self::Lazy(q) => q.len(),
            Self::Indexed(q) => q.heap.len(),
            Self::Radix(q) => q.size,
            Self::Dial(q) => q.size,
            Self::ZeroOne(q, _) => q.len(),
        }
    }
}
/// Independent synchronous Bellman-Ford oracle, using the previous round's labels.
///
/// # Errors
/// Returns [`Error::InvalidVertex`] when the source is outside the graph.
///
/// # Panics
/// Panics if a finite oracle distance exceeds `u64`, which graph admission excludes.
pub fn oracle(g: &Graph, source: usize) -> Result<Vec<u64>, Error> {
    if source >= g.len() {
        return Err(Error::InvalidVertex);
    }
    let mut old = vec![None::<u128>; g.len()];
    old[source] = Some(0);
    for _ in 1..g.len() {
        let mut next = old.clone();
        for (u, row) in g.edges.iter().enumerate() {
            if let Some(cost) = old[u] {
                for &(v, w) in row {
                    let value = cost + u128::from(w);
                    if next[v].is_none_or(|x| value < x) {
                        next[v] = Some(value);
                    }
                }
            }
        }
        if old == next {
            break;
        }
        old = next;
    }
    Ok(old
        .into_iter()
        .map(|x| x.map_or(INF, |v| u64::try_from(v).unwrap()))
        .collect())
}

#[cfg(test)]
mod queue_tests {
    use super::{Indexed, Radix};
    #[test]
    fn full_width_radix_keys_and_equal_minimum() {
        let mut q = Radix {
            buckets: vec![Vec::new(); 65],
            last: 0,
            size: 0,
        };
        for (v, key) in [u64::MAX, 1_u64 << 63, 0, 1, 1_u64 << 32]
            .into_iter()
            .enumerate()
        {
            q.push(key, v);
        }
        assert_eq!(q.pop().unwrap().0, 0);
        q.push(0, 9);
        let mut got = Vec::new();
        while let Some((key, _)) = q.pop() {
            got.push(key);
        }
        assert_eq!(got, [0, 1, 1_u64 << 32, 1_u64 << 63, u64::MAX]);
    }
    #[test]
    fn indexed_position_map_tracks_swaps_and_decreases() {
        let mut q = Indexed {
            heap: Vec::new(),
            pos: vec![usize::MAX; 50],
        };
        for v in 0..50 {
            q.push(100 + u64::try_from(v).unwrap(), v);
        }
        for v in (0..50).rev() {
            q.push(u64::try_from(v).unwrap(), v);
        }
        for v in 0..50 {
            assert_eq!(q.pop(), Some((u64::try_from(v).unwrap(), v)));
            for (i, &(_, node)) in q.heap.iter().enumerate() {
                assert_eq!(q.pos[node], i);
            }
        }
        assert!(q.pop().is_none());
    }
}
