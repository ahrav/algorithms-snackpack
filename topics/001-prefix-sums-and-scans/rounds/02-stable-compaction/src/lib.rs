//! Stable out-of-place compaction of u32 values by a boolean selection mask.
#![forbid(unsafe_code)]

use std::num::NonZeroUsize;

fn check_lengths(values: &[u32], keep: &[bool], output: &[u32]) {
    assert_eq!(values.len(), keep.len(), "one flag per value");
    assert!(
        output.len() >= values.len(),
        "output capacity for every input"
    );
}

/// Fuses the running rank with each selected write.
///
/// All candidates require equal value and flag lengths and an output slice at
/// least as long as the input. They return the initialized prefix length and
/// preserve the output suffix. Invalid lengths panic before mutation.
#[inline(never)]
pub fn cursor(values: &[u32], keep: &[bool], output: &mut [u32]) -> usize {
    check_lengths(values, keep, output);
    let mut next = 0;
    for (&value, &selected) in values.iter().zip(keep) {
        if selected {
            output[next] = value;
            next += 1;
        }
    }
    next
}

/// Materializes exclusive ranks, then scatters selected records.
///
/// The rank scratch needs one entry per flag. It is checked before writes.
#[inline(never)]
pub fn ranks(values: &[u32], keep: &[bool], output: &mut [u32], scratch: &mut [usize]) -> usize {
    check_lengths(values, keep, output);
    let ranks = &mut scratch[..keep.len()];
    let mut count = 0;
    for (&selected, rank) in keep.iter().zip(ranks.iter_mut()) {
        *rank = count;
        count += usize::from(selected);
    }
    for ((&value, &selected), &rank) in values.iter().zip(keep).zip(ranks.iter()) {
        if selected {
            output[rank] = value;
        }
    }
    count
}

/// Stores one exclusive offset per tile, then uses a local cursor for scatter.
///
/// Scratch needs `ceil(keep.len() / tile_len)` entries. Tile lengths may
/// exceed the input length. The count pass and scatter run on the caller.
#[inline(never)]
pub fn tiles(
    values: &[u32],
    keep: &[bool],
    output: &mut [u32],
    tile_len: NonZeroUsize,
    scratch: &mut [usize],
) -> usize {
    check_lengths(values, keep, output);
    let tile_len = tile_len.get();
    let offsets = &mut scratch[..keep.len().div_ceil(tile_len)];
    let mut count = 0;
    for (flags, offset) in keep.chunks(tile_len).zip(offsets.iter_mut()) {
        *offset = count;
        // Each batch fits in u32. Sum the batch totals at index width.
        // This also avoids an incorrect native Arm usize reduction observed
        // on rustc 1.98.1. The full-width total remains bounded by input length.
        for batch in flags.chunks(256) {
            count += batch
                .iter()
                .map(|&selected| u32::from(selected))
                .sum::<u32>() as usize;
        }
    }
    for ((values, flags), &offset) in values
        .chunks(tile_len)
        .zip(keep.chunks(tile_len))
        .zip(offsets.iter())
    {
        let mut next = offset;
        for (&value, &selected) in values.iter().zip(flags) {
            if selected {
                output[next] = value;
                next += 1;
            }
        }
    }
    count
}

/// Copies contiguous selected runs into consecutive output ranges.
///
/// Run discovery starts at the first unprocessed flag. This candidate uses
/// no caller scratch and preserves order within and across runs.
#[inline(never)]
pub fn runs(values: &[u32], keep: &[bool], output: &mut [u32]) -> usize {
    check_lengths(values, keep, output);
    let mut next = 0;
    let mut remaining = 0;
    while let Some(skip) = keep[remaining..].iter().position(|&selected| selected) {
        let start = remaining + skip;
        let end = start
            + keep[start..]
                .iter()
                .take_while(|&&selected| selected)
                .count();
        let len = end - start;
        output[next..next + len].copy_from_slice(&values[start..end]);
        next += len;
        remaining = end;
    }
    next
}

/// Returns the stable selection using a backward walk and a reversal.
pub fn reference(values: &[u32], keep: &[bool]) -> Vec<u32> {
    assert_eq!(values.len(), keep.len());
    let mut result = Vec::new();
    for i in (0..keep.len()).rev() {
        if keep[i] {
            result.push(values[i]);
        }
    }
    result.reverse();
    result
}
