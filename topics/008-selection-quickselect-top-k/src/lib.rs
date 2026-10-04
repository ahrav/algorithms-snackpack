//! Exact, sorted smallest-K portfolios for immutable integer input.
//!
//! Values retain multiplicity. `k > input.len()` returns `None`; zero returns
//! an empty vector. No candidate mutates the caller's input.
#![forbid(unsafe_code)]

use std::collections::BinaryHeap;

/// Full-sort baseline. Allocates an N-element working vector for nonzero K.
#[must_use]
pub fn full_sort(input: &[i64], k: usize) -> Option<Vec<i64>> {
    if k > input.len() {
        return None;
    }
    if k == 0 {
        return Some(Vec::new());
    }
    let mut values = input.to_vec();
    values.sort_unstable();
    values.truncate(k);
    Some(values)
}

/// Library selection followed by sorting exactly the selected prefix.
#[must_use]
pub fn standard(input: &[i64], k: usize) -> Option<Vec<i64>> {
    if k > input.len() {
        return None;
    }
    if k == 0 {
        return Some(Vec::new());
    }
    let mut values = input.to_vec();
    if k < values.len() {
        values.select_nth_unstable(k - 1);
    }
    values.truncate(k);
    values.sort_unstable();
    Some(values)
}

/// Work counters for the teaching selector, outside performance timing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SelectionStats {
    /// Elements classified by the three-way partitions.
    pub classified: usize,
    /// Explicit swaps, including self swaps; not machine stores.
    pub swaps: usize,
    /// Completed partitions.
    pub partitions: usize,
    /// Whether the scan budget caused a full active-slice sort.
    pub fallback: bool,
}

/// In-place teaching selector for zero-based rank, with a cumulative scan cap.
///
/// Returns `None` for invalid rank without changing the input. The selected
/// value has the same rank as in a full sort. Both side slices are unordered.
/// A three-way equal band avoids repeatedly selecting through duplicate values.
/// Sorting the active slice after an 8*N scan budget gives O(N log N) worst-case
/// work, not the library's linear guarantee.
#[must_use]
pub fn select_three_way(values: &mut [i64], rank: usize) -> Option<SelectionStats> {
    if rank >= values.len() {
        return None;
    }
    let mut stats = SelectionStats::default();
    let budget = values.len().saturating_mul(8);
    let (mut lo, mut hi) = (0, values.len());
    while hi - lo > 1 {
        let size = hi - lo;
        if size > budget.saturating_sub(stats.classified) {
            values[lo..hi].sort_unstable();
            stats.fallback = true;
            return Some(stats);
        }
        let a = values[lo];
        let b = values[lo + size / 2];
        let c = values[hi - 1];
        let pivot = a.max(b.min(c)).min(b.max(c));
        let (mut less, mut scan, mut greater) = (lo, lo, hi);
        stats.partitions += 1;
        // [lo,less) < pivot; [less,scan) == pivot;
        // [scan,greater) unknown; [greater,hi) > pivot.
        while scan < greater {
            stats.classified += 1;
            match values[scan].cmp(&pivot) {
                std::cmp::Ordering::Less => {
                    values.swap(less, scan);
                    stats.swaps += 1;
                    less += 1;
                    scan += 1;
                }
                std::cmp::Ordering::Equal => scan += 1,
                std::cmp::Ordering::Greater => {
                    greater -= 1;
                    values.swap(scan, greater);
                    stats.swaps += 1;
                }
            }
        }
        if rank < less {
            hi = less;
        } else if rank >= greater {
            lo = greater;
        } else {
            return Some(stats);
        }
    }
    Some(stats)
}

/// Three-way teaching selection plus sorting the K-element result.
#[must_use]
pub fn three_way(input: &[i64], k: usize) -> Option<Vec<i64>> {
    if k > input.len() {
        return None;
    }
    if k == 0 {
        return Some(Vec::new());
    }
    let mut values = input.to_vec();
    if k < values.len() {
        let _ = select_three_way(&mut values, k - 1);
    }
    values.truncate(k);
    values.sort_unstable();
    Some(values)
}

/// Streaming-compatible max-heap: root is the worst retained small value.
///
/// Retains K integers. The borrowed slice remains resident in this experiment.
#[must_use]
pub fn heap(input: &[i64], k: usize) -> Option<Vec<i64>> {
    if k > input.len() {
        return None;
    }
    if k == 0 {
        return Some(Vec::new());
    }
    let mut retained = BinaryHeap::from(input[..k].to_vec());
    for &value in &input[k..] {
        if value < *retained.peek()? {
            *retained.peek_mut()? = value;
        }
    }
    Some(retained.into_sorted_vec())
}

/// Sorted retained buffer: bounded memory but up to K moves per accepted value.
#[must_use]
pub fn buffer(input: &[i64], k: usize) -> Option<Vec<i64>> {
    if k > input.len() {
        return None;
    }
    if k == 0 {
        return Some(Vec::new());
    }
    let mut retained = input[..k].to_vec();
    retained.sort_unstable();
    for &value in &input[k..] {
        if value < retained[k - 1] {
            let position = retained.partition_point(|&x| x < value);
            retained.copy_within(position..k - 1, position + 1);
            retained[position] = value;
        }
    }
    Some(retained)
}

/// Candidate function type shared by tests and the frozen harness.
pub type Candidate = fn(&[i64], usize) -> Option<Vec<i64>>;
/// All serious candidates, with stable experiment names.
pub const CANDIDATES: [(&str, Candidate); 5] = [
    ("sort", full_sort),
    ("standard", standard),
    ("three_way", three_way),
    ("heap", heap),
    ("buffer", buffer),
];
