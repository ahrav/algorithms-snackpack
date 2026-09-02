//! Exact counting of budget-bounded contiguous subarrays over nonnegative values.
//!
//! Every function in this crate implements the same contract. Given `values`
//! and `budget`, it counts the nonempty half-open ranges `[start, end)` whose
//! mathematical sum is at most `budget`. Equal values at different indices are
//! distinct inputs, so their ranges are counted separately. The result is exact
//! in `u128`.
//!
//! The optimized sliding-window implementations rely on one crucial property:
//! values are `u64`, so extending a range cannot decrease its sum. That
//! monotonicity does not hold for signed inputs.

/// Counts budget-bounded subarrays with the default direct sliding window.
///
/// This is the production-facing entry point. It runs in `O(n)` time, uses
/// `O(1)` auxiliary space, and delegates to [`direct_sliding`].
#[must_use]
pub fn count_bounded_subarrays(values: &[u64], budget: u128) -> u128 {
    direct_sliding(values, budget)
}

/// Counts ranges by recomputing every candidate range from scratch.
///
/// This deliberately independent `O(n^3)` implementation is the correctness
/// oracle. It does not reuse a running sum, stop early, or call another
/// candidate.
#[must_use]
pub fn recompute_reference(values: &[u64], budget: u128) -> u128 {
    let mut count = 0_u128;

    for start in 0..values.len() {
        for end in (start + 1)..=values.len() {
            let sum = values[start..end]
                .iter()
                .map(|&value| u128::from(value))
                .sum::<u128>();
            if sum <= budget {
                count += 1;
            }
        }
    }

    count
}

/// Counts ranges with an incremental sum and an early exit per start index.
///
/// Because every value is nonnegative, once a range exceeds the budget, every
/// longer range with the same start also exceeds it. The worst case is
/// `O(n^2)` time and `O(1)` auxiliary space.
#[must_use]
pub fn quadratic_early_exit(values: &[u64], budget: u128) -> u128 {
    let mut count = 0_u128;

    for start in 0..values.len() {
        let mut sum = 0_u128;
        for &value in &values[start..] {
            sum += u128::from(value);
            if sum > budget {
                break;
            }
            count += 1;
        }
    }

    count
}

/// Counts ranges with the direct variable-width sliding-window algorithm.
///
/// After processing each right endpoint, `left` is the smallest start whose
/// suffix ending there is within budget. All later starts are also valid, so
/// that endpoint contributes `right + 1 - left` ranges. Each element enters
/// and leaves the window at most once, giving `O(n)` time and `O(1)` auxiliary
/// space.
#[must_use]
pub fn direct_sliding(values: &[u64], budget: u128) -> u128 {
    let mut left = 0_usize;
    let mut sum = 0_u128;
    let mut count = 0_u128;

    for (right, &value) in values.iter().enumerate() {
        sum += u128::from(value);
        while sum > budget {
            sum -= u128::from(values[left]);
            left += 1;
        }

        count += usize_as_u128(right + 1 - left);
    }

    count
}

/// Counts ranges with a sliding window that explicitly resets on oversized values.
///
/// A value larger than `budget` cannot belong to a valid range, so it separates
/// the input into independent runs. This variant makes that separator fast path
/// explicit. It has the same `O(n)` time and `O(1)` auxiliary-space bounds as
/// [`direct_sliding`].
#[must_use]
pub fn oversized_reset_sliding(values: &[u64], budget: u128) -> u128 {
    let mut left = 0_usize;
    let mut sum = 0_u128;
    let mut count = 0_u128;

    for (right, &value) in values.iter().enumerate() {
        let value = u128::from(value);
        if value > budget {
            left = right + 1;
            sum = 0;
            continue;
        }

        sum += value;
        while sum > budget {
            sum -= u128::from(values[left]);
            left += 1;
        }

        count += usize_as_u128(right + 1 - left);
    }

    count
}

/// Counts ranges by binary-searching a nondecreasing prefix-sum array.
///
/// For each exclusive end `e`, a start `s < e` is valid exactly when
/// `prefix[s] >= prefix[e] - budget`. [`slice::partition_point`] finds the first
/// such prefix, including the correct boundary when zero values create equal
/// prefix sums. This implementation runs in `O(n log n)` time and uses `O(n)`
/// auxiliary space.
#[must_use]
pub fn prefix_binary_search(values: &[u64], budget: u128) -> u128 {
    let mut prefix = Vec::with_capacity(values.len() + 1);
    prefix.push(0_u128);
    let mut running_sum = 0_u128;
    for &value in values {
        running_sum += u128::from(value);
        prefix.push(running_sum);
    }

    let mut count = 0_u128;
    for end in 1..prefix.len() {
        let threshold = prefix[end].saturating_sub(budget);
        let first_valid = prefix[..end].partition_point(|&sum| sum < threshold);
        count += usize_as_u128(end - first_valid);
    }

    count
}

fn usize_as_u128(value: usize) -> u128 {
    u128::try_from(value).expect("usize is no wider than u128 on supported targets")
}
