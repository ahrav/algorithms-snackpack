//! Bounded interleaving model for concurrent graph marking.
#![forbid(unsafe_code)]

use std::collections::{HashSet, VecDeque};

const NODES: usize = 4;
const SLOTS: usize = 2;
const ROOT: usize = 0;
type Edges = [[Option<usize>; SLOTS]; NODES];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Color {
    White,
    Gray,
    Black,
}

/// Select the pointer a write barrier shades.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Barrier {
    /// Store without recording the pointer.
    None,
    /// Shade the new target of a store into a scanned node.
    Insertion,
    /// Shade the overwritten target before storing the new pointer.
    Deletion,
}

/// Select the work visible to the termination check.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Termination {
    /// Look only at the shared work list.
    SharedOnly,
    /// Check the shared list, private buffer and held scan.
    Accounted,
}

/// Initial heap with a single fixed root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Heap {
    /// Root reaches two branches, one of which reaches the fourth node.
    Fork,
    /// A root path leads into a cycle.
    Cycle,
    /// Two nodes start unreachable from the root.
    Disconnected,
}

impl Heap {
    const fn edges(self) -> Edges {
        match self {
            Self::Fork => [
                [Some(1), Some(2)],
                [None, None],
                [Some(3), None],
                [None, None],
            ],
            Self::Cycle => [
                [Some(1), None],
                [Some(2), None],
                [Some(1), Some(3)],
                [Some(0), None],
            ],
            Self::Disconnected => [
                [Some(1), None],
                [None, None],
                [Some(3), None],
                [Some(2), None],
            ],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct State {
    edges: Edges,
    colors: [Color; NODES],
    shared: u8,
    private: u8,
    held: Option<usize>,
    writes: u8,
}

const fn bit(node: usize) -> u8 {
    1 << node
}

impl State {
    const fn start(heap: Heap) -> Self {
        Self {
            edges: heap.edges(),
            colors: [Color::Gray, Color::White, Color::White, Color::White],
            shared: bit(ROOT),
            private: 0,
            held: None,
            writes: 0,
        }
    }

    fn shade(&mut self, node: Option<usize>) {
        if let Some(node) = node
            && self.colors[node] == Color::White
        {
            self.colors[node] = Color::Gray;
            self.private |= bit(node);
        }
    }

    fn write(&mut self, barrier: Barrier, owner: usize, slot: usize, next: Option<usize>) {
        match barrier {
            Barrier::Insertion if self.colors[owner] == Color::Black => self.shade(next),
            Barrier::Deletion => self.shade(self.edges[owner][slot]),
            Barrier::None | Barrier::Insertion => {}
        }
        self.edges[owner][slot] = next;
        self.writes += 1;
    }

    fn take(&mut self, node: usize) {
        self.shared &= !bit(node);
        self.held = Some(node);
    }

    fn flush(&mut self) {
        self.shared |= self.private;
        self.private = 0;
    }

    fn scan(&mut self, node: usize) {
        debug_assert_eq!(self.held, Some(node));
        for target in self.edges[node] {
            self.shade(target);
        }
        self.held = None;
        self.colors[node] = Color::Black;
    }

    const fn terminates(self, termination: Termination) -> bool {
        self.shared == 0
            && match termination {
                Termination::SharedOnly => true,
                Termination::Accounted => self.private == 0 && self.held.is_none(),
            }
    }

    fn black(self) -> u8 {
        self.colors
            .iter()
            .enumerate()
            .filter(|(_, color)| **color == Color::Black)
            .fold(0, |mask, (node, _)| mask | bit(node))
    }

    fn successors(self, barrier: Barrier, max_writes: u8, live: u8) -> Vec<Self> {
        let mut next = Vec::new();
        if let Some(node) = self.held {
            let mut scanned = self;
            scanned.scan(node);
            next.push(scanned);
        } else {
            for node in 0..NODES {
                if self.shared & bit(node) != 0 {
                    let mut taken = self;
                    taken.take(node);
                    next.push(taken);
                }
            }
        }
        if self.private != 0 {
            let mut flushed = self;
            flushed.flush();
            next.push(flushed);
        }
        if self.writes < max_writes {
            for owner in 0..NODES {
                if live & bit(owner) == 0 {
                    continue;
                }
                for slot in 0..SLOTS {
                    for target in (0..NODES)
                        .filter(|node| live & bit(*node) != 0)
                        .map(Some)
                        .chain(std::iter::once(None))
                    {
                        if self.edges[owner][slot] != target {
                            let mut written = self;
                            written.write(barrier, owner, slot, target);
                            next.push(written);
                        }
                    }
                }
            }
        }
        next
    }
}

// Transitive closure uses the edge relation alone. It ignores all marking state.
fn reachable(edges: &Edges) -> u8 {
    let mut reach = [[false; NODES]; NODES];
    for (node, targets) in edges.iter().enumerate() {
        reach[node][node] = true;
        for &target in targets.iter().flatten() {
            reach[node][target] = true;
        }
    }
    for intermediate in 0..NODES {
        let through = reach[intermediate];
        for row in &mut reach {
            if row[intermediate] {
                for (target, via) in row.iter_mut().zip(through) {
                    *target |= via;
                }
            }
        }
    }
    reach[ROOT]
        .iter()
        .enumerate()
        .filter(|(_, yes)| **yes)
        .fold(0, |mask, (node, _)| mask | bit(node))
}

/// Counts of distinct model states and failures at possible termination points.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct Summary {
    /// Distinct states visited.
    pub states: usize,
    /// States where the selected termination check can succeed.
    pub termination_points: usize,
    /// Termination points with a live node that has not been scanned.
    pub live_misses: usize,
    /// Termination points with an initially reachable node that has not been scanned.
    pub snapshot_misses: usize,
    /// Termination points that retain at least one node outside the current live set.
    pub extra_retention: usize,
}

/// Explore all interleavings within the admitted write bound.
///
/// # Panics
///
/// Panics if the bound exceeds two writes. This keeps the exhaustive search small.
#[must_use]
pub fn explore(heap: Heap, barrier: Barrier, termination: Termination, max_writes: u8) -> Summary {
    assert!(max_writes <= 2);
    let initial = State::start(heap);
    let snapshot = reachable(&initial.edges);
    let mut found = HashSet::from([initial]);
    let mut queue = VecDeque::from([initial]);
    let mut report = Summary::default();
    while let Some(state) = queue.pop_front() {
        let live = reachable(&state.edges);
        let black = state.black();
        if state.terminates(termination) {
            report.termination_points += 1;
            report.live_misses += usize::from(live & !black != 0);
            report.snapshot_misses += usize::from(snapshot & !black != 0);
            report.extra_retention += usize::from(black & !live != 0);
        }
        for next in state.successors(barrier, max_writes, live) {
            if found.insert(next) {
                queue.push_back(next);
            }
        }
    }
    report.states = found.len();
    report
}

/// Return whether the fourth node is marked after moving its last incoming edge.
///
/// # Panics
///
/// Panics if the fixed witness fails its reachability or work accounting checks.
#[must_use]
pub fn moved_pointer_is_marked(barrier: Barrier) -> bool {
    let mut state = State::start(Heap::Fork);
    state.take(ROOT);
    state.scan(ROOT);
    state.flush();
    state.take(1);
    state.scan(1);
    state.write(barrier, 1, 0, Some(3));
    state.write(barrier, 2, 0, None);
    state.take(2);
    state.scan(2);
    state.flush();
    if state.shared & bit(3) != 0 {
        state.take(3);
        state.scan(3);
    }
    assert!(state.terminates(Termination::Accounted));
    assert_ne!(reachable(&state.edges) & bit(3), 0);
    state.black() & bit(3) != 0
}
