//! Stable merge, duplicate-preserving inner join, and merge schedules.
#![forbid(unsafe_code)]
// Index names follow the co-ranking equations used in the lesson.
#![allow(clippy::many_single_char_names)]
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// A key and identity. Identity is payload, not a sorting key.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Row {
    /// Ascending key.
    pub key: u64,
    /// Identity preserved by merge and join.
    pub id: u64,
}

/// Independent stable concatenate-and-sort merge oracle.
#[must_use]
pub fn sort_merge(a: &[Row], b: &[Row]) -> Vec<Row> {
    let mut out = [a, b].concat();
    out.sort_by_key(|r| r.key);
    out
}

/// Stable two-pointer merge. Inputs must be sorted by key.
#[must_use]
pub fn linear_merge(a: &[Row], b: &[Row]) -> Vec<Row> {
    let mut out = Vec::with_capacity(a.len() + b.len());
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        if a[i].key <= b[j].key {
            out.push(a[i]);
            i += 1;
        } else {
            out.push(b[j]);
            j += 1;
        }
    }
    out.extend_from_slice(&a[i..]);
    out.extend_from_slice(&b[j..]);
    out
}

// Find the prefix satisfying a monotone predicate by exponential search,
// then binary search. This avoids searching the whole suffix for short runs.
fn prefix<T>(s: &[T], pred: impl Fn(&T) -> bool) -> usize {
    if s.is_empty() || !pred(&s[0]) {
        return 0;
    }
    let mut bound = 1;
    while bound < s.len() && pred(&s[bound]) {
        bound = bound.saturating_mul(2);
    }
    let lo = bound / 2;
    let hi = bound.saturating_add(1).min(s.len());
    lo + s[lo..hi].partition_point(pred)
}

/// Stable block merge with exponential prefix searches.
#[must_use]
pub fn gallop_merge(mut a: &[Row], mut b: &[Row]) -> Vec<Row> {
    let mut out = Vec::with_capacity(a.len() + b.len());
    while !a.is_empty() && !b.is_empty() {
        if a[0].key <= b[0].key {
            let n = prefix(a, |r| r.key <= b[0].key);
            out.extend_from_slice(&a[..n]);
            a = &a[n..];
        } else {
            let n = prefix(b, |r| r.key < a[0].key);
            out.extend_from_slice(&b[..n]);
            b = &b[n..];
        }
    }
    out.extend_from_slice(a);
    out.extend_from_slice(b);
    out
}

/// Input prefix lengths producing an output prefix of `d` rows.
///
/// Inputs must be ascending; `d <= a.len() + b.len()`.
/// Equal keys in `a` precede equal keys in `b`.
///
/// # Panics
/// Panics if `d` exceeds the total input length.
#[must_use]
pub fn co_rank(d: usize, a: &[Row], b: &[Row]) -> (usize, usize) {
    assert!(d <= a.len() + b.len());
    let mut lo = d.saturating_sub(b.len());
    let mut hi = d.min(a.len());
    loop {
        let i = lo + (hi - lo) / 2;
        let j = d - i;
        if i > 0 && j < b.len() && a[i - 1].key > b[j].key {
            hi = i - 1;
        } else if j > 0 && i < a.len() && b[j - 1].key >= a[i].key {
            lo = i + 1;
        } else {
            return (i, j);
        }
    }
}

/// Safe scoped-thread Merge Path merge, including output initialization.
///
/// Inputs must be sorted. `workers` must be positive. Thread launch, join,
/// co-ranking, allocation, and initialization are part of this API's cost.
///
/// # Panics
/// Panics if `workers` is zero or a spawned thread panics.
#[must_use]
pub fn parallel_merge(a: &[Row], b: &[Row], workers: usize) -> Vec<Row> {
    assert!(workers > 0);
    let n = a.len() + b.len();
    if n == 0 {
        return Vec::new();
    }
    let chunk = n.div_ceil(workers);
    let mut out = vec![Row::default(); n];
    std::thread::scope(|scope| {
        for (part, dest) in out.chunks_mut(chunk).enumerate() {
            let d = part * chunk;
            let (i, j) = co_rank(d, a, b);
            let (ie, je) = co_rank(d + dest.len(), a, b);
            scope.spawn(move || {
                let (aa, bb) = (&a[i..ie], &b[j..je]);
                let (mut x, mut y) = (0, 0);
                for slot in dest {
                    if y == bb.len() || (x < aa.len() && aa[x].key <= bb[y].key) {
                        *slot = aa[x];
                        x += 1;
                    } else {
                        *slot = bb[y];
                        y += 1;
                    }
                }
            });
        }
    });
    out
}

/// Binary-probe join; all matching pairs, left-major then right-major order.
/// Inputs must be sorted. Allocation failure follows `Vec` behavior.
#[must_use]
pub fn probe_join(a: &[Row], b: &[Row]) -> Vec<(u64, u64)> {
    let mut out = Vec::new();
    for x in a {
        let lo = b.partition_point(|r| r.key < x.key);
        let hi = b.partition_point(|r| r.key <= x.key);
        for y in &b[lo..hi] {
            out.push((x.id, y.id));
        }
    }
    out
}

/// Run-replay join. No deduplication: equal runs form a Cartesian product.
/// Inputs must be sorted. Output allocation and growth are included.
#[must_use]
pub fn replay_join(a: &[Row], b: &[Row]) -> Vec<(u64, u64)> {
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        match a[i].key.cmp(&b[j].key) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                let key = a[i].key;
                let mut ie = i + 1;
                let mut je = j + 1;
                while ie < a.len() && a[ie].key == key {
                    ie += 1;
                }
                while je < b.len() && b[je].key == key {
                    je += 1;
                }
                for x in &a[i..ie] {
                    for y in &b[j..je] {
                        out.push((x.id, y.id));
                    }
                }
                i = ie;
                j = je;
            }
        }
    }
    out
}

/// Count matching pairs in `u128` without materializing them.
/// Inputs must be sorted. This is a different contract from a row join.
#[must_use]
#[allow(clippy::comparison_chain)]
pub fn count_join(a: &[Row], b: &[Row]) -> u128 {
    let (mut i, mut j, mut total) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        if a[i].key < b[j].key {
            i += 1;
        } else if a[i].key > b[j].key {
            j += 1;
        } else {
            let key = a[i].key;
            let (start_i, start_j) = (i, j);
            while i < a.len() && a[i].key == key {
                i += 1;
            }
            while j < b.len() && b[j].key == key {
                j += 1;
            }
            total += (i - start_i) as u128 * (j - start_j) as u128;
        }
    }
    total
}

/// Left-fold cascade over sorted runs, preserving original run order on ties.
#[must_use]
pub fn left_cascade(runs: &[Vec<Row>]) -> Vec<Row> {
    runs.iter().fold(Vec::new(), |acc, r| linear_merge(&acc, r))
}

/// Adjacent balanced cascade; arbitrary reordering would break tie stability.
#[must_use]
pub fn balanced_cascade(runs: &[Vec<Row>]) -> Vec<Row> {
    fn go(r: &[Vec<Row>]) -> Vec<Row> {
        match r.len() {
            0 => Vec::new(),
            1 => r[0].clone(),
            _ => {
                let m = r.len() / 2;
                linear_merge(&go(&r[..m]), &go(&r[m..]))
            }
        }
    }
    go(runs)
}

/// Heap merge. `(key, run, position)` establishes stable run precedence.
#[must_use]
pub fn heap_cascade(runs: &[Vec<Row>]) -> Vec<Row> {
    let mut heap = BinaryHeap::new();
    for (r, run) in runs.iter().enumerate() {
        if let Some(x) = run.first() {
            heap.push(Reverse((x.key, r, 0)));
        }
    }
    let mut out = Vec::with_capacity(runs.iter().map(Vec::len).sum());
    while let Some(Reverse((_, r, p))) = heap.pop() {
        out.push(runs[r][p]);
        if let Some(x) = runs[r].get(p + 1) {
            heap.push(Reverse((x.key, r, p + 1)));
        }
    }
    out
}

/// Independent stable sort oracle for a cascade.
#[must_use]
pub fn sort_cascade(runs: &[Vec<Row>]) -> Vec<Row> {
    let mut out: Vec<Row> = runs.iter().flatten().copied().collect();
    out.sort_by_key(|r| r.key);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn row(keys: &[u64], base: u64) -> Vec<Row> {
        keys.iter()
            .enumerate()
            .map(|(i, &key)| Row {
                key,
                id: base + i as u64,
            })
            .collect()
    }
    #[test]
    fn exhaustive_sorted_pairs_and_every_diagonal() {
        let mut vectors = Vec::new();
        for n in 0..=5 {
            for word in 0..3_usize.pow(n) {
                let mut v = Vec::new();
                let mut w = word;
                for _ in 0..n {
                    v.push((w % 3) as u64);
                    w /= 3;
                }
                if v.windows(2).all(|p| p[0] <= p[1]) {
                    vectors.push(v);
                }
            }
        }
        for ak in &vectors {
            for bk in &vectors {
                let a = row(ak, 0);
                let b = row(bk, 100);
                let want = sort_merge(&a, &b);
                assert_eq!(linear_merge(&a, &b), want);
                assert_eq!(gallop_merge(&a, &b), want);
                for d in 0..=want.len() {
                    let (i, j) = co_rank(d, &a, &b);
                    assert_eq!(sort_merge(&a[..i], &b[..j]), want[..d]);
                }
                let naive: Vec<_> = a
                    .iter()
                    .flat_map(|x| {
                        b.iter()
                            .filter(move |y| x.key == y.key)
                            .map(move |y| (x.id, y.id))
                    })
                    .collect();
                assert_eq!(probe_join(&a, &b), naive);
                assert_eq!(replay_join(&a, &b), naive);
                assert_eq!(count_join(&a, &b), naive.len() as u128);
            }
        }
        assert_eq!(vectors.len(), 56);
    }
    #[test]
    fn parallel_tie_boundaries_and_empty_chunks() {
        for n in 0..=24 {
            let a = row(&vec![7; n], 0);
            let b = row(&vec![7; n + 3], 100);
            for workers in [1, 2, 4, 32] {
                assert_eq!(parallel_merge(&a, &b, workers), sort_merge(&a, &b));
            }
        }
        assert!(parallel_merge(&[], &[], 4).is_empty());
    }
    #[test]
    fn cascade_stability_and_skew() {
        for sizes in [vec![], vec![0], vec![1_usize, 8, 0, 2, 13], vec![8; 17]] {
            let runs: Vec<_> = sizes
                .iter()
                .enumerate()
                .map(|(i, &n)| {
                    row(
                        &(0..n).map(|x| (x / 3) as u64).collect::<Vec<_>>(),
                        i as u64 * 100,
                    )
                })
                .collect();
            let want = sort_cascade(&runs);
            assert_eq!(left_cascade(&runs), want);
            assert_eq!(balanced_cascade(&runs), want);
            assert_eq!(heap_cascade(&runs), want);
        }
    }
    #[test]
    fn seeded_differential_and_extremes() {
        let mut seed = 17_u64;
        for n in 0_usize..200 {
            let mut make = |base| {
                let mut v = Vec::new();
                for i in 0..n {
                    seed ^= seed << 13;
                    seed ^= seed >> 7;
                    seed ^= seed << 17;
                    v.push(Row {
                        key: seed % 37,
                        id: base + i as u64,
                    });
                }
                v.sort_by_key(|r| r.key);
                v
            };
            let a = make(0);
            let b = make(1000);
            let want = sort_merge(&a, &b);
            assert_eq!(linear_merge(&a, &b), want);
            assert_eq!(gallop_merge(&a, &b), want);
            assert_eq!(parallel_merge(&a, &b, 4), want);
            assert_eq!(replay_join(&a, &b), probe_join(&a, &b));
        }
        let a = row(&[0, 0, u64::MAX], 0);
        let b = row(&[0, u64::MAX, u64::MAX], 100);
        assert_eq!(gallop_merge(&a, &b), sort_merge(&a, &b));
        assert_eq!(count_join(&a, &b), 4);
    }
}
