//! Batch range addition with four implementations.
//!
//! Every update uses a half-open range. Arithmetic wraps modulo `2^64`, which
//! matches the behavior of [`i64::wrapping_add`] and [`i64::wrapping_sub`].

use std::fmt;

/// One addition over the half-open range `start..end`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RangeUpdate {
    /// The first updated index.
    pub start: usize,
    /// The first index after the updated range.
    pub end: usize,
    /// The value added to every covered element with wrapping arithmetic.
    pub delta: i64,
}

/// The first invalid update in input order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RangeError {
    /// The zero-based position of the invalid update.
    pub update_index: usize,
    /// The invalid range start.
    pub start: usize,
    /// The invalid range end.
    pub end: usize,
    /// The input length used to validate the range.
    pub len: usize,
}

impl fmt::Display for RangeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "update {} has invalid half-open range {}..{} for input length {}",
            self.update_index, self.start, self.end, self.len
        )
    }
}

impl std::error::Error for RangeError {}

/// Applies each update directly to every covered element.
///
/// This deliberately simple implementation is the independent reference
/// model. It validates the complete batch before cloning or changing data.
///
/// # Errors
///
/// Returns the first update for which `start > end` or `end > input.len()`.
pub fn apply_reference(input: &[i64], updates: &[RangeUpdate]) -> Result<Vec<i64>, RangeError> {
    for (update_index, update) in updates.iter().enumerate() {
        if update.start > update.end || update.end > input.len() {
            return Err(RangeError {
                update_index,
                start: update.start,
                end: update.end,
                len: input.len(),
            });
        }
    }

    let mut output = input.to_vec();
    for update in updates {
        for value in &mut output[update.start..update.end] {
            *value = value.wrapping_add(update.delta);
        }
    }
    Ok(output)
}

/// Applies the batch through a dense sidecar of boundary changes.
///
/// The sidecar has `input.len() + 1` entries, including a stop sentinel. A
/// prefix scan over the first `input.len()` entries turns boundary changes into
/// the active delta at each index.
///
/// # Errors
///
/// Returns the first update for which `start > end` or `end > input.len()`.
pub fn apply_dense_sidecar(input: &[i64], updates: &[RangeUpdate]) -> Result<Vec<i64>, RangeError> {
    for (update_index, update) in updates.iter().enumerate() {
        if update.start > update.end || update.end > input.len() {
            return Err(RangeError {
                update_index,
                start: update.start,
                end: update.end,
                len: input.len(),
            });
        }
    }

    let mut boundaries = vec![0_i64; input.len() + 1];
    for update in updates {
        if update.start == update.end || update.delta == 0 {
            continue;
        }
        boundaries[update.start] = boundaries[update.start].wrapping_add(update.delta);
        boundaries[update.end] = boundaries[update.end].wrapping_sub(update.delta);
    }

    let mut active = 0_i64;
    let mut output = Vec::with_capacity(input.len());
    for (&value, boundary) in input.iter().zip(boundaries) {
        active = active.wrapping_add(boundary);
        output.push(value.wrapping_add(active));
    }
    Ok(output)
}

/// Applies the batch after converting the output buffer to adjacent differences.
///
/// This candidate uses the returned vector as its difference array, so it does
/// not allocate a second `input.len()` sidecar.
///
/// # Errors
///
/// Returns the first update for which `start > end` or `end > input.len()`.
pub fn apply_in_place_difference(
    input: &[i64],
    updates: &[RangeUpdate],
) -> Result<Vec<i64>, RangeError> {
    for (update_index, update) in updates.iter().enumerate() {
        if update.start > update.end || update.end > input.len() {
            return Err(RangeError {
                update_index,
                start: update.start,
                end: update.end,
                len: input.len(),
            });
        }
    }

    let mut output = input.to_vec();
    for index in (1..output.len()).rev() {
        output[index] = output[index].wrapping_sub(output[index - 1]);
    }

    for update in updates {
        if update.start == update.end || update.delta == 0 {
            continue;
        }
        output[update.start] = output[update.start].wrapping_add(update.delta);
        if update.end < output.len() {
            output[update.end] = output[update.end].wrapping_sub(update.delta);
        }
    }

    for index in 1..output.len() {
        output[index] = output[index].wrapping_add(output[index - 1]);
    }
    Ok(output)
}

/// Applies the batch through sorted sparse boundary events.
///
/// This candidate stores boundaries only for nonempty updates with nonzero
/// deltas. Sorting the events lets one forward scan apply them to the input.
///
/// # Errors
///
/// Returns the first update for which `start > end` or `end > input.len()`.
pub fn apply_sorted_events(input: &[i64], updates: &[RangeUpdate]) -> Result<Vec<i64>, RangeError> {
    for (update_index, update) in updates.iter().enumerate() {
        if update.start > update.end || update.end > input.len() {
            return Err(RangeError {
                update_index,
                start: update.start,
                end: update.end,
                len: input.len(),
            });
        }
    }

    let mut events = Vec::with_capacity(updates.len().saturating_mul(2));
    for update in updates {
        if update.start == update.end || update.delta == 0 {
            continue;
        }
        events.push((update.start, update.delta));
        if update.end < input.len() {
            events.push((update.end, 0_i64.wrapping_sub(update.delta)));
        }
    }
    events.sort_unstable_by_key(|event| event.0);

    let mut output = Vec::with_capacity(input.len());
    let mut event_index = 0;
    let mut active = 0_i64;
    for (index, &value) in input.iter().enumerate() {
        while event_index < events.len() && events[event_index].0 == index {
            active = active.wrapping_add(events[event_index].1);
            event_index += 1;
        }
        output.push(value.wrapping_add(active));
    }
    Ok(output)
}
