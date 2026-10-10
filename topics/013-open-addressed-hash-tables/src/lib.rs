//! Linear-probing deletion portfolio and a bounded extendible-hashing model.
//! All integer keys/values are valid. These are single-threaded educational maps.
#![forbid(unsafe_code)]

/// Deletion and maintenance choice; the probe sequence is always linear.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    /// Preserve deleted slots until reused or live occupancy forces growth.
    Lazy,
    /// Rebuild before a new insert if deleted slots exceed one eighth capacity.
    Rebuild,
    /// Move only entries whose cyclic probe path crosses the deletion hole.
    Shift,
}
/// Hash input for controlled experiments. Neither option resists hostile keys.
#[derive(Clone, Copy, Debug)]
pub enum HashMode {
    /// Fixed integer mixer; not a randomized independence guarantee.
    Mixed,
    /// Low key bits determine the home slot, exposing adversarial clusters.
    Identity,
}
impl HashMode {
    fn hash(self, mut x: u64) -> u64 {
        if matches!(self, Self::Identity) {
            return x;
        }
        x ^= x >> 30;
        x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
        x ^= x >> 27;
        x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
        x ^ (x >> 31)
    }
}
#[derive(Clone, Copy, Debug)]
enum Slot {
    Empty,
    Deleted,
    Full(u64, u64),
}
/// Observable maintenance work, not a CPU-performance explanation.
#[derive(Clone, Copy, Debug, Default)]
pub struct Work {
    /// Number of complete array rebuilds including growth.
    pub rebuilds: usize,
    /// Live entries reinserted by rebuilds.
    pub reinserted: usize,
    /// Entries moved by backward shift.
    pub shifted: usize,
}
/// Growable map with bounded probes, explicit deleted state, and borrowed iteration.
#[derive(Clone, Debug)]
pub struct LinearMap {
    slots: Vec<Slot>,
    len: usize,
    deleted: usize,
    policy: Policy,
    mode: HashMode,
    work: Work,
}
impl LinearMap {
    /// Allocate at least eight power-of-two slots. Allocation failure panics.
    ///
    /// # Panics
    /// Panics if capacity rounding overflows or allocation fails.
    #[must_use]
    pub fn new(capacity: usize, policy: Policy, mode: HashMode) -> Self {
        Self {
            slots: vec![Slot::Empty; capacity.max(8).next_power_of_two()],
            len: 0,
            deleted: 0,
            policy,
            mode,
            work: Work::default(),
        }
    }
    /// Number of live keys.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }
    /// Whether no live keys remain.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// Slot count, including empty and deleted slots.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }
    /// Number of deleted slots that do not terminate lookup.
    #[must_use]
    pub fn deleted(&self) -> usize {
        self.deleted
    }
    /// Maintenance totals since construction.
    #[must_use]
    pub fn work(&self) -> Work {
        self.work
    }
    #[allow(clippy::cast_possible_truncation)]
    fn home(&self, key: u64) -> usize {
        (self.mode.hash(key) as usize) & (self.capacity() - 1)
    }
    fn find(&self, key: u64) -> (Option<usize>, usize) {
        let mask = self.capacity() - 1;
        let home = self.home(key);
        for step in 0..self.capacity() {
            let i = (home + step) & mask;
            match self.slots[i] {
                Slot::Empty => return (None, step + 1),
                Slot::Full(k, _) if k == key => return (Some(i), step + 1),
                _ => (),
            }
        }
        (None, self.capacity())
    }
    /// Lookup, stopping at a true empty slot or after one full circuit.
    #[must_use]
    pub fn get(&self, key: u64) -> Option<u64> {
        self.find(key).0.and_then(|i| match self.slots[i] {
            Slot::Full(_, v) => Some(v),
            _ => None,
        })
    }
    /// Count inspected slots for a lookup, including the terminating slot.
    #[must_use]
    pub fn lookup_probes(&self, key: u64) -> usize {
        self.find(key).1
    }
    fn place_new(&mut self, key: u64, value: u64) {
        let mask = self.capacity() - 1;
        let home = self.home(key);
        for step in 0..self.capacity() {
            let i = (home + step) & mask;
            if !matches!(self.slots[i], Slot::Full(_, _)) {
                if matches!(self.slots[i], Slot::Deleted) {
                    self.deleted -= 1;
                }
                self.slots[i] = Slot::Full(key, value);
                self.len += 1;
                return;
            }
        }
        unreachable!("live load remains at most seven eighths")
    }
    fn rehash(&mut self, capacity: usize) {
        let old = std::mem::replace(&mut self.slots, vec![Slot::Empty; capacity]);
        self.work.rebuilds += 1;
        self.work.reinserted += self.len;
        self.len = 0;
        self.deleted = 0;
        for slot in old {
            if let Slot::Full(k, v) = slot {
                self.place_new(k, v);
            }
        }
    }
    /// Insert or replace. Check for an existing key before reusing a tombstone.
    /// Live occupancy grows above seven eighths; Rebuild cleans above one eighth deleted.
    ///
    /// # Panics
    /// Panics if growth capacity overflows or allocation fails.
    pub fn insert(&mut self, key: u64, value: u64) -> Option<u64> {
        if let Some(i) = self.find(key).0
            && let Slot::Full(_, old) = self.slots[i]
        {
            self.slots[i] = Slot::Full(key, value);
            return Some(old);
        }
        if self.len + 1 > self.capacity() - self.capacity() / 8 {
            self.rehash(self.capacity().checked_mul(2).expect("capacity overflow"));
        } else if self.policy == Policy::Rebuild && self.deleted > self.capacity() / 8 {
            self.rehash(self.capacity());
        }
        self.place_new(key, value);
        None
    }
    /// Remove without breaking reachability. Shift applies only to this linear sequence.
    pub fn remove(&mut self, key: u64) -> Option<u64> {
        let i = self.find(key).0?;
        let Slot::Full(_, value) = self.slots[i] else {
            unreachable!()
        };
        self.len -= 1;
        if self.policy != Policy::Shift {
            self.slots[i] = Slot::Deleted;
            self.deleted += 1;
            return Some(value);
        }
        let mask = self.capacity() - 1;
        let mut hole = i;
        self.slots[hole] = Slot::Empty;
        for step in 1..self.capacity() {
            let pos = (i + step) & mask;
            let Slot::Full(k, v) = self.slots[pos] else {
                break;
            };
            let origin = self.home(k);
            if ((hole.wrapping_sub(origin)) & mask) < ((pos.wrapping_sub(origin)) & mask) {
                self.slots[hole] = Slot::Full(k, v);
                self.slots[pos] = Slot::Empty;
                hole = pos;
                self.work.shifted += 1;
            }
        }
        Some(value)
    }
    /// Borrowed iteration has no order guarantee. Structural mutation requires ending the borrow.
    ///
    /// ```compile_fail
    /// use open_addressed_hash_tables::{LinearMap, Policy, HashMode};
    /// let mut map = LinearMap::new(8, Policy::Shift, HashMode::Identity);
    /// map.insert(1, 10);
    /// for (key, _) in map.iter() { map.insert(*key + 1, 20); }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (&u64, &u64)> {
        self.slots.iter().filter_map(|slot| match slot {
            Slot::Full(k, v) => Some((k, v)),
            _ => None,
        })
    }
    /// Visit mutable values while preserving keys and structural ownership.
    pub fn visit_values(&mut self, mut f: impl FnMut(u64, &mut u64)) {
        for slot in &mut self.slots {
            if let Slot::Full(k, v) = slot {
                f(*k, v);
            }
        }
    }
    /// An owned key snapshot allows later structural mutation. Values are read later.
    #[must_use]
    pub fn snapshot_keys(&self) -> Vec<u64> {
        self.iter().map(|(k, _)| *k).collect()
    }
}
#[derive(Debug)]
struct Bucket {
    depth: u8,
    rows: Vec<(u64, u64)>,
}
/// Operation-count demonstrator of low-bit extendible hashing; no latency claim.
/// Directory depth is capped at 12, then buckets use vector overflow. No merging.
#[derive(Debug)]
pub struct ExtendibleMap {
    directory: Vec<usize>,
    buckets: Vec<Bucket>,
    depth: u8,
    bucket_limit: usize,
    pointer_visits: usize,
    moved: usize,
}
impl ExtendibleMap {
    /// Create one bucket; zero bucket limit becomes one. Uses identity hashing.
    #[must_use]
    pub fn new(bucket_limit: usize) -> Self {
        Self {
            directory: vec![0],
            buckets: vec![Bucket {
                depth: 0,
                rows: vec![],
            }],
            depth: 0,
            bucket_limit: bucket_limit.max(1),
            pointer_visits: 0,
            moved: 0,
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    fn index(&self, key: u64) -> usize {
        (key as usize) & (self.directory.len() - 1)
    }
    /// Lookup in the selected local bucket, including capped-depth overflow.
    #[must_use]
    pub fn get(&self, key: u64) -> Option<u64> {
        self.buckets[self.directory[self.index(key)]]
            .rows
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| *v)
    }
    /// Insert or update, splitting one bucket at a time; repeated splits are possible.
    pub fn insert(&mut self, key: u64, value: u64) -> Option<u64> {
        loop {
            let id = self.directory[self.index(key)];
            if let Some((_, v)) = self.buckets[id].rows.iter_mut().find(|(k, _)| *k == key) {
                return Some(std::mem::replace(v, value));
            }
            if self.buckets[id].rows.len() < self.bucket_limit || self.buckets[id].depth == 12 {
                self.buckets[id].rows.push((key, value));
                return None;
            }
            let bit = self.buckets[id].depth;
            if bit == self.depth {
                self.pointer_visits += self.directory.len();
                self.directory.extend_from_within(..);
                self.depth += 1;
            }
            let new_id = self.buckets.len();
            let rows = std::mem::take(&mut self.buckets[id].rows);
            self.moved += rows.len();
            self.buckets[id].depth += 1;
            self.buckets.push(Bucket {
                depth: bit + 1,
                rows: vec![],
            });
            for row in rows {
                let dest = if (row.0 >> bit) & 1 == 0 { id } else { new_id };
                self.buckets[dest].rows.push(row);
            }
            self.pointer_visits += self.directory.len();
            for (i, p) in self.directory.iter_mut().enumerate() {
                if *p == id && ((i >> bit) & 1) == 1 {
                    *p = new_id;
                }
            }
        }
    }
    /// Remove a key. This model does not coalesce buckets or shrink its directory.
    pub fn remove(&mut self, key: u64) -> Option<u64> {
        let id = self.directory[self.index(key)];
        let pos = self.buckets[id].rows.iter().position(|(k, _)| *k == key)?;
        Some(self.buckets[id].rows.swap_remove(pos).1)
    }
    /// Directory size, allocated buckets, pointer visits, and redistributed rows.
    #[must_use]
    pub fn stats(&self) -> (usize, usize, usize, usize) {
        (
            self.directory.len(),
            self.buckets.len(),
            self.pointer_visits,
            self.moved,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    #[test]
    fn deletion_update_wrap_and_extremes() {
        for policy in [Policy::Lazy, Policy::Rebuild, Policy::Shift] {
            let mut map = LinearMap::new(8, policy, HashMode::Identity);
            for k in [7, 15, 23, u64::MAX] {
                assert_eq!(map.insert(k, k), None);
            }
            assert_eq!(map.remove(7), Some(7));
            assert_eq!(map.insert(23, 42), Some(23));
            assert_eq!(map.len(), 3);
            for k in [15, 23, u64::MAX] {
                assert!(map.get(k).is_some());
            }
            assert_eq!(map.get(31), None);
            assert_eq!(map.insert(0, 0), None);
        }
    }
    #[test]
    fn all_used_but_not_all_live_terminates() {
        let mut map = LinearMap::new(8, Policy::Lazy, HashMode::Identity);
        for k in 0..8 {
            map.insert(k, k);
            map.remove(k);
        }
        assert_eq!(map.deleted(), 8);
        assert_eq!(map.lookup_probes(16), 8);
        assert_eq!(map.get(16), None);
        map.insert(16, 99);
        assert_eq!(map.get(16), Some(99));
    }
    #[test]
    fn do_not_shift_key_before_its_home() {
        let mut map = LinearMap::new(8, Policy::Shift, HashMode::Identity);
        for k in [1, 2, 9] {
            map.insert(k, k);
        }
        map.remove(1);
        assert_eq!(map.get(2), Some(2));
        assert_eq!(map.get(9), Some(9));
        assert_eq!(map.lookup_probes(2), 1);
        assert_eq!(map.lookup_probes(9), 1);
    }
    #[test]
    fn bounded_exhaustive_histories() {
        // 8 choices at each of five steps: inserts/removes on four colliding keys.
        for history in 0_u32..32_768 {
            for policy in [Policy::Lazy, Policy::Rebuild, Policy::Shift] {
                let mut map = LinearMap::new(8, policy, HashMode::Identity);
                let mut oracle = BTreeMap::new();
                let mut code = history;
                for step in 0..5 {
                    let op = code % 8;
                    code /= 8;
                    let key = u64::from(op % 4) * 8 + 7;
                    if op < 4 {
                        assert_eq!(map.insert(key, step), oracle.insert(key, step));
                    } else {
                        assert_eq!(map.remove(key), oracle.remove(&key));
                    }
                    assert_eq!(map.len(), oracle.len());
                    for k in [7, 15, 23, 31, 39] {
                        assert_eq!(map.get(k), oracle.get(&k).copied());
                    }
                }
            }
        }
    }
    #[test]
    fn seeded_grow_rebuild_and_shift() {
        for policy in [Policy::Lazy, Policy::Rebuild, Policy::Shift] {
            let mut map = LinearMap::new(8, policy, HashMode::Mixed);
            let mut oracle = BTreeMap::new();
            let mut state = 17_u64;
            for i in 0..20_000 {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                let k = (state >> 32) % 257;
                let actual = match i % 3 {
                    0 => map.insert(k, state),
                    1 => map.remove(k),
                    _ => map.get(k),
                };
                let expected = match i % 3 {
                    0 => oracle.insert(k, state),
                    1 => oracle.remove(&k),
                    _ => oracle.get(&k).copied(),
                };
                assert_eq!(actual, expected);
                assert_eq!(map.len(), oracle.len());
            }
            let rows: BTreeMap<_, _> = map.iter().map(|(k, v)| (*k, *v)).collect();
            assert_eq!(rows, oracle);
        }
    }
    #[test]
    fn snapshot_and_value_mutation() {
        let mut map = LinearMap::new(8, Policy::Shift, HashMode::Identity);
        for k in [1, 9, 17] {
            map.insert(k, k);
        }
        map.visit_values(|_, v| *v += 1);
        for k in map.snapshot_keys() {
            map.remove(k);
            map.insert(k + 100, 7);
        }
        assert_eq!(map.len(), 3);
        assert_eq!(map.get(101), Some(7));
        assert_eq!(map.get(1), None);
    }
    #[test]
    fn extendible_split_alias_update_delete_overflow() {
        let mut map = ExtendibleMap::new(2);
        let mut oracle = BTreeMap::new();
        for k in [1, 9, 17, 0, 2, 4, 6, u64::MAX, 4097, 8193, 12289] {
            assert_eq!(map.insert(k, k), oracle.insert(k, k));
            for (&k, &v) in &oracle {
                assert_eq!(map.get(k), Some(v));
            }
        }
        assert_eq!(map.insert(17, 99), oracle.insert(17, 99));
        for k in [1, 17, 99, u64::MAX] {
            assert_eq!(map.remove(k), oracle.remove(&k));
        }
        assert!(map.stats().0 <= 4096);
        assert!(map.stats().2 > 0);
        for (&k, &v) in &oracle {
            assert_eq!(map.get(k), Some(v));
        }
    }
}
