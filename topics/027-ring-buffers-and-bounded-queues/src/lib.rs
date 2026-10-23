//! Safe, single-owner bounded FIFO storage. Full pushes return the original value.
#![forbid(unsafe_code)]
use std::collections::VecDeque;

/// Common exact-capacity FIFO contract for the measured `u64` candidates.
pub trait Fifo: Sized {
    /// Construct an empty queue with the exact logical capacity.
    fn new(capacity: usize) -> Self;
    /// Return an unaccepted value without modifying the queue when full.
    ///
    /// # Errors
    /// Returns `Err(value)` exactly when the logical capacity is exhausted.
    fn push(&mut self, value: u64) -> Result<(), u64>;
    /// Move out the oldest value, or return `None` when empty.
    fn pop(&mut self) -> Option<u64>;
    /// Number of live values.
    fn len(&self) -> usize;
    /// Whether no values remain.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Contiguous baseline: front removal shifts every remaining value.
pub struct Shift {
    values: Vec<u64>,
    capacity: usize,
}
impl Fifo for Shift {
    fn new(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
            capacity,
        }
    }
    fn push(&mut self, value: u64) -> Result<(), u64> {
        if self.values.len() == self.capacity {
            return Err(value);
        }
        self.values.push(value);
        Ok(())
    }
    fn pop(&mut self) -> Option<u64> {
        if self.values.is_empty() {
            None
        } else {
            Some(self.values.remove(0))
        }
    }
    fn len(&self) -> usize {
        self.values.len()
    }
}

/// Standard growable deque with an explicit logical bound checked before insertion.
pub struct Deque {
    values: VecDeque<u64>,
    capacity: usize,
}
impl Fifo for Deque {
    fn new(capacity: usize) -> Self {
        Self {
            values: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
    fn push(&mut self, value: u64) -> Result<(), u64> {
        if self.values.len() == self.capacity {
            return Err(value);
        }
        self.values.push_back(value);
        Ok(())
    }
    fn pop(&mut self) -> Option<u64> {
        self.values.pop_front()
    }
    fn len(&self) -> usize {
        self.values.len()
    }
}

/// Safe owning ring; MODE 0 uses remainder, 1 conditional wrap, 2 a power-of-two mask.
///
/// Logical capacity stays exact in every mode. Mask mode rounds backing slots upward.
/// Allocation failure follows standard infallible `Vec` construction behavior.
pub struct Ring<T, const MODE: u8> {
    slots: Vec<Option<T>>,
    head: usize,
    len: usize,
    capacity: usize,
}
impl<T, const MODE: u8> Ring<T, MODE> {
    /// Construct empty storage.
    ///
    /// # Panics
    /// Panics for invalid mode, overflowing rounded capacity or invalid allocation layout.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        assert!(MODE <= 2);
        let physical = if MODE == 2 {
            capacity
                .max(1)
                .checked_next_power_of_two()
                .expect("capacity overflow")
        } else {
            capacity.max(1)
        };
        Self {
            slots: std::iter::repeat_with(|| None).take(physical).collect(),
            head: 0,
            len: 0,
            capacity,
        }
    }
    fn index(&self, offset: usize) -> usize {
        let size = self.slots.len();
        if MODE == 0 {
            (self.head + offset) % size
        } else if MODE == 1 {
            let remaining = size - self.head;
            if offset >= remaining {
                offset - remaining
            } else {
                self.head + offset
            }
        } else {
            self.head.wrapping_add(offset) & (size - 1)
        }
    }
    /// Return the original owned value on full, preserving all existing values.
    ///
    /// # Errors
    /// Returns `Err(value)` when `len() == capacity()`.
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.len == self.capacity {
            return Err(value);
        }
        let tail = self.index(self.len);
        self.slots[tail] = Some(value);
        self.len += 1;
        Ok(())
    }
    /// Move out the oldest value and clear its slot before reuse.
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let value = self.slots[self.head].take();
        self.head = self.index(1);
        self.len -= 1;
        value
    }
    /// Live element count.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }
    /// Empty queues have no initialized payloads.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// Exact public capacity, which can be smaller than backing storage.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }
    /// Allocated slot count, including mask-mode padding and a zero-capacity sentinel.
    #[must_use]
    pub fn backing_slots(&self) -> usize {
        self.slots.len()
    }
    /// Two initialized slot ranges in FIFO order. Each live slot contains `Some(T)`.
    #[must_use]
    pub fn as_slices(&self) -> (&[Option<T>], &[Option<T>]) {
        let first = self.len.min(self.slots.len() - self.head);
        (
            &self.slots[self.head..self.head + first],
            &self.slots[..self.len - first],
        )
    }
}
impl<const MODE: u8> Fifo for Ring<u64, MODE> {
    fn new(capacity: usize) -> Self {
        Self::new(capacity)
    }
    fn push(&mut self, value: u64) -> Result<(), u64> {
        Self::push(self, value)
    }
    fn pop(&mut self) -> Option<u64> {
        Self::pop(self)
    }
    fn len(&self) -> usize {
        self.len
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, collections::BTreeMap, rc::Rc};
    fn histories<Q: Fifo>() {
        for cap in [0, 1, 2, 3, 4, 7, 8] {
            for history in 0..6561 {
                let mut encoded = history;
                let mut q = Q::new(cap);
                let mut oracle = BTreeMap::new();
                let mut ticket = 0_u64;
                for step in 0_u64..8 {
                    match encoded % 3 {
                        0 | 1 => {
                            let value = if encoded % 3 == 0 { step } else { 0 };
                            let result = q.push(value);
                            let expected = if oracle.len() == cap {
                                Err(value)
                            } else {
                                oracle.insert(ticket, value);
                                ticket += 1;
                                Ok(())
                            };
                            assert_eq!(result, expected, "cap {cap}, history {history}");
                        }
                        _ => assert_eq!(q.pop(), oracle.pop_first().map(|(_, v)| v)),
                    }
                    assert_eq!(q.len(), oracle.len());
                    encoded /= 3;
                }
                while let Some((_, v)) = oracle.pop_first() {
                    assert_eq!(q.pop(), Some(v));
                }
                assert_eq!(q.pop(), None);
            }
        }
    }
    #[test]
    fn exhaustive_histories() {
        histories::<Shift>();
        histories::<Deque>();
        histories::<Ring<u64, 0>>();
        histories::<Ring<u64, 1>>();
        histories::<Ring<u64, 2>>();
    }
    #[test]
    fn wrapped_slices_and_exact_bound() {
        let mut q = Ring::<u64, 1>::new(4);
        for n in 10..14 {
            q.push(n).unwrap();
        }
        assert_eq!(q.push(14), Err(14));
        assert_eq!(q.pop(), Some(10));
        q.push(14).unwrap();
        assert_eq!(
            q.as_slices(),
            (&[Some(11), Some(12), Some(13)][..], &[Some(14)][..])
        );
        let mut q = Ring::<u64, 2>::new(3);
        assert_eq!(q.backing_slots(), 4);
        for n in 0..3 {
            q.push(n).unwrap();
        }
        assert_eq!(q.push(99), Err(99));
    }
    #[test]
    fn owning_values_drop_once_and_rejection_returns_ownership() {
        struct Token(Rc<Cell<usize>>);
        impl Drop for Token {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }
        let drops = Rc::new(Cell::new(0));
        {
            let mut q = Ring::<Token, 1>::new(2);
            assert!(q.push(Token(drops.clone())).is_ok());
            assert!(q.push(Token(drops.clone())).is_ok());
            let rejected = q.push(Token(drops.clone())).err().unwrap();
            assert_eq!(drops.get(), 0);
            drop(rejected);
            assert_eq!(drops.get(), 1);
            drop(q.pop());
            assert_eq!(drops.get(), 2);
            assert!(q.push(Token(drops.clone())).is_ok());
        }
        assert_eq!(drops.get(), 4);
    }
    #[test]
    fn many_laps_zero_and_unit_payloads() {
        let mut q = Ring::<(), 2>::new(3);
        for _ in 0..10000 {
            for _ in 0..3 {
                q.push(()).unwrap();
            }
            assert_eq!(q.push(()), Err(()));
            for _ in 0..3 {
                assert_eq!(q.pop(), Some(()));
            }
        }
        let mut zero = Ring::<String, 0>::new(0);
        assert_eq!(zero.push("owned".into()), Err("owned".into()));
        assert!(zero.pop().is_none());
    }
}
