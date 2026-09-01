//! Strict fresh-process harness for Topic 003 bitmap candidates.

#![allow(clippy::too_many_lines)]

use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::process::ExitCode;
use std::time::Instant;

use topic_003_density_adaptive_bitmaps::{
    AdaptiveBitmap, ContainerKind, DenseBitSet, ReferenceSet, SortedSet,
};

const SCHEMA_VERSION: u32 = 1;
const GENERATOR_VERSION: &str = "topic003-density-fixtures-v1";
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn main() -> ExitCode {
    if std::env::args_os().nth(1).is_none() {
        return ExitCode::SUCCESS;
    }
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("bitmap_sets benchmark: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let config = Config::parse()?;
    let fixture = Fixture::generate(config.workload, config.seed)?;
    let canary = Canary::derive(&fixture)?;
    verify_candidate(config.candidate, &fixture, &canary)?;

    for _ in 0..config.warmups {
        black_box(run_candidate(config.candidate, &fixture)?);
    }

    let mut total_sample_ns = Vec::with_capacity(config.samples);
    let mut build_sample_ns = Vec::with_capacity(config.samples);
    let mut contains_sample_ns = Vec::with_capacity(config.samples);
    let mut intersection_sample_ns = Vec::with_capacity(config.samples);
    let mut aggregate_checksum = 0_u64;
    for _ in 0..config.samples {
        let mut total = 0_u128;
        let mut build = 0_u128;
        let mut contains = 0_u128;
        let mut intersection = 0_u128;
        for _ in 0..config.profile_repetitions {
            let sample = run_candidate(config.candidate, &fixture)?;
            total = total
                .checked_add(sample.total_ns)
                .ok_or_else(|| "total timer accumulation overflowed".to_owned())?;
            build = build
                .checked_add(sample.build_ns)
                .ok_or_else(|| "build timer accumulation overflowed".to_owned())?;
            contains = contains
                .checked_add(sample.contains_ns)
                .ok_or_else(|| "contains timer accumulation overflowed".to_owned())?;
            intersection = intersection
                .checked_add(sample.intersection_ns)
                .ok_or_else(|| "intersection timer accumulation overflowed".to_owned())?;
            aggregate_checksum ^= sample.checksum.rotate_left(13);
        }
        if total != build + contains + intersection {
            return Err("composite timer did not equal its component timers".to_owned());
        }
        total_sample_ns.push(total);
        build_sample_ns.push(build);
        contains_sample_ns.push(contains);
        intersection_sample_ns.push(intersection);
    }
    black_box(aggregate_checksum);

    let candidate_pair = build_pair(config.candidate, &fixture)?;
    let payload_bytes_a = candidate_pair.0.payload_bytes();
    let payload_bytes_b = candidate_pair.1.payload_bytes();
    let adaptive_a = AdaptiveBitmap::try_new(fixture.universe, &fixture.raw_a)
        .map_err(|error| format!("adaptive summary A failed: {error}"))?;
    let adaptive_b = AdaptiveBitmap::try_new(fixture.universe, &fixture.raw_b)
        .map_err(|error| format!("adaptive summary B failed: {error}"))?;
    let summary_a = SummaryCounts::from_adaptive(&adaptive_a);
    let summary_b = SummaryCounts::from_adaptive(&adaptive_b);

    print_record(
        &config,
        &fixture,
        &canary,
        payload_bytes_a,
        payload_bytes_b,
        &summary_a,
        &summary_b,
        &total_sample_ns,
        &build_sample_ns,
        &contains_sample_ns,
        &intersection_sample_ns,
        aggregate_checksum,
    );
    Ok(())
}

#[derive(Clone, Copy, Debug)]
enum Candidate {
    Reference,
    Sorted,
    Dense,
    Adaptive,
}

impl Candidate {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "reference" => Ok(Self::Reference),
            "sorted" => Ok(Self::Sorted),
            "dense" => Ok(Self::Dense),
            "adaptive" => Ok(Self::Adaptive),
            _ => Err(format!("unsupported candidate {value:?}")),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Reference => "reference",
            Self::Sorted => "sorted",
            Self::Dense => "dense",
            Self::Adaptive => "adaptive",
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Workload {
    TinySparseShuffled,
    WideSparseLowOverlap,
    SkewedSparse,
    DenseLocalHighOverlap,
    DenseWideMediumOverlap,
    LongRuns,
    MixedChunkShapes,
}

impl Workload {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "tiny_sparse_shuffled" => Ok(Self::TinySparseShuffled),
            "wide_sparse_low_overlap" => Ok(Self::WideSparseLowOverlap),
            "skewed_sparse" => Ok(Self::SkewedSparse),
            "dense_local_high_overlap" => Ok(Self::DenseLocalHighOverlap),
            "dense_wide_medium_overlap" => Ok(Self::DenseWideMediumOverlap),
            "long_runs" => Ok(Self::LongRuns),
            "mixed_chunk_shapes" => Ok(Self::MixedChunkShapes),
            _ => Err(format!("unsupported workload {value:?}")),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::TinySparseShuffled => "tiny_sparse_shuffled",
            Self::WideSparseLowOverlap => "wide_sparse_low_overlap",
            Self::SkewedSparse => "skewed_sparse",
            Self::DenseLocalHighOverlap => "dense_local_high_overlap",
            Self::DenseWideMediumOverlap => "dense_wide_medium_overlap",
            Self::LongRuns => "long_runs",
            Self::MixedChunkShapes => "mixed_chunk_shapes",
        }
    }
}

struct Config {
    candidate: Candidate,
    workload: Workload,
    seed: u64,
    warmups: usize,
    samples: usize,
    profile_repetitions: usize,
}

impl Config {
    fn parse() -> Result<Self, String> {
        let mut flags = parse_flags()?;
        let candidate = Candidate::parse(&take(&mut flags, "candidate")?)?;
        let workload = Workload::parse(&take(&mut flags, "workload")?)?;
        let seed = parse_number(&take(&mut flags, "seed")?, "seed")?;
        let warmups = parse_number(&take(&mut flags, "warmups")?, "warmups")?;
        let samples = parse_number(&take(&mut flags, "samples")?, "samples")?;
        let profile_repetitions = parse_number(
            &take(&mut flags, "profile-repetitions")?,
            "profile-repetitions",
        )?;
        if samples == 0 || profile_repetitions == 0 {
            return Err("samples and profile-repetitions must be positive".to_owned());
        }
        if !flags.is_empty() {
            return Err(format!(
                "unknown flags: {}",
                flags.keys().cloned().collect::<Vec<_>>().join(", ")
            ));
        }
        Ok(Self {
            candidate,
            workload,
            seed,
            warmups,
            samples,
            profile_repetitions,
        })
    }
}

fn parse_flags() -> Result<BTreeMap<String, String>, String> {
    let mut arguments = std::env::args().skip(1);
    let mut flags = BTreeMap::new();
    while let Some(flag) = arguments.next() {
        if flag == "--bench" {
            continue;
        }
        let name = flag
            .strip_prefix("--")
            .ok_or_else(|| format!("expected --flag, found {flag:?}"))?;
        let value = arguments
            .next()
            .ok_or_else(|| format!("missing value for --{name}"))?;
        if flags.insert(name.to_owned(), value).is_some() {
            return Err(format!("duplicate flag --{name}"));
        }
    }
    Ok(flags)
}

fn take(flags: &mut BTreeMap<String, String>, name: &str) -> Result<String, String> {
    flags
        .remove(name)
        .ok_or_else(|| format!("missing required flag --{name}"))
}

fn parse_number<T>(value: &str, name: &str) -> Result<T, String>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    value
        .parse()
        .map_err(|error| format!("invalid --{name} value {value:?}: {error}"))
}

struct Fixture {
    universe: u64,
    raw_a: Vec<u32>,
    raw_b: Vec<u32>,
    probes: Vec<Probe>,
    intersection_calls: usize,
    build_repetitions: usize,
    expected_unique_a: usize,
    expected_unique_b: usize,
    expected_overlap: Option<usize>,
    expected_hit_percent: usize,
}

#[derive(Clone, Copy)]
enum Operand {
    A,
    B,
}

#[derive(Clone, Copy)]
struct Probe {
    operand: Operand,
    value: u32,
}

impl Fixture {
    fn generate(workload: Workload, seed: u64) -> Result<Self, String> {
        let mut rng = Lcg::new(seed);
        let mut fixture = match workload {
            Workload::TinySparseShuffled => {
                let (a, b) = uniform_pair(1_024, 24, 24, 8, &mut rng)?;
                let raw_a = with_duplicates(&a, a.len());
                let raw_b = with_duplicates(&b, b.len());
                Self::new(1_024, raw_a, raw_b, 256, 50, 256, 128, Some(8), &mut rng)?
            }
            Workload::WideSparseLowOverlap => {
                let (a, b) = uniform_pair(1 << 24, 4_096, 4_096, 128, &mut rng)?;
                let raw_a = with_duplicates(&a, a.len() / 10);
                let raw_b = with_duplicates(&b, b.len() / 10);
                Self::new(1 << 24, raw_a, raw_b, 4_096, 25, 64, 4, Some(128), &mut rng)?
            }
            Workload::SkewedSparse => {
                let (a, b) = uniform_pair(1 << 24, 64, 65_536, 32, &mut rng)?;
                Self::new(1 << 24, a, b, 4_096, 50, 64, 1, Some(32), &mut rng)?
            }
            Workload::DenseLocalHighOverlap => {
                let (a, b) = uniform_pair(65_536, 49_152, 49_152, 40_960, &mut rng)?;
                let raw_a = with_duplicates(&a, a.len() * 5 / 100);
                let raw_b = with_duplicates(&b, b.len() * 5 / 100);
                Self::new(
                    65_536,
                    raw_a,
                    raw_b,
                    4_096,
                    90,
                    64,
                    1,
                    Some(40_960),
                    &mut rng,
                )?
            }
            Workload::DenseWideMediumOverlap => {
                let (a, b) = uniform_pair(1 << 18, 131_072, 131_072, 65_536, &mut rng)?;
                Self::new(1 << 18, a, b, 4_096, 50, 64, 1, Some(65_536), &mut rng)?
            }
            Workload::LongRuns => {
                let (a, b) = long_run_pair();
                Self::new(1 << 20, a, b, 4_096, 50, 64, 2, Some(16_384), &mut rng)?
            }
            Workload::MixedChunkShapes => {
                let (a, b) = mixed_chunk_pair();
                Self::new(1 << 20, a, b, 4_096, 50, 64, 1, Some(65_798), &mut rng)?
            }
        };
        rng.shuffle(&mut fixture.raw_a);
        rng.shuffle(&mut fixture.raw_b);
        Ok(fixture)
    }

    #[allow(clippy::too_many_arguments)]
    fn new(
        universe: u64,
        raw_a: Vec<u32>,
        raw_b: Vec<u32>,
        probe_count: usize,
        hit_percent: usize,
        intersection_calls: usize,
        build_repetitions: usize,
        expected_overlap: Option<usize>,
        rng: &mut Lcg,
    ) -> Result<Self, String> {
        let unique_a = raw_a.iter().copied().collect::<BTreeSet<_>>();
        let unique_b = raw_b.iter().copied().collect::<BTreeSet<_>>();
        let probes = generate_probes(
            &unique_a,
            &unique_b,
            universe,
            probe_count,
            hit_percent,
            rng,
        )?;
        Ok(Self {
            universe,
            raw_a,
            raw_b,
            probes,
            intersection_calls,
            build_repetitions,
            expected_unique_a: unique_a.len(),
            expected_unique_b: unique_b.len(),
            expected_overlap,
            expected_hit_percent: hit_percent,
        })
    }
}

fn uniform_pair(
    universe: u64,
    count_a: usize,
    count_b: usize,
    overlap: usize,
    rng: &mut Lcg,
) -> Result<(Vec<u32>, Vec<u32>), String> {
    if overlap > count_a || overlap > count_b {
        return Err("overlap exceeds an operand cardinality".to_owned());
    }
    let union = count_a + count_b - overlap;
    let universe_size = usize::try_from(universe).map_err(|_| "universe does not fit usize")?;
    if union > universe_size || !universe.is_power_of_two() {
        return Err("uniform generator requires a fitting power-of-two universe".to_owned());
    }
    let chunks = usize::try_from(universe.div_ceil(65_536)).map_err(|_| "chunk count")?;
    let multiplier = u32::try_from((rng.next_u64() | 1) & 0xffff).expect("masked multiplier");
    let offset = u32::try_from(rng.next_u64() & 0xffff).expect("masked offset");
    let value_at = |index: usize| -> Result<u32, String> {
        let chunk = index % chunks;
        let round = index / chunks;
        let round_u32 = u32::try_from(round).map_err(|_| "round does not fit u32")?;
        let low_mask = if universe < 65_536 {
            u32::try_from(universe - 1).map_err(|_| "tiny universe mask does not fit u32")?
        } else {
            0xffff
        };
        let low = round_u32.wrapping_mul(multiplier).wrapping_add(offset) & low_mask;
        let high = u32::try_from(chunk).map_err(|_| "chunk does not fit u32")? << 16;
        Ok(high | low)
    };
    let mut common = Vec::with_capacity(overlap);
    let mut a_only = Vec::with_capacity(count_a - overlap);
    let mut b_only = Vec::with_capacity(count_b - overlap);
    for index in 0..union {
        let value = value_at(index)?;
        if index < overlap {
            common.push(value);
        } else if index < count_a {
            a_only.push(value);
        } else {
            b_only.push(value);
        }
    }
    let mut a = common.clone();
    a.extend(a_only);
    let mut b = common;
    b.extend(b_only);
    if a.iter().copied().collect::<BTreeSet<_>>().len() != count_a
        || b.iter().copied().collect::<BTreeSet<_>>().len() != count_b
    {
        return Err("uniform generator did not produce the requested cardinality".to_owned());
    }
    Ok((a, b))
}

fn with_duplicates(values: &[u32], duplicate_count: usize) -> Vec<u32> {
    let mut output = values.to_vec();
    output.extend((0..duplicate_count).map(|index| values[index % values.len()]));
    output
}

fn generate_probes(
    present_a: &BTreeSet<u32>,
    present_b: &BTreeSet<u32>,
    universe: u64,
    count: usize,
    hit_percent: usize,
    rng: &mut Lcg,
) -> Result<Vec<Probe>, String> {
    if !count.is_multiple_of(2) {
        return Err("balanced operand probe count must be even".to_owned());
    }
    let hit_count = count * hit_percent / 100;
    if !hit_count.is_multiple_of(2) {
        return Err("balanced operand hit count must be even".to_owned());
    }
    let per_operand_count = count / 2;
    let per_operand_hits = hit_count / 2;
    let mut a = generate_operand_probes(
        present_a,
        universe,
        per_operand_count,
        per_operand_hits,
        rng,
    )?;
    let mut b = generate_operand_probes(
        present_b,
        universe,
        per_operand_count,
        per_operand_hits,
        rng,
    )?;
    let mut probes = Vec::with_capacity(count);
    for index in 0..per_operand_count {
        probes.push(Probe {
            operand: Operand::A,
            value: a[index],
        });
        probes.push(Probe {
            operand: Operand::B,
            value: b[index],
        });
    }
    a.clear();
    b.clear();
    Ok(probes)
}

fn generate_operand_probes(
    present: &BTreeSet<u32>,
    universe: u64,
    count: usize,
    hit_count: usize,
    rng: &mut Lcg,
) -> Result<Vec<u32>, String> {
    let miss_count = count - hit_count;
    if present.is_empty() && hit_count != 0 {
        return Err("cannot generate hit probes for an empty set".to_owned());
    }
    let present_values = present.iter().copied().collect::<Vec<_>>();
    let mut probes = (0..hit_count)
        .map(|index| present_values[index % present_values.len()])
        .collect::<Vec<_>>();
    let mut candidate = 0_u64;
    while probes.len() < count {
        if candidate >= universe {
            return Err(format!("could not generate {miss_count} miss probes"));
        }
        let value = u32::try_from(candidate).map_err(|_| "probe does not fit u32")?;
        if !present.contains(&value) {
            probes.push(value);
        }
        candidate += 1;
    }
    rng.shuffle(&mut probes);
    Ok(probes)
}

fn long_run_pair() -> (Vec<u32>, Vec<u32>) {
    let mut a = Vec::with_capacity(32_768);
    let mut b = Vec::with_capacity(32_768);
    for run in 0_u32..64 {
        let start = run * 16_384;
        a.extend(start..start + 512);
        b.extend(start + 256..start + 768);
    }
    (a, b)
}

fn push_chunk(target: &mut Vec<u32>, key: u16, lows: impl IntoIterator<Item = u16>) {
    let high = u32::from(key) << 16;
    target.extend(lows.into_iter().map(|low| high | u32::from(low)));
}

fn scattered(count: usize, offset: u16) -> Vec<u16> {
    (0..count)
        .map(|index| {
            let index = u32::try_from(index).expect("local index fits u32");
            u16::try_from((index * 251 + u32::from(offset)) & 0xffff).expect("masked low fits u16")
        })
        .collect()
}

fn contiguous(start: u16, count: usize) -> Vec<u16> {
    let start = u32::from(start);
    (0..count)
        .map(|offset| {
            u16::try_from(start + u32::try_from(offset).expect("offset fits u32"))
                .expect("contiguous range fits u16")
        })
        .collect()
}

fn fragmented(offset: u16) -> Vec<u16> {
    (0_u32..32_768)
        .filter(|value| (value + u32::from(offset)) % 4 < 2)
        .map(|value| u16::try_from(value).expect("fragmented low fits u16"))
        .collect()
}

fn mixed_chunk_pair() -> (Vec<u32>, Vec<u32>) {
    let mut a = Vec::with_capacity(193_304);
    let mut b = Vec::with_capacity(193_304);
    push_chunk(&mut a, 0, scattered(24, 0));
    push_chunk(&mut b, 0, scattered(24, 8));
    push_chunk(&mut a, 1, scattered(4_095, 1));
    push_chunk(&mut b, 1, scattered(4_095, 2_049));
    push_chunk(&mut a, 2, scattered(4_096, 2));
    push_chunk(&mut b, 2, scattered(4_096, 2_050));
    push_chunk(&mut a, 3, scattered(4_097, 3));
    push_chunk(&mut b, 3, scattered(4_097, 2_051));
    push_chunk(
        &mut a,
        4,
        (0_u32..65_536)
            .step_by(2)
            .map(|value| u16::try_from(value).expect("even low")),
    );
    push_chunk(
        &mut b,
        4,
        (1_u32..65_536)
            .step_by(2)
            .map(|value| u16::try_from(value).expect("odd low")),
    );
    push_chunk(&mut a, 5, contiguous(0, 49_152));
    push_chunk(&mut b, 5, contiguous(16_384, 49_152));
    push_chunk(&mut a, 6, contiguous(0, 16_384));
    push_chunk(&mut b, 6, contiguous(8_192, 16_384));
    push_chunk(&mut a, 7, fragmented(0));
    push_chunk(&mut b, 7, fragmented(1));
    push_chunk(&mut a, 8, scattered(256, 8));
    push_chunk(&mut b, 8, fragmented(0));
    push_chunk(&mut a, 9, fragmented(0));
    push_chunk(&mut b, 9, scattered(256, 9));
    push_chunk(&mut a, 10, contiguous(0, 16_384));
    push_chunk(&mut b, 10, scattered(256, 10));
    push_chunk(&mut a, 11, scattered(256, 11));
    push_chunk(&mut b, 11, contiguous(8_192, 16_384));
    push_chunk(&mut a, 12, contiguous(0, 16_384));
    push_chunk(&mut b, 12, fragmented(0));
    push_chunk(&mut a, 13, fragmented(0));
    push_chunk(&mut b, 13, contiguous(8_192, 16_384));
    push_chunk(&mut a, 14, scattered(256, 14));
    push_chunk(&mut b, 15, scattered(256, 15));
    assert_eq!(a.len(), 193_304);
    assert_eq!(b.len(), 193_304);
    (a, b)
}

struct Lcg(u64);

impl Lcg {
    const fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }

    fn below(&mut self, bound: usize) -> usize {
        if bound <= 1 {
            return 0;
        }
        let bound_u64 = u64::try_from(bound).expect("shuffle bound fits u64");
        usize::try_from(self.next_u64() % bound_u64).expect("bounded value fits usize")
    }

    fn shuffle<T>(&mut self, values: &mut [T]) {
        for upper in (1..values.len()).rev() {
            let other = self.below(upper + 1);
            values.swap(upper, other);
        }
    }
}

struct Canary {
    unique_a: Vec<u32>,
    unique_b: Vec<u32>,
    expected_intersection: usize,
    probe_hits: usize,
    probe_hits_a: usize,
    probe_hits_b: usize,
    probe_count_a: usize,
    probe_count_b: usize,
    occupied_chunks_a: usize,
    occupied_chunks_b: usize,
    expected_containers_a: Vec<ContainerRecord>,
    expected_containers_b: Vec<ContainerRecord>,
    fixture_hash: u64,
    expected_hash: u64,
}

impl Canary {
    fn derive(fixture: &Fixture) -> Result<Self, String> {
        let unique_a = fixture
            .raw_a
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let unique_b = fixture
            .raw_b
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if unique_a.len() != fixture.expected_unique_a {
            return Err(format!(
                "fixture A cardinality was {}, expected {}",
                unique_a.len(),
                fixture.expected_unique_a
            ));
        }
        if unique_b.len() != fixture.expected_unique_b {
            return Err(format!(
                "fixture B cardinality was {}, expected {}",
                unique_b.len(),
                fixture.expected_unique_b
            ));
        }
        let expected_intersection = sorted_intersection_len(&unique_a, &unique_b);
        if fixture
            .expected_overlap
            .is_some_and(|expected| expected != expected_intersection)
        {
            return Err(format!(
                "fixture overlap was {expected_intersection}, expected {}",
                fixture.expected_overlap.unwrap_or_default()
            ));
        }
        let unique_a_set = unique_a.iter().copied().collect::<BTreeSet<_>>();
        let unique_b_set = unique_b.iter().copied().collect::<BTreeSet<_>>();
        let probe_hits_a = fixture
            .probes
            .iter()
            .filter(|probe| {
                matches!(probe.operand, Operand::A) && unique_a_set.contains(&probe.value)
            })
            .count();
        let probe_hits_b = fixture
            .probes
            .iter()
            .filter(|probe| {
                matches!(probe.operand, Operand::B) && unique_b_set.contains(&probe.value)
            })
            .count();
        let probe_hits = probe_hits_a + probe_hits_b;
        let probe_count_a = fixture
            .probes
            .iter()
            .filter(|probe| matches!(probe.operand, Operand::A))
            .count();
        let probe_count_b = fixture
            .probes
            .iter()
            .filter(|probe| matches!(probe.operand, Operand::B))
            .count();
        if probe_count_a != fixture.probes.len() / 2 || probe_count_b != fixture.probes.len() / 2 {
            return Err(format!(
                "fixture probe operands were A={probe_count_a}, B={probe_count_b}, expected {} each",
                fixture.probes.len() / 2
            ));
        }
        let expected_probe_hits = fixture.probes.len() * fixture.expected_hit_percent / 100;
        if probe_hits != expected_probe_hits {
            return Err(format!(
                "fixture probe hits were {probe_hits}, expected {expected_probe_hits}"
            ));
        }
        let occupied_chunks_a = occupied_chunks(&unique_a);
        let occupied_chunks_b = occupied_chunks(&unique_b);
        let expected_containers_a = expected_container_manifest(&unique_a);
        let expected_containers_b = expected_container_manifest(&unique_b);
        let fixture_hash = fixture_fingerprint(fixture);
        let expected_hash = semantic_fingerprint(
            fixture.universe,
            &unique_a,
            &unique_b,
            expected_intersection,
            probe_hits,
            probe_hits_a,
            probe_hits_b,
        );
        Ok(Self {
            unique_a,
            unique_b,
            expected_intersection,
            probe_hits,
            probe_hits_a,
            probe_hits_b,
            probe_count_a,
            probe_count_b,
            occupied_chunks_a,
            occupied_chunks_b,
            expected_containers_a,
            expected_containers_b,
            fixture_hash,
            expected_hash,
        })
    }
}

fn expected_container_manifest(values: &[u32]) -> Vec<ContainerRecord> {
    let mut records = Vec::new();
    let mut start = 0_usize;
    while start < values.len() {
        let key = u16::try_from(values[start] >> 16).expect("high key fits u16");
        let mut end = start + 1;
        while end < values.len() && values[end] >> 16 == u32::from(key) {
            end += 1;
        }
        let lows = values[start..end]
            .iter()
            .map(|value| u16::try_from(value & 0xffff).expect("low value fits u16"))
            .collect::<Vec<_>>();
        let run_count = 1 + lows
            .windows(2)
            .filter(|pair| u32::from(pair[1]) != u32::from(pair[0]) + 1)
            .count();
        let cardinality = lows.len();
        let baseline_payload = if cardinality <= 4_096 {
            u64::try_from(cardinality).expect("cardinality fits u64") * 2
        } else {
            8_192
        };
        let run_payload = 2 + u64::try_from(run_count).expect("run count fits u64") * 4;
        let (kind, payload_bytes) = if run_payload < baseline_payload {
            (ContainerKind::Run, run_payload)
        } else if cardinality <= 4_096 {
            (ContainerKind::Array, baseline_payload)
        } else {
            (ContainerKind::Bitmap, baseline_payload)
        };
        records.push(ContainerRecord {
            key,
            kind,
            cardinality,
            run_count,
            payload_bytes,
        });
        start = end;
    }
    records
}

fn sorted_intersection_len(left: &[u32], right: &[u32]) -> usize {
    let mut left_index = 0_usize;
    let mut right_index = 0_usize;
    let mut count = 0_usize;
    while left_index < left.len() && right_index < right.len() {
        match left[left_index].cmp(&right[right_index]) {
            std::cmp::Ordering::Less => left_index += 1,
            std::cmp::Ordering::Greater => right_index += 1,
            std::cmp::Ordering::Equal => {
                count += 1;
                left_index += 1;
                right_index += 1;
            }
        }
    }
    count
}

fn occupied_chunks(values: &[u32]) -> usize {
    values
        .iter()
        .map(|value| value >> 16)
        .collect::<BTreeSet<_>>()
        .len()
}

fn hash_u64(mut state: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        state ^= u64::from(byte);
        state = state.wrapping_mul(FNV_PRIME);
    }
    state
}

fn hash_values(mut state: u64, values: &[u32]) -> u64 {
    state = hash_u64(
        state,
        u64::try_from(values.len()).expect("value count fits u64"),
    );
    for &value in values {
        state = hash_u64(state, u64::from(value));
    }
    state
}

fn hash_probes(mut state: u64, probes: &[Probe]) -> u64 {
    state = hash_u64(
        state,
        u64::try_from(probes.len()).expect("probe count fits u64"),
    );
    for probe in probes {
        state = hash_u64(
            state,
            match probe.operand {
                Operand::A => 0,
                Operand::B => 1,
            },
        );
        state = hash_u64(state, u64::from(probe.value));
    }
    state
}

fn fixture_fingerprint(fixture: &Fixture) -> u64 {
    let mut state = hash_u64(FNV_OFFSET, fixture.universe);
    state = hash_values(state, &fixture.raw_a);
    state = hash_values(state, &fixture.raw_b);
    state = hash_probes(state, &fixture.probes);
    state = hash_u64(
        state,
        u64::try_from(fixture.intersection_calls).expect("call count fits u64"),
    );
    hash_u64(
        state,
        u64::try_from(fixture.build_repetitions).expect("build count fits u64"),
    )
}

fn semantic_fingerprint(
    universe: u64,
    unique_a: &[u32],
    unique_b: &[u32],
    expected_intersection: usize,
    probe_hits: usize,
    probe_hits_a: usize,
    probe_hits_b: usize,
) -> u64 {
    let mut state = hash_u64(FNV_OFFSET, universe);
    state = hash_values(state, unique_a);
    state = hash_values(state, unique_b);
    state = hash_u64(
        state,
        u64::try_from(expected_intersection).expect("intersection count fits u64"),
    );
    state = hash_u64(
        state,
        u64::try_from(probe_hits).expect("probe hit count fits u64"),
    );
    state = hash_u64(
        state,
        u64::try_from(probe_hits_a).expect("A probe hit count fits u64"),
    );
    hash_u64(
        state,
        u64::try_from(probe_hits_b).expect("B probe hit count fits u64"),
    )
}

trait BenchSet: Sized {
    fn construct(universe: u64, values: &[u32]) -> Result<Self, String>;
    fn len(&self) -> usize;
    fn contains(&self, value: u32) -> bool;
    fn intersection_len(&self, other: &Self) -> Result<usize, String>;
    fn payload_bytes(&self) -> u64;
}

macro_rules! impl_bench_set {
    ($type:ty) => {
        impl BenchSet for $type {
            fn construct(universe: u64, values: &[u32]) -> Result<Self, String> {
                Self::try_new(universe, values).map_err(|error| error.to_string())
            }

            fn len(&self) -> usize {
                self.len()
            }

            fn contains(&self, value: u32) -> bool {
                self.contains(value)
            }

            fn intersection_len(&self, other: &Self) -> Result<usize, String> {
                self.intersection_len(other)
                    .map_err(|error| error.to_string())
            }

            fn payload_bytes(&self) -> u64 {
                self.payload_bytes()
            }
        }
    };
}

impl_bench_set!(ReferenceSet);
impl_bench_set!(SortedSet);
impl_bench_set!(DenseBitSet);
impl_bench_set!(AdaptiveBitmap);

enum SetValue {
    Reference(ReferenceSet),
    Sorted(SortedSet),
    Dense(DenseBitSet),
    Adaptive(AdaptiveBitmap),
}

impl SetValue {
    fn len(&self) -> usize {
        match self {
            Self::Reference(value) => value.len(),
            Self::Sorted(value) => value.len(),
            Self::Dense(value) => value.len(),
            Self::Adaptive(value) => value.len(),
        }
    }

    fn contains(&self, needle: u32) -> bool {
        match self {
            Self::Reference(value) => value.contains(needle),
            Self::Sorted(value) => value.contains(needle),
            Self::Dense(value) => value.contains(needle),
            Self::Adaptive(value) => value.contains(needle),
        }
    }

    fn values(&self) -> Vec<u32> {
        match self {
            Self::Reference(value) => value.iter().collect(),
            Self::Sorted(value) => value.iter().collect(),
            Self::Dense(value) => value.iter().collect(),
            Self::Adaptive(value) => value.iter().collect(),
        }
    }

    fn intersection_len(&self, other: &Self) -> Result<usize, String> {
        match (self, other) {
            (Self::Reference(left), Self::Reference(right)) => left
                .intersection_len(right)
                .map_err(|error| error.to_string()),
            (Self::Sorted(left), Self::Sorted(right)) => left
                .intersection_len(right)
                .map_err(|error| error.to_string()),
            (Self::Dense(left), Self::Dense(right)) => left
                .intersection_len(right)
                .map_err(|error| error.to_string()),
            (Self::Adaptive(left), Self::Adaptive(right)) => left
                .intersection_len(right)
                .map_err(|error| error.to_string()),
            _ => Err("candidate pair contained different representation types".to_owned()),
        }
    }

    fn payload_bytes(&self) -> u64 {
        match self {
            Self::Reference(value) => value.payload_bytes(),
            Self::Sorted(value) => value.payload_bytes(),
            Self::Dense(value) => value.payload_bytes(),
            Self::Adaptive(value) => value.payload_bytes(),
        }
    }
}

fn build_pair(candidate: Candidate, fixture: &Fixture) -> Result<(SetValue, SetValue), String> {
    let build_one = |values: &[u32]| -> Result<SetValue, String> {
        match candidate {
            Candidate::Reference => ReferenceSet::try_new(fixture.universe, values)
                .map(SetValue::Reference)
                .map_err(|error| error.to_string()),
            Candidate::Sorted => SortedSet::try_new(fixture.universe, values)
                .map(SetValue::Sorted)
                .map_err(|error| error.to_string()),
            Candidate::Dense => DenseBitSet::try_new(fixture.universe, values)
                .map(SetValue::Dense)
                .map_err(|error| error.to_string()),
            Candidate::Adaptive => AdaptiveBitmap::try_new(fixture.universe, values)
                .map(SetValue::Adaptive)
                .map_err(|error| error.to_string()),
        }
    };
    Ok((build_one(&fixture.raw_a)?, build_one(&fixture.raw_b)?))
}

fn verify_candidate(
    candidate: Candidate,
    fixture: &Fixture,
    canary: &Canary,
) -> Result<(), String> {
    let adaptive_a = AdaptiveBitmap::try_new(fixture.universe, &fixture.raw_a)
        .map_err(|error| error.to_string())?;
    let adaptive_b = AdaptiveBitmap::try_new(fixture.universe, &fixture.raw_b)
        .map_err(|error| error.to_string())?;
    let actual_containers_a = SummaryCounts::from_adaptive(&adaptive_a).containers;
    let actual_containers_b = SummaryCounts::from_adaptive(&adaptive_b).containers;
    if actual_containers_a != canary.expected_containers_a
        || actual_containers_b != canary.expected_containers_b
    {
        return Err(
            "adaptive per-key container manifest disagreed with independent canary".to_owned(),
        );
    }
    let (left, right) = build_pair(candidate, fixture)?;
    if left.len() != canary.unique_a.len() || right.len() != canary.unique_b.len() {
        return Err(format!(
            "{} cardinality disagreed with canary",
            candidate.label()
        ));
    }
    let expected_payload_a = match candidate {
        Candidate::Reference | Candidate::Sorted => {
            u64::try_from(canary.unique_a.len()).expect("A cardinality fits u64") * 4
        }
        Candidate::Dense => fixture.universe.div_ceil(64) * 8,
        Candidate::Adaptive => canary
            .expected_containers_a
            .iter()
            .map(|container| container.payload_bytes)
            .sum(),
    };
    let expected_payload_b = match candidate {
        Candidate::Reference | Candidate::Sorted => {
            u64::try_from(canary.unique_b.len()).expect("B cardinality fits u64") * 4
        }
        Candidate::Dense => fixture.universe.div_ceil(64) * 8,
        Candidate::Adaptive => canary
            .expected_containers_b
            .iter()
            .map(|container| container.payload_bytes)
            .sum(),
    };
    if left.payload_bytes() != expected_payload_a || right.payload_bytes() != expected_payload_b {
        return Err(format!(
            "{} payloads were ({}, {}), expected ({expected_payload_a}, {expected_payload_b})",
            candidate.label(),
            left.payload_bytes(),
            right.payload_bytes()
        ));
    }
    if left.values() != canary.unique_a || right.values() != canary.unique_b {
        return Err(format!(
            "{} iteration disagreed with canary",
            candidate.label()
        ));
    }
    let observed_hits = fixture
        .probes
        .iter()
        .filter(|probe| match probe.operand {
            Operand::A => {
                left.contains(probe.value) == canary.unique_a.binary_search(&probe.value).is_ok()
            }
            Operand::B => {
                right.contains(probe.value) == canary.unique_b.binary_search(&probe.value).is_ok()
            }
        })
        .count();
    if observed_hits != fixture.probes.len() {
        return Err(format!(
            "{} had {} probe answers disagree with canary",
            candidate.label(),
            fixture.probes.len() - observed_hits
        ));
    }
    let observed_intersection = left.intersection_len(&right)?;
    if observed_intersection != canary.expected_intersection {
        return Err(format!(
            "{} intersection was {observed_intersection}, expected {}",
            candidate.label(),
            canary.expected_intersection
        ));
    }
    Ok(())
}

struct TimedSample {
    total_ns: u128,
    build_ns: u128,
    contains_ns: u128,
    intersection_ns: u128,
    checksum: u64,
}

#[inline(never)]
fn topic003_clock_now() -> Instant {
    Instant::now()
}

#[inline(never)]
fn topic003_consume_result(value: u64) {
    black_box(value);
}

#[inline(never)]
fn topic003_timed_loop<T: BenchSet>(fixture: &Fixture) -> Result<TimedSample, String> {
    let mut pairs = Vec::with_capacity(fixture.build_repetitions);
    let build_started = topic003_clock_now();
    let mut build_checksum = 0_u64;
    for _ in 0..fixture.build_repetitions {
        let left = T::construct(fixture.universe, black_box(&fixture.raw_a))?;
        let right = T::construct(fixture.universe, black_box(&fixture.raw_b))?;
        build_checksum = build_checksum
            .wrapping_add(u64::try_from(left.len()).unwrap_or_default())
            .wrapping_add(u64::try_from(right.len()).unwrap_or_default())
            ^ left.payload_bytes().rotate_left(11)
            ^ right.payload_bytes().rotate_left(23);
        pairs.push((left, right));
    }
    let build_ns = build_started.elapsed().as_nanos();
    let (left, right) = pairs
        .pop()
        .ok_or_else(|| "build repetitions unexpectedly empty".to_owned())?;
    drop(pairs);

    let contains_started = topic003_clock_now();
    let mut membership_hits = 0_usize;
    for &probe in &fixture.probes {
        let hit = match probe.operand {
            Operand::A => black_box(&left).contains(black_box(probe.value)),
            Operand::B => black_box(&right).contains(black_box(probe.value)),
        };
        membership_hits += usize::from(hit);
    }
    let contains_ns = contains_started.elapsed().as_nanos();

    let intersection_started = topic003_clock_now();
    let mut intersection_sum = 0_usize;
    for _ in 0..fixture.intersection_calls {
        intersection_sum =
            intersection_sum.wrapping_add(black_box(&left).intersection_len(black_box(&right))?);
    }
    let intersection_ns = intersection_started.elapsed().as_nanos();
    let checksum = u64::try_from(membership_hits)
        .unwrap_or_default()
        .wrapping_mul(0x9e37_79b9)
        ^ u64::try_from(intersection_sum)
            .unwrap_or_default()
            .rotate_left(17)
        ^ u64::try_from(left.len())
            .unwrap_or_default()
            .rotate_left(31)
        ^ left.payload_bytes().rotate_left(43)
        ^ build_checksum.rotate_left(53);
    topic003_consume_result(checksum);
    Ok(TimedSample {
        total_ns: build_ns + contains_ns + intersection_ns,
        build_ns,
        contains_ns,
        intersection_ns,
        checksum,
    })
}

#[inline(never)]
fn topic003_reference_workload(fixture: &Fixture) -> Result<TimedSample, String> {
    topic003_timed_loop::<ReferenceSet>(fixture)
}

#[inline(never)]
fn topic003_sorted_workload(fixture: &Fixture) -> Result<TimedSample, String> {
    topic003_timed_loop::<SortedSet>(fixture)
}

#[inline(never)]
fn topic003_dense_workload(fixture: &Fixture) -> Result<TimedSample, String> {
    topic003_timed_loop::<DenseBitSet>(fixture)
}

#[inline(never)]
fn topic003_adaptive_workload(fixture: &Fixture) -> Result<TimedSample, String> {
    topic003_timed_loop::<AdaptiveBitmap>(fixture)
}

fn run_candidate(candidate: Candidate, fixture: &Fixture) -> Result<TimedSample, String> {
    match candidate {
        Candidate::Reference => topic003_reference_workload(fixture),
        Candidate::Sorted => topic003_sorted_workload(fixture),
        Candidate::Dense => topic003_dense_workload(fixture),
        Candidate::Adaptive => topic003_adaptive_workload(fixture),
    }
}

#[derive(Clone)]
struct SummaryCounts {
    array: usize,
    bitmap: usize,
    run: usize,
    containers: Vec<ContainerRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ContainerRecord {
    key: u16,
    kind: ContainerKind,
    cardinality: usize,
    run_count: usize,
    payload_bytes: u64,
}

impl SummaryCounts {
    fn from_adaptive(bitmap: &AdaptiveBitmap) -> Self {
        let mut counts = Self {
            array: 0,
            bitmap: 0,
            run: 0,
            containers: Vec::new(),
        };
        for summary in bitmap.container_summaries() {
            match summary.kind {
                ContainerKind::Array => counts.array += 1,
                ContainerKind::Bitmap => counts.bitmap += 1,
                ContainerKind::Run => counts.run += 1,
            }
            counts.containers.push(ContainerRecord {
                key: summary.key,
                kind: summary.kind,
                cardinality: summary.cardinality,
                run_count: summary.run_count,
                payload_bytes: summary.payload_bytes,
            });
        }
        counts
    }

    const fn total(&self) -> usize {
        self.array + self.bitmap + self.run
    }
}

#[allow(clippy::too_many_arguments)]
fn print_record(
    config: &Config,
    fixture: &Fixture,
    canary: &Canary,
    payload_bytes_a: u64,
    payload_bytes_b: u64,
    summary_a: &SummaryCounts,
    summary_b: &SummaryCounts,
    total_sample_ns: &[u128],
    build_sample_ns: &[u128],
    contains_sample_ns: &[u128],
    intersection_sample_ns: &[u128],
    aggregate_checksum: u64,
) {
    let scaled = config.profile_repetitions;
    let raw_values_read =
        scaled * fixture.build_repetitions * (fixture.raw_a.len() + fixture.raw_b.len());
    let unique_values =
        scaled * fixture.build_repetitions * (canary.unique_a.len() + canary.unique_b.len());
    let duplicate_values = raw_values_read - unique_values;
    print!(
        "{{\"schema_version\":{SCHEMA_VERSION},\"candidate\":\"{}\",\"workload\":\"{}\",\"generator_version\":\"{GENERATOR_VERSION}\",\"seed\":{},\"warmups\":{},\"samples\":{},\"profile_repetitions\":{},\"universe\":{},\"dense_logical_words\":{},\"raw_a\":{},\"raw_b\":{},\"unique_a\":{},\"unique_b\":{},\"duplicate_a\":{},\"duplicate_b\":{},\"overlap\":{},\"probe_count\":{},\"probe_count_a\":{},\"probe_count_b\":{},\"probe_hits\":{},\"probe_hits_a\":{},\"probe_hits_b\":{},\"probe_misses\":{},\"intersection_calls\":{},\"build_repetitions\":{},\"fixture_hash\":\"{:016x}\",\"expected_hash\":\"{:016x}\",\"payload_bytes_a\":{},\"payload_bytes_b\":{},",
        config.candidate.label(),
        config.workload.label(),
        config.seed,
        config.warmups,
        config.samples,
        config.profile_repetitions,
        fixture.universe,
        fixture.universe.div_ceil(64),
        fixture.raw_a.len(),
        fixture.raw_b.len(),
        canary.unique_a.len(),
        canary.unique_b.len(),
        fixture.raw_a.len() - canary.unique_a.len(),
        fixture.raw_b.len() - canary.unique_b.len(),
        canary.expected_intersection,
        fixture.probes.len(),
        canary.probe_count_a,
        canary.probe_count_b,
        canary.probe_hits,
        canary.probe_hits_a,
        canary.probe_hits_b,
        fixture.probes.len() - canary.probe_hits,
        fixture.intersection_calls,
        fixture.build_repetitions,
        canary.fixture_hash,
        canary.expected_hash,
        payload_bytes_a,
        payload_bytes_b,
    );
    print!("\"total_sample_ns\":");
    print_integer_array(total_sample_ns);
    print!(",\"build_sample_ns\":");
    print_integer_array(build_sample_ns);
    print!(",\"contains_sample_ns\":");
    print_integer_array(contains_sample_ns);
    print!(",\"intersection_sample_ns\":");
    print_integer_array(intersection_sample_ns);
    print!(",\"aggregate_checksum\":{aggregate_checksum},\"adaptive_containers_a\":");
    print_container_records(&summary_a.containers);
    print!(",\"adaptive_containers_b\":");
    print_container_records(&summary_b.containers);
    println!(
        ",\"work\":{{\"raw_values_read\":{raw_values_read},\"unique_values\":{unique_values},\"duplicate_values\":{duplicate_values},\"membership_probes\":{},\"membership_hits\":{},\"intersection_calls\":{},\"build_calls\":{},\"expected_intersection\":{},\"occupied_chunks_a\":{},\"occupied_chunks_b\":{},\"adaptive_array_containers_a\":{},\"adaptive_bitmap_containers_a\":{},\"adaptive_run_containers_a\":{},\"adaptive_array_containers_b\":{},\"adaptive_bitmap_containers_b\":{},\"adaptive_run_containers_b\":{}}}}}",
        scaled * fixture.probes.len(),
        scaled * canary.probe_hits,
        scaled * fixture.intersection_calls,
        scaled * fixture.build_repetitions * 2,
        canary.expected_intersection,
        canary.occupied_chunks_a,
        canary.occupied_chunks_b,
        summary_a.array,
        summary_a.bitmap,
        summary_a.run,
        summary_b.array,
        summary_b.bitmap,
        summary_b.run,
    );
    debug_assert_eq!(summary_a.total(), canary.occupied_chunks_a);
    debug_assert_eq!(summary_b.total(), canary.occupied_chunks_b);
}

fn print_container_records(records: &[ContainerRecord]) {
    print!("[");
    for (index, record) in records.iter().enumerate() {
        if index != 0 {
            print!(",");
        }
        let kind = match record.kind {
            ContainerKind::Array => "array",
            ContainerKind::Bitmap => "bitmap",
            ContainerKind::Run => "run",
        };
        print!(
            "{{\"key\":{},\"kind\":\"{kind}\",\"cardinality\":{},\"run_count\":{},\"payload_bytes\":{}}}",
            record.key, record.cardinality, record.run_count, record.payload_bytes
        );
    }
    print!("]");
}

fn print_integer_array(values: &[u128]) {
    print!("[");
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            print!(",");
        }
        print!("{value}");
    }
    print!("]");
}
