//! Dominance for a directed graph with one designated entry.
//! Vertices outside the entry's reachable set have no dominator information.
use std::collections::VecDeque;
/// Adjacency lists, with dense vertex IDs. Duplicate edges and loops are allowed.
pub type Graph = Vec<Vec<usize>>;
/// Dense sets of dominators, indexed by vertex then by 64-bit word.
pub type Sets = Vec<Vec<u64>>;
/// Scan counts for algorithmic work. Allocation and machine events are unmeasured.
#[derive(Default, Debug, Clone, Copy)]
pub struct Work {
    /// Fixed-point passes or worklist pops.
    pub visits: usize,
    /// Predecessor edges processed.
    pub edges: usize,
    /// Word intersections or parent-chain steps.
    pub steps: usize,
}
fn reachable(g: &Graph, root: usize) -> Vec<bool> {
    let mut seen = vec![false; g.len()];
    if root < g.len() {
        seen[root] = true;
        let mut stack = vec![root];
        while let Some(v) = stack.pop() {
            for &u in &g[v] {
                if !seen[u] {
                    seen[u] = true;
                    stack.push(u);
                }
            }
        }
    }
    seen
}
fn predecessors(g: &Graph) -> Graph {
    let mut p = vec![vec![]; g.len()];
    for (u, row) in g.iter().enumerate() {
        for &v in row {
            p[v].push(u);
        }
    }
    p
}
fn solve_row(p: &Graph, d: &Sets, seen: &[bool], v: usize, work: &mut Work) -> Vec<u64> {
    let mut row = vec![u64::MAX; d[v].len()];
    for &u in &p[v] {
        if seen[u] {
            work.edges += 1;
            for (a, b) in row.iter_mut().zip(&d[u]) {
                *a &= b;
                work.steps += 1;
            }
        }
    }
    row[v / 64] |= 1 << (v % 64);
    row
}
/// Compute full sets by descending iteration from the reachable universe.
///
/// # Panics
/// Panics if any edge endpoint is outside `0..g.len()`.
#[must_use]
pub fn bitsets(g: &Graph, root: usize) -> (Sets, Work) {
    let seen = reachable(g, root);
    let p = predecessors(g);
    let mut universe = vec![0_u64; g.len().div_ceil(64)];
    for (v, &live) in seen.iter().enumerate() {
        if live {
            universe[v / 64] |= 1 << (v % 64);
        }
    }
    let mut d: Sets = seen
        .iter()
        .map(|&live| {
            if live {
                universe.clone()
            } else {
                vec![0; universe.len()]
            }
        })
        .collect();
    if root < g.len() {
        d[root].fill(0);
        d[root][root / 64] |= 1 << (root % 64);
    }
    let mut work = Work::default();
    loop {
        let mut changed = false;
        for v in 0..g.len() {
            if seen[v] && v != root {
                work.visits += 1;
                let row = solve_row(&p, &d, &seen, v, &mut work);
                if row != d[v] {
                    d[v] = row;
                    changed = true;
                }
            }
        }
        if !changed {
            return (d, work);
        }
    }
}
/// Extract immediate dominators from full sets. Root points to itself.
#[must_use]
pub fn parents(d: &Sets, root: usize) -> Vec<Option<usize>> {
    let depth: Vec<u32> = d
        .iter()
        .map(|r| r.iter().map(|w| w.count_ones()).sum())
        .collect();
    (0..d.len())
        .map(|v| {
            if depth[v] == 0 {
                return None;
            }
            if v == root {
                return Some(root);
            }
            (0..d.len())
                .filter(|&u| u != v && (d[v][u / 64] >> (u % 64)) & 1 == 1)
                .max_by_key(|&u| depth[u])
        })
        .collect()
}
fn postorder(g: &Graph, root: usize) -> Vec<usize> {
    let mut seen = vec![false; g.len()];
    let mut out = vec![];
    if root >= g.len() {
        return out;
    }
    seen[root] = true;
    let mut stack = vec![(root, 0)];
    while let Some((v, i)) = stack.pop() {
        if i == g[v].len() {
            out.push(v);
        } else {
            stack.push((v, i + 1));
            let u = g[v][i];
            if !seen[u] {
                seen[u] = true;
                stack.push((u, 0));
            }
        }
    }
    out
}
/// Compute immediate dominators with reverse-postorder parent intersection (CHK).
/// Root points to itself; unreachable vertices have `None`.
///
/// # Panics
/// Panics if any edge endpoint is outside `0..g.len()`.
#[must_use]
pub fn chk(g: &Graph, root: usize) -> (Vec<Option<usize>>, Work) {
    let order = postorder(g, root);
    let mut idom = vec![None; g.len()];
    let mut work = Work::default();
    if order.is_empty() {
        return (idom, work);
    }
    let p = predecessors(g);
    let mut rank = vec![0; g.len()];
    for (i, &v) in order.iter().enumerate() {
        rank[v] = i;
    }
    idom[root] = Some(root);
    loop {
        let mut changed = false;
        for &v in order.iter().rev().skip(1) {
            work.visits += 1;
            let mut next: Option<usize> = None;
            for &u in &p[v] {
                if idom[u].is_some() {
                    work.edges += 1;
                    next = Some(if let Some(mut a) = next {
                        let mut b = u;
                        while a != b {
                            while rank[a] < rank[b] {
                                a = idom[a].unwrap();
                                work.steps += 1;
                            }
                            while rank[b] < rank[a] {
                                b = idom[b].unwrap();
                                work.steps += 1;
                            }
                        }
                        a
                    } else {
                        u
                    });
                }
            }
            if next != idom[v] {
                idom[v] = next;
                changed = true;
            }
        }
        if !changed {
            return (idom, work);
        }
    }
}
/// Insert one edge and shrink cached dominator sets with a propagation worklist.
/// New reachability triggers a fresh greatest-fixed-point computation.
/// Caller supplies the exact sets for `g` and `root` before insertion.
///
/// # Panics
/// Panics for invalid edge endpoints or an incompatible cached set shape.
#[allow(clippy::many_single_char_names)]
pub fn insert(g: &mut Graph, root: usize, d: &mut Sets, u: usize, v: usize) -> Work {
    assert!(
        u < g.len() && v < g.len(),
        "edge endpoint outside the graph"
    );
    let seen = reachable(g, root);
    g[u].push(v);
    if !seen[u] {
        return Work::default();
    }
    if !seen[v] {
        let (next, w) = bitsets(g, root);
        *d = next;
        return w;
    }
    let p = predecessors(g);
    let mut queued = vec![false; g.len()];
    let mut q = VecDeque::from([v]);
    queued[v] = true;
    let mut work = Work::default();
    while let Some(x) = q.pop_front() {
        queued[x] = false;
        if x == root {
            continue;
        }
        work.visits += 1;
        let row = solve_row(&p, d, &seen, x, &mut work);
        if row != d[x] {
            d[x] = row;
            for &y in &g[x] {
                if !queued[y] {
                    queued[y] = true;
                    q.push_back(y);
                }
            }
        }
    }
    work
}
/// Independent oracle: remove each possible dominator and test reachability.
/// Kept outside all timing boundaries.
///
/// # Panics
/// Panics if any edge endpoint is outside `0..g.len()`.
#[must_use]
pub fn oracle(g: &Graph, root: usize) -> Sets {
    let n = g.len();
    let search = |blocked: usize| {
        let mut live = vec![false; n];
        let mut todo = VecDeque::new();
        if root < n && root != blocked {
            live[root] = true;
            todo.push_back(root);
        }
        while let Some(a) = todo.pop_front() {
            for &b in &g[a] {
                if b != blocked && !live[b] {
                    live[b] = true;
                    todo.push_back(b);
                }
            }
        }
        live
    };
    let base = search(n);
    let mut d = vec![vec![0; n.div_ceil(64)]; n];
    for blocked in 0..n {
        let live = search(blocked);
        for v in 0..n {
            if base[v] && !live[v] {
                d[v][blocked / 64] |= 1 << (blocked % 64);
            }
        }
    }
    d
}
/// Expand immediate-dominator chains into sets for independent validation.
///
/// # Panics
/// Panics on an invalid parent ID or a parent cycle longer than one vertex.
#[must_use]
pub fn expand(p: &[Option<usize>]) -> Sets {
    let mut d = vec![vec![0; p.len().div_ceil(64)]; p.len()];
    for (v, row) in d.iter_mut().enumerate() {
        let mut x = v;
        for step in 0..=p.len() {
            let Some(next) = p[x] else {
                break;
            };
            assert!(step < p.len());
            row[x / 64] |= 1 << (x % 64);
            if next == x {
                break;
            }
            x = next;
        }
    }
    d
}
/// Deterministic workload with loops, joins, unreachable vertices or dense edges.
///
/// # Panics
/// Panics if `shape` is unknown and `n` is nonzero.
#[must_use]
pub fn workload(shape: &str, n: usize) -> Graph {
    let mut g = vec![vec![]; n];
    if n == 0 {
        return g;
    }
    for (u, row) in g.iter_mut().enumerate().take(n - 1) {
        row.push(u + 1);
    }
    match shape {
        "chain" | "tiny" | "noop" | "bypass" => {}
        "joins" => {
            for (u, row) in g.iter_mut().enumerate().take(n.saturating_sub(3)) {
                row.push(u + 3);
            }
        }
        "cycle" => {
            g[n - 1].push(0);
            for (u, row) in g.iter_mut().enumerate().skip(2) {
                row.push(u - 2);
            }
        }
        "unreachable" => {
            if n > 1 {
                g[n / 2 - 1].clear();
            }
        }
        "dense" => {
            for (u, row) in g.iter_mut().enumerate() {
                for v in 0..n {
                    if (u * 31 + v * 17) % 7 == 0 {
                        row.push(v);
                    }
                }
            }
        }
        "permuted" => {
            let old = g;
            g = vec![vec![]; n];
            let map = |v| if v == 0 { 0 } else { n - v };
            for u in 0..n {
                for &v in &old[u] {
                    g[map(u)].push(map(v));
                }
            }
        }
        _ => panic!("unknown shape"),
    }
    g
}
