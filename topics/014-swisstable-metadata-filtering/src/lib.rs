//! Bounded grouped hash table for studying exact metadata filtering.
//! This owned teaching layout uses linear groups, not hashbrown's probe protocol.
#![forbid(unsafe_code)]

const EMPTY: u8 = 255;
const DELETED: u8 = 128;
const WIDTH: usize = 16;

/// Lookup mechanism on one common table layout.
#[derive(Clone, Copy, Debug)]
pub enum Filter {
    /// Compare every full slot's key.
    Unfiltered,
    /// Reject a slot by its tag before comparing its key.
    Scalar,
    /// Form an exact 16-byte matching mask using integer operations.
    Word,
}

/// Trusted prehash input policy. These intentionally bad modes are test controls.
#[derive(Clone, Copy, Debug)]
pub enum HashShape {
    /// Mixed placement and tag bits.
    Mixed,
    /// Normal placement, but every stored tag is zero.
    ConstantTag,
    /// Every key starts in group zero; tags still vary.
    Clustered,
}

/// A fixed key with selectable comparison width.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key(pub [u64; 8]);
impl Key {
    /// All keys share a 56-byte prefix; the last word identifies the key.
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self([0, 0, 0, 0, 0, 0, 0, id])
    }
}

/// Stable deterministic mixing for trusted synthetic keys, not a security hasher.
#[must_use]
pub fn hash(id: u64, shape: HashShape) -> u64 {
    let mut x = id.wrapping_add(0x9e37_79b9_7f4a_7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^= x >> 31;
    match shape {
        HashShape::Mixed => x,
        HashShape::ConstantTag => x & ((1_u64 << 57) - 1),
        HashShape::Clustered => x & (127_u64 << 57),
    }
}

/// Exact high-bit-per-byte zero mask. Unlike subtract-one tricks, no borrow false positives.
#[must_use]
pub fn matching_mask(bytes: [u8; 16], tag: u8) -> u128 {
    let ones = u128::from_le_bytes([1; 16]);
    let low = u128::from_le_bytes([127; 16]);
    let high = u128::from_le_bytes([128; 16]);
    let x = u128::from_le_bytes(bytes) ^ (ones * u128::from(tag));
    !(((x & low) + low) | x | low) & high
}

fn tag(hash: u64) -> u8 {
    hash.to_be_bytes()[0] >> 1
}

fn lane(mask: u128) -> usize {
    // Nonzero masks have a bit index at most 127, fitting every supported usize.
    usize::try_from(mask.trailing_zeros()).unwrap() / 8
}

/// Lookup output plus untimed diagnostic counts.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Lookup {
    /// Associated value, if the exact key exists.
    pub value: Option<u64>,
    /// Number of control groups examined.
    pub groups: usize,
    /// Number of exact key comparisons.
    pub equalities: usize,
}

/// Fixed-capacity table. Keys and values are separate from control bytes.
/// No references escape; erase always leaves a tombstone. No resize or cleanup.
pub struct Table {
    controls: Vec<[u8; WIDTH]>,
    keys: Vec<Key>,
    values: Vec<u64>,
    wide: bool,
}

impl Table {
    /// Capacity must be a power of two and at least 16.
    ///
    /// # Panics
    /// Panics if the capacity contract is violated.
    #[must_use]
    pub fn new(capacity: usize, wide: bool) -> Self {
        assert!(capacity >= WIDTH && capacity.is_power_of_two());
        Self {
            controls: vec![[EMPTY; WIDTH]; capacity / WIDTH],
            keys: vec![Key::new(0); capacity],
            values: vec![0; capacity],
            wide,
        }
    }

    fn same(&self, index: usize, key: Key) -> bool {
        if self.wide {
            self.keys[index] == key
        } else {
            self.keys[index].0[7] == key.0[7]
        }
    }
    fn start(&self, hash: u64) -> usize {
        usize::try_from(hash & u64::try_from(self.controls.len() - 1).unwrap()).unwrap()
    }

    /// Insert or replace, returning the old value or a capacity failure.
    /// Prehash must be identical for every operation on a key.
    /// Searches past tombstones to prevent duplicate-key insertion.
    ///
    /// # Errors
    /// Returns the supplied value if no reusable slot exists.
    pub fn insert(&mut self, key: Key, hash: u64, value: u64) -> Result<Option<u64>, u64> {
        let tag = tag(hash);
        let mut reusable = None;
        for step in 0..self.controls.len() {
            let group = (self.start(hash) + step) & (self.controls.len() - 1);
            let mut empty = false;
            for lane in 0..WIDTH {
                let ctrl = self.controls[group][lane];
                let index = group * WIDTH + lane;
                if ctrl == tag && self.same(index, key) {
                    return Ok(Some(std::mem::replace(&mut self.values[index], value)));
                }
                if ctrl >= DELETED && reusable.is_none() {
                    reusable = Some((group, lane));
                }
                empty |= ctrl == EMPTY;
            }
            if empty {
                break;
            }
        }
        let Some((group, lane)) = reusable else {
            return Err(value);
        };
        self.keys[group * WIDTH + lane] = key;
        self.values[group * WIDTH + lane] = value;
        self.controls[group][lane] = tag;
        Ok(None)
    }

    /// Remove an exact key. Always leaves DELETED rather than EMPTY.
    pub fn remove(&mut self, key: Key, hash: u64) -> Option<u64> {
        for step in 0..self.controls.len() {
            let group = (self.start(hash) + step) & (self.controls.len() - 1);
            for lane in 0..WIDTH {
                if self.controls[group][lane] == tag(hash) && self.same(group * WIDTH + lane, key) {
                    self.controls[group][lane] = DELETED;
                    return Some(self.values[group * WIDTH + lane]);
                }
            }
            if self.controls[group].contains(&EMPTY) {
                return None;
            }
        }
        None
    }

    /// Compare all candidates in the current group before considering EMPTY.
    /// The bounded full-table scan also terminates when there are no empty slots.
    #[must_use]
    pub fn get(&self, key: Key, hash: u64, filter: Filter) -> Lookup {
        let tag = tag(hash);
        let mut out = Lookup::default();
        for step in 0..self.controls.len() {
            let group = (self.start(hash) + step) & (self.controls.len() - 1);
            out.groups += 1;
            let controls = self.controls[group];
            let mut mask = match filter {
                Filter::Word => matching_mask(controls, tag),
                _ => controls
                    .iter()
                    .enumerate()
                    .fold(0_u128, |m, (lane, &ctrl)| {
                        let candidate = match filter {
                            Filter::Unfiltered => ctrl < DELETED,
                            _ => ctrl == tag,
                        };
                        m | (u128::from(candidate) << (lane * 8 + 7))
                    }),
            };
            while mask != 0 {
                let lane = lane(mask);
                mask &= mask - 1;
                out.equalities += 1;
                let index = group * WIDTH + lane;
                if self.same(index, key) {
                    out.value = Some(self.values[index]);
                    return out;
                }
            }
            if controls.contains(&EMPTY) {
                return out;
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    const FILTERS: [Filter; 3] = [Filter::Unfiltered, Filter::Scalar, Filter::Word];

    #[test]
    fn every_byte_pair_mask_is_exact() {
        for tag in 0..=127 {
            for byte in 0..=255 {
                let mut bytes = [byte; 16];
                bytes[0] = tag;
                bytes[15] = 255;
                let mask = matching_mask(bytes, tag);
                for (i, b) in bytes.into_iter().enumerate() {
                    assert_eq!((mask >> (i * 8 + 7)) & 1, u128::from(b == tag));
                }
            }
        }
        let bytes = [0, 1, 0, 1, 128, 255, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1];
        assert_eq!(matching_mask(bytes, 0).count_ones(), 7);
    }
    #[test]
    fn candidate_after_empty_is_checked() {
        let mut t = Table::new(16, true);
        t.controls[0][15] = 7;
        t.keys[15] = Key::new(9);
        t.values[15] = 42;
        for f in FILTERS {
            assert_eq!(t.get(Key::new(9), 7 << 57, f).value, Some(42));
        }
    }
    #[test]
    fn wrap_tombstone_duplicate_and_full_cycle() {
        let mut t = Table::new(32, true);
        let h = 1;
        for id in 0..32 {
            assert_eq!(t.insert(Key::new(id), h, id), Ok(None));
        }
        assert_eq!(t.insert(Key::new(100), h, 100), Err(100));
        assert_eq!(t.remove(Key::new(0), h), Some(0));
        assert_eq!(t.insert(Key::new(31), h, 777), Ok(Some(31)));
        for f in FILTERS {
            assert_eq!(t.get(Key::new(31), h, f).value, Some(777));
            assert_eq!(t.get(Key::new(100), h, f).value, None);
        }
        assert_eq!(t.insert(Key::new(100), h, 100), Ok(None));
        for f in FILTERS {
            assert_eq!(t.get(Key::new(100), h, f).value, Some(100));
        }
    }
    #[test]
    fn all_tags_collide_still_exact() {
        let mut t = Table::new(32, true);
        for id in 0..24 {
            t.insert(Key::new(id), hash(id, HashShape::ConstantTag), id)
                .unwrap();
        }
        for f in FILTERS {
            for id in 0..48 {
                assert_eq!(
                    t.get(Key::new(id), hash(id, HashShape::ConstantTag), f)
                        .value,
                    (id < 24).then_some(id)
                );
            }
        }
    }
    #[test]
    fn seeded_operation_histories_against_ordered_map() {
        for wide in [false, true] {
            for shape in [
                HashShape::Mixed,
                HashShape::Clustered,
                HashShape::ConstantTag,
            ] {
                let mut t = Table::new(128, wide);
                let mut oracle = BTreeMap::new();
                let mut rng = 1_u64;
                for step in 0..4_000 {
                    rng = rng.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                    let id = (rng >> 20) % 80;
                    let key = Key::new(id);
                    let h = hash(id, shape);
                    if rng.is_multiple_of(3) {
                        assert_eq!(
                            t.remove(key, h),
                            oracle.remove(&key),
                            "step {step} key {id}"
                        );
                    } else {
                        assert_eq!(
                            t.insert(key, h, rng).unwrap(),
                            oracle.insert(key, rng),
                            "step {step} key {id}"
                        );
                    }
                    for id in 0..80 {
                        let key = Key::new(id);
                        let expected = oracle.get(&key).copied();
                        for f in FILTERS {
                            assert_eq!(
                                t.get(key, hash(id, shape), f).value,
                                expected,
                                "step {step} wide {wide} shape {shape:?} key {id} filter {f:?}"
                            );
                        }
                    }
                }
            }
        }
    }
}
