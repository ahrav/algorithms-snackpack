//! Equality inner-join counting, preserving duplicate multiplicity.
//! Inputs are non-null u64 keys. Results count pairs without materializing them.
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

/// Independent quadratic reference for bounded tests.
#[must_use]
pub fn nested(left: &[u64], right: &[u64]) -> u128 {
    let mut count = 0;
    for a in left {
        for b in right {
            count += u128::from(a == b);
        }
    }
    count
}

/// Build a frequency table and probe it. Counts every matching occurrence.
#[must_use]
pub fn hash(left: &[u64], right: &[u64]) -> u128 {
    let mut table = HashMap::<u64, u64>::new();
    for &key in left {
        *table.entry(key).or_default() += 1;
    }
    right
        .iter()
        .map(|key| u128::from(*table.get(key).unwrap_or(&0)))
        .sum()
}

/// Sort copied inputs and multiply equal-run lengths. Includes copying/sorting.
#[must_use]
pub fn merge(left: &[u64], right: &[u64]) -> u128 {
    let mut a = left.to_vec();
    let mut b = right.to_vec();
    a.sort_unstable();
    b.sort_unstable();
    let (mut i, mut j, mut count) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                let (start_i, start_j, key) = (i, j, a[i]);
                while i < a.len() && a[i] == key {
                    i += 1;
                }
                while j < b.len() && b[j] == key {
                    j += 1;
                }
                count += (i - start_i) as u128 * (j - start_j) as u128;
            }
        }
    }
    count
}

fn mixed(mut key: u64) -> u64 {
    key ^= key >> 30;
    key = key.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    key ^= key >> 27;
    key = key.wrapping_mul(0x94d0_49bb_1331_11eb);
    key ^ (key >> 31)
}

fn bucket(key: u64, shift: u32, mask: usize) -> usize {
    usize::try_from((mixed(key) >> shift) & mask as u64).expect("masked bucket")
}

/// Resident 16-way partition followed by local tables. No hard memory bound.
#[must_use]
pub fn partition(left: &[u64], right: &[u64]) -> u128 {
    let mut a: [Vec<u64>; 16] = std::array::from_fn(|_| Vec::new());
    let mut b: [Vec<u64>; 16] = std::array::from_fn(|_| Vec::new());
    for &key in left {
        a[bucket(key, 0, 15)].push(key);
    }
    for &key in right {
        b[bucket(key, 0, 15)].push(key);
    }
    a.iter().zip(&b).map(|(x, y)| hash(x, y)).sum()
}

/// Cap build input rows per table and rescan the probe for each chunk.
/// This bounds table input rows, not total allocated bytes or resident inputs.
///
/// # Panics
/// Panics if `budget` is zero.
#[must_use]
pub fn chunk(left: &[u64], right: &[u64], budget: usize) -> u128 {
    assert!(budget > 0);
    left.chunks(budget).map(|part| hash(part, right)).sum()
}

/// Logical file traffic and progress evidence. OS cache traffic is not measured.
#[derive(Debug, Default)]
pub struct SpillStats {
    /// Records read by the join after source creation.
    pub reads: u64,
    /// Records written during repartition, excluding original source creation.
    pub writes: u64,
    /// Number of oversized nodes partitioned.
    pub partitions: usize,
    /// Number of nodes using chunked rescans after depth/no-progress guards.
    pub fallbacks: usize,
    /// Largest build chunk loaded at a leaf.
    pub max_build_rows: usize,
}

/// Write headerless little-endian keys. Refuses to replace an existing file.
///
/// # Errors
/// Returns creation/write errors.
pub fn write_keys(path: &Path, keys: &[u64]) -> io::Result<()> {
    let mut out = BufWriter::new(File::create_new(path)?);
    for key in keys {
        out.write_all(&key.to_le_bytes())?;
    }
    out.flush()
}

fn next_key(input: &mut impl Read) -> io::Result<Option<u64>> {
    let mut bytes = [0; 8];
    if input.read(&mut bytes[..1])? == 0 {
        return Ok(None);
    }
    input.read_exact(&mut bytes[1..])?;
    Ok(Some(u64::from_le_bytes(bytes)))
}

fn file_rows(path: &Path) -> io::Result<usize> {
    let bytes = path.metadata()?.len();
    if bytes % 8 != 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "partial key"));
    }
    usize::try_from(bytes / 8).map_err(io::Error::other)
}

struct Spill<'a> {
    scratch: &'a Path,
    budget: usize,
    max_depth: u32,
    serial: usize,
    stats: SpillStats,
}

impl Spill<'_> {
    fn split(&mut self, path: &Path, depth: u32) -> io::Result<Vec<PathBuf>> {
        let id = self.serial;
        self.serial += 1;
        let paths: Vec<_> = (0..4)
            .map(|b| self.scratch.join(format!("{id}-{b}.keys")))
            .collect();
        let mut out: Vec<_> = paths
            .iter()
            .map(|p| File::create_new(p).map(BufWriter::new))
            .collect::<io::Result<_>>()?;
        let mut input = BufReader::new(File::open(path)?);
        while let Some(key) = next_key(&mut input)? {
            self.stats.reads += 1;
            self.stats.writes += 1;
            out[bucket(key, depth * 2, 3)].write_all(&key.to_le_bytes())?;
        }
        for writer in &mut out {
            writer.flush()?;
        }
        Ok(paths)
    }

    fn leaf(&mut self, build: &Path, probe: &Path) -> io::Result<u128> {
        let mut input = BufReader::new(File::open(build)?);
        let mut result = 0;
        loop {
            let mut keys = Vec::with_capacity(self.budget.min(file_rows(build)?));
            for _ in 0..self.budget {
                if let Some(key) = next_key(&mut input)? {
                    keys.push(key);
                    self.stats.reads += 1;
                } else {
                    break;
                }
            }
            if keys.is_empty() {
                break;
            }
            self.stats.max_build_rows = self.stats.max_build_rows.max(keys.len());
            let mut table = HashMap::<u64, u64>::new();
            for key in keys {
                *table.entry(key).or_default() += 1;
            }
            let mut scan = BufReader::new(File::open(probe)?);
            while let Some(key) = next_key(&mut scan)? {
                self.stats.reads += 1;
                result += u128::from(*table.get(&key).unwrap_or(&0));
            }
        }
        Ok(result)
    }

    fn join(&mut self, build: &Path, probe: &Path, depth: u32) -> io::Result<u128> {
        let (n, m) = (file_rows(build)?, file_rows(probe)?);
        if n == 0 || m == 0 {
            return Ok(0);
        }
        if n <= self.budget {
            return self.leaf(build, probe);
        }
        if depth == self.max_depth {
            self.stats.fallbacks += 1;
            return self.leaf(build, probe);
        }
        self.stats.partitions += 1;
        let a = self.split(build, depth)?;
        let b = self.split(probe, depth)?;
        let mut result = 0;
        for (x, y) in a.iter().zip(&b) {
            if file_rows(x)? == n && file_rows(y)? > 0 {
                // The fallback handles stalled partitioning when the probe child has rows.
                self.stats.fallbacks += 1;
                result += self.leaf(x, y)?;
            } else {
                result += self.join(x, y, depth + 1)?;
            }
        }
        Ok(result)
    }
}

/// Recursive four-way file join with bounded-depth and no-progress fallback.
/// `scratch` must already exist and be exclusively owned by this call.
/// Leaves generated files for caller retention/cleanup, including on I/O errors.
/// `budget` is a build-row cap, not a process memory or allocator-byte cap.
///
/// # Errors
/// Returns invalid budget/depth, truncated-file, creation/read/write errors.
pub fn spill(
    build: &Path,
    probe: &Path,
    scratch: &Path,
    budget: usize,
    max_depth: u32,
) -> io::Result<(u128, SpillStats)> {
    if budget == 0 || max_depth > 31 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "budget/depth"));
    }
    let mut state = Spill {
        scratch,
        budget,
        max_depth,
        serial: 0,
        stats: SpillStats::default(),
    };
    let count = state.join(build, probe, 0)?;
    Ok((count, state.stats))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exhaustive_multisets() {
        let mut inputs = vec![vec![]];
        for len in 1..=4 {
            for code in 0..3_usize.pow(len) {
                let mut code = code;
                inputs.push(
                    (0..len)
                        .map(|_| {
                            let key = (code % 3) as u64;
                            code /= 3;
                            key
                        })
                        .collect(),
                );
            }
        }
        for a in &inputs {
            for b in &inputs {
                let expected = nested(a, b);
                assert_eq!(hash(a, b), expected);
                assert_eq!(merge(a, b), expected);
                assert_eq!(partition(a, b), expected);
                for budget in [1, 2, 5] {
                    assert_eq!(chunk(a, b, budget), expected);
                }
            }
        }
    }
    #[test]
    fn extreme_keys_and_output_growth() {
        let a = [0, u64::MAX, u64::MAX];
        let b = [u64::MAX, 0, u64::MAX];
        assert_eq!(nested(&a, &b), 5);
        assert_eq!(partition(&a, &b), 5);
        assert_eq!(merge(&vec![7; 1000], &vec![7; 1000]), 1_000_000);
    }
}
