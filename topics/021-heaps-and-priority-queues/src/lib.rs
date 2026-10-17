//! Keyed minimum-priority queues with explicit FIFO and stale-entry contracts.
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

/// A live record, ordered by priority, original arrival, ID, then unique stamp.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Entry {
    /// Smaller priorities are returned first.
    pub priority: u64,
    /// Preserved by updates; renewed after removal and reinsertion.
    pub arrival: u64,
    /// Stable ID in the queue's fixed universe.
    pub id: usize,
    /// Unique per successful set, including same-priority updates.
    pub stamp: u64,
}

/// Implemented, semantically equivalent candidates.
#[derive(Clone, Copy, Debug)]
pub enum Kind {
    /// Scan the fixed ID universe.
    Scan,
    /// Indexed binary heap with bottom-up construction.
    Binary,
    /// Indexed binary heap constructed by repeated insertion.
    BinaryPush,
    /// Indexed four-child heap.
    Four,
    /// Indexed eight-child heap.
    Eight,
    /// Standard binary heap with version validation and no compaction.
    Lazy,
    /// Lazy heap rebuilt when physical records exceed twice live plus 32.
    Compact,
}

impl Kind {
    /// All candidates in fixed experiment order.
    pub const ALL: [Self; 7] = [
        Self::Scan,
        Self::Binary,
        Self::BinaryPush,
        Self::Four,
        Self::Eight,
        Self::Lazy,
        Self::Compact,
    ];
    /// Stable command-line spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Scan => "scan",
            Self::Binary => "binary",
            Self::BinaryPush => "binary_push",
            Self::Four => "four",
            Self::Eight => "eight",
            Self::Lazy => "lazy",
            Self::Compact => "compact",
        }
    }
}

/// Rejections occur before any logical mutation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// ID outside the fixed universe.
    InvalidId,
    /// No next unique arrival or stamp is available.
    CounterExhausted,
}

#[derive(Debug)]
struct Indexed<const D: usize> {
    heap: Vec<Entry>,
    pos: Vec<usize>,
}

impl<const D: usize> Indexed<D> {
    fn new(entries: Vec<Entry>, universe: usize, pushes: bool) -> Self {
        let mut result = Self {
            heap: Vec::with_capacity(entries.len()),
            pos: vec![usize::MAX; universe],
        };
        if pushes {
            for e in entries {
                result.set(e);
            }
        } else {
            result.heap = entries;
            for (i, e) in result.heap.iter().enumerate() {
                result.pos[e.id] = i;
            }
            if result.heap.len() > 1 {
                for i in (0..=(result.heap.len() - 2) / D).rev() {
                    result.down(i);
                }
            }
        }
        result
    }
    fn swap(&mut self, a: usize, b: usize) {
        self.heap.swap(a, b);
        self.pos[self.heap[a].id] = a;
        self.pos[self.heap[b].id] = b;
    }
    fn up(&mut self, mut i: usize) {
        while i > 0 {
            let parent = (i - 1) / D;
            if self.heap[parent] <= self.heap[i] {
                break;
            }
            self.swap(parent, i);
            i = parent;
        }
    }
    fn down(&mut self, mut i: usize) {
        // i < len/D bounds multiplication by a real child position.
        while self.heap.len() > 1 && i <= (self.heap.len() - 2) / D {
            let first = i * D + 1;
            let end = first.saturating_add(D).min(self.heap.len());
            let mut best = first;
            for child in first + 1..end {
                if self.heap[child] < self.heap[best] {
                    best = child;
                }
            }
            if self.heap[i] <= self.heap[best] {
                break;
            }
            self.swap(i, best);
            i = best;
        }
    }
    fn repair(&mut self, i: usize) {
        if i > 0 && self.heap[i] < self.heap[(i - 1) / D] {
            self.up(i);
        } else {
            self.down(i);
        }
    }
    fn set(&mut self, e: Entry) {
        let i = self.pos[e.id];
        if i == usize::MAX {
            self.pos[e.id] = self.heap.len();
            self.heap.push(e);
            self.up(self.heap.len() - 1);
        } else {
            self.heap[i] = e;
            self.repair(i);
        }
    }
    fn remove(&mut self, id: usize) -> Option<Entry> {
        let i = self.pos[id];
        if i == usize::MAX {
            return None;
        }
        let removed = self.heap.swap_remove(i);
        self.pos[id] = usize::MAX;
        if i < self.heap.len() {
            self.pos[self.heap[i].id] = i;
            self.repair(i);
        }
        Some(removed)
    }
    fn valid(&self, live: &[Option<Entry>]) -> bool {
        self.heap.iter().enumerate().all(|(i, e)| {
            self.pos[e.id] == i
                && live[e.id] == Some(*e)
                && (i == 0 || self.heap[(i - 1) / D] <= *e)
        }) && self.pos.iter().enumerate().all(|(id, &p)| match live[id] {
            Some(e) => self.heap.get(p) == Some(&e),
            None => p == usize::MAX,
        })
    }
}

#[derive(Debug)]
enum Storage {
    Scan,
    Binary(Indexed<2>),
    Four(Indexed<4>),
    Eight(Indexed<8>),
    Lazy {
        heap: BinaryHeap<Reverse<Entry>>,
        compact: bool,
    },
}

/// Sequential queue. Allocation failure follows Rust's ordinary allocation policy.
#[derive(Debug)]
pub struct Queue {
    live: Vec<Option<Entry>>,
    storage: Storage,
    next_arrival: u64,
    next_stamp: u64,
    len: usize,
    peak_records: usize,
    rebuilds: usize,
    stale_pops: usize,
}

impl Queue {
    /// Build one live job per input priority. IDs and FIFO order follow slice order.
    ///
    /// # Panics
    /// Panics if the input length cannot be represented by the u64 counters.
    #[must_use]
    pub fn from_priorities(kind: Kind, priorities: &[u64]) -> Self {
        let entries: Vec<_> = priorities
            .iter()
            .enumerate()
            .map(|(id, &priority)| Entry {
                priority,
                arrival: u64::try_from(id).expect("IDs fit u64"),
                id,
                stamp: u64::try_from(id).expect("IDs fit u64") + 1,
            })
            .collect();
        let n = entries.len();
        let live = entries.iter().copied().map(Some).collect();
        let storage = match kind {
            Kind::Scan => Storage::Scan,
            Kind::Binary | Kind::BinaryPush => {
                Storage::Binary(Indexed::new(entries, n, matches!(kind, Kind::BinaryPush)))
            }
            Kind::Four => Storage::Four(Indexed::new(entries, n, false)),
            Kind::Eight => Storage::Eight(Indexed::new(entries, n, false)),
            Kind::Lazy | Kind::Compact => Storage::Lazy {
                heap: BinaryHeap::from(entries.into_iter().map(Reverse).collect::<Vec<_>>()),
                compact: matches!(kind, Kind::Compact),
            },
        };
        Self {
            live,
            storage,
            next_arrival: u64::try_from(n).expect("IDs fit u64"),
            next_stamp: u64::try_from(n).expect("IDs fit u64") + 1,
            len: n,
            peak_records: n,
            rebuilds: 0,
            stale_pops: 0,
        }
    }
    /// Live record count.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }
    /// Whether there are no live records, regardless of retained stale records.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// Insert or update; an update preserves the live job's original FIFO rank.
    ///
    /// # Errors
    /// Rejects an invalid ID or exhausted counter before mutation.
    pub fn set(&mut self, id: usize, priority: u64) -> Result<(), Error> {
        let old = *self.live.get(id).ok_or(Error::InvalidId)?;
        let next_stamp = self
            .next_stamp
            .checked_add(1)
            .ok_or(Error::CounterExhausted)?;
        let (arrival, next_arrival) = if let Some(e) = old {
            (e.arrival, self.next_arrival)
        } else {
            (
                self.next_arrival,
                self.next_arrival
                    .checked_add(1)
                    .ok_or(Error::CounterExhausted)?,
            )
        };
        let entry = Entry {
            priority,
            arrival,
            id,
            stamp: self.next_stamp,
        };
        match &mut self.storage {
            Storage::Scan => (),
            Storage::Binary(h) => h.set(entry),
            Storage::Four(h) => h.set(entry),
            Storage::Eight(h) => h.set(entry),
            Storage::Lazy { heap, .. } => heap.push(Reverse(entry)),
        }
        self.live[id] = Some(entry);
        self.next_stamp = next_stamp;
        self.next_arrival = next_arrival;
        if old.is_none() {
            self.len += 1;
        }
        self.maintain();
        Ok(())
    }
    /// Cancel a live ID; an absent valid ID returns false.
    ///
    /// # Errors
    /// Rejects IDs outside the fixed universe.
    pub fn cancel(&mut self, id: usize) -> Result<bool, Error> {
        let old = self.live.get_mut(id).ok_or(Error::InvalidId)?.take();
        if old.is_none() {
            return Ok(false);
        }
        match &mut self.storage {
            Storage::Binary(h) => {
                h.remove(id);
            }
            Storage::Four(h) => {
                h.remove(id);
            }
            Storage::Eight(h) => {
                h.remove(id);
            }
            Storage::Scan | Storage::Lazy { .. } => (),
        }
        self.len -= 1;
        self.maintain();
        Ok(true)
    }
    /// Return the earliest live record. Lazy validation work belongs to this call.
    pub fn peek(&mut self) -> Option<Entry> {
        match &mut self.storage {
            Storage::Scan => self.live.iter().flatten().copied().min(),
            Storage::Binary(h) => h.heap.first().copied(),
            Storage::Four(h) => h.heap.first().copied(),
            Storage::Eight(h) => h.heap.first().copied(),
            Storage::Lazy { heap, .. } => {
                while let Some(&Reverse(e)) = heap.peek() {
                    if self.live[e.id] == Some(e) {
                        return Some(e);
                    }
                    heap.pop();
                    self.stale_pops += 1;
                }
                None
            }
        }
    }
    /// Remove the earliest live record, preserving FIFO within equal priorities.
    pub fn pop(&mut self) -> Option<Entry> {
        let e = self.peek()?;
        match &mut self.storage {
            Storage::Binary(h) => {
                h.remove(e.id);
            }
            Storage::Four(h) => {
                h.remove(e.id);
            }
            Storage::Eight(h) => {
                h.remove(e.id);
            }
            Storage::Lazy { heap, .. } => {
                heap.pop();
            }
            Storage::Scan => (),
        }
        self.live[e.id] = None;
        self.len -= 1;
        self.maintain();
        Some(e)
    }
    fn maintain(&mut self) {
        if let Storage::Lazy { heap, compact } = &mut self.storage {
            self.peak_records = self.peak_records.max(heap.len());
            if *compact && heap.len() > self.len.saturating_mul(2).saturating_add(32) {
                // The authoritative universe scan is charged, even when live count is tiny.
                *heap = BinaryHeap::from(
                    self.live
                        .iter()
                        .flatten()
                        .copied()
                        .map(Reverse)
                        .collect::<Vec<_>>(),
                );
                self.rebuilds += 1;
            }
        }
    }
    /// Peak physical records, compactions and discarded stale records. Not allocation bytes.
    #[must_use]
    pub const fn counters(&self) -> (usize, usize, usize) {
        (self.peak_records, self.rebuilds, self.stale_pops)
    }
    /// Sorted authoritative live state, outside benchmark timing.
    #[must_use]
    pub fn snapshot(&self) -> Vec<Entry> {
        let mut v: Vec<_> = self.live.iter().flatten().copied().collect();
        v.sort_unstable();
        v
    }
    /// Check order, reverse maps, membership and lazy live-record coverage.
    #[must_use]
    pub fn valid(&self) -> bool {
        self.len == self.live.iter().flatten().count()
            && match &self.storage {
                Storage::Scan => true,
                Storage::Binary(h) => h.valid(&self.live),
                Storage::Four(h) => h.valid(&self.live),
                Storage::Eight(h) => h.valid(&self.live),
                Storage::Lazy { heap, compact } => {
                    let represented: BTreeSet<_> = heap.iter().map(|e| e.0).collect();
                    self.live.iter().flatten().all(|e| represented.contains(e))
                        && (!compact || heap.len() <= self.len.saturating_mul(2).saturating_add(32))
                }
            }
    }
}

/// Trace operation.
#[derive(Clone, Copy, Debug)]
pub enum Op {
    /// Insert/update.
    Set(usize, u64),
    /// Cancel.
    Cancel(usize),
    /// Remove minimum.
    Pop,
}

/// Independent ordered-map oracle; no heap helpers or production clocks are used.
pub struct Oracle {
    by_id: BTreeMap<usize, (u64, u64, u64)>,
    ordered: BTreeSet<(u64, u64, usize, u64)>,
    universe: usize,
    arrival: u64,
    stamp: u64,
}
impl Oracle {
    /// Initialize fixed IDs from slice order.
    #[must_use]
    pub fn new(priorities: &[u64]) -> Self {
        let mut model = Self {
            by_id: BTreeMap::new(),
            ordered: BTreeSet::new(),
            universe: priorities.len(),
            arrival: 0,
            stamp: 1,
        };
        for &p in priorities {
            let id = model.by_id.len();
            model.apply(Op::Set(id, p));
        }
        model
    }
    /// Apply a valid trace operation and return a removed record, if any.
    ///
    /// # Panics
    /// Panics on an invalid set ID or exhausted oracle counter.
    pub fn apply(&mut self, op: Op) -> Option<Entry> {
        match op {
            Op::Set(id, priority) => {
                assert!(id < self.universe);
                let birth = if let Some((p, a, s)) = self.by_id.remove(&id) {
                    self.ordered.remove(&(p, a, id, s));
                    a
                } else {
                    let a = self.arrival;
                    self.arrival = self.arrival.checked_add(1).expect("oracle arrival");
                    a
                };
                let stamp = self.stamp;
                self.stamp = self.stamp.checked_add(1).expect("oracle stamp");
                self.by_id.insert(id, (priority, birth, stamp));
                self.ordered.insert((priority, birth, id, stamp));
                None
            }
            Op::Cancel(id) => {
                if let Some((p, a, s)) = self.by_id.remove(&id) {
                    self.ordered.remove(&(p, a, id, s));
                }
                None
            }
            Op::Pop => {
                let (priority, arrival, id, stamp) = self.ordered.pop_first()?;
                self.by_id.remove(&id);
                Some(Entry {
                    priority,
                    arrival,
                    id,
                    stamp,
                })
            }
        }
    }
    /// Sorted complete live state, using the oracle's ordered tree.
    #[must_use]
    pub fn snapshot(&self) -> Vec<Entry> {
        self.ordered
            .iter()
            .map(|&(priority, arrival, id, stamp)| Entry {
                priority,
                arrival,
                id,
                stamp,
            })
            .collect()
    }
}

/// Execute one operation using the candidate.
///
/// # Panics
/// Panics if a set or cancel is rejected; benchmark traces use valid inputs.
pub fn apply(queue: &mut Queue, op: Op) -> Option<Entry> {
    match op {
        Op::Set(id, p) => {
            queue.set(id, p).expect("valid trace");
            None
        }
        Op::Cancel(id) => {
            queue.cancel(id).expect("valid trace");
            None
        }
        Op::Pop => queue.pop(),
    }
}

#[cfg(test)]
mod tests {
    #[allow(
        unused_imports,
        reason = "Cargo checks this module in the harness-free standalone bench too."
    )]
    use super::*;
    #[test]
    fn fifo_update_cancel_reinsert_and_stale_same_key() {
        for kind in Kind::ALL {
            let mut q = Queue::from_priorities(kind, &[5, 5, 5]);
            q.set(0, 1).unwrap();
            q.set(0, 5).unwrap();
            q.cancel(1).unwrap();
            q.set(1, 5).unwrap();
            assert_eq!(
                [
                    q.pop().unwrap().id,
                    q.pop().unwrap().id,
                    q.pop().unwrap().id
                ],
                [0, 2, 1]
            );
            assert_eq!(q.pop(), None);
            assert!(q.valid());
        }
    }
    #[test]
    fn invalid_and_exhausted_rejection_is_unchanged() {
        for kind in Kind::ALL {
            let mut q = Queue::from_priorities(kind, &[1, 2]);
            let before = q.snapshot();
            assert_eq!(q.set(2, 0), Err(Error::InvalidId));
            assert_eq!(q.cancel(2), Err(Error::InvalidId));
            q.next_stamp = u64::MAX;
            assert_eq!(q.set(0, 0), Err(Error::CounterExhausted));
            assert_eq!(q.snapshot(), before);
            q.next_stamp = 10;
            q.cancel(0).unwrap();
            q.next_arrival = u64::MAX;
            let before = q.snapshot();
            assert_eq!(q.set(0, 0), Err(Error::CounterExhausted));
            assert_eq!(q.snapshot(), before);
            assert!(q.valid());
        }
    }
    #[test]
    fn exhaustive_small_transitions_match_independent_trees() {
        let ops = [
            Op::Set(0, 0),
            Op::Set(0, 2),
            Op::Set(1, 0),
            Op::Set(1, 2),
            Op::Cancel(0),
            Op::Cancel(1),
            Op::Pop,
        ];
        for code in 0..7_usize.pow(5) {
            for kind in Kind::ALL {
                let mut q = Queue::from_priorities(kind, &[1, 1]);
                let mut m = Oracle::new(&[1, 1]);
                let mut c = code;
                for _ in 0..5 {
                    let op = ops[c % 7];
                    c /= 7;
                    assert_eq!(apply(&mut q, op), m.apply(op));
                    assert_eq!(q.snapshot(), m.snapshot());
                    assert!(q.valid());
                }
            }
        }
    }
    #[test]
    fn bulk_partial_children_both_directions_and_last_delete() {
        for n in 0..35 {
            let priorities: Vec<_> = (0..n).map(|i| (n - i) as u64).collect();
            for kind in Kind::ALL {
                let mut q = Queue::from_priorities(kind, &priorities);
                let mut m = Oracle::new(&priorities);
                for id in 0..n {
                    for p in [u64::MAX, 0, 5] {
                        let op = Op::Set(id, p);
                        assert_eq!(apply(&mut q, op), m.apply(op));
                        assert!(q.valid());
                    }
                }
                if n > 0 {
                    let op = Op::Cancel(n - 1);
                    apply(&mut q, op);
                    m.apply(op);
                }
                loop {
                    let expected = m.apply(Op::Pop);
                    assert_eq!(q.pop(), expected);
                    assert!(q.valid());
                    if expected.is_none() {
                        break;
                    }
                }
            }
        }
    }
    #[test]
    fn compaction_bounds_records_and_preserves_latest_entry() {
        let mut q = Queue::from_priorities(Kind::Compact, &[100_000]);
        for p in 0..1000 {
            q.set(0, p).unwrap();
            assert!(q.valid());
        }
        assert!(q.counters().1 > 0);
        assert_eq!(q.pop().unwrap().priority, 999);
        assert_eq!(q.pop(), None);
    }
}
