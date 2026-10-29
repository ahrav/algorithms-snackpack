//! Iterative strongly connected component algorithms on immutable directed graphs.
#![forbid(unsafe_code)]
use std::collections::VecDeque;

/// Directed adjacency lists. Vertices are `0..len`; loops and duplicates are allowed.
pub type Graph = Vec<Vec<usize>>;
const NONE: usize = usize::MAX;

/// Reject invalid destinations before a traversal starts.
#[must_use]
pub fn valid(g: &Graph) -> bool {
    g.iter().flatten().all(|&v| v < g.len())
}

/// Reverse every edge, retaining loops and duplicates.
#[must_use]
pub fn transpose(g: &Graph) -> Graph {
    assert!(valid(g));
    let mut r = vec![Vec::new(); g.len()];
    for (v, edges) in g.iter().enumerate() {
        for &w in edges {
            r[w].push(v);
        }
    }
    r
}

/// Tarjan with explicit DFS continuation frames and a separate unresolved stack.
///
/// Returns dense component IDs in sink-first order. IDs describe this snapshot.
/// # Panics
/// Panics when an edge destination is outside the vertex range.
#[must_use]
pub fn tarjan(g: &Graph) -> Vec<usize> {
    assert!(valid(g));
    let n = g.len();
    let mut index = vec![NONE; n];
    let mut low = vec![0; n];
    let mut id = vec![NONE; n];
    let mut active = vec![false; n];
    let mut pending = Vec::with_capacity(n);
    let mut frames: Vec<(usize, usize)> = Vec::with_capacity(n);
    let (mut clock, mut component) = (0, 0);
    for root in 0..n {
        if index[root] != NONE {
            continue;
        }
        index[root] = clock;
        low[root] = clock;
        clock += 1;
        active[root] = true;
        pending.push(root);
        frames.push((root, 0));
        while let Some(&(v, next)) = frames.last() {
            if next < g[v].len() {
                let w = g[v][next];
                frames.last_mut().unwrap().1 += 1;
                if index[w] == NONE {
                    index[w] = clock;
                    low[w] = clock;
                    clock += 1;
                    active[w] = true;
                    pending.push(w);
                    frames.push((w, 0));
                } else if active[w] {
                    low[v] = low[v].min(index[w]);
                }
            } else {
                frames.pop();
                if low[v] == index[v] {
                    loop {
                        let w = pending.pop().unwrap();
                        active[w] = false;
                        id[w] = component;
                        if w == v {
                            break;
                        }
                    }
                    component += 1;
                }
                if let Some(&(parent, _)) = frames.last() {
                    low[parent] = low[parent].min(low[v]);
                }
            }
        }
    }
    id
}

/// Kosaraju's two traversals, including construction and teardown of the transpose.
/// # Panics
/// Panics for an invalid destination.
#[must_use]
pub fn kosaraju(g: &Graph) -> Vec<usize> {
    kosaraju_prepared(g, &transpose(g))
}

/// Kosaraju with a caller-provided exact transpose from the same graph snapshot.
///
/// The caller must rebuild `reverse` after every graph edit.
/// # Panics
/// Panics for invalid destinations or unequal vertex counts.
#[must_use]
pub fn kosaraju_prepared(g: &Graph, reverse: &Graph) -> Vec<usize> {
    assert!(valid(g) && valid(reverse) && g.len() == reverse.len());
    let n = g.len();
    let mut seen = vec![false; n];
    let mut order = Vec::with_capacity(n);
    let mut frames = Vec::with_capacity(n);
    for root in 0..n {
        if seen[root] {
            continue;
        }
        seen[root] = true;
        frames.push((root, 0));
        while let Some(&(v, next)) = frames.last() {
            if next < g[v].len() {
                let w = g[v][next];
                frames.last_mut().unwrap().1 += 1;
                if !seen[w] {
                    seen[w] = true;
                    frames.push((w, 0));
                }
            } else {
                frames.pop();
                order.push(v);
            }
        }
    }
    let mut id = vec![NONE; n];
    let mut stack = Vec::with_capacity(n);
    let mut component = 0;
    for &root in order.iter().rev() {
        if id[root] != NONE {
            continue;
        }
        stack.push(root);
        id[root] = component;
        while let Some(v) = stack.pop() {
            for &w in &reverse[v] {
                if id[w] == NONE {
                    id[w] = component;
                    stack.push(w);
                }
            }
        }
        component += 1;
    }
    id
}

/// Gabow's path-based algorithm with unresolved and candidate-root stacks.
/// # Panics
/// Panics for an invalid destination.
#[must_use]
pub fn gabow(g: &Graph) -> Vec<usize> {
    assert!(valid(g));
    let n = g.len();
    let mut pre = vec![NONE; n];
    let mut id = vec![NONE; n];
    let mut pending = Vec::with_capacity(n);
    let mut roots = Vec::with_capacity(n);
    let mut frames = Vec::with_capacity(n);
    let (mut clock, mut component) = (0, 0);
    for root in 0..n {
        if pre[root] != NONE {
            continue;
        }
        pre[root] = clock;
        clock += 1;
        pending.push(root);
        roots.push(root);
        frames.push((root, 0));
        while let Some(&(v, next)) = frames.last() {
            if next < g[v].len() {
                let w = g[v][next];
                frames.last_mut().unwrap().1 += 1;
                if pre[w] == NONE {
                    pre[w] = clock;
                    clock += 1;
                    pending.push(w);
                    roots.push(w);
                    frames.push((w, 0));
                } else if id[w] == NONE {
                    while pre[*roots.last().unwrap()] > pre[w] {
                        roots.pop();
                    }
                }
            } else {
                frames.pop();
                if roots.last() == Some(&v) {
                    roots.pop();
                    loop {
                        let w = pending.pop().unwrap();
                        id[w] = component;
                        if w == v {
                            break;
                        }
                    }
                    component += 1;
                }
            }
        }
    }
    id
}

/// Independent small-graph oracle using repeated breadth-first reachability.
/// Labels are the minimum vertex in each mutual-reachability class.
#[must_use]
pub fn oracle(g: &Graph) -> Vec<usize> {
    assert!(valid(g));
    let n = g.len();
    let mut reach = vec![vec![false; n]; n];
    for (root, row) in reach.iter_mut().enumerate() {
        row[root] = true;
        let mut q = VecDeque::from([root]);
        while let Some(v) = q.pop_front() {
            for &w in &g[v] {
                if !row[w] {
                    row[w] = true;
                    q.push_back(w);
                }
            }
        }
    }
    (0..n)
        .map(|v| (0..n).find(|&w| reach[v][w] && reach[w][v]).unwrap())
        .collect()
}

/// Canonicalize arbitrary component labels to their minimum member.
#[must_use]
pub fn canonical(id: &[usize]) -> Vec<usize> {
    let mut minima = vec![NONE; id.len()];
    for (v, &c) in id.iter().enumerate() {
        minima[c] = minima[c].min(v);
    }
    id.iter().map(|&c| minima[c]).collect()
}

/// Certify a full partition independently: each group is strongly connected,
/// and the graph of groups is acyclic. Returns false for malformed dense IDs.
#[must_use]
pub fn certify(g: &Graph, id: &[usize]) -> bool {
    let n = g.len();
    if !valid(g) || id.len() != n || id.iter().any(|&c| c >= n) {
        return false;
    }
    if n == 0 {
        return true;
    }
    let k = *id.iter().max().unwrap() + 1;
    let mut groups = vec![Vec::new(); k];
    for (v, &c) in id.iter().enumerate() {
        groups[c].push(v);
    }
    if groups.iter().any(Vec::is_empty) {
        return false;
    }
    let rev = transpose(g);
    for edges in [g, &rev] {
        let mut seen = vec![false; n];
        for (c, group) in groups.iter().enumerate() {
            let mut stack = vec![group[0]];
            seen[group[0]] = true;
            let mut count = 0;
            while let Some(v) = stack.pop() {
                count += 1;
                for &w in &edges[v] {
                    if id[w] == c && !seen[w] {
                        seen[w] = true;
                        stack.push(w);
                    }
                }
            }
            if count != group.len() {
                return false;
            }
        }
    }
    let mut degree = vec![0; k];
    let mut out = vec![Vec::new(); k];
    for (v, edges) in g.iter().enumerate() {
        for &w in edges {
            if id[v] != id[w] {
                out[id[v]].push(id[w]);
                degree[id[w]] += 1;
            }
        }
    }
    let mut ready: Vec<_> = (0..k).filter(|&c| degree[c] == 0).collect();
    let mut count = 0;
    while let Some(c) = ready.pop() {
        count += 1;
        for &d in &out[c] {
            degree[d] -= 1;
            if degree[d] == 0 {
                ready.push(d);
            }
        }
    }
    count == k
}

/// Seven-vertex example: two cyclic groups and two singleton vertices.
#[must_use]
pub fn example() -> Graph {
    vec![
        vec![1],
        vec![2],
        vec![0, 3],
        vec![4],
        vec![3, 5],
        vec![6],
        vec![],
    ]
}
