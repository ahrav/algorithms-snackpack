//! Stable named-owner placement and deterministic bounded batch replay.
//! Hash version 1 uses fixed-width integer fields and `SplitMix64` finalization.
//! This educational noncryptographic hash assumes trusted keys.

/// Candidate placement policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    /// Remainder into sorted stable IDs.
    Modulo,
    /// One ring token per owner.
    Ring1,
    /// Thirty-two ring tokens per owner.
    Ring32,
    /// Highest key-owner score; smallest ID wins ties.
    Rendezvous,
    /// Ring32 replay with a shared physical-owner capacity of ceil(5K/4N).
    Bounded,
}

/// Deterministic integer hash. Arithmetic wraps modulo 2^64.
#[must_use]
pub const fn hash(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

/// Validated immutable owner map.
#[derive(Debug)]
pub struct Map {
    policy: Policy,
    owners: Vec<u64>,
    // (position, stable owner ID, token number, index in owners)
    tokens: Vec<(u64, u64, usize, usize)>,
}

impl Map {
    /// Build a map with distinct nonempty IDs. Caller input order is canonicalized.
    ///
    /// # Errors
    /// Returns an error for empty membership, duplicate IDs, or token-count overflow.
    pub fn new(policy: Policy, ids: &[u64]) -> Result<Self, &'static str> {
        if ids.is_empty() {
            return Err("empty membership");
        }
        let mut owners = ids.to_vec();
        owners.sort_unstable();
        if owners.windows(2).any(|w| w[0] == w[1]) {
            return Err("duplicate owner");
        }
        let v = match policy {
            Policy::Ring1 => 1,
            Policy::Ring32 | Policy::Bounded => 32,
            _ => 0,
        };
        let token_count = owners.len().checked_mul(v).ok_or("token count overflow")?;
        let mut tokens = Vec::with_capacity(token_count);
        for (index, &id) in owners.iter().enumerate() {
            for token in 0..v {
                tokens.push((
                    hash(id ^ hash(token as u64 ^ 0xd1b5_4a32_d192_ed03)),
                    id,
                    token,
                    index,
                ));
            }
        }
        tokens.sort_unstable();
        Ok(Self {
            policy,
            owners,
            tokens,
        })
    }

    fn start(&self, key: u64) -> usize {
        let i = self.tokens.partition_point(|p| p.0 < hash(key));
        if i == self.tokens.len() { 0 } else { i }
    }

    /// Return one stable owner per input item in caller-supplied replay order.
    /// Each input position represents one unit item. Repeated values represent
    /// distinct items with the same hash, and can spill to different owners.
    /// For a key-to-owner directory, callers supply distinct keys and retain it.
    /// Bounded replay depends on the whole batch and its order.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "Remainder is below the owner vector length."
    )]
    pub fn assign(&self, keys: &[u64]) -> Vec<u64> {
        let mut result = Vec::with_capacity(keys.len());
        let mut counts = if self.policy == Policy::Bounded {
            vec![0; self.owners.len()]
        } else {
            Vec::new()
        };
        let cap = (5 * keys.len()).div_ceil(4 * self.owners.len());
        for &key in keys {
            let index = match self.policy {
                Policy::Modulo => (hash(key) % self.owners.len() as u64) as usize,
                Policy::Rendezvous => self
                    .owners
                    .iter()
                    .enumerate()
                    .max_by_key(|&(_, &id)| {
                        (
                            hash(key ^ hash(id ^ 0xa076_1d64_78bd_642f)),
                            std::cmp::Reverse(id),
                        )
                    })
                    .map_or(0, |(i, _)| i),
                Policy::Ring1 | Policy::Ring32 => self.tokens[self.start(key)].3,
                Policy::Bounded => {
                    let mut i = self.start(key);
                    while counts[self.tokens[i].3] == cap {
                        i = (i + 1) % self.tokens.len();
                    }
                    let owner = self.tokens[i].3;
                    counts[owner] += 1;
                    owner
                }
            };
            result.push(self.owners[index]);
        }
        result
    }

    /// Retained vector payload bytes; excludes struct, capacity slack and allocator overhead.
    #[must_use]
    pub fn payload_bytes(&self) -> usize {
        self.owners.len() * std::mem::size_of::<u64>()
            + self.tokens.len() * std::mem::size_of::<(u64, u64, usize, usize)>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn collisions_wrap_and_capacity() {
        let mut map = Map::new(Policy::Bounded, &[30, 10, 20]).unwrap();
        map.tokens = vec![(0, 10, 0, 0), (0, 20, 0, 1), (0, 30, 0, 2)];
        assert_eq!(map.assign(&[1; 6]), [10, 10, 10, 20, 20, 20]);
        map.policy = Policy::Ring1;
        assert_eq!(map.assign(&[u64::MAX; 6]), [10; 6]);
        // An exact position match chooses the first stable-ID tie.
        map.tokens = vec![
            (hash(7), 10, 0, 0),
            (hash(7), 20, 0, 1),
            (u64::MAX, 30, 0, 2),
        ];
        assert_eq!(map.assign(&[7]), [10]);
    }
}
