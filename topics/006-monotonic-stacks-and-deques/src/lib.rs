//! Forward-only candidate pruning over totally ordered integer samples.
//!
//! Window queries return the newest tied maximum's index. Invalid widths return
//! `None`. Next-greater queries require strict inequality and return `None` for
//! unresolved positions. No timing or per-arrival latency guarantee is made.
//!
//! ```
//! use monotonic_stacks_deques::{window_deque, next_greater_stack};
//! let a = [5, 3, 3, 4, 2, 6, 1, 6];
//! assert_eq!(window_deque(&a, 3), Some(vec![0, 3, 3, 5, 5, 7]));
//! assert_eq!(next_greater_stack(&a)[5], None);
//! ```

use std::collections::VecDeque;

/// Abstract operations, collected in separate untimed calls.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    /// Comparisons between input values, including unsuccessful comparisons.
    pub comparisons: usize,
    /// Candidate indices appended.
    pub pushes: usize,
    /// Candidates removed by dominance or next-greater resolution.
    pub back_pops: usize,
    /// Candidates removed by expiration.
    pub front_pops: usize,
    /// Largest number of live candidate indices.
    pub peak: usize,
    /// Largest back-removal burst caused by one arrival.
    pub max_burst: usize,
}

/// Independent direct window oracle. Newer indices win equal-value ties.
#[must_use]
pub fn window_scan(a: &[i64], w: usize) -> Option<Vec<usize>> {
    if w == 0 || w > a.len() {
        return None;
    }
    let mut result = Vec::with_capacity(a.len() - w + 1);
    for start in 0..=a.len() - w {
        let mut best = start;
        for j in start + 1..start + w {
            if a[j] >= a[best] {
                best = j;
            }
        }
        result.push(best);
    }
    Some(result)
}

/// Online monotonic deque with at most `w` live indices.
///
/// Total work is linear, but one arrival may remove a window-sized suffix.
#[must_use]
pub fn window_deque(a: &[i64], w: usize) -> Option<Vec<usize>> {
    deque_impl::<false>(a, w).map(|(out, _)| out)
}

/// The same deque with operation instrumentation enabled outside timing.
#[must_use]
pub fn window_deque_counted(a: &[i64], w: usize) -> Option<(Vec<usize>, Counts)> {
    deque_impl::<true>(a, w)
}

fn deque_impl<const COUNT: bool>(a: &[i64], w: usize) -> Option<(Vec<usize>, Counts)> {
    if w == 0 || w > a.len() {
        return None;
    }
    let mut q: VecDeque<usize> = VecDeque::with_capacity(w);
    let mut out = Vec::with_capacity(a.len() - w + 1);
    let mut counts = Counts::default();
    for i in 0..a.len() {
        while q.front().is_some_and(|&j| i - j >= w) {
            q.pop_front();
            if COUNT {
                counts.front_pops += 1;
            }
        }
        let mut burst = 0;
        while let Some(&j) = q.back() {
            if COUNT {
                counts.comparisons += 1;
            }
            if a[j] > a[i] {
                break;
            }
            q.pop_back();
            if COUNT {
                counts.back_pops += 1;
                burst += 1;
            }
        }
        q.push_back(i);
        if COUNT {
            counts.pushes += 1;
            counts.peak = counts.peak.max(q.len());
            counts.max_burst = counts.max_burst.max(burst);
        }
        if i >= w - 1 {
            out.push(*q.front().expect("insertion makes the deque nonempty"));
        }
    }
    Some((out, counts))
}

/// Batch prefix/suffix maxima. Includes all preprocessing and allocations.
///
/// Uses two input-sized summary arrays. Ties choose the newest index.
#[must_use]
pub fn window_blocks(a: &[i64], w: usize) -> Option<Vec<usize>> {
    blocks_impl::<false>(a, w).map(|(out, _)| out)
}

/// The batch algorithm with untimed value-comparison counts.
#[must_use]
pub fn window_blocks_counted(a: &[i64], w: usize) -> Option<(Vec<usize>, usize)> {
    blocks_impl::<true>(a, w)
}

fn blocks_impl<const COUNT: bool>(a: &[i64], w: usize) -> Option<(Vec<usize>, usize)> {
    if w == 0 || w > a.len() {
        return None;
    }
    let mut prefix = vec![0; a.len()];
    let mut suffix = vec![0; a.len()];
    let mut comparisons = 0;
    for i in 0..a.len() {
        prefix[i] = if i % w == 0 {
            i
        } else {
            best::<COUNT>(a, prefix[i - 1], i, &mut comparisons)
        };
    }
    for i in (0..a.len()).rev() {
        suffix[i] = if i == a.len() - 1 || (i + 1) % w == 0 {
            i
        } else {
            best::<COUNT>(a, i, suffix[i + 1], &mut comparisons)
        };
    }
    let out = (0..=a.len() - w)
        .map(|start| best::<COUNT>(a, suffix[start], prefix[start + w - 1], &mut comparisons))
        .collect();
    Some((out, comparisons))
}

fn best<const COUNT: bool>(a: &[i64], left: usize, right: usize, count: &mut usize) -> usize {
    if COUNT {
        *count += 1;
    }
    // One value comparison plus an index comparison on equal values.
    match a[left].cmp(&a[right]) {
        std::cmp::Ordering::Greater => left,
        std::cmp::Ordering::Less => right,
        std::cmp::Ordering::Equal => left.max(right),
    }
}

/// Independent suffix-search oracle for the first strictly greater index.
#[must_use]
pub fn next_greater_scan(a: &[i64]) -> Vec<Option<usize>> {
    (0..a.len())
        .map(|i| (i + 1..a.len()).find(|&j| a[j] > a[i]))
        .collect()
}

/// Resolve waiting indices in a forward monotonic stack.
#[must_use]
pub fn next_greater_stack(a: &[i64]) -> Vec<Option<usize>> {
    stack_impl::<false>(a).0
}

/// The stack with untimed comparison, storage, and burst counts.
#[must_use]
pub fn next_greater_stack_counted(a: &[i64]) -> (Vec<Option<usize>>, Counts) {
    stack_impl::<true>(a)
}

fn stack_impl<const COUNT: bool>(a: &[i64]) -> (Vec<Option<usize>>, Counts) {
    let mut waiting: Vec<usize> = Vec::with_capacity(a.len());
    let mut out = vec![None; a.len()];
    let mut counts = Counts::default();
    for i in 0..a.len() {
        let mut burst = 0;
        while let Some(&j) = waiting.last() {
            if COUNT {
                counts.comparisons += 1;
            }
            if a[j] >= a[i] {
                break;
            }
            waiting.pop();
            out[j] = Some(i);
            if COUNT {
                counts.back_pops += 1;
                burst += 1;
            }
        }
        waiting.push(i);
        if COUNT {
            counts.pushes += 1;
            counts.peak = counts.peak.max(waiting.len());
            counts.max_burst = counts.max_burst.max(burst);
        }
    }
    (out, counts)
}
