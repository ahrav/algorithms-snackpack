//! Fresh-process native benchmark harness for Topic 004 interval sets.

#![allow(clippy::too_many_lines)]

use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::process::ExitCode;
use std::time::Instant;

use topic_004_interval_sets_and_compressed_ranges::{
    BTreeIntervalSet, BoundaryEventSet, DOMAIN_END, FlatIntervalSet, Interval, IntervalSet,
    PackedIntervalSet, PointOracle,
};

const SCHEMA_VERSION: u32 = 1;
const GENERATOR_VERSION: &str = "topic004-fixture-v1";
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn main() -> ExitCode {
    if !std::env::args()
        .any(|argument| argument == "--candidate" || argument.starts_with("--candidate="))
    {
        return ExitCode::SUCCESS;
    }
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("interval_sets benchmark: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let config = Config::parse()?;
    let fixture = Fixture::generate(config.workload, config.seed)?;
    let verification = verify_candidate(config.candidate, config.phase, &fixture)?;

    for _ in 0..config.warmup {
        black_box(run_inner(
            config.candidate,
            config.phase,
            &fixture,
            config.inner,
        )?);
    }

    let mut sample_ns = Vec::with_capacity(config.samples);
    let mut timed_checksum = 0_u64;
    for sample_index in 0..config.samples {
        let started = topic004_clock_now();
        let checksum = run_inner(config.candidate, config.phase, &fixture, config.inner)?;
        let elapsed = started.elapsed().as_nanos();
        if elapsed == 0 {
            return Err(format!(
                "sample {sample_index} had zero elapsed nanoseconds"
            ));
        }
        sample_ns.push(elapsed);
        timed_checksum ^= checksum
            .rotate_left(u32::try_from(sample_index % 64).expect("sample rotation is below 64"));
    }
    black_box(timed_checksum);

    let work = Work::count(config.candidate, config.phase, &fixture, config.inner)?;
    print_record(&config, &fixture, &verification, &sample_ns, &work);
    Ok(())
}

#[derive(Clone, Copy, Debug)]
enum Candidate {
    Oracle,
    Flat,
    Packed,
    Events,
    Btree,
}

impl Candidate {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "oracle" => Ok(Self::Oracle),
            "flat" => Ok(Self::Flat),
            "packed" => Ok(Self::Packed),
            "events" => Ok(Self::Events),
            "btree" => Ok(Self::Btree),
            _ => Err(format!("unsupported candidate {value:?}")),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Oracle => "oracle",
            Self::Flat => "flat",
            Self::Packed => "packed",
            Self::Events => "events",
            Self::Btree => "btree",
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Workload {
    TinySparseSortedUnique,
    CacheClusteredShuffledDuplicates,
    LargeSparseReverseUnique,
    LargeAdjacentShuffledDuplicates,
}

impl Workload {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "tiny_sparse_sorted_unique" => Ok(Self::TinySparseSortedUnique),
            "cache_clustered_shuffled_duplicates" => Ok(Self::CacheClusteredShuffledDuplicates),
            "large_sparse_reverse_unique" => Ok(Self::LargeSparseReverseUnique),
            "large_adjacent_shuffled_duplicates" => Ok(Self::LargeAdjacentShuffledDuplicates),
            _ => Err(format!("unsupported workload {value:?}")),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::TinySparseSortedUnique => "tiny_sparse_sorted_unique",
            Self::CacheClusteredShuffledDuplicates => "cache_clustered_shuffled_duplicates",
            Self::LargeSparseReverseUnique => "large_sparse_reverse_unique",
            Self::LargeAdjacentShuffledDuplicates => "large_adjacent_shuffled_duplicates",
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Phase {
    Build,
    BuildMembership,
}

impl Phase {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "build" => Ok(Self::Build),
            "build_membership" => Ok(Self::BuildMembership),
            _ => Err(format!("unsupported phase {value:?}")),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::BuildMembership => "build_membership",
        }
    }

    const fn includes_membership(self) -> bool {
        matches!(self, Self::BuildMembership)
    }
}

struct Config {
    candidate: Candidate,
    workload: Workload,
    phase: Phase,
    seed: u64,
    warmup: usize,
    samples: usize,
    inner: usize,
}

impl Config {
    fn parse() -> Result<Self, String> {
        let mut flags = parse_flags()?;
        let candidate = Candidate::parse(&take(&mut flags, "candidate")?)?;
        let workload = Workload::parse(&take(&mut flags, "workload")?)?;
        let phase = Phase::parse(&take(&mut flags, "phase")?)?;
        let seed = parse_number(&take(&mut flags, "seed")?, "seed")?;
        let warmup = parse_number(&take(&mut flags, "warmup")?, "warmup")?;
        let samples = parse_number(&take(&mut flags, "samples")?, "samples")?;
        let inner = parse_number(&take(&mut flags, "inner")?, "inner")?;
        if samples == 0 || inner == 0 {
            return Err("samples and inner must both be positive".to_owned());
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
            phase,
            seed,
            warmup,
            samples,
            inner,
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
        let body = flag
            .strip_prefix("--")
            .ok_or_else(|| format!("expected --flag, found {flag:?}"))?;
        let (name, value) = if let Some((name, value)) = body.split_once('=') {
            (name.to_owned(), value.to_owned())
        } else {
            let value = arguments
                .next()
                .ok_or_else(|| format!("missing value for --{body}"))?;
            (body.to_owned(), value)
        };
        if name.is_empty() {
            return Err(format!("expected --flag, found {flag:?}"));
        }
        if flags.insert(name.clone(), value).is_some() {
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
    input: Vec<Interval>,
    queries: Vec<u32>,
    unique_input_intervals: usize,
    duplicate_intervals: usize,
    output_runs: usize,
    point_count: u64,
    fingerprint: u64,
}

impl Fixture {
    fn generate(workload: Workload, seed: u64) -> Result<Self, String> {
        let mut random = Lcg::new(seed);
        let (input, expected_input, expected_unique, expected_duplicates, expected_runs) =
            match workload {
                Workload::TinySparseSortedUnique => {
                    let input = vec![
                        Interval::new(1, 2),
                        Interval::new(4, 6),
                        Interval::new(16, 17),
                        Interval::new(64, 68),
                        Interval::new(1_024, 1_025),
                        Interval::new(65_535, 65_536),
                        Interval::new(1_000_000, 1_000_003),
                        Interval::new(DOMAIN_END - 1, DOMAIN_END),
                    ];
                    (input, 8, 8, 0, 8)
                }
                Workload::CacheClusteredShuffledDuplicates => {
                    let mut input = Vec::with_capacity(320);
                    for cluster in 0..8_u64 {
                        let base = cluster * 65_536 + 1_024;
                        for run in 0..32_u64 {
                            let item = Interval::new(base + run * 16, base + run * 16 + 32);
                            input.push(item);
                            if run % 4 == 0 {
                                input.push(item);
                            }
                        }
                    }
                    random.shuffle(&mut input);
                    (input, 320, 256, 64, 8)
                }
                Workload::LargeSparseReverseUnique => {
                    let input = (0..4_096_u64)
                        .rev()
                        .map(|index| Interval::new(index * 1_024, index * 1_024 + 1))
                        .collect();
                    (input, 4_096, 4_096, 0, 4_096)
                }
                Workload::LargeAdjacentShuffledDuplicates => {
                    let mut input = Vec::with_capacity(4_608);
                    let base = 16_000_000_u64;
                    for index in 0..4_096_u64 {
                        let item = Interval::new(base + index * 32, base + (index + 1) * 32);
                        input.push(item);
                        if index % 8 == 0 {
                            input.push(item);
                        }
                    }
                    random.shuffle(&mut input);
                    (input, 4_608, 4_096, 512, 1)
                }
            };
        if input.len() != expected_input {
            return Err("generator input-count canary failed".to_owned());
        }

        let unique_input_intervals = input.iter().copied().collect::<BTreeSet<_>>().len();
        let duplicate_intervals = input.len() - unique_input_intervals;
        if unique_input_intervals != expected_unique || duplicate_intervals != expected_duplicates {
            return Err("generator duplicate-count canary failed".to_owned());
        }

        let oracle = PointOracle::try_from_intervals(&input)
            .map_err(|error| format!("fixture oracle construction failed: {error}"))?;
        let output_runs = oracle.canonical_intervals().len();
        if output_runs != expected_runs {
            return Err(format!(
                "generator produced {output_runs} runs, expected {expected_runs}"
            ));
        }
        let query_count = match workload {
            Workload::TinySparseSortedUnique => 64,
            Workload::CacheClusteredShuffledDuplicates => 1_024,
            Workload::LargeSparseReverseUnique | Workload::LargeAdjacentShuffledDuplicates => 4_096,
        };
        let queries = generate_queries(&oracle, query_count, &mut random)?;
        let fingerprint = fixture_fingerprint(workload, &input, &queries);
        Ok(Self {
            input,
            queries,
            unique_input_intervals,
            duplicate_intervals,
            output_runs,
            point_count: oracle.cardinality(),
            fingerprint,
        })
    }
}

fn generate_queries(
    oracle: &PointOracle,
    count: usize,
    random: &mut Lcg,
) -> Result<Vec<u32>, String> {
    let points = oracle.points().collect::<Vec<_>>();
    if points.is_empty() {
        return Err("benchmark fixture unexpectedly has no points".to_owned());
    }
    let mut queries = Vec::with_capacity(count);
    for index in 0..count {
        if index % 2 == 0 {
            let point_index = random.bounded_usize(points.len());
            queries.push(points[point_index]);
        } else {
            loop {
                let candidate = random.next_u32();
                if !oracle.contains(candidate) {
                    queries.push(candidate);
                    break;
                }
            }
        }
    }
    Ok(queries)
}

enum SetValue {
    Oracle(PointOracle),
    Flat(FlatIntervalSet),
    Packed(PackedIntervalSet),
    Events(BoundaryEventSet),
    Btree(BTreeIntervalSet),
}

impl SetValue {
    fn build(candidate: Candidate, input: &[Interval]) -> Result<Self, String> {
        match candidate {
            Candidate::Oracle => PointOracle::try_from_intervals(input)
                .map(Self::Oracle)
                .map_err(|error| error.to_string()),
            Candidate::Flat => FlatIntervalSet::try_from_intervals(input)
                .map(Self::Flat)
                .map_err(|error| error.to_string()),
            Candidate::Packed => PackedIntervalSet::try_from_intervals(input)
                .map(Self::Packed)
                .map_err(|error| error.to_string()),
            Candidate::Events => BoundaryEventSet::try_from_intervals(input)
                .map(Self::Events)
                .map_err(|error| error.to_string()),
            Candidate::Btree => BTreeIntervalSet::try_from_intervals(input)
                .map(Self::Btree)
                .map_err(|error| error.to_string()),
        }
    }

    fn as_view(&self) -> &dyn IntervalSet {
        match self {
            Self::Oracle(value) => value,
            Self::Flat(value) => value,
            Self::Packed(value) => value,
            Self::Events(value) => value,
            Self::Btree(value) => value,
        }
    }
}

struct Verification {
    output_hash: u64,
    packed_max_decode: u64,
}

fn verify_candidate(
    candidate: Candidate,
    phase: Phase,
    fixture: &Fixture,
) -> Result<Verification, String> {
    let oracle = PointOracle::try_from_intervals(&fixture.input)
        .map_err(|error| format!("self-check oracle failed: {error}"))?;
    let observed = SetValue::build(candidate, &fixture.input)?;
    let observed = observed.as_view();
    if observed.canonical_intervals() != oracle.canonical_intervals()
        || observed.cardinality() != oracle.cardinality()
    {
        return Err(format!(
            "{} canonical projection disagreed with the point oracle",
            candidate.label()
        ));
    }
    for &query in &fixture.queries {
        if observed.contains(query) != oracle.contains(query) {
            return Err(format!(
                "{} membership disagreed with the point oracle at {query}",
                candidate.label()
            ));
        }
    }

    let packed_max = PackedIntervalSet::try_from_intervals(&[Interval::new(0, DOMAIN_END)])
        .map_err(|error| format!("packed max canary failed: {error}"))?;
    let packed_max_decode = packed_max
        .packed_runs()
        .first()
        .ok_or_else(|| "packed max canary emitted no run".to_owned())?
        .decode()
        .end();
    if packed_max_decode != DOMAIN_END {
        return Err(format!(
            "packed max endpoint decoded as {packed_max_decode}, expected {DOMAIN_END}"
        ));
    }

    Ok(Verification {
        output_hash: semantic_fingerprint(observed, phase, &fixture.queries),
        packed_max_decode,
    })
}

#[inline(never)]
fn topic004_clock_now() -> Instant {
    Instant::now()
}

#[inline(never)]
fn topic004_consume_result(value: u64) {
    black_box(value);
}

#[inline(never)]
fn topic004_timed_loop(
    candidate: Candidate,
    phase: Phase,
    fixture: &Fixture,
    inner: usize,
) -> Result<u64, String> {
    let mut checksum = FNV_OFFSET;
    for iteration in 0..inner {
        let set = SetValue::build(candidate, black_box(&fixture.input))?;
        let view = black_box(set.as_view());
        checksum = hash_u64(checksum, view.cardinality());
        checksum = hash_u64(
            checksum,
            u64::try_from(iteration).expect("iteration index fits u64"),
        );
        if phase.includes_membership() {
            for &query in black_box(&fixture.queries) {
                checksum = hash_u64(checksum, u64::from(query));
                checksum = hash_u64(checksum, u64::from(view.contains(black_box(query))));
            }
        }
        black_box(set);
    }
    topic004_consume_result(checksum);
    Ok(checksum)
}

#[inline(never)]
fn topic004_oracle_workload(phase: Phase, fixture: &Fixture, inner: usize) -> Result<u64, String> {
    topic004_timed_loop(Candidate::Oracle, phase, fixture, inner)
}

#[inline(never)]
fn topic004_flat_workload(phase: Phase, fixture: &Fixture, inner: usize) -> Result<u64, String> {
    topic004_timed_loop(Candidate::Flat, phase, fixture, inner)
}

#[inline(never)]
fn topic004_packed_workload(phase: Phase, fixture: &Fixture, inner: usize) -> Result<u64, String> {
    topic004_timed_loop(Candidate::Packed, phase, fixture, inner)
}

#[inline(never)]
fn topic004_events_workload(phase: Phase, fixture: &Fixture, inner: usize) -> Result<u64, String> {
    topic004_timed_loop(Candidate::Events, phase, fixture, inner)
}

#[inline(never)]
fn topic004_btree_workload(phase: Phase, fixture: &Fixture, inner: usize) -> Result<u64, String> {
    topic004_timed_loop(Candidate::Btree, phase, fixture, inner)
}

fn run_inner(
    candidate: Candidate,
    phase: Phase,
    fixture: &Fixture,
    inner: usize,
) -> Result<u64, String> {
    match candidate {
        Candidate::Oracle => topic004_oracle_workload(phase, fixture, inner),
        Candidate::Flat => topic004_flat_workload(phase, fixture, inner),
        Candidate::Packed => topic004_packed_workload(phase, fixture, inner),
        Candidate::Events => topic004_events_workload(phase, fixture, inner),
        Candidate::Btree => topic004_btree_workload(phase, fixture, inner),
    }
}

struct Work {
    input_intervals: u64,
    unique_input_intervals: u64,
    duplicate_intervals: u64,
    sort_comparisons_count_pass: u64,
    merge_comparisons: u64,
    output_runs: u64,
    membership_queries: u64,
    canonical_binary_search_comparisons: u64,
    result_scalar_slots: u64,
}

impl Work {
    fn count(
        candidate: Candidate,
        phase: Phase,
        fixture: &Fixture,
        inner: usize,
    ) -> Result<Self, String> {
        let inner = u64::try_from(inner).map_err(|error| error.to_string())?;
        let sort_comparisons = sort_comparison_count(candidate, &fixture.input);
        let merge_comparisons = merge_comparison_count(candidate, &fixture.input);
        let canonical_binary_search_comparisons = if phase.includes_membership() {
            membership_comparison_count(candidate, fixture)?
        } else {
            0
        };
        let membership_queries = if phase.includes_membership() {
            fixture.queries.len()
        } else {
            0
        };
        let result_scalar_slots = match candidate {
            Candidate::Oracle => fixture.point_count,
            Candidate::Flat | Candidate::Packed | Candidate::Events | Candidate::Btree => {
                u64::try_from(fixture.output_runs).expect("run count fits u64") * 2
            }
        };

        Ok(Self {
            input_intervals: scale(fixture.input.len(), inner),
            unique_input_intervals: scale(fixture.unique_input_intervals, inner),
            duplicate_intervals: scale(fixture.duplicate_intervals, inner),
            sort_comparisons_count_pass: sort_comparisons.saturating_mul(inner),
            merge_comparisons: merge_comparisons.saturating_mul(inner),
            output_runs: scale(fixture.output_runs, inner),
            membership_queries: scale(membership_queries, inner),
            canonical_binary_search_comparisons: canonical_binary_search_comparisons
                .saturating_mul(inner),
            result_scalar_slots: result_scalar_slots.saturating_mul(inner),
        })
    }
}

fn scale(value: usize, inner: u64) -> u64 {
    u64::try_from(value)
        .expect("fixture count fits u64")
        .saturating_mul(inner)
}

fn sort_comparison_count(candidate: Candidate, input: &[Interval]) -> u64 {
    let mut comparisons = 0_u64;
    match candidate {
        Candidate::Oracle | Candidate::Btree => {}
        Candidate::Flat | Candidate::Packed => {
            let mut intervals = input
                .iter()
                .copied()
                .filter(|item| !item.is_empty())
                .collect::<Vec<_>>();
            intervals.sort_unstable_by(|left, right| {
                comparisons += 1;
                (left.start(), left.end()).cmp(&(right.start(), right.end()))
            });
        }
        Candidate::Events => {
            let mut events = Vec::with_capacity(input.len() * 2);
            for item in input.iter().copied().filter(|item| !item.is_empty()) {
                events.push(item.start());
                events.push(item.end());
            }
            events.sort_unstable_by(|left, right| {
                comparisons += 1;
                left.cmp(right)
            });
        }
    }
    comparisons
}

fn merge_comparison_count(candidate: Candidate, input: &[Interval]) -> u64 {
    match candidate {
        Candidate::Oracle => 0,
        Candidate::Flat | Candidate::Packed => u64::try_from(
            input
                .iter()
                .filter(|item| !item.is_empty())
                .count()
                .saturating_sub(1),
        )
        .expect("merge count fits u64"),
        Candidate::Events => event_transition_comparisons(input),
        Candidate::Btree => btree_relation_comparisons(input),
    }
}

fn event_transition_comparisons(input: &[Interval]) -> u64 {
    let mut events = Vec::with_capacity(input.len() * 2);
    for item in input.iter().copied().filter(|item| !item.is_empty()) {
        events.push((item.start(), 1_i64));
        events.push((item.end(), -1_i64));
    }
    events.sort_unstable_by_key(|event| event.0);
    let mut index = 0;
    let mut depth = 0_i64;
    let mut comparisons = 0_u64;
    while index < events.len() {
        let coordinate = events[index].0;
        let mut delta = 0_i64;
        while index < events.len() && events[index].0 == coordinate {
            delta += events[index].1;
            index += 1;
        }
        let next_depth = depth + delta;
        comparisons += 1;
        if depth == 0 {
            comparisons += 1;
            black_box(next_depth > 0);
        } else {
            comparisons += 2;
            black_box(depth > 0 && next_depth == 0);
        }
        depth = next_depth;
    }
    comparisons
}

fn btree_relation_comparisons(input: &[Interval]) -> u64 {
    let mut tree = BTreeMap::new();
    let mut comparisons = 0_u64;
    for item in input.iter().copied().filter(|item| !item.is_empty()) {
        let mut start = item.start();
        let mut end = item.end();
        if let Some((&previous_start, &previous_end)) = tree.range(..=start).next_back() {
            comparisons += 1;
            if previous_end >= start {
                start = previous_start;
                end = end.max(previous_end);
                tree.remove(&previous_start);
            }
        }
        loop {
            let next = tree
                .range(start..)
                .next()
                .map(|(&key, &value)| (key, value));
            let Some((next_start, next_end)) = next else {
                break;
            };
            comparisons += 1;
            if next_start > end {
                break;
            }
            end = end.max(next_end);
            tree.remove(&next_start);
        }
        tree.insert(start, end);
    }
    comparisons
}

fn membership_comparison_count(candidate: Candidate, fixture: &Fixture) -> Result<u64, String> {
    match candidate {
        Candidate::Oracle => {
            let oracle = PointOracle::try_from_intervals(&fixture.input)
                .map_err(|error| error.to_string())?;
            let points = oracle.points().collect::<Vec<_>>();
            Ok(fixture
                .queries
                .iter()
                .map(|&query| counted_point_binary_search(&points, query))
                .sum())
        }
        Candidate::Flat | Candidate::Packed | Candidate::Events | Candidate::Btree => {
            let set = SetValue::build(candidate, &fixture.input)?;
            let intervals = set.as_view().canonical_intervals();
            Ok(fixture
                .queries
                .iter()
                .map(|&query| counted_interval_search(&intervals, query))
                .sum())
        }
    }
}

fn counted_point_binary_search(points: &[u32], query: u32) -> u64 {
    let mut low = 0;
    let mut high = points.len();
    let mut comparisons = 0_u64;
    while low < high {
        let middle = low + (high - low) / 2;
        comparisons += 1;
        match points[middle].cmp(&query) {
            std::cmp::Ordering::Less => low = middle + 1,
            std::cmp::Ordering::Equal => return comparisons,
            std::cmp::Ordering::Greater => high = middle,
        }
    }
    comparisons
}

fn counted_interval_search(intervals: &[Interval], query: u32) -> u64 {
    let query = u64::from(query);
    let mut low = 0;
    let mut high = intervals.len();
    let mut comparisons = 0_u64;
    while low < high {
        let middle = low + (high - low) / 2;
        comparisons += 1;
        if intervals[middle].start() <= query {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    if low != 0 {
        comparisons += 1;
        black_box(query < intervals[low - 1].end());
    }
    comparisons
}

fn semantic_fingerprint(set: &dyn IntervalSet, phase: Phase, queries: &[u32]) -> u64 {
    let mut state = hash_u64(FNV_OFFSET, set.cardinality());
    let intervals = set.canonical_intervals();
    state = hash_u64(
        state,
        u64::try_from(intervals.len()).expect("run count fits u64"),
    );
    for item in intervals {
        state = hash_u64(state, item.start());
        state = hash_u64(state, item.end());
    }
    state = hash_u64(state, u64::from(phase.includes_membership()));
    if phase.includes_membership() {
        for &query in queries {
            state = hash_u64(state, u64::from(query));
            state = hash_u64(state, u64::from(set.contains(query)));
        }
    }
    state
}

fn fixture_fingerprint(workload: Workload, input: &[Interval], queries: &[u32]) -> u64 {
    let mut state = hash_bytes(FNV_OFFSET, GENERATOR_VERSION.as_bytes());
    state = hash_bytes(state, workload.label().as_bytes());
    state = hash_u64(
        state,
        u64::try_from(input.len()).expect("input count fits u64"),
    );
    for item in input {
        state = hash_u64(state, item.start());
        state = hash_u64(state, item.end());
    }
    state = hash_u64(
        state,
        u64::try_from(queries.len()).expect("query count fits u64"),
    );
    for &query in queries {
        state = hash_u64(state, u64::from(query));
    }
    state
}

fn hash_u64(state: u64, value: u64) -> u64 {
    hash_bytes(state, &value.to_le_bytes())
}

fn hash_bytes(mut state: u64, bytes: &[u8]) -> u64 {
    for &byte in bytes {
        state ^= u64::from(byte);
        state = state.wrapping_mul(FNV_PRIME);
    }
    state
}

fn print_record(
    config: &Config,
    fixture: &Fixture,
    verification: &Verification,
    sample_ns: &[u128],
    work: &Work,
) {
    print!(
        "{{\"schema_version\":{SCHEMA_VERSION},\"candidate\":\"{}\",\"workload\":\"{}\",\"phase\":\"{}\",\"seed\":{},\"warmup\":{},\"samples\":{},\"inner\":{},\"generator_version\":\"{GENERATOR_VERSION}\",\"fixture_hash\":\"{:016x}\",\"sample_ns\":",
        config.candidate.label(),
        config.workload.label(),
        config.phase.label(),
        config.seed,
        config.warmup,
        config.samples,
        config.inner,
        fixture.fingerprint,
    );
    print_integer_array(sample_ns);
    println!(
        ",\"output_hash\":\"{:016x}\",\"work\":{{\"input_intervals\":{},\"unique_input_intervals\":{},\"duplicate_intervals\":{},\"sort_comparisons_count_pass\":{},\"merge_comparisons\":{},\"output_runs\":{},\"membership_queries\":{},\"canonical_binary_search_comparisons\":{},\"result_scalar_slots\":{}}},\"canaries\":{{\"oracle_match\":true,\"domain_end\":{DOMAIN_END},\"packed_max_decode\":{},\"membership_match\":true}}}}",
        verification.output_hash,
        work.input_intervals,
        work.unique_input_intervals,
        work.duplicate_intervals,
        work.sort_comparisons_count_pass,
        work.merge_comparisons,
        work.output_runs,
        work.membership_queries,
        work.canonical_binary_search_comparisons,
        work.result_scalar_slots,
        verification.packed_max_decode,
    );
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

struct Lcg(u64);

impl Lcg {
    const fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }

    fn next_u32(&mut self) -> u32 {
        u32::try_from(self.next() & u64::from(u32::MAX)).expect("masked RNG value fits u32")
    }

    fn bounded_usize(&mut self, exclusive: usize) -> usize {
        let bound = u64::try_from(exclusive).expect("slice length fits u64");
        usize::try_from(self.next() % bound).expect("bounded RNG index fits usize")
    }

    fn shuffle<T>(&mut self, values: &mut [T]) {
        for index in (1..values.len()).rev() {
            let other = self.bounded_usize(index + 1);
            values.swap(index, other);
        }
    }
}
