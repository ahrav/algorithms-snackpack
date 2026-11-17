//! Insert-only ordered sets with owning and embedded-link red-black representations.
//! Arena slots retain their key identity for the lifetime of the index.
use std::collections::BTreeSet;

type Link = Option<Box<Node>>;
struct Node {
    key: u64,
    red: bool,
    left: Link,
    right: Link,
}
fn red(n: &Link) -> bool {
    n.as_ref().is_some_and(|n| n.red)
}
fn boxed_put(n: Link, key: u64, added: &mut bool) -> Box<Node> {
    let Some(mut h) = n else {
        *added = true;
        return Box::new(Node {
            key,
            red: true,
            left: None,
            right: None,
        });
    };
    match key.cmp(&h.key) {
        std::cmp::Ordering::Less => h.left = Some(boxed_put(h.left.take(), key, added)),
        std::cmp::Ordering::Greater => h.right = Some(boxed_put(h.right.take(), key, added)),
        std::cmp::Ordering::Equal => return h,
    }
    if red(&h.right) && !red(&h.left) {
        let mut x = h.right.take().expect("red child exists");
        h.right = x.left.take();
        x.red = h.red;
        h.red = true;
        x.left = Some(h);
        h = x;
    }
    if red(&h.left) && red(&h.left.as_ref().expect("red child exists").left) {
        let mut x = h.left.take().expect("red child exists");
        h.left = x.right.take();
        x.red = h.red;
        h.red = true;
        x.right = Some(h);
        h = x;
    }
    if red(&h.left) && red(&h.right) {
        h.red = !h.red;
        h.left.as_mut().expect("red child exists").red = false;
        h.right.as_mut().expect("red child exists").red = false;
    }
    h
}
#[derive(Clone)]
struct Record {
    key: u64,
    red: bool,
    left: Option<usize>,
    right: Option<usize>,
}
#[derive(Default)]
struct Arena {
    records: Vec<Record>,
    root: Option<usize>,
}
impl Arena {
    fn red(&self, i: Option<usize>) -> bool {
        i.is_some_and(|i| self.records[i].red)
    }
    fn put(&mut self, i: Option<usize>, key: u64) -> usize {
        let Some(mut h) = i else {
            let i = self.records.len();
            self.records.push(Record {
                key,
                red: true,
                left: None,
                right: None,
            });
            return i;
        };
        match key.cmp(&self.records[h].key) {
            std::cmp::Ordering::Less => {
                self.records[h].left = Some(self.put(self.records[h].left, key));
            }
            std::cmp::Ordering::Greater => {
                self.records[h].right = Some(self.put(self.records[h].right, key));
            }
            std::cmp::Ordering::Equal => return h,
        }
        if self.red(self.records[h].right) && !self.red(self.records[h].left) {
            let x = self.records[h].right.expect("red child exists");
            self.records[h].right = self.records[x].left;
            self.records[x].red = self.records[h].red;
            self.records[h].red = true;
            self.records[x].left = Some(h);
            h = x;
        }
        let left = self.records[h].left;
        if self.red(left) && self.red(self.records[left.expect("red child exists")].left) {
            let x = left.expect("red child exists");
            self.records[h].left = self.records[x].right;
            self.records[x].red = self.records[h].red;
            self.records[h].red = true;
            self.records[x].right = Some(h);
            h = x;
        }
        if self.red(self.records[h].left) && self.red(self.records[h].right) {
            self.records[h].red = !self.records[h].red;
            let l = self.records[h].left.expect("red child exists");
            let r = self.records[h].right.expect("red child exists");
            self.records[l].red = false;
            self.records[r].red = false;
        }
        h
    }
}
enum Storage {
    Flat(Vec<u64>),
    Boxed(Link),
    Arena(Arena),
    Btree(BTreeSet<u64>),
}
/// An insert-only unique-key set. Keys remain immutable after insertion.
pub struct Index {
    storage: Storage,
}
impl Index {
    /// Create a candidate, reserving capacity for `capacity` insertion attempts.
    /// Returns `None` for an unknown candidate name.
    #[must_use]
    pub fn new(kind: &str, capacity: usize) -> Option<Self> {
        let storage = match kind {
            "flat" => Storage::Flat(Vec::with_capacity(capacity)),
            "boxed" => Storage::Boxed(None),
            "arena" => Storage::Arena(Arena {
                records: Vec::with_capacity(capacity),
                root: None,
            }),
            "btree" => Storage::Btree(BTreeSet::new()),
            _ => return None,
        };
        Some(Self { storage })
    }
    /// Insert a key and preserve the existing record when the key already exists.
    /// Returns whether the set gained a member.
    /// # Panics
    /// Panics if private tree invariants are violated or allocation fails.
    pub fn insert(&mut self, key: u64) -> bool {
        match &mut self.storage {
            Storage::Flat(v) => match v.binary_search(&key) {
                Ok(_) => false,
                Err(i) => {
                    v.insert(i, key);
                    true
                }
            },
            Storage::Btree(t) => t.insert(key),
            Storage::Boxed(t) => {
                let mut added = false;
                let mut root = boxed_put(t.take(), key, &mut added);
                root.red = false;
                *t = Some(root);
                added
            }
            Storage::Arena(a) => {
                let before = a.records.len();
                let root = a.put(a.root, key);
                a.records[root].red = false;
                a.root = Some(root);
                a.records.len() != before
            }
        }
    }
    /// Test membership with the candidate's native lookup path.
    #[must_use]
    pub fn contains(&self, key: u64) -> bool {
        match &self.storage {
            Storage::Flat(v) => v.binary_search(&key).is_ok(),
            Storage::Btree(t) => t.contains(&key),
            Storage::Boxed(t) => boxed_contains(t, key),
            Storage::Arena(a) => {
                let mut next = a.root;
                while let Some(i) = next {
                    let n = &a.records[i];
                    match key.cmp(&n.key) {
                        std::cmp::Ordering::Equal => return true,
                        std::cmp::Ordering::Less => next = n.left,
                        std::cmp::Ordering::Greater => next = n.right,
                    }
                }
                false
            }
        }
    }
    /// Return keys in ascending order, outside the measured workload.
    #[must_use]
    pub fn keys(&self) -> Vec<u64> {
        fn walk(n: &Link, out: &mut Vec<u64>) {
            if let Some(n) = n {
                walk(&n.left, out);
                out.push(n.key);
                walk(&n.right, out);
            }
        }
        fn slots(a: &Arena, i: Option<usize>, out: &mut Vec<u64>) {
            if let Some(i) = i {
                let n = &a.records[i];
                slots(a, n.left, out);
                out.push(n.key);
                slots(a, n.right, out);
            }
        }
        match &self.storage {
            Storage::Flat(v) => v.clone(),
            Storage::Btree(t) => t.iter().copied().collect(),
            Storage::Boxed(t) => {
                let mut v = Vec::new();
                walk(t, &mut v);
                v
            }
            Storage::Arena(a) => {
                let mut v = Vec::new();
                slots(a, a.root, &mut v);
                v
            }
        }
    }
    /// Return arena keys in birth-slot order; other representations return an empty vector.
    #[must_use]
    pub fn slot_keys(&self) -> Vec<u64> {
        match &self.storage {
            Storage::Arena(a) => a.records.iter().map(|n| n.key).collect(),
            _ => Vec::new(),
        }
    }
    /// Check ordering, unique reachability, black height, and left-leaning invariants.
    #[must_use]
    pub fn valid(&self) -> bool {
        fn check(n: &Link, lo: Option<u64>, hi: Option<u64>) -> Option<usize> {
            let Some(n) = n else { return Some(0) };
            if lo.is_some_and(|x| n.key <= x)
                || hi.is_some_and(|x| n.key >= x)
                || red(&n.right)
                || (n.red && red(&n.left))
            {
                return None;
            }
            let l = check(&n.left, lo, Some(n.key))?;
            let r = check(&n.right, Some(n.key), hi)?;
            (l == r).then_some(l + usize::from(!n.red))
        }
        fn check_slots(
            a: &Arena,
            i: Option<usize>,
            lo: Option<u64>,
            hi: Option<u64>,
            seen: &mut [bool],
        ) -> Option<usize> {
            let Some(i) = i else { return Some(0) };
            if i >= seen.len() || seen[i] {
                return None;
            }
            seen[i] = true;
            let n = &a.records[i];
            if lo.is_some_and(|x| n.key <= x)
                || hi.is_some_and(|x| n.key >= x)
                || a.red(n.right)
                || (n.red && a.red(n.left))
            {
                return None;
            }
            let left_height = check_slots(a, n.left, lo, Some(n.key), seen)?;
            let right_height = check_slots(a, n.right, Some(n.key), hi, seen)?;
            (left_height == right_height).then_some(left_height + usize::from(!n.red))
        }
        match &self.storage {
            Storage::Boxed(t) => !red(t) && check(t, None, None).is_some(),
            Storage::Arena(a) => {
                let mut seen = vec![false; a.records.len()];
                !a.red(a.root)
                    && check_slots(a, a.root, None, None, &mut seen).is_some()
                    && seen.iter().all(|x| *x)
            }
            Storage::Flat(v) => v.windows(2).all(|w| w[0] < w[1]),
            Storage::Btree(_) => true,
        }
    }
}
fn boxed_contains(n: &Link, key: u64) -> bool {
    let mut next = n.as_ref();
    while let Some(n) = next {
        match key.cmp(&n.key) {
            std::cmp::Ordering::Equal => return true,
            std::cmp::Ordering::Less => next = n.left.as_ref(),
            std::cmp::Ordering::Greater => next = n.right.as_ref(),
        }
    }
    false
}
/// Deterministic insertion attempts and membership queries, generated outside timing.
/// # Panics
/// Panics when the input dimension exceeds the supported integer workload domain.
#[must_use]
pub fn workload(n: usize, shape: &str, access: &str) -> (Vec<u64>, Vec<u64>) {
    let n64 = u64::try_from(n).expect("usize fits workload domain");
    let keys = (0..n64)
        .map(|i| match shape {
            "descending" => (n64 - 1 - i) * 2,
            "duplicates" => ((i * 37 + 11) % (n64 / 4).max(1)) * 2,
            _ => i * 2,
        })
        .collect();
    let queries = (0..8192_u64)
        .map(|i| {
            if access == "hot" {
                i % 8
            } else {
                (i * 11939 + 7) % (n64 * 2).max(1)
            }
        })
        .collect();
    (keys, queries)
}
