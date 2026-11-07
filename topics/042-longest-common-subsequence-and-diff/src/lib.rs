//! Exact byte LCS reconstruction and minimum insertion/deletion scripts.
//!
//! Inputs must fit addressable memory. Allocation failure follows Rust's allocator policy.
//! All candidates return one optimal alignment; tie choices may differ.
//! ```
//! use topic_042_lcs_diff::{diff, Candidate, Edit};
//! let (script, _) = diff(b"ABCABBA", b"CBABAC", Candidate::Myers);
//! assert_eq!(script.iter().filter(|e| !matches!(e, Edit::Keep(_))).count(), 5);
//! ```

/// Exact unit-cost operations, including the affected byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edit {
    /// Consume and emit the same byte.
    Keep(u8),
    /// Consume a source byte.
    Delete(u8),
    /// Emit a target byte.
    Insert(u8),
}

/// Script-producing algorithms with the same exact contract.
#[derive(Clone, Copy, Debug)]
pub enum Candidate {
    /// Full LCS table and backward reconstruction.
    Matrix,
    /// Divide-and-conquer reconstruction using released score rows.
    Hirschberg,
    /// Edit-count search with compact retained history.
    Myers,
}

/// Deterministic algorithmic counts, excluding input and result buffers.
#[derive(Default, Debug)]
pub struct Stats {
    /// DP cells or frontier states examined; counts have candidate-specific meanings.
    pub states: usize,
    /// Exact byte comparisons, including equal runs in Myers search.
    pub comparisons: usize,
    /// Maximum live score slots, excluding vector headers and output positions.
    pub score_slots: usize,
    /// Score-buffer allocations requested, excluding output and history headers.
    pub score_allocations: usize,
}

type Pairs = Vec<(usize, usize)>;

/// Return an optimal exact byte script and algorithmic counters.
///
/// Time includes output construction for callers measuring this function. Matrix uses
/// O(mn) scores; Hirschberg uses O(min(m,n)) live scores; this Myers variant retains
/// O(D²) score history. All also allocate O(m+n) output and O(min(m,n)) pairs.
///
/// # Panics
/// Panics if dimension arithmetic exceeds `usize`. Practical allocations can fail earlier.
#[must_use]
pub fn diff(a: &[u8], b: &[u8], candidate: Candidate) -> (Vec<Edit>, Stats) {
    let mut stats = Stats::default();
    let pairs = match candidate {
        Candidate::Matrix => matrix(a, b, &mut stats),
        Candidate::Hirschberg => {
            let mut pairs = Vec::with_capacity(a.len().min(b.len()));
            if b.len() <= a.len() {
                hirschberg(a, b, 0, 0, &mut pairs, &mut stats);
            } else {
                hirschberg(b, a, 0, 0, &mut pairs, &mut stats);
                for pair in &mut pairs {
                    *pair = (pair.1, pair.0);
                }
            }
            pairs
        }
        Candidate::Myers => myers(a, b, &mut stats),
    };
    let mut script = Vec::with_capacity(a.len().checked_add(b.len()).unwrap());
    let (mut i, mut j) = (0, 0);
    for (x, y) in pairs {
        script.extend(a[i..x].iter().copied().map(Edit::Delete));
        script.extend(b[j..y].iter().copied().map(Edit::Insert));
        script.push(Edit::Keep(a[x]));
        i = x + 1;
        j = y + 1;
    }
    script.extend(a[i..].iter().copied().map(Edit::Delete));
    script.extend(b[j..].iter().copied().map(Edit::Insert));
    (script, stats)
}

fn matrix(a: &[u8], b: &[u8], stats: &mut Stats) -> Pairs {
    let width = b.len().checked_add(1).unwrap();
    let size = a.len().checked_add(1).unwrap().checked_mul(width).unwrap();
    let mut table = vec![0usize; size];
    stats.score_slots = size;
    stats.score_allocations = 1;
    for (i, x) in a.iter().enumerate() {
        for (j, y) in b.iter().enumerate() {
            stats.states += 1;
            stats.comparisons += 1;
            table[(i + 1) * width + j + 1] = if x == y {
                table[i * width + j] + 1
            } else {
                table[i * width + j + 1].max(table[(i + 1) * width + j])
            };
        }
    }
    let (mut i, mut j) = (a.len(), b.len());
    let mut pairs = Vec::with_capacity(a.len().min(b.len()));
    while i > 0 && j > 0 {
        stats.comparisons += 1;
        if a[i - 1] == b[j - 1] {
            pairs.push((i - 1, j - 1));
            i -= 1;
            j -= 1;
        } else if table[(i - 1) * width + j] >= table[i * width + j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    pairs.reverse();
    pairs
}

fn row(a: &[u8], b: &[u8], reverse: bool, stats: &mut Stats) -> Vec<usize> {
    let mut scores = vec![0usize; b.len() + 1];
    stats.score_allocations += 1;
    for i in 0..a.len() {
        let x = if reverse { a[a.len() - 1 - i] } else { a[i] };
        let mut diagonal = 0;
        for j in 0..b.len() {
            let y = if reverse { b[b.len() - 1 - j] } else { b[j] };
            let old = scores[j + 1];
            stats.states += 1;
            stats.comparisons += 1;
            scores[j + 1] = if x == y {
                diagonal + 1
            } else {
                old.max(scores[j])
            };
            diagonal = old;
        }
    }
    scores
}

fn hirschberg(a: &[u8], b: &[u8], ai: usize, bi: usize, pairs: &mut Pairs, stats: &mut Stats) {
    if a.is_empty() || b.is_empty() {
        return;
    }
    if a.len() == 1 {
        if let Some(j) = b.iter().position(|&y| {
            stats.comparisons += 1;
            a[0] == y
        }) {
            pairs.push((ai, bi + j));
        }
        return;
    }
    let mid = a.len() / 2;
    let left = row(&a[..mid], b, false, stats);
    let right = row(&a[mid..], b, true, stats);
    stats.score_slots = stats.score_slots.max(left.len() + right.len());
    let mut split = 0;
    for j in 1..=b.len() {
        if left[j] + right[b.len() - j] > left[split] + right[b.len() - split] {
            split = j;
        }
    }
    // Release parent rows before recursion to retain the linear live-score bound.
    drop(left);
    drop(right);
    hirschberg(&a[..mid], &b[..split], ai, bi, pairs, stats);
    hirschberg(&a[mid..], &b[split..], ai + mid, bi + split, pairs, stats);
}

// Diagonals are represented by t in 0..=d, where k=2t-d. The y coordinate
// x-k is computed as x+d-2t with checked subtraction for unreachable states.
fn myers(a: &[u8], b: &[u8], stats: &mut Stats) -> Pairs {
    let max = a.len().checked_add(b.len()).unwrap();
    let mut history: Vec<Vec<usize>> = Vec::new();
    let mut total_slots = 0;
    for d in 0..=max {
        let mut current = Vec::with_capacity(d + 1);
        for t in 0..=d {
            let mut x = if d == 0 {
                0
            } else {
                let prev = &history[d - 1];
                if t == 0 || (t < d && prev[t - 1] < prev[t]) {
                    prev[t]
                } else {
                    prev[t - 1] + 1
                }
            };
            let mut y = (x + d).checked_sub(2 * t).unwrap();
            stats.states += 1;
            while x < a.len() && y < b.len() {
                stats.comparisons += 1;
                if a[x] != b[y] {
                    break;
                }
                x += 1;
                y += 1;
            }
            current.push(x);
            if x >= a.len() && y >= b.len() {
                total_slots += current.capacity();
                stats.score_slots = total_slots;
                stats.score_allocations = d + 1;
                history.push(current);
                return backtrack(&history, a.len(), b.len());
            }
        }
        total_slots += current.len();
        history.push(current);
    }
    unreachable!("deleting and inserting everything is reachable")
}

fn backtrack(history: &[Vec<usize>], mut x: usize, mut y: usize) -> Pairs {
    let mut pairs = Vec::with_capacity(x.min(y));
    for d in (1..history.len()).rev() {
        let t = (x + d - y) / 2;
        let prev = &history[d - 1];
        let (px, py) = if t == 0 || (t < d && prev[t - 1] < prev[t]) {
            let px = prev[t];
            (px, px + d - 1 - 2 * t)
        } else {
            let px = prev[t - 1];
            (px, px + d - 1 - 2 * (t - 1))
        };
        while x > px && y > py {
            x -= 1;
            y -= 1;
            pairs.push((x, y));
        }
        x = px;
        y = py;
    }
    while x > 0 && y > 0 {
        x -= 1;
        y -= 1;
        pairs.push((x, y));
    }
    pairs.reverse();
    pairs
}

/// Independent shortest insertion/deletion edit-graph distance.
///
/// Uses forward relaxation over all prefix states, separately from LCS reconstruction.
/// # Panics
/// Panics if dimensions overflow. Allocations must fit available memory.
#[must_use]
pub fn oracle(a: &[u8], b: &[u8]) -> usize {
    let w = b.len().checked_add(1).unwrap();
    let size = a.len().checked_add(1).unwrap().checked_mul(w).unwrap();
    let mut distance = vec![usize::MAX / 4; size];
    distance[0] = 0;
    for i in 0..=a.len() {
        for j in 0..=b.len() {
            let value = distance[i * w + j];
            if i < a.len() {
                distance[(i + 1) * w + j] = distance[(i + 1) * w + j].min(value + 1);
            }
            if j < b.len() {
                distance[i * w + j + 1] = distance[i * w + j + 1].min(value + 1);
            }
            if i < a.len() && j < b.len() && a[i] == b[j] {
                distance[(i + 1) * w + j + 1] = distance[(i + 1) * w + j + 1].min(value);
            }
        }
    }
    distance[size - 1]
}

/// Validate source consumption, exact target output, and independent optimal cost.
#[must_use]
pub fn valid(a: &[u8], b: &[u8], script: &[Edit], expected_cost: usize) -> bool {
    let mut consumed = Vec::new();
    let mut output = Vec::new();
    let mut cost = 0;
    for &edit in script {
        match edit {
            Edit::Keep(x) => {
                consumed.push(x);
                output.push(x);
            }
            Edit::Delete(x) => {
                consumed.push(x);
                cost += 1;
            }
            Edit::Insert(x) => {
                output.push(x);
                cost += 1;
            }
        }
    }
    consumed == a && output == b && cost == expected_cost
}

/// The function builds source bytes of length `n` and derives the target according to `shape`.
/// # Panics
/// Panics when `shape` is not one of `identical`, `sparse`, `unrelated`, `repeat`, `unequal`.
#[must_use]
pub fn workload(shape: &str, n: usize) -> (Vec<u8>, Vec<u8>) {
    let a: Vec<u8> = (0..n)
        .map(|i| u8::try_from((i * 73 + i / 7) % 251).unwrap())
        .collect();
    let mut b = a.clone();
    match shape {
        "identical" => {}
        "unrelated" => {
            b.fill(255);
        }
        "repeat" => {
            b.fill(b'a');
            return (
                vec![b'a'; n],
                [vec![b'b'; n / 2], b[n / 2..].to_vec()].concat(),
            );
        }
        "unequal" => {
            b.truncate(n / 16);
        }
        "sparse" => {
            for i in (0..n).step_by(64) {
                b[i] = 255;
            }
        }
        _ => panic!("unknown shape"),
    }
    (a, b)
}
