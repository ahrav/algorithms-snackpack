//! Fixed-capacity exact maps comparing local probing with cuckoo relocation.
//! All keys and values are `u64`. Rejected insertion preserves the whole map.

/// Deliberately deterministic layouts. `Mixed` is not a hash-collision denial-of-service defense.
#[derive(Clone, Copy, Debug)]
pub enum Layout {
    /// Two seeded integer mixes; no independence theorem is claimed.
    Mixed,
    /// Every key has the same pair, for an adversarial admission test.
    SamePair,
    /// Four named keys for the lesson's relocation example.
    Example,
}
fn mix(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}
fn pair(key: u64, buckets: usize, layout: Layout) -> [usize; 2] {
    let mask = buckets - 1;
    let raw = match layout {
        Layout::Mixed => [
            mix(key ^ 0x243f_6a88_85a3_08d3),
            mix(key ^ 0x1319_8a2e_0370_7344),
        ],
        Layout::SamePair => [0, 0],
        Layout::Example => match key {
            10 => [0, 1],
            20 | 40 | 50 => [0, 0],
            30 => [1, 1],
            _ => [2, 2],
        },
    };
    raw.map(|h| {
        usize::try_from(h & u64::try_from(mask).expect("mask fits u64"))
            .expect("masked hash fits usize")
    })
}
/// An exact entry. `Option<Entry>` reserves no key value as an empty sentinel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Exact key.
    pub key: u64,
    /// Associated value.
    pub value: u64,
}
/// Why an unchanged-map insertion was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    /// Every linear-probing slot is occupied.
    Full,
    /// The reachable cuckoo search component contains no vacancy.
    Exhausted,
    /// The finite planning budget was reached; feasibility is unknown.
    Budget,
}
/// Ownership of the incoming entry is returned on rejection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rejected {
    /// The entry that was not inserted.
    pub entry: Entry,
    /// Failure reason.
    pub reason: Reason,
}
/// Single-owner fixed-capacity exact-map contract.
pub trait Table {
    /// Return the exact value for a key.
    fn get(&self, key: u64) -> Option<u64>;
    /// Update duplicates without allocation. Return the previous value.
    /// On `Err`, all entries and values are unchanged.
    ///
    /// # Errors
    /// Returns the incoming entry when capacity or planning limits reject it.
    fn try_insert(&mut self, key: u64, value: u64) -> Result<Option<u64>, Rejected>;
    /// Remove a key, returning its previous value.
    fn remove(&mut self, key: u64) -> Option<u64>;
    /// Return sorted entries for an independent oracle.
    fn entries(&self) -> Vec<(u64, u64)>;
    /// Number of reserved slots; excludes allocator bookkeeping.
    fn capacity(&self) -> usize;
    /// Count slot examinations for a diagnostic query outside timing.
    fn examined(&self, key: u64) -> usize;
}
/// Simple linear-probing baseline with backward-shift deletion.
pub struct Linear {
    slots: Vec<Option<Entry>>,
    layout: Layout,
}
impl Linear {
    /// Reserve a positive power-of-two slot count.
    ///
    /// # Panics
    /// Panics when `capacity` is not a power of two.
    #[must_use]
    pub fn new(capacity: usize, layout: Layout) -> Self {
        assert!(capacity.is_power_of_two());
        Self {
            slots: vec![None; capacity],
            layout,
        }
    }
    fn home(&self, key: u64) -> usize {
        pair(key, self.slots.len(), self.layout)[0]
    }
    fn locate(&self, key: u64) -> (Option<usize>, usize) {
        let mask = self.slots.len() - 1;
        for d in 0..self.slots.len() {
            let i = (self.home(key) + d) & mask;
            match self.slots[i] {
                Some(e) if e.key == key => return (Some(i), d + 1),
                None => return (None, d + 1),
                _ => (),
            }
        }
        (None, self.slots.len())
    }
}
impl Table for Linear {
    fn get(&self, key: u64) -> Option<u64> {
        self.locate(key)
            .0
            .and_then(|i| self.slots[i].map(|e| e.value))
    }
    fn try_insert(&mut self, key: u64, value: u64) -> Result<Option<u64>, Rejected> {
        let mask = self.slots.len() - 1;
        for d in 0..self.slots.len() {
            let i = (self.home(key) + d) & mask;
            match self.slots[i] {
                Some(e) if e.key == key => {
                    self.slots[i] = Some(Entry { key, value });
                    return Ok(Some(e.value));
                }
                None => {
                    self.slots[i] = Some(Entry { key, value });
                    return Ok(None);
                }
                _ => (),
            }
        }
        Err(Rejected {
            entry: Entry { key, value },
            reason: Reason::Full,
        })
    }
    fn remove(&mut self, key: u64) -> Option<u64> {
        let (Some(mut hole), _) = self.locate(key) else {
            return None;
        };
        let old = self.slots[hole].take()?.value;
        let mask = self.slots.len() - 1;
        let mut next = (hole + 1) & mask;
        for _ in 0..self.slots.len() - 1 {
            let Some(e) = self.slots[next] else {
                break;
            };
            let origin = self.home(e.key);
            if (hole.wrapping_sub(origin) & mask) < (next.wrapping_sub(origin) & mask) {
                self.slots[hole] = self.slots[next].take();
                hole = next;
            }
            next = (next + 1) & mask;
        }
        Some(old)
    }
    fn entries(&self) -> Vec<(u64, u64)> {
        sorted(&self.slots)
    }
    fn capacity(&self) -> usize {
        self.slots.len()
    }
    fn examined(&self, key: u64) -> usize {
        self.locate(key).1
    }
}
fn sorted(slots: &[Option<Entry>]) -> Vec<(u64, u64)> {
    let mut entries: Vec<_> = slots.iter().flatten().map(|e| (e.key, e.value)).collect();
    entries.sort_unstable();
    entries
}
#[derive(Clone, Copy)]
struct Node {
    slot: usize,
    parent: Option<usize>,
}
/// Two banks of `B`-slot buckets. Search plans a path before moving any entry.
/// No stash, growth, rehash, or concurrent readers are implemented.
pub struct Cuckoo<const B: usize> {
    slots: Vec<Option<Entry>>,
    buckets: usize,
    layout: Layout,
    budget: usize,
}
impl<const B: usize> Cuckoo<B> {
    /// Reserve equal banks; `budget` caps distinct planned occupied slots.
    ///
    /// # Panics
    /// Panics unless `B > 0`, capacity is divisible by `2*B`, banks contain
    /// a power-of-two bucket count, and budget covers both root buckets.
    #[must_use]
    pub fn new(capacity: usize, layout: Layout, budget: usize) -> Self {
        assert!(B > 0 && capacity.is_multiple_of(2 * B));
        let buckets = capacity / (2 * B);
        assert!(buckets.is_power_of_two() && budget >= 2 * B);
        Self {
            slots: vec![None; capacity],
            buckets,
            layout,
            budget,
        }
    }
    fn starts(&self, key: u64) -> [usize; 2] {
        let p = pair(key, self.buckets, self.layout);
        [p[0] * B, (self.buckets + p[1]) * B]
    }
    fn locate(&self, key: u64) -> (Option<usize>, usize) {
        let mut count = 0;
        for start in self.starts(key) {
            for i in start..start + B {
                count += 1;
                if self.slots[i].is_some_and(|e| e.key == key) {
                    return (Some(i), count);
                }
            }
        }
        (None, count)
    }
    fn commit(&mut self, nodes: &[Node], mut parent: usize, mut vacancy: usize, entry: Entry) {
        loop {
            let node = nodes[parent];
            self.slots[vacancy] = self.slots[node.slot].take();
            vacancy = node.slot;
            match node.parent {
                Some(p) => parent = p,
                None => break,
            }
        }
        self.slots[vacancy] = Some(entry);
    }
}
impl<const B: usize> Table for Cuckoo<B> {
    fn get(&self, key: u64) -> Option<u64> {
        self.locate(key)
            .0
            .and_then(|i| self.slots[i].map(|e| e.value))
    }
    fn try_insert(&mut self, key: u64, value: u64) -> Result<Option<u64>, Rejected> {
        let entry = Entry { key, value };
        if let (Some(i), _) = self.locate(key) {
            let old = self.slots[i]
                .replace(entry)
                .expect("located occupied entry");
            return Ok(Some(old.value));
        }
        let starts = self.starts(key);
        for start in starts {
            for i in start..start + B {
                if self.slots[i].is_none() {
                    self.slots[i] = Some(entry);
                    return Ok(None);
                }
            }
        }
        let mut nodes = Vec::with_capacity(self.budget);
        for start in starts {
            for slot in start..start + B {
                nodes.push(Node { slot, parent: None });
            }
        }
        let mut cursor = 0;
        let mut budget_hit = false;
        while cursor < nodes.len() {
            let node = nodes[cursor];
            let occupant = self.slots[node.slot].expect("planning reads occupied slots");
            let bank = node.slot / (self.buckets * B);
            let alternate = self.starts(occupant.key)[1 - bank];
            for slot in alternate..alternate + B {
                if self.slots[slot].is_none() {
                    self.commit(&nodes, cursor, slot, entry);
                    return Ok(None);
                }
                if nodes.iter().all(|n| n.slot != slot) {
                    if nodes.len() == self.budget {
                        budget_hit = true;
                    } else {
                        nodes.push(Node {
                            slot,
                            parent: Some(cursor),
                        });
                    }
                }
            }
            cursor += 1;
        }
        Err(Rejected {
            entry,
            reason: if budget_hit {
                Reason::Budget
            } else {
                Reason::Exhausted
            },
        })
    }
    fn remove(&mut self, key: u64) -> Option<u64> {
        self.locate(key)
            .0
            .and_then(|i| self.slots[i].take().map(|e| e.value))
    }
    fn entries(&self) -> Vec<(u64, u64)> {
        sorted(&self.slots)
    }
    fn capacity(&self) -> usize {
        self.slots.len()
    }
    fn examined(&self, key: u64) -> usize {
        self.locate(key).1
    }
}
/// Run a bounded differential trace. Rejected operations leave the oracle unchanged.
///
/// # Panics
/// Panics on a mismatch, lost entry, duplicate, or changed rejection state.
pub fn check<T: Table>(mut table: T, steps: u64) {
    let mut model = std::collections::BTreeMap::new();
    for step in 0..steps {
        let key = (step.wrapping_mul(71) ^ (step / 11)) % 97;
        match step % 5 {
            0 => assert_eq!(table.remove(key), model.remove(&key)),
            1 => assert_eq!(table.get(key), model.get(&key).copied()),
            _ => {
                let before = table.entries();
                match table.try_insert(key, step) {
                    Ok(old) => assert_eq!(old, model.insert(key, step)),
                    Err(e) => {
                        assert_eq!(e.entry, Entry { key, value: step });
                        assert_eq!(table.entries(), before);
                    }
                }
            }
        }
        assert_eq!(
            table.entries(),
            model.iter().map(|(&k, &v)| (k, v)).collect::<Vec<_>>()
        );
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn oracle_mixed_and_colliding() {
        for layout in [Layout::Mixed, Layout::SamePair] {
            for capacity in [8, 32, 128] {
                check(Linear::new(capacity, layout), 2000);
                check(Cuckoo::<1>::new(capacity, layout, 64), 2000);
                check(Cuckoo::<4>::new(capacity, layout, 64), 2000);
            }
        }
    }
    #[test]
    fn relocation_and_global_slack_obstruction() {
        let mut t = Cuckoo::<1>::new(8, Layout::Example, 64);
        for key in [10, 20, 30, 40] {
            assert_eq!(t.try_insert(key, key + 1), Ok(None));
        }
        assert_eq!(t.slots[0].map(|e| e.key), Some(40));
        assert_eq!(t.slots[5].map(|e| e.key), Some(10));
        let before = t.entries();
        assert_eq!(
            t.try_insert(50, 51).map_err(|e| e.reason),
            Err(Reason::Exhausted)
        );
        assert_eq!(t.entries(), before);
        assert_eq!(t.entries().len(), 4);
        assert_eq!(t.capacity(), 8);
    }
    #[test]
    fn duplicate_update_on_full_pair_and_deletion() {
        let mut t = Cuckoo::<4>::new(32, Layout::SamePair, 64);
        for key in 0..8 {
            assert_eq!(t.try_insert(key, key), Ok(None));
        }
        assert_eq!(t.try_insert(3, 999), Ok(Some(3)));
        let before = t.entries();
        assert!(t.try_insert(8, 8).is_err());
        assert_eq!(t.entries(), before);
        assert_eq!(t.remove(3), Some(999));
        assert_eq!(t.remove(3), None);
        assert_eq!(t.try_insert(u64::MAX, 42), Ok(None));
        assert_eq!(t.get(u64::MAX), Some(42));
    }
    #[test]
    fn finite_budget_is_not_infeasibility() {
        // Find a deterministic valid insertion needing more than two planned roots.
        let mut small = Cuckoo::<1>::new(256, Layout::Mixed, 2);
        let mut large = Cuckoo::<1>::new(256, Layout::Mixed, 64);
        let mut found = false;
        for key in 0..120 {
            let before = small.entries();
            let a = small.try_insert(key, key);
            let b = large.try_insert(key, key);
            if a.is_err_and(|e| e.reason == Reason::Budget) && b.is_ok() {
                assert_eq!(small.entries(), before);
                found = true;
                break;
            }
            assert_eq!(a, b);
        }
        assert!(found, "frozen valid-but-budget-rejected witness");
    }
    #[test]
    fn budget_is_charged_only_for_recorded_slots() {
        // Key 27's search reaches the cap at an unrecorded occupied slot; a
        // vacancy scanned after that point needs no new record.
        let mut t = Cuckoo::<4>::new(32, Layout::Mixed, 8);
        for key in 0..27 {
            assert_eq!(t.try_insert(key, key), Ok(None), "key {key}");
        }
        assert_eq!(t.try_insert(27, 27), Ok(None));
        assert_eq!(t.entries().len(), 28);
    }
    #[test]
    fn all_keys_are_legal_and_queries_bounded() {
        let mut t = Cuckoo::<4>::new(32, Layout::Mixed, 64);
        for key in [0, u64::MAX, 1] {
            assert!(t.try_insert(key, key).is_ok());
        }
        for key in [0, 1, 3, u64::MAX] {
            assert!(t.examined(key) <= 8);
        }
    }
    #[test]
    fn full_linear_delete_wraparound() {
        let mut t = Linear::new(8, Layout::SamePair);
        for key in 0..8 {
            t.try_insert(key, key).unwrap();
        }
        assert_eq!(t.remove(0), Some(0));
        for key in 1..8 {
            assert_eq!(t.get(key), Some(key));
        }
        assert_eq!(t.try_insert(9, 9), Ok(None));
    }
}
