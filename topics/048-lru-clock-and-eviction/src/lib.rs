//! Sequential equal-size cache replacement with dense integer keys.
//!
//! Every miss admits, capacity zero always misses, and invalid keys preserve state.
//! Segmented LRU protects at most half the capacity; probation borrows unused space.
//!
//! ```
//! use topic048_eviction::{Cache, Policy};
//! let mut cache = Cache::new(Policy::Linked, 3, 5);
//! let hits: Vec<_> = [0,1,0,1,2,3,4,0,1].map(|k| cache.access(k).unwrap()).into();
//! assert_eq!(hits.iter().filter(|&&h| h).count(), 2);
//! assert_eq!(cache.snapshot().queues[0], [4,0,1]);
//! ```

const NONE: usize = usize::MAX;

/// Replacement policy and concrete realization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    /// Exact LRU using timestamps and a resident victim scan.
    Scan,
    /// Exact LRU using indexed doubly linked lists.
    Linked,
    /// Second-chance CLOCK, initially at slot zero.
    Clock,
    /// Segmented LRU with a protected quota of floor(capacity / 2).
    Segmented,
}

/// Complete observable policy state, excluding implementation links and counters.
#[derive(Debug, PartialEq, Eq)]
pub struct Snapshot {
    /// Oldest-first lists; segmented uses probation then protected.
    pub queues: Vec<Vec<usize>>,
    /// CLOCK slots, with `usize::MAX` for an empty slot.
    pub slots: Vec<usize>,
    /// CLOCK reference bits in slot order.
    pub bits: Vec<bool>,
    /// CLOCK's next candidate slot, zero for the other policies.
    pub hand: usize,
}

/// Bounded sequential cache metadata; keys must be below the declared universe.
pub struct Cache {
    policy: Policy,
    capacity: usize,
    pos: Vec<usize>,
    slots: Vec<usize>,
    stamps: Vec<usize>,
    bits: Vec<bool>,
    prev: Vec<usize>,
    next: Vec<usize>,
    head: [usize; 2],
    tail: [usize; 2],
    len: [usize; 2],
    used: usize,
    hand: usize,
    tick: usize,
    /// Victim candidates inspected, including CLOCK's initial empty slots.
    pub inspections: usize,
    /// List unlink and append operations.
    pub list_operations: usize,
}

impl Cache {
    /// Construct empty metadata. Allocations scale with the dense universe and capacity.
    #[must_use]
    pub fn new(policy: Policy, capacity: usize, universe: usize) -> Self {
        let linked = matches!(policy, Policy::Linked | Policy::Segmented);
        Self {
            policy,
            capacity,
            pos: vec![NONE; universe],
            slots: vec![NONE; if linked { 0 } else { capacity }],
            stamps: vec![0; if policy == Policy::Scan { capacity } else { 0 }],
            bits: vec![false; if policy == Policy::Clock { capacity } else { 0 }],
            prev: vec![NONE; if linked { universe } else { 0 }],
            next: vec![NONE; if linked { universe } else { 0 }],
            head: [NONE; 2],
            tail: [NONE; 2],
            len: [0; 2],
            used: 0,
            hand: 0,
            tick: 0,
            inspections: 0,
            list_operations: 0,
        }
    }

    fn unlink(&mut self, key: usize) {
        let q = self.pos[key];
        let p = self.prev[key];
        let n = self.next[key];
        if p == NONE {
            self.head[q] = n;
        } else {
            self.next[p] = n;
        }
        if n == NONE {
            self.tail[q] = p;
        } else {
            self.prev[n] = p;
        }
        self.len[q] -= 1;
        self.pos[key] = NONE;
        self.list_operations += 1;
    }

    fn append(&mut self, key: usize, q: usize) {
        let old = self.tail[q];
        self.prev[key] = old;
        self.next[key] = NONE;
        if old == NONE {
            self.head[q] = key;
        } else {
            self.next[old] = key;
        }
        self.tail[q] = key;
        self.pos[key] = q;
        self.len[q] += 1;
        self.list_operations += 1;
    }

    /// Report prior residency and apply the replacement transition.
    ///
    /// # Errors
    /// Returns `key outside universe` or `request counter exhausted` before mutation.
    pub fn access(&mut self, key: usize) -> Result<bool, &'static str> {
        if key >= self.pos.len() {
            return Err("key outside universe");
        }
        let tick = self
            .tick
            .checked_add(1)
            .ok_or("request counter exhausted")?;
        self.tick = tick;
        if self.capacity == 0 {
            return Ok(false);
        }
        let old = self.pos[key];
        let hit = old != NONE;
        match self.policy {
            Policy::Scan => {
                let slot = if hit {
                    old
                } else if self.used < self.capacity {
                    let i = self.used;
                    self.used += 1;
                    i
                } else {
                    let mut best = 0;
                    for i in 0..self.capacity {
                        self.inspections += 1;
                        if self.stamps[i] < self.stamps[best] {
                            best = i;
                        }
                    }
                    self.pos[self.slots[best]] = NONE;
                    best
                };
                self.slots[slot] = key;
                self.pos[key] = slot;
                self.stamps[slot] = tick;
            }
            Policy::Clock => {
                if hit {
                    self.bits[old] = true;
                } else {
                    loop {
                        let slot = self.hand;
                        self.inspections += 1;
                        self.hand = (self.hand + 1) % self.capacity;
                        if self.slots[slot] == NONE || !self.bits[slot] {
                            if self.slots[slot] != NONE {
                                self.pos[self.slots[slot]] = NONE;
                            }
                            self.slots[slot] = key;
                            self.pos[key] = slot;
                            self.bits[slot] = true;
                            break;
                        }
                        self.bits[slot] = false;
                    }
                }
            }
            Policy::Linked | Policy::Segmented => {
                if hit {
                    self.unlink(key);
                } else if self.len[0] + self.len[1] == self.capacity {
                    self.unlink(self.head[0]);
                }
                let q = usize::from(hit && self.policy == Policy::Segmented);
                self.append(key, q);
                if q == 1 && self.len[1] > self.capacity / 2 {
                    let demoted = self.head[1];
                    self.unlink(demoted);
                    self.append(demoted, 0);
                }
            }
        }
        Ok(hit)
    }

    /// Copy the exact logical state. Snapshot allocation belongs outside timing.
    #[must_use]
    pub fn snapshot(&self) -> Snapshot {
        let mut queues = Vec::new();
        if self.policy == Policy::Scan {
            let mut entries: Vec<_> = (0..self.used)
                .map(|i| (self.stamps[i], self.slots[i]))
                .collect();
            entries.sort_unstable();
            queues.push(entries.into_iter().map(|(_, key)| key).collect());
        } else if self.policy != Policy::Clock {
            for q in 0..if self.policy == Policy::Segmented {
                2
            } else {
                1
            } {
                let mut keys = Vec::new();
                let mut k = self.head[q];
                while k != NONE {
                    keys.push(k);
                    k = self.next[k];
                }
                queues.push(keys);
            }
        }
        Snapshot {
            queues,
            slots: if self.policy == Policy::Clock {
                self.slots.clone()
            } else {
                Vec::new()
            },
            bits: self.bits.clone(),
            hand: self.hand,
        }
    }
}
