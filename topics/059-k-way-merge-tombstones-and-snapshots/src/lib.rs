//! Forward snapshot scans over sorted point histories and half-open range deletes.

use std::cell::Cell;
use std::cmp::{Ordering, Reverse};
use std::collections::{BTreeMap, BinaryHeap};

/// A put or point delete. A missing value represents a point delete.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Point {
    /// User key.
    pub key: u32,
    /// Globally unique operation sequence.
    pub seq: u64,
    /// Payload, or a deletion marker.
    pub value: Option<u32>,
}

impl Point {
    /// Sort key for a run: ascending user key, then descending sequence.
    #[must_use]
    pub fn order(self) -> (u32, Reverse<u64>) {
        (self.key, Reverse(self.seq))
    }
}

/// A deletion of keys in a half-open interval.
#[derive(Clone, Copy, Debug)]
pub struct RangeDelete {
    /// Included lower bound.
    pub start: u32,
    /// Excluded upper bound. An empty or reversed range has no effect.
    pub end: u32,
    /// Globally unique operation sequence.
    pub seq: u64,
}

/// The snapshot as sorted key and value pairs.
pub type Rows = Vec<(u32, u32)>;

/// A source of points in forward internal-key order.
#[derive(Clone, Copy, Debug)]
pub enum Selector {
    /// Collect and comparison-sort all points.
    Flat,
    /// Keep one head per nonempty run in a binary heap.
    Heap,
    /// Replay a tournament whose interior nodes keep the losing run.
    Loser,
}

#[derive(Default)]
struct Work {
    comparisons: Cell<u64>,
}

struct Head<'a, const COUNT: bool> {
    order: (u32, Reverse<u64>),
    run: usize,
    offset: usize,
    work: &'a Work,
}

impl<const COUNT: bool> Ord for Head<'_, COUNT> {
    fn cmp(&self, other: &Self) -> Ordering {
        if COUNT {
            self.work.comparisons.set(self.work.comparisons.get() + 1);
        }
        (other.order, other.run, other.offset).cmp(&(self.order, self.run, self.offset))
    }
}

impl<const COUNT: bool> PartialOrd for Head<'_, COUNT> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const COUNT: bool> PartialEq for Head<'_, COUNT> {
    fn eq(&self, other: &Self) -> bool {
        (self.order, self.run, self.offset) == (other.order, other.run, other.offset)
    }
}

impl<const COUNT: bool> Eq for Head<'_, COUNT> {}

fn from_heap<'a, const COUNT: bool>(
    runs: &'a [Vec<Point>],
    work: &'a Work,
) -> impl Iterator<Item = Point> + 'a {
    let mut queue = BinaryHeap::from(
        runs.iter()
            .enumerate()
            .filter_map(|(run, data)| {
                Some(Head::<COUNT> {
                    order: data.first()?.order(),
                    run,
                    offset: 0,
                    work,
                })
            })
            .collect::<Vec<_>>(),
    );
    std::iter::from_fn(move || {
        let mut root = queue.peek_mut()?;
        let point = runs[root.run][root.offset];
        if let Some(next) = runs[root.run].get(root.offset + 1) {
            root.order = next.order();
            root.offset += 1;
        } else {
            std::collections::binary_heap::PeekMut::pop(root);
        }
        Some(point)
    })
}

struct LoserTree<'a, const COUNT: bool> {
    runs: &'a [Vec<Point>],
    offsets: Vec<usize>,
    losers: Vec<usize>,
    winner: usize,
    leaves: usize,
    work: &'a Work,
}

impl<'a, const COUNT: bool> LoserTree<'a, COUNT> {
    fn new(runs: &'a [Vec<Point>], work: &'a Work) -> Self {
        let leaves = runs.len().next_power_of_two();
        let mut tree = Self {
            runs,
            offsets: vec![0; runs.len()],
            losers: vec![0; leaves],
            winner: 0,
            leaves,
            work,
        };
        tree.winner = tree.build(1);
        tree
    }

    fn point(&self, run: usize) -> Option<Point> {
        self.runs.get(run)?.get(self.offsets[run]).copied()
    }

    fn contest(&mut self, node: usize, left: usize, right: usize) -> usize {
        if COUNT {
            self.work.comparisons.set(self.work.comparisons.get() + 1);
        }
        let left_key = self.point(left).map(Point::order);
        let right_key = self.point(right).map(Point::order);
        let (winner, loser) = match (left_key, right_key) {
            (None, Some(_)) => (right, left),
            (Some(a), Some(b)) if (a, left) > (b, right) => (right, left),
            _ => (left, right),
        };
        self.losers[node] = loser;
        winner
    }

    fn build(&mut self, node: usize) -> usize {
        if node >= self.leaves {
            return node - self.leaves;
        }
        let left = self.build(2 * node);
        let right = self.build(2 * node + 1);
        self.contest(node, left, right)
    }
}

impl<const COUNT: bool> Iterator for LoserTree<'_, COUNT> {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        let mut advancing = self.winner;
        let point = self.point(advancing)?;
        self.offsets[advancing] += 1;
        let mut node = usize::midpoint(self.leaves, advancing);
        while node != 0 {
            advancing = self.contest(node, advancing, self.losers[node]);
            node /= 2;
        }
        self.winner = advancing;
        Some(point)
    }
}

fn resolve(points: impl Iterator<Item = Point>, ranges: &[RangeDelete], snapshot: u64) -> Rows {
    let mut starts: Vec<_> = ranges
        .iter()
        .filter(|r| r.seq <= snapshot && r.start < r.end)
        .collect();
    starts.sort_unstable_by_key(|r| r.start);
    let mut next_range = 0;
    let mut active = BinaryHeap::<(u64, u32)>::new();
    let mut previous = None;
    let mut out = Rows::new();

    for point in points {
        if point.seq > snapshot || previous == Some(point.key) {
            continue;
        }
        previous = Some(point.key);
        let Some(value) = point.value else { continue };
        while let Some(range) = starts.get(next_range).filter(|r| r.start <= point.key) {
            active.push((range.seq, range.end));
            next_range += 1;
        }
        while active.peek().is_some_and(|&(_, end)| end <= point.key) {
            active.pop();
        }
        if active.peek().is_none_or(|&(seq, _)| seq < point.seq) {
            out.push((point.key, value));
        }
    }
    out
}

fn scan<const COUNT: bool>(
    runs: &[Vec<Point>],
    ranges: &[RangeDelete],
    snapshot: u64,
    selector: Selector,
    work: &Work,
) -> Rows {
    match selector {
        Selector::Flat => {
            let count = runs.iter().map(Vec::len).sum();
            let mut points = Vec::with_capacity(count);
            for run in runs {
                points.extend_from_slice(run);
            }
            points.sort_unstable_by(|a, b| {
                if COUNT {
                    work.comparisons.set(work.comparisons.get() + 1);
                }
                a.order().cmp(&b.order())
            });
            resolve(points.into_iter(), ranges, snapshot)
        }
        Selector::Heap => resolve(from_heap::<COUNT>(runs, work), ranges, snapshot),
        Selector::Loser => resolve(LoserTree::<COUNT>::new(runs, work), ranges, snapshot),
    }
}

/// Read the view after all operations through `snapshot`.
///
/// Point runs must already be sorted by [`Point::order`]. Sequences must be
/// unique across all operations. Ranges may be supplied in any order.
#[must_use]
pub fn read(
    runs: &[Vec<Point>],
    ranges: &[RangeDelete],
    snapshot: u64,
    selector: Selector,
) -> Rows {
    scan::<false>(runs, ranges, snapshot, selector, &Work::default())
}

/// Read the same view as [`read`] and count point-selector comparisons.
#[must_use]
pub fn measure_work(
    runs: &[Vec<Point>],
    ranges: &[RangeDelete],
    snapshot: u64,
    selector: Selector,
) -> (Rows, u64) {
    let work = Work::default();
    let rows = scan::<true>(runs, ranges, snapshot, selector, &work);
    (rows, work.comparisons.get())
}

#[derive(Clone, Copy)]
enum Operation {
    Point(Point),
    Range(RangeDelete),
}

/// Independently materialize a snapshot by replaying operations in sequence order.
///
/// Sequences must be unique. Point runs may be supplied in any order.
#[must_use]
pub fn replay(runs: &[Vec<Point>], ranges: &[RangeDelete], snapshot: u64) -> Rows {
    let mut history = Vec::new();
    for point in runs.iter().flatten().filter(|point| point.seq <= snapshot) {
        history.push((point.seq, Operation::Point(*point)));
    }
    for range in ranges.iter().filter(|range| range.seq <= snapshot) {
        history.push((range.seq, Operation::Range(*range)));
    }
    history.sort_unstable_by_key(|&(seq, _)| seq);
    let mut live = BTreeMap::new();
    let mut remove = Vec::new();
    for (_, op) in history {
        match op {
            Operation::Point(Point {
                key,
                value: Some(value),
                ..
            }) => {
                live.insert(key, value);
            }
            Operation::Point(Point {
                key, value: None, ..
            }) => {
                live.remove(&key);
            }
            Operation::Range(RangeDelete { start, end, .. }) if start < end => {
                remove.extend(live.range(start..end).map(|(&key, _)| key));
                for key in remove.drain(..) {
                    live.remove(&key);
                }
            }
            Operation::Range(_) => {}
        }
    }
    live.into_iter().collect()
}
