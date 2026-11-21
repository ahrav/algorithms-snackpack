//! Exact equal-weight isotonic regression for an ordered chain.
#![forbid(unsafe_code)]

use std::cmp::Ordering;

/// Maximum supported number of observations.
pub const MAX_LEN: usize = 32_768;

/// One maximal run of equal fitted values in the original input order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Block {
    /// Sum of original observations in the block.
    pub sum: i64,
    /// Number of observations in the block. Returned blocks always have positive length.
    pub len: i64,
}

impl Block {
    fn singleton(value: i32) -> Self {
        Self {
            sum: i64::from(value),
            len: 1,
        }
    }

    fn mean_cmp(self, right: Self) -> Ordering {
        (self.sum * right.len).cmp(&(right.sum * self.len))
    }

    fn join(self, right: Self) -> Self {
        Self {
            sum: self.sum + right.sum,
            len: self.len + right.len,
        }
    }
}

/// Fit a nondecreasing sequence by least squares using a ordered vector with shifting deletion.
///
/// # Panics
///
/// Panics if the input has more than [`MAX_LEN`] observations.
#[must_use]
pub fn shifting(input: &[i32]) -> Vec<Block> {
    assert!(input.len() <= MAX_LEN);
    let mut blocks: Vec<_> = input.iter().copied().map(Block::singleton).collect();
    let mut left = 0;
    while left + 1 < blocks.len() {
        let right = blocks[left + 1];
        if blocks[left].mean_cmp(right).is_lt() {
            left += 1;
        } else {
            blocks[left] = blocks[left].join(right);
            blocks.remove(left + 1);
            left = left.saturating_sub(1);
        }
    }
    blocks
}

/// Fit a nondecreasing sequence by least squares using a pooling stack.
///
/// # Panics
///
/// Panics if the input has more than [`MAX_LEN`] observations.
#[must_use]
pub fn stack(input: &[i32]) -> Vec<Block> {
    assert!(input.len() <= MAX_LEN);
    let mut blocks: Vec<Block> = Vec::with_capacity(input.len());
    for &value in input {
        let mut block = Block::singleton(value);
        while let Some(&left) = blocks.last() {
            if left.mean_cmp(block).is_lt() {
                break;
            }
            blocks.pop();
            block = left.join(block);
        }
        blocks.push(block);
    }
    blocks
}

#[derive(Clone, Copy)]
struct Point {
    count: i64,
    sum: i64,
}

impl Point {
    fn slope_to(self, right: Self) -> Block {
        Block {
            sum: right.sum - self.sum,
            len: right.count - self.count,
        }
    }
}

/// Fit a nondecreasing sequence by least squares using a lower convex hull of the cumulative sums.
///
/// # Panics
///
/// Panics if the input has more than [`MAX_LEN`] observations.
#[must_use]
pub fn hull(input: &[i32]) -> Vec<Block> {
    assert!(input.len() <= MAX_LEN);
    let origin = Point { count: 0, sum: 0 };
    let mut vertices = Vec::with_capacity(input.len() + 1);
    vertices.push(origin);
    let mut point = origin;
    for &value in input {
        point.count += 1;
        point.sum += i64::from(value);
        while let [.., a, b] = vertices.as_slice() {
            if a.slope_to(*b).mean_cmp(b.slope_to(point)).is_lt() {
                break;
            }
            vertices.pop();
        }
        vertices.push(point);
    }
    vertices
        .windows(2)
        .map(|edge| edge[0].slope_to(edge[1]))
        .collect()
}
