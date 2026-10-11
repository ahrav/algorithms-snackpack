//! Bounded maps isolating Robin Hood placement and early miss termination.
#![forbid(unsafe_code)]

/// Placement and lookup policy. All candidates use the same slot layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    /// First empty slot; cyclic eligibility during backward deletion.
    Linear,
    /// Larger displacement wins; resident displacement can terminate a miss.
    Robin,
    /// Robin Hood placement, but scan misses to an empty slot as a control.
    RobinScan,
}

/// Hasher modes for trusted-key experiments, never collision-attack defenses.
#[derive(Clone, Copy, Debug)]
pub enum HashMode {
    /// Fixed deterministic avalanche mixing.
    Mixed,
    /// Every key starts at slot zero, deliberately adversarial.
    Constant,
    /// Home is the low key bits, useful for exact invariant witnesses.
    Identity,
}

/// Fixed mixing used by every timed candidate. Hashing is inside timed operations.
#[must_use]
pub fn mix(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

#[derive(Clone, Copy, Debug)]
struct Entry {
    key: u64,
    value: u64,
    dib: usize,
}

/// Capacity rejection. A failed insert leaves every existing key and value intact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Full;

/// Fixed-capacity, single-owner map with no tombstones and no stable-address contract.
#[derive(Clone, Debug)]
pub struct Table {
    slots: Vec<Option<Entry>>,
    len: usize,
    policy: Policy,
    hash: HashMode,
}

impl Table {
    /// Allocate a power-of-two capacity of at least two slots.
    ///
    /// # Panics
    /// Panics for an invalid capacity or failed allocation.
    #[must_use]
    pub fn new(capacity: usize, policy: Policy, hash: HashMode) -> Self {
        assert!(capacity >= 2 && capacity.is_power_of_two());
        Self {
            slots: vec![None; capacity],
            len: 0,
            policy,
            hash,
        }
    }

    fn home(&self, key: u64) -> usize {
        let hash = match self.hash {
            HashMode::Mixed => mix(key),
            HashMode::Constant => 0,
            HashMode::Identity => key,
        };
        // Only low bits are used. Truncation is intentional on smaller targets.
        #[allow(clippy::cast_possible_truncation)]
        let index = hash as usize;
        index & (self.slots.len() - 1)
    }

    fn locate(&self, key: u64) -> (Option<usize>, usize) {
        let mask = self.slots.len() - 1;
        let home = self.home(key);
        for d in 0..self.slots.len() {
            let i = (home + d) & mask;
            let Some(e) = self.slots[i] else {
                return (None, d + 1);
            };
            if self.policy == Policy::Robin && e.dib < d {
                return (None, d + 1);
            }
            if e.key == key {
                return (Some(i), d + 1);
            }
        }
        (None, self.slots.len())
    }

    /// Return the value and the number of inspected slots, including a terminating slot.
    ///
    /// # Panics
    /// Panics if an internal occupied-slot invariant is violated.
    #[must_use]
    pub fn get_counted(&self, key: u64) -> (Option<u64>, usize) {
        let (index, probes) = self.locate(key);
        (
            index.map(|i| self.slots[i].expect("located entry").value),
            probes,
        )
    }

    /// Return a copied value. Hashing and slot inspection occur in this call.
    #[must_use]
    pub fn get(&self, key: u64) -> Option<u64> {
        self.get_counted(key).0
    }

    /// Insert or update. An update works even when full.
    ///
    /// # Errors
    /// Returns [`Full`] for a new key in a full table, before any relocation.
    ///
    /// # Panics
    /// Panics if an internal occupied-slot invariant is violated.
    pub fn insert(&mut self, key: u64, value: u64) -> Result<Option<u64>, Full> {
        if let (Some(i), _) = self.locate(key) {
            let e = self.slots[i].as_mut().expect("located entry");
            let old = e.value;
            e.value = value;
            return Ok(Some(old));
        }
        if self.len == self.slots.len() {
            return Err(Full);
        }
        let mask = self.slots.len() - 1;
        let mut i = self.home(key);
        let mut incoming = Entry { key, value, dib: 0 };
        loop {
            match self.slots[i].as_mut() {
                None => {
                    self.slots[i] = Some(incoming);
                    self.len += 1;
                    return Ok(None);
                }
                Some(resident) if self.policy != Policy::Linear && incoming.dib > resident.dib => {
                    std::mem::swap(resident, &mut incoming);
                }
                Some(_) => {}
            }
            incoming.dib += 1;
            i = (i + 1) & mask;
        }
    }

    /// Delete, preserving lookup invariants. Return the removed value.
    ///
    /// # Panics
    /// Panics if an internal occupied-slot invariant is violated.
    pub fn remove(&mut self, key: u64) -> Option<u64> {
        let i = self.locate(key).0?;
        let old = self.slots[i].take().expect("located entry").value;
        self.len -= 1;
        let mask = self.slots.len() - 1;
        let mut hole = i;
        let mut j = (i + 1) & mask;
        // At most capacity-1 survivors need inspection, even if the table was full.
        for _ in 0..mask {
            let Some(mut e) = self.slots[j] else { break };
            if self.policy != Policy::Linear && e.dib == 0 {
                break;
            }
            let gap = j.wrapping_sub(hole) & mask;
            if e.dib >= gap {
                e.dib -= gap;
                self.slots[hole] = Some(e);
                self.slots[j] = None;
                hole = j;
            }
            j = (j + 1) & mask;
        }
        Some(old)
    }

    /// Occupied count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether there are no keys.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Snapshot `(slot, key, distance)` for the visual and invariant witnesses.
    #[must_use]
    pub fn layout(&self) -> Vec<(usize, u64, usize)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(i, e)| e.map(|e| (i, e.key, e.dib)))
            .collect()
    }

    /// Check displacement bookkeeping and reachability for every resident key.
    /// This is a diagnostic, not the independent correctness oracle.
    #[must_use]
    pub fn valid(&self) -> bool {
        self.slots.iter().flatten().count() == self.len
            && self.slots.iter().enumerate().all(|(i, e)| {
                e.is_none_or(|e| {
                    e.dib == (i.wrapping_sub(self.home(e.key)) & (self.slots.len() - 1))
                        && self.get(e.key) == Some(e.value)
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn independent_oracle_mixed_operations() {
        for policy in [Policy::Linear, Policy::Robin, Policy::RobinScan] {
            for mode in [HashMode::Mixed, HashMode::Identity, HashMode::Constant] {
                for capacity in [2, 8, 64] {
                    let mut t = Table::new(capacity, policy, mode);
                    let mut oracle = BTreeMap::new();
                    let mut state = 17_u64;
                    for step in 0..4000_u64 {
                        // Independent LCG, deliberately not the table's mixing helper.
                        state = state
                            .wrapping_mul(6_364_136_223_846_793_005)
                            .wrapping_add(1);
                        let k = (state >> 32) % 97;
                        match state % 4 {
                            0 | 1 => {
                                if oracle.contains_key(&k) || oracle.len() < capacity {
                                    assert_eq!(t.insert(k, step), Ok(oracle.insert(k, step)));
                                } else {
                                    assert_eq!(t.insert(k, step), Err(Full));
                                }
                            }
                            2 => assert_eq!(t.remove(k), oracle.remove(&k)),
                            _ => assert_eq!(t.get(k), oracle.get(&k).copied()),
                        }
                        assert_eq!(t.len(), oracle.len());
                        assert!(t.valid());
                        for (&k, &v) in &oracle {
                            assert_eq!(t.get(k), Some(v));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn displacement_is_not_globally_sorted() {
        let mut t = Table::new(8, Policy::Robin, HashMode::Identity);
        for k in [0, 8, 2] {
            t.insert(k, k).unwrap();
        }
        assert_eq!(t.layout(), vec![(0, 0, 0), (1, 8, 1), (2, 2, 0)]);
        assert_eq!(t.get_counted(16), (None, 3));
    }

    #[test]
    fn swap_and_wrap_delete() {
        let mut t = Table::new(8, Policy::Robin, HashMode::Identity);
        for k in [6, 7, 14] {
            t.insert(k, k).unwrap();
        }
        assert_eq!(t.layout(), vec![(0, 7, 1), (6, 6, 0), (7, 14, 1)]);
        assert_eq!(t.remove(6), Some(6));
        assert_eq!(t.layout(), vec![(6, 14, 0), (7, 7, 0)]);
        assert!(t.valid());
    }

    #[test]
    fn linear_must_skip_at_home_then_continue() {
        let mut t = Table::new(8, Policy::Linear, HashMode::Identity);
        for k in [0, 1, 8] {
            t.insert(k, k).unwrap();
        }
        assert_eq!(t.remove(0), Some(0));
        assert_eq!(t.get(8), Some(8));
        assert_eq!(t.layout(), vec![(0, 8, 0), (1, 1, 0)]);
    }

    #[test]
    fn full_failure_and_update_preserve_contents() {
        for p in [Policy::Linear, Policy::Robin, Policy::RobinScan] {
            let mut t = Table::new(8, p, HashMode::Identity);
            for k in [0, 1, 2, 3, 8, 9, 10, 11] {
                t.insert(k, k).unwrap();
            }
            let before = t.layout();
            assert_eq!(t.insert(16, 99), Err(Full));
            assert_eq!(t.layout(), before);
            assert_eq!(t.insert(8, 99), Ok(Some(8)));
            assert_eq!(t.get(8), Some(99));
            assert_eq!(t.get(16), None);
            assert_eq!(t.remove(1), Some(1));
            assert!(t.valid());
        }
    }
}
