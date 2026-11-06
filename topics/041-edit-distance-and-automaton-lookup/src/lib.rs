//! Global unit-cost byte edit distance and complete dictionary lookup.
//!
//! Insertion, deletion and substitution cost one. Duplicate dictionary entries
//! retain separate IDs. Results contain every qualifying ID in input order.

/// Exact global Levenshtein distance with one reusable row.
#[must_use]
pub fn distance(query: &[u8], term: &[u8]) -> usize {
    let mut row: Vec<usize> = (0..=query.len()).collect();
    for (i, &byte) in term.iter().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, &q) in query.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = (above + 1)
                .min(row[j] + 1)
                .min(diagonal + usize::from(q != byte));
            diagonal = above;
        }
    }
    row[query.len()]
}

/// Thresholded global distance. Returns the exact distance through `k`.
///
/// Each row computes only columns within `k` of its diagonal. Values outside
/// that interval are reset to `k + 1`, preventing stale values from reentering.
#[must_use]
pub fn bounded(query: &[u8], term: &[u8], k: usize) -> Option<usize> {
    if query.len().abs_diff(term.len()) > k {
        return None;
    }
    if k >= query.len().max(term.len()) {
        return Some(distance(query, term));
    }
    let infinity = k + 1;
    let m = query.len();
    let mut row: Vec<usize> = (0..=m).map(|j| j.min(infinity)).collect();
    let mut old_hi = m.min(k);
    for (i0, &byte) in term.iter().enumerate() {
        let i = i0 + 1;
        let lo = i.saturating_sub(k).max(1);
        let hi = m.min(i.saturating_add(k));
        let mut diagonal = row[lo - 1];
        row[0] = i.min(infinity);
        if lo > 1 {
            row[lo - 1] = infinity;
        }
        let mut minimum = row[0];
        for j in lo..=hi {
            let above = if j <= old_hi { row[j] } else { infinity };
            row[j] = (above + 1)
                .min(row[j - 1] + 1)
                .min(diagonal + usize::from(query[j - 1] != byte))
                .min(infinity);
            minimum = minimum.min(row[j]);
            diagonal = above;
        }
        old_hi = hi;
        if minimum > k {
            return None;
        }
    }
    (row[m] <= k).then_some(row[m])
}

/// Prepared one-word global Myers matcher; patterns longer than 64 use rows.
#[derive(Clone)]
pub struct Myers {
    query: Vec<u8>,
    equality: [u64; 256],
}

impl Myers {
    /// Build byte equality masks once per query.
    #[must_use]
    pub fn new(query: &[u8]) -> Self {
        let mut equality = [0; 256];
        if query.len() <= 64 {
            for (i, &byte) in query.iter().enumerate() {
                equality[usize::from(byte)] |= 1_u64 << i;
            }
        }
        Self {
            query: query.to_vec(),
            equality,
        }
    }

    /// Exact global distance. The injected low bit charges for the text prefix.
    #[must_use]
    pub fn distance(&self, term: &[u8]) -> usize {
        let m = self.query.len();
        if m == 0 {
            return term.len();
        }
        if m > 64 {
            return distance(&self.query, term);
        }
        let high = 1_u64 << (m - 1);
        let (mut pv, mut mv, mut score) = (!0_u64, 0_u64, m);
        for &byte in term {
            let eq = self.equality[usize::from(byte)];
            let xv = eq | mv;
            let xh = ((eq & pv).wrapping_add(pv) ^ pv) | eq;
            let mut ph = mv | !(xh | pv);
            let mut mh = pv & xh;
            if ph & high != 0 {
                score += 1;
            }
            if mh & high != 0 {
                score -= 1;
            }
            ph = (ph << 1) | 1;
            mh <<= 1;
            pv = mh | !(xv | ph);
            mv = ph & xv;
        }
        score
    }
}

#[derive(Default)]
struct Node {
    edges: Vec<(u8, usize)>,
    ids: Vec<usize>,
}

/// Prefix trie with a dynamic-programming row as its automaton state.
#[derive(Default)]
pub struct Trie {
    nodes: Vec<Node>,
}

impl Trie {
    /// Build an immutable index. Duplicate terms retain their input IDs.
    #[must_use]
    pub fn new(terms: &[Vec<u8>]) -> Self {
        let mut nodes = vec![Node::default()];
        for (id, term) in terms.iter().enumerate() {
            let mut at = 0;
            for &byte in term {
                let next = nodes[at]
                    .edges
                    .iter()
                    .find(|&&(b, _)| b == byte)
                    .map(|&(_, n)| n);
                at = if let Some(n) = next {
                    n
                } else {
                    let n = nodes.len();
                    nodes.push(Node::default());
                    nodes[at].edges.push((byte, n));
                    n
                };
            }
            nodes[at].ids.push(id);
        }
        Self { nodes }
    }

    /// Return every matching ID and the number of evaluated trie edges.
    ///
    /// A row minimum exceeding `k` rejects the entire descendant subtree.
    /// Iterative traversal bounds stack use independently of term length.
    #[must_use]
    pub fn search(&self, query: &[u8], k: usize) -> (Vec<usize>, usize) {
        if self.nodes.is_empty() {
            return (Vec::new(), 0);
        }
        let mut todo = vec![(0, (0..=query.len()).collect::<Vec<_>>())];
        let (mut out, mut visited) = (Vec::new(), 0);
        while let Some((at, row)) = todo.pop() {
            if row[query.len()] <= k {
                out.extend_from_slice(&self.nodes[at].ids);
            }
            for &(byte, child) in &self.nodes[at].edges {
                visited += 1;
                let mut next = vec![row[0] + 1];
                for (j, &q) in query.iter().enumerate() {
                    next.push(
                        (next[j] + 1)
                            .min(row[j + 1] + 1)
                            .min(row[j] + usize::from(q != byte)),
                    );
                }
                if next.iter().copied().min().unwrap_or(0) <= k {
                    todo.push((child, next));
                }
            }
        }
        out.sort_unstable();
        (out, visited)
    }

    /// Allocated node count, including the root.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

/// Scan a dictionary using `row`, `band`, or `myers`, retaining all IDs.
///
/// Each method prepares only its own state before the scan, so a timed scan
/// measures the chosen method alone.
#[must_use]
pub fn scan(terms: &[Vec<u8>], query: &[u8], k: usize, method: &str) -> Vec<usize> {
    match method {
        "band" => retain(terms, |term| bounded(query, term, k).is_some()),
        "myers" => {
            let myers = Myers::new(query);
            retain(terms, |term| myers.distance(term) <= k)
        }
        _ => retain(terms, |term| distance(query, term) <= k),
    }
}

fn retain(terms: &[Vec<u8>], keep: impl Fn(&[u8]) -> bool) -> Vec<usize> {
    terms
        .iter()
        .enumerate()
        .filter_map(|(id, term)| keep(term).then_some(id))
        .collect()
}
