//! Exact inclusive range counts over immutable blocks of signed readings.

use std::num::NonZeroUsize;

/// Work observed while counting a predicate.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Scan {
    /// Rows satisfying the predicate.
    pub matches: usize,
    /// Block summaries visited.
    pub summaries: usize,
    /// Blocks admitted by their summaries.
    pub candidate_blocks: usize,
    /// Original rows checked.
    pub checked_rows: usize,
}

#[derive(Clone, Copy)]
struct Bounds {
    low: i32,
    high: i32,
}

impl Bounds {
    fn overlaps(self, low: i32, high: i32) -> bool {
        self.low <= high && low <= self.high
    }
}

fn count(rows: &[i32], low: i32, high: i32) -> usize {
    rows.iter()
        .map(|&value| usize::from(value >= low && value <= high))
        .sum()
}

/// Count matching rows by visiting every row. Reversed endpoints are empty.
#[must_use]
pub fn full_scan(rows: &[i32], low: i32, high: i32) -> Scan {
    if low > high {
        return Scan::default();
    }
    Scan {
        matches: count(rows, low, high),
        checked_rows: rows.len(),
        ..Scan::default()
    }
}

/// Summary representation for each block.
#[derive(Clone, Copy)]
pub enum Layout {
    /// Store the block's inclusive minimum and maximum.
    MinMax,
    /// Split a sorted copy at its largest adjacent gap.
    TwoRanges,
}

enum Summaries {
    MinMax(Vec<Bounds>),
    TwoRanges(Vec<[Bounds; 2]>),
}

/// Block summaries bound to the original immutable rows.
pub struct Index<'a> {
    rows: &'a [i32],
    width: NonZeroUsize,
    summaries: Summaries,
}

impl<'a> Index<'a> {
    /// Build summaries over physical chunks, including a short final chunk.
    #[must_use]
    pub fn build(rows: &'a [i32], width: NonZeroUsize, layout: Layout) -> Self {
        let summaries = match layout {
            Layout::MinMax => Summaries::MinMax(rows.chunks(width.get()).map(minmax).collect()),
            Layout::TwoRanges => {
                let mut scratch = Vec::with_capacity(width.get().min(rows.len()));
                let summaries = rows
                    .chunks(width.get())
                    .map(|block| {
                        scratch.clear();
                        scratch.extend_from_slice(block);
                        scratch.sort_unstable();
                        split_bounds(&scratch)
                    })
                    .collect();
                Summaries::TwoRanges(summaries)
            }
        };
        Self {
            rows,
            width,
            summaries,
        }
    }

    /// Count matching rows by scanning candidate blocks. Reversed endpoints are empty.
    #[must_use]
    pub fn scan(&self, low: i32, high: i32) -> Scan {
        if low > high {
            return Scan::default();
        }
        match &self.summaries {
            Summaries::MinMax(summaries) => self.scan_candidates(
                summaries.iter().map(|&bounds| bounds.overlaps(low, high)),
                low,
                high,
            ),
            Summaries::TwoRanges(summaries) => self.scan_candidates(
                summaries
                    .iter()
                    .map(|&[left, right]| left.overlaps(low, high) || right.overlaps(low, high)),
                low,
                high,
            ),
        }
    }

    /// Return the summary payload bytes, excluding the vector header.
    #[must_use]
    pub fn summary_bytes(&self) -> usize {
        match &self.summaries {
            Summaries::MinMax(summaries) => std::mem::size_of_val(summaries.as_slice()),
            Summaries::TwoRanges(summaries) => std::mem::size_of_val(summaries.as_slice()),
        }
    }

    fn scan_candidates(&self, candidates: impl Iterator<Item = bool>, low: i32, high: i32) -> Scan {
        let mut scan = Scan::default();
        for (block, candidate) in self.rows.chunks(self.width.get()).zip(candidates) {
            scan.summaries += 1;
            if candidate {
                scan.candidate_blocks += 1;
                scan.checked_rows += block.len();
                scan.matches += count(block, low, high);
            }
        }
        scan
    }
}

fn minmax(rows: &[i32]) -> Bounds {
    let first = rows[0];
    rows[1..].iter().fold(
        Bounds {
            low: first,
            high: first,
        },
        |bounds, &value| Bounds {
            low: bounds.low.min(value),
            high: bounds.high.max(value),
        },
    )
}

fn split_bounds(sorted: &[i32]) -> [Bounds; 2] {
    let first = sorted[0];
    let last = sorted[sorted.len() - 1];
    let mut split = 0;
    let mut gap = 0;
    for (i, pair) in sorted.windows(2).enumerate() {
        let next_gap = i64::from(pair[1]) - i64::from(pair[0]);
        if next_gap > gap {
            split = i + 1;
            gap = next_gap;
        }
    }
    if split == 0 {
        return [Bounds {
            low: first,
            high: last,
        }; 2];
    }
    [
        Bounds {
            low: first,
            high: sorted[split - 1],
        },
        Bounds {
            low: sorted[split],
            high: last,
        },
    ]
}
