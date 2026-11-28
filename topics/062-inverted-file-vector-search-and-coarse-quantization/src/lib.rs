//! Fixed-centroid inverted files, residual product quantization, and exact reranking.

use std::collections::BinaryHeap;

/// An eight-dimensional vector with exact integer coordinates.
pub type Vector = [i16; 8];
/// A squared distance paired with an original row ID. Lower tuples rank first.
pub type Hit = (i64, usize);
type Code = [u8; 4];
const WORDS: [[i64; 2]; 9] = [
    [0, 0],
    [0, -12],
    [0, 12],
    [-12, 0],
    [-12, -12],
    [-12, 12],
    [12, 0],
    [12, -12],
    [12, 12],
];

/// Available search paths. Probe counts clamp to the centroid count.
#[derive(Clone, Copy, Debug)]
pub enum Mode {
    /// Score every original vector.
    Flat,
    /// Score originals in the selected inverted lists.
    Raw(usize),
    /// Score residual codes in the selected inverted lists.
    Pq(usize),
    /// Keep a compressed shortlist and score its original vectors.
    Rerank {
        /// Number of lists to open.
        probes: usize,
        /// Compressed shortlist capacity. Values below k may underfill.
        shortlist: usize,
    },
}

/// Sorted hits and actual row work for one search.
#[derive(Debug, PartialEq, Eq)]
pub struct Search {
    /// Up to k hits, ordered by (squared distance, original row ID).
    pub hits: Vec<Hit>,
    /// Entries scored on the first pass.
    pub scanned: usize,
    /// Originals scored on the refinement pass.
    pub reranked: usize,
}

/// A static corpus and its disjoint centroid lists.
pub struct Index {
    rows: Vec<Vector>,
    centroids: Vec<Vector>,
    lists: Vec<List>,
}

#[derive(Clone, Default)]
struct List {
    ids: Vec<usize>,
    codes: Vec<Code>,
}

fn distance(left: &Vector, right: &Vector) -> i64 {
    left.iter()
        .zip(right)
        .map(|(&x, &y)| {
            let delta = i64::from(x) - i64::from(y);
            delta * delta
        })
        .sum()
}

fn table(row: &Vector, center: &Vector) -> [[i64; 9]; 4] {
    std::array::from_fn(|j| {
        let x = i64::from(row[2 * j]) - i64::from(center[2 * j]);
        let y = i64::from(row[2 * j + 1]) - i64::from(center[2 * j + 1]);
        WORDS.map(|[a, b]| (x - a).pow(2) + (y - b).pow(2))
    })
}

fn encode(row: &Vector, center: &Vector) -> Code {
    table(row, center).map(|scores| {
        let best = scores
            .iter()
            .enumerate()
            .min_by_key(|&(_, score)| score)
            .unwrap()
            .0;
        u8::try_from(best).expect("nine codewords fit in one byte")
    })
}

fn code_distance(tables: &[[i64; 9]; 4], code: Code) -> i64 {
    tables
        .iter()
        .zip(code)
        .map(|(t, id)| t[usize::from(id)])
        .sum()
}

fn keep(best: &mut BinaryHeap<Hit>, limit: usize, hit: Hit) {
    if best.len() < limit {
        best.push(hit);
    } else if let Some(mut worst) = best.peek_mut()
        && hit < *worst
    {
        *worst = hit;
    }
}

impl Index {
    /// Assign each row to its nearest centroid, breaking ties by centroid index.
    ///
    /// # Panics
    ///
    /// Panics when no centroids are supplied.
    #[must_use]
    pub fn new(rows: Vec<Vector>, centroids: Vec<Vector>) -> Self {
        assert!(!centroids.is_empty(), "at least one centroid is required");
        let mut lists = vec![List::default(); centroids.len()];
        for (id, row) in rows.iter().enumerate() {
            let (key, center) = centroids
                .iter()
                .enumerate()
                .min_by_key(|&(_, center)| distance(row, center))
                .unwrap();
            lists[key].ids.push(id);
            lists[key].codes.push(encode(row, center));
        }
        Self {
            rows,
            centroids,
            lists,
        }
    }

    /// Search under the selected scorer. Zero k requests no work.
    #[must_use]
    pub fn search(&self, query: &Vector, k: usize, mode: Mode) -> Search {
        let mut out = Search {
            hits: Vec::new(),
            scanned: 0,
            reranked: 0,
        };
        if k == 0 || self.rows.is_empty() {
            return out;
        }
        let cap = k.min(self.rows.len());
        let (probes, coded, refine) = match mode {
            Mode::Flat => {
                let mut best = BinaryHeap::with_capacity(cap);
                for (id, row) in self.rows.iter().enumerate() {
                    keep(&mut best, cap, (distance(query, row), id));
                }
                out.scanned = self.rows.len();
                out.hits = best.into_sorted_vec();
                return out;
            }
            Mode::Raw(probes) => (probes, false, None),
            Mode::Pq(probes) => (probes, true, None),
            Mode::Rerank { probes, shortlist } => (probes, true, Some(shortlist)),
        };
        let limit = refine.unwrap_or(cap).min(self.rows.len());
        if limit == 0 || probes == 0 {
            return out;
        }

        let probe_count = probes.min(self.centroids.len());
        let mut nearest = BinaryHeap::with_capacity(probe_count);
        for (key, center) in self.centroids.iter().enumerate() {
            keep(&mut nearest, probe_count, (distance(query, center), key));
        }

        let mut candidates = BinaryHeap::with_capacity(limit);
        for (_, key) in nearest.into_sorted_vec() {
            let list = &self.lists[key];
            out.scanned += list.ids.len();
            if coded {
                let tables = table(query, &self.centroids[key]);
                for (&id, &code) in list.ids.iter().zip(&list.codes) {
                    keep(&mut candidates, limit, (code_distance(&tables, code), id));
                }
            } else {
                for &id in &list.ids {
                    keep(
                        &mut candidates,
                        limit,
                        (distance(query, &self.rows[id]), id),
                    );
                }
            }
        }
        if refine.is_some() {
            let mut best = BinaryHeap::with_capacity(cap.min(candidates.len()));
            out.reranked = candidates.len();
            for (_, id) in candidates {
                keep(&mut best, cap, (distance(query, &self.rows[id]), id));
            }
            out.hits = best.into_sorted_vec();
        } else {
            out.hits = candidates.into_sorted_vec();
        }
        out
    }
}

/// Full sorting with widened direct differences, independent of index search.
#[must_use]
pub fn reference(rows: &[Vector], query: &Vector, k: usize) -> Vec<Hit> {
    let mut ranked = Vec::with_capacity(rows.len());
    for (id, row) in rows.iter().enumerate() {
        let mut sum = 0;
        for dim in 0..8 {
            let diff = i64::from(i32::from(query[dim]) - i32::from(row[dim]));
            sum += diff * diff;
        }
        ranked.push((sum, id));
    }
    ranked.sort_unstable();
    ranked.truncate(k);
    ranked
}
