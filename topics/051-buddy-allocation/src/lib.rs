//! Fixed-arena, lowest-address, power-of-two reservation portfolios.
use std::collections::BTreeSet;

/// Reservation metadata for one serial arena. IDs identify live allocations.
pub struct Pool {
    n: usize,
    live: Vec<Option<(usize, usize)>>,
    arena: Arena,
    work: u64,
}
enum Arena {
    Scan(Vec<bool>),
    Buddy(Vec<BTreeSet<usize>>),
    Tree(Vec<usize>),
}
impl Pool {
    /// Creates an empty arena and a fixed live-ID table.
    ///
    /// # Panics
    /// Panics unless `n` is a power of two in `1..=1_048_576`, or kind is unknown.
    #[must_use]
    pub fn new(kind: &str, n: usize, ids: usize) -> Self {
        assert!(n.is_power_of_two() && n <= 1_048_576);
        let arena = match kind {
            "scan" => Arena::Scan(vec![false; n]),
            "buddy" => {
                let mut sets = vec![BTreeSet::new(); n.ilog2() as usize + 1];
                sets[n.ilog2() as usize].insert(0);
                Arena::Buddy(sets)
            }
            "tree" => {
                let mut tree = vec![0; 2 * n];
                for (i, value) in tree.iter_mut().enumerate().skip(1) {
                    *value = n >> i.ilog2();
                }
                Arena::Tree(tree)
            }
            _ => panic!("unknown kind"),
        };
        Self {
            n,
            live: vec![None; ids],
            arena,
            work: 0,
        }
    }
    /// Reserves the lowest aligned power-of-two block. Failure preserves state.
    pub fn alloc(&mut self, id: usize, request: usize) -> Option<usize> {
        if id >= self.live.len() || self.live[id].is_some() || request == 0 {
            return None;
        }
        let size = request.checked_next_power_of_two()?;
        if size > self.n {
            return None;
        }
        let start = match &mut self.arena {
            Arena::Scan(bits) => {
                let mut found = None;
                for a in (0..self.n).step_by(size) {
                    let mut available = true;
                    for used in &bits[a..a + size] {
                        self.work += 1;
                        if *used {
                            available = false;
                            break;
                        }
                    }
                    if available {
                        found = Some(a);
                        break;
                    }
                }
                let a = found?;
                bits[a..a + size].fill(true);
                a
            }
            Arena::Buddy(sets) => {
                let order = size.ilog2() as usize;
                let mut best: Option<(usize, usize)> = None;
                for (k, set) in sets.iter().enumerate().skip(order) {
                    self.work += 1;
                    if let Some(&a) = set.first()
                        && best.is_none_or(|(b, _)| a < b)
                    {
                        best = Some((a, k));
                    }
                }
                let (a, mut k) = best?;
                sets[k].remove(&a);
                while k > order {
                    k -= 1;
                    self.work += 1;
                    sets[k].insert(a + (1 << k));
                }
                a
            }
            Arena::Tree(tree) => {
                self.work += 1;
                if tree[1] < size {
                    return None;
                }
                let mut node = 1;
                let mut width = self.n;
                while width > size {
                    node *= 2;
                    width /= 2;
                    self.work += 1;
                    if tree[node] < size {
                        node += 1;
                    }
                }
                tree[node] = 0;
                let a = (node - self.n / size) * size;
                repair(tree, node, size, &mut self.work);
                a
            }
        };
        self.live[id] = Some((start, size));
        Some(start)
    }
    /// Releases a live ID once. Invalid or repeated frees preserve state.
    pub fn free(&mut self, id: usize) -> bool {
        let Some(slot) = self.live.get_mut(id) else {
            return false;
        };
        let Some((mut a, size)) = slot.take() else {
            return false;
        };
        match &mut self.arena {
            Arena::Scan(bits) => {
                bits[a..a + size].fill(false);
                self.work += size as u64;
            }
            Arena::Buddy(sets) => {
                let mut k = size.ilog2() as usize;
                while k + 1 < sets.len() {
                    self.work += 1;
                    let buddy = a ^ (1 << k);
                    if !sets[k].remove(&buddy) {
                        break;
                    }
                    a = a.min(buddy);
                    k += 1;
                }
                sets[k].insert(a);
            }
            Arena::Tree(tree) => {
                let node = self.n / size + a / size;
                tree[node] = size;
                repair(tree, node, size, &mut self.work);
            }
        }
        true
    }
    /// Logical probes and summary updates, specific to each representation.
    #[must_use]
    pub fn work(&self) -> u64 {
        self.work
    }
    /// Complete reservation state reconstructed from live IDs.
    #[must_use]
    pub fn occupied(&self) -> Vec<u8> {
        let mut bits = vec![0; self.n];
        for &(a, s) in self.live.iter().flatten() {
            bits[a..a + s].fill(1);
        }
        bits
    }
}
fn repair(tree: &mut [usize], mut node: usize, mut width: usize, work: &mut u64) {
    while node > 1 {
        node /= 2;
        let left = tree[node * 2];
        let right = tree[node * 2 + 1];
        tree[node] = if left == width && right == width {
            width * 2
        } else {
            left.max(right)
        };
        width *= 2;
        *work += 1;
    }
}
/// One deterministic reservation operation.
#[derive(Clone, Copy, Debug)]
pub enum Op {
    /// Allocate an ID with a positive requested size.
    Alloc(usize, usize),
    /// Release an ID.
    Free(usize),
}
/// Fixed synthetic traces. Each round ends empty, except the fragmentation probe.
///
/// # Panics
/// Panics for an unknown workload.
#[must_use]
pub fn workload(n: usize, shape: &str) -> Vec<Op> {
    let mut ops = Vec::new();
    match shape {
        "holes" => {
            for id in 0..n {
                ops.push(Op::Alloc(id, 1));
            }
            for id in (0..n).step_by(2) {
                ops.push(Op::Free(id));
            }
            for id in (0..n).step_by(2) {
                ops.push(Op::Alloc(id, 2));
            }
            for id in 0..n {
                ops.push(Op::Free(id));
            }
        }
        "tiny" | "mixed" | "churn" => {
            for round in 0..8 {
                let count = if shape == "churn" { 8 } else { n / 2 };
                for id in 0..count {
                    let request = match shape {
                        "tiny" => 1,
                        "mixed" => [1, 3, 5, 9, 17, 33][(id + round) % 6],
                        _ => [n / 2 - 1, 3, 1, n / 4 + 1][id % 4],
                    };
                    ops.push(Op::Alloc(id, request));
                }
                for id in (0..count).rev() {
                    ops.push(Op::Free(id));
                }
            }
        }
        _ => panic!("unknown workload"),
    }
    ops
}
