//! Integer prefix scans with an independent quadratic model and three linear candidates.
//!
//! Every function preserves input order and uses [`i64::wrapping_add`]. The
//! resulting arithmetic is addition modulo `2^64`, represented as `i64`.

use std::fmt;

/// An invalid configuration for a scan candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScanError {
    /// The blocked scan received a block size of zero.
    ZeroBlockSize,
    /// The parallel scan received a worker count of zero.
    ZeroWorkers,
}

impl fmt::Display for ScanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroBlockSize => formatter.write_str("block size must be greater than zero"),
            Self::ZeroWorkers => formatter.write_str("worker count must be greater than zero"),
        }
    }
}

impl std::error::Error for ScanError {}

/// Computes an inclusive scan by recomputing each prefix from its start.
///
/// This deliberately simple, quadratic implementation is the independent
/// reference model for optimized inclusive candidates. Output position `i`
/// contains the wrapping sum of `input[0..=i]`.
#[must_use]
pub fn reference_inclusive(input: &[i64]) -> Vec<i64> {
    let mut output = Vec::with_capacity(input.len());

    for prefix_len in 1..=input.len() {
        let mut sum = 0_i64;
        for &value in &input[..prefix_len] {
            sum = sum.wrapping_add(value);
        }
        output.push(sum);
    }

    output
}

/// Computes an exclusive scan by recomputing each prefix from its start.
///
/// This deliberately simple, quadratic implementation is the independent
/// reference model for the optimized exclusive candidate. Output position `i`
/// contains `init.wrapping_add(sum(input[0..i]))`, with every addition applied
/// in input order.
#[must_use]
pub fn reference_exclusive(input: &[i64], init: i64) -> Vec<i64> {
    let mut output = Vec::with_capacity(input.len());

    for prefix_len in 0..input.len() {
        let mut sum = init;
        for &value in &input[..prefix_len] {
            sum = sum.wrapping_add(value);
        }
        output.push(sum);
    }

    output
}

/// Computes an inclusive scan in one left-to-right pass.
///
/// Output position `i` contains the wrapping sum of `input[0..=i]`.
#[must_use]
pub fn linear_inclusive(input: &[i64]) -> Vec<i64> {
    let mut output = Vec::with_capacity(input.len());
    let Some((&first, remaining)) = input.split_first() else {
        return output;
    };
    let mut sum = first;
    output.push(first);

    for &value in remaining {
        sum = sum.wrapping_add(value);
        output.push(sum);
    }

    output
}

/// Computes an exclusive scan in one left-to-right pass.
///
/// Output position `i` contains `init.wrapping_add(sum(input[0..i]))`, with
/// every addition applied in input order. The implementation does not add the
/// final input value because no output can observe that sum.
#[must_use]
pub fn linear_exclusive(input: &[i64], init: i64) -> Vec<i64> {
    let mut output = Vec::with_capacity(input.len());
    let mut sum = init;
    let values_that_feed_an_output = input.len().saturating_sub(1);

    for &value in &input[..values_that_feed_an_output] {
        output.push(sum);
        sum = sum.wrapping_add(value);
    }
    if !input.is_empty() {
        output.push(sum);
    }

    output
}

/// Computes an inclusive scan in local blocks and then applies block offsets.
///
/// The candidate writes each local prefix before adding the total of all
/// preceding blocks. It returns [`ScanError::ZeroBlockSize`] before reading or
/// copying the input when `block_size` is zero.
///
/// # Errors
///
/// Returns [`ScanError::ZeroBlockSize`] when `block_size` is zero.
pub fn blocked_inclusive(input: &[i64], block_size: usize) -> Result<Vec<i64>, ScanError> {
    if block_size == 0 {
        return Err(ScanError::ZeroBlockSize);
    }

    let block_count = input.len().div_ceil(block_size);
    let mut output = Vec::with_capacity(input.len());
    let mut block_totals = Vec::with_capacity(block_count);

    for block in input.chunks(block_size) {
        let Some((&first, remaining)) = block.split_first() else {
            continue;
        };
        let mut local_sum = first;
        output.push(first);
        for &value in remaining {
            local_sum = local_sum.wrapping_add(value);
            output.push(local_sum);
        }
        block_totals.push(local_sum);
    }

    let Some((&first_total, remaining_totals)) = block_totals.split_first() else {
        return Ok(output);
    };

    let mut output_blocks = output.chunks_mut(block_size);
    let _first_block = output_blocks.next();
    let remaining_count = remaining_totals.len();
    let mut carry = first_total;

    for (index, (block, &block_total)) in output_blocks.zip(remaining_totals).enumerate() {
        for value in block {
            *value = carry.wrapping_add(*value);
        }
        if index + 1 < remaining_count {
            carry = carry.wrapping_add(block_total);
        }
    }

    Ok(output)
}

/// Computes an inclusive scan with scoped worker threads and ordered offsets.
///
/// The effective worker count is `min(workers, input.len())`. Empty inputs use
/// no worker. An effective count of one scans on the calling thread. Every
/// effective count greater than one starts that many scoped threads in the
/// first phase and one fewer in the second; there is no input-length dispatch
/// threshold. The first phase scans contiguous, nonempty chunks. The caller
/// computes ordered chunk offsets and retains chunk zero, whose offset is the
/// identity. The second phase applies the remaining offsets. Output position
/// `i` is therefore the wrapping sum of `input[0..=i]`.
///
/// # Errors
///
/// Returns [`ScanError::ZeroWorkers`] when `workers` is zero, including for an
/// empty input.
pub fn parallel_inclusive(input: &[i64], workers: usize) -> Result<Vec<i64>, ScanError> {
    if workers == 0 {
        return Err(ScanError::ZeroWorkers);
    }
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let worker_count = workers.min(input.len());
    if worker_count == 1 {
        return Ok(linear_inclusive(input));
    }

    let base_chunk_len = input.len() / worker_count;
    let longer_chunk_count = input.len() % worker_count;
    let partials = std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(worker_count);
        let mut start = 0;

        for worker_index in 0..worker_count {
            let chunk_len = base_chunk_len + usize::from(worker_index < longer_chunk_count);
            let end = start + chunk_len;
            let chunk = &input[start..end];
            handles.push(scope.spawn(move || {
                let mut local_output = Vec::with_capacity(chunk.len());
                let Some((&first, remaining)) = chunk.split_first() else {
                    return (local_output, 0_i64);
                };
                let mut local_sum = first;
                local_output.push(first);
                for &value in remaining {
                    local_sum = local_sum.wrapping_add(value);
                    local_output.push(local_sum);
                }
                (local_output, local_sum)
            }));
            start = end;
        }

        handles
            .into_iter()
            .map(|handle| match handle.join() {
                Ok(partial) => partial,
                Err(payload) => std::panic::resume_unwind(payload),
            })
            .collect::<Vec<_>>()
    });

    let mut offsets = Vec::with_capacity(worker_count - 1);
    let mut carry = partials[0].1;
    for (chunk_index, (_, chunk_total)) in partials.iter().enumerate().skip(1) {
        offsets.push(carry);
        if chunk_index + 1 < worker_count {
            carry = carry.wrapping_add(*chunk_total);
        }
    }

    let mut partials = partials.into_iter();
    let Some((first_chunk, _first_total)) = partials.next() else {
        return Ok(Vec::new());
    };
    let repaired_chunks = std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(worker_count - 1);
        for ((mut chunk, _chunk_total), offset) in partials.zip(offsets) {
            handles.push(scope.spawn(move || {
                for value in &mut chunk {
                    *value = offset.wrapping_add(*value);
                }
                chunk
            }));
        }

        handles
            .into_iter()
            .map(|handle| match handle.join() {
                Ok(chunk) => chunk,
                Err(payload) => std::panic::resume_unwind(payload),
            })
            .collect::<Vec<_>>()
    });

    let mut output = Vec::with_capacity(input.len());
    output.extend(first_chunk);
    for chunk in repaired_chunks {
        output.extend(chunk);
    }

    Ok(output)
}
