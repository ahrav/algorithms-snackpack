//! Unweighted directed graph traversal with explicit result contracts.
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

/// A validated directed adjacency list. Neighbor order is preserved.
#[derive(Clone, Debug)]
pub struct Graph {
    adj: Vec<Vec<usize>>,
}
impl Graph {
    /// Validate all endpoints. Empty graphs are allowed.
    ///
    /// # Errors
    /// Returns an error when an endpoint is outside the vertex range.
    pub fn new(adj: Vec<Vec<usize>>) -> Result<Self, &'static str> {
        if adj.iter().flatten().any(|&v| v >= adj.len()) {
            return Err("endpoint out of range");
        }
        Ok(Self { adj })
    }
    /// Number of vertices.
    #[must_use]
    pub fn len(&self) -> usize {
        self.adj.len()
    }
    /// Whether the graph is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.adj.is_empty()
    }
    /// Directed edge count, including duplicates and self loops.
    #[must_use]
    pub fn edges(&self) -> usize {
        self.adj.iter().map(Vec::len).sum()
    }
    /// Neighbors in input order.
    ///
    /// # Panics
    /// Panics when `u` is outside `0..self.len()`.
    #[must_use]
    pub fn neighbors(&self, u: usize) -> &[usize] {
        &self.adj[u]
    }
}
/// Compressed sparse row layout: one offsets array and one neighbor array.
pub struct Csr {
    offsets: Vec<usize>,
    targets: Vec<usize>,
}
impl Csr {
    /// Copy adjacency entries into contiguous storage without reordering.
    #[must_use]
    pub fn new(g: &Graph) -> Self {
        let mut offsets = vec![0];
        let mut targets = Vec::with_capacity(g.edges());
        for u in 0..g.len() {
            targets.extend_from_slice(g.neighbors(u));
            offsets.push(targets.len());
        }
        Self { offsets, targets }
    }
    fn len(&self) -> usize {
        self.offsets.len() - 1
    }
    fn neighbors(&self, u: usize) -> &[usize] {
        &self.targets[self.offsets[u]..self.offsets[u + 1]]
    }
}
/// The sentinel for an unreachable vertex.
pub const UNREACHABLE: usize = usize::MAX;
/// Queue-based breadth-first search. Invalid sources are ignored.
///
/// Each distinct valid source has distance zero. Duplicate sources, duplicate
/// edges, self loops, and cycles preserve one discovery per reached vertex.
#[must_use]
pub fn bfs_deque(g: &Graph, sources: &[usize]) -> Vec<usize> {
    let mut d = vec![UNREACHABLE; g.len()];
    let mut q = VecDeque::new();
    for &s in sources {
        if s < g.len() && d[s] == UNREACHABLE {
            d[s] = 0;
            q.push_back(s);
        }
    }
    while let Some(u) = q.pop_front() {
        for &v in g.neighbors(u) {
            if d[v] == UNREACHABLE {
                d[v] = d[u] + 1;
                q.push_back(v);
            }
        }
    }
    d
}
/// Breadth-first search using a vector and a monotonically advancing head.
#[must_use]
pub fn bfs_vec(g: &Graph, sources: &[usize]) -> Vec<usize> {
    let mut d = vec![UNREACHABLE; g.len()];
    let mut q = Vec::new();
    let mut head = 0;
    for &s in sources {
        if s < g.len() && d[s] == UNREACHABLE {
            d[s] = 0;
            q.push(s);
        }
    }
    while head < q.len() {
        let u = q[head];
        head += 1;
        for &v in g.neighbors(u) {
            if d[v] == UNREACHABLE {
                d[v] = d[u] + 1;
                q.push(v);
            }
        }
    }
    d
}
/// Vector-queue breadth-first search over compressed sparse rows.
#[must_use]
pub fn bfs_csr(g: &Csr, sources: &[usize]) -> Vec<usize> {
    let mut d = vec![UNREACHABLE; g.len()];
    let mut q = Vec::new();
    let mut head = 0;
    for &s in sources {
        if s < g.len() && d[s] == UNREACHABLE {
            d[s] = 0;
            q.push(s);
        }
    }
    while head < q.len() {
        let u = q[head];
        head += 1;
        for &v in g.neighbors(u) {
            if d[v] == UNREACHABLE {
                d[v] = d[u] + 1;
                q.push(v);
            }
        }
    }
    d
}
/// Level-synchronous breadth-first search using two frontier vectors.
#[must_use]
pub fn bfs_layers(g: &Csr, sources: &[usize]) -> Vec<usize> {
    let mut d = vec![UNREACHABLE; g.len()];
    let mut current = Vec::new();
    let mut next = Vec::new();
    for &s in sources {
        if s < g.len() && d[s] == UNREACHABLE {
            d[s] = 0;
            current.push(s);
        }
    }
    let mut level = 0;
    while !current.is_empty() {
        level += 1;
        for &u in &current {
            for &v in g.neighbors(u) {
                if d[v] == UNREACHABLE {
                    d[v] = level;
                    next.push(v);
                }
            }
        }
        current.clear();
        std::mem::swap(&mut current, &mut next);
    }
    d
}
/// Independent shortest-distance oracle using repeated minimum-key extraction.
///
/// A binary heap and relaxation permit duplicate pending entries. This oracle
/// deliberately has a different scheduling and update rule from BFS.
#[must_use]
pub fn distance_oracle(g: &Graph, sources: &[usize]) -> Vec<usize> {
    let mut d = vec![UNREACHABLE; g.len()];
    let mut heap = BinaryHeap::new();
    for &s in sources {
        if s < g.len() {
            heap.push(Reverse((0, s)));
        }
    }
    while let Some(Reverse((cost, u))) = heap.pop() {
        if cost >= d[u] {
            continue;
        }
        d[u] = cost;
        for &v in g.neighbors(u) {
            if cost + 1 < d[v] {
                heap.push(Reverse((cost + 1, v)));
            }
        }
    }
    d
}
/// Ordered DFS events from a supplied root list. Finish order records exit.
#[derive(Debug, PartialEq, Eq)]
pub struct Dfs {
    /// First-entry order, respecting root and neighbor order.
    pub preorder: Vec<usize>,
    /// Order of exits after every outgoing edge was examined.
    pub finish: Vec<usize>,
    /// Whether a directed edge reached an active ancestor.
    pub cyclic: bool,
}
/// Depth-first search with one `(vertex, next-neighbor)` frame per active vertex.
#[must_use]
pub fn dfs_frames(g: &Graph, roots: &[usize]) -> Dfs {
    let mut out = Dfs {
        preorder: Vec::new(),
        finish: Vec::new(),
        cyclic: false,
    };
    let mut color = vec![0_u8; g.len()];
    let mut stack = Vec::new();
    for &root in roots {
        if root >= g.len() || color[root] != 0 {
            continue;
        }
        color[root] = 1;
        out.preorder.push(root);
        stack.push((root, 0));
        while let Some(&(u, i)) = stack.last() {
            if i == g.neighbors(u).len() {
                stack.pop();
                color[u] = 2;
                out.finish.push(u);
                continue;
            }
            if let Some(frame) = stack.last_mut() {
                frame.1 += 1;
            }
            let v = g.neighbors(u)[i];
            if color[v] == 1 {
                out.cyclic = true;
            }
            if color[v] == 0 {
                color[v] = 1;
                out.preorder.push(v);
                stack.push((v, 0));
            }
        }
    }
    out
}
/// DFS with explicit entry/exit events. Pending events can require O(E) space.
#[must_use]
pub fn dfs_events(g: &Graph, roots: &[usize]) -> Dfs {
    let mut out = Dfs {
        preorder: Vec::new(),
        finish: Vec::new(),
        cyclic: false,
    };
    let mut color = vec![0_u8; g.len()];
    let mut stack = Vec::new();
    for &root in roots {
        if root >= g.len() || color[root] != 0 {
            continue;
        }
        stack.push((root, false));
        while let Some((u, exit)) = stack.pop() {
            if exit {
                color[u] = 2;
                out.finish.push(u);
                continue;
            }
            if color[u] == 1 {
                out.cyclic = true;
                continue;
            }
            if color[u] == 2 {
                continue;
            }
            color[u] = 1;
            out.preorder.push(u);
            stack.push((u, true));
            for &v in g.neighbors(u).iter().rev() {
                stack.push((v, false));
            }
        }
    }
    out
}
