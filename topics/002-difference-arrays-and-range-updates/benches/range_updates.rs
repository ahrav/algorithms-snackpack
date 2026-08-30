//! Fresh-process benchmark binary for batch range-update candidates.

use std::collections::BTreeMap;
use std::hint::black_box;
use std::process::ExitCode;
use std::time::Instant;

use topic_002_difference_arrays_and_range_updates::{
    RangeError, RangeUpdate, apply_dense_sidecar, apply_in_place_difference, apply_reference,
    apply_sorted_events,
};

type CandidateFn = fn(&[i64], &[RangeUpdate]) -> Result<Vec<i64>, RangeError>;

const GENERATOR_VERSION: &str = "topic002-fixture-v1";

fn main() -> ExitCode {
    // `cargo test --all-targets` executes harness-free bench binaries without
    // arguments. The measurement runner always supplies the full protocol.
    if std::env::args_os().nth(1).is_none() {
        return ExitCode::SUCCESS;
    }
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("range_updates benchmark: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let config = Config::parse()?;
    let mut rng = Lcg::new(config.seed);
    let input = generate_input(config.len, &mut rng);
    let mut updates = generate_updates(&config, &mut rng)?;
    apply_order(config.order, &mut updates, &mut rng);

    let expected = eager_canary(&input, &updates);
    let candidate_fn = config.candidate.function();
    let observed = candidate_fn(&input, &updates)
        .map_err(|error| format!("generated candidate input was invalid: {error:?}"))?;
    if observed != expected {
        return Err(format!(
            "candidate {} disagreed with the reference before timing",
            config.candidate.label()
        ));
    }
    let input_hash = hash_i64_slice(&input);
    let updates_hash = hash_updates(&updates);
    let output_hash = hash_i64_slice(&expected);
    drop(observed);
    drop(expected);
    let work = WorkCounts::for_call(config.candidate, &input, &updates);

    for _ in 0..config.warmup {
        let output = candidate_fn(black_box(&input), black_box(&updates))
            .map_err(|error| format!("warmup failed: {error:?}"))?;
        black_box(output);
    }

    let mut sample_ns = Vec::with_capacity(config.samples);
    for _ in 0..config.samples {
        let timed_input = black_box(input.as_slice());
        let timed_updates = black_box(updates.as_slice());
        let start = Instant::now();
        let result = candidate_fn(timed_input, timed_updates);
        let elapsed_ns = start.elapsed().as_nanos();
        let output = result.map_err(|error| format!("timed call failed: {error:?}"))?;
        sample_ns.push(elapsed_ns);
        black_box(output);
    }

    print_record(
        &config,
        &sample_ns,
        input_hash,
        updates_hash,
        output_hash,
        work,
    );
    Ok(())
}

#[derive(Clone, Copy)]
enum Candidate {
    Reference,
    Dense,
    InPlace,
    SortedEvents,
}

impl Candidate {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "reference" => Ok(Self::Reference),
            "dense" => Ok(Self::Dense),
            "in_place" => Ok(Self::InPlace),
            "sorted_events" => Ok(Self::SortedEvents),
            _ => Err(format!("unsupported candidate {value:?}")),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Reference => "reference",
            Self::Dense => "dense",
            Self::InPlace => "in_place",
            Self::SortedEvents => "sorted_events",
        }
    }

    const fn function(self) -> CandidateFn {
        match self {
            Self::Reference => apply_reference,
            Self::Dense => apply_dense_sidecar,
            Self::InPlace => apply_in_place_difference,
            Self::SortedEvents => apply_sorted_events,
        }
    }
}

#[derive(Clone, Copy)]
enum Pattern {
    Point,
    Full,
    Uniform,
    Clustered,
}

impl Pattern {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "point" => Ok(Self::Point),
            "full" => Ok(Self::Full),
            "uniform" => Ok(Self::Uniform),
            "clustered" => Ok(Self::Clustered),
            _ => Err(format!("unsupported pattern {value:?}")),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Point => "point",
            Self::Full => "full",
            Self::Uniform => "uniform",
            Self::Clustered => "clustered",
        }
    }
}

#[derive(Clone, Copy)]
enum Order {
    Sorted,
    Reverse,
    Shuffled,
}

impl Order {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "sorted" => Ok(Self::Sorted),
            "reverse" => Ok(Self::Reverse),
            "shuffled" => Ok(Self::Shuffled),
            _ => Err(format!("unsupported order {value:?}")),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Sorted => "sorted",
            Self::Reverse => "reverse",
            Self::Shuffled => "shuffled",
        }
    }
}

struct Config {
    candidate: Candidate,
    workload: String,
    pattern: Pattern,
    order: Order,
    len: usize,
    updates: usize,
    max_span: usize,
    seed: u64,
    warmup: usize,
    samples: usize,
    inner: usize,
}

impl Config {
    fn parse() -> Result<Self, String> {
        let mut values = parse_flag_pairs()?;
        let candidate = Candidate::parse(&take(&mut values, "candidate")?)?;
        let workload = take(&mut values, "workload")?;
        if !matches!(
            workload.as_str(),
            "tiny_point_sorted"
                | "small_full_repeated"
                | "cache_short_shuffled"
                | "cache_clustered"
                | "large_sparse_sorted"
                | "large_wide_shuffled"
        ) {
            return Err(format!("unsupported workload {workload:?}"));
        }
        let pattern = Pattern::parse(&take(&mut values, "pattern")?)?;
        let order = Order::parse(&take(&mut values, "order")?)?;
        let len = parse_number(&take(&mut values, "len")?, "len")?;
        let updates = parse_number(&take(&mut values, "updates")?, "updates")?;
        let max_span = parse_number(&take(&mut values, "max-span")?, "max-span")?;
        let seed = parse_number(&take(&mut values, "seed")?, "seed")?;
        let warmup = parse_number(&take(&mut values, "warmup")?, "warmup")?;
        let samples = parse_number(&take(&mut values, "samples")?, "samples")?;
        let inner = parse_number(&take(&mut values, "inner")?, "inner")?;
        if samples == 0 {
            return Err("samples must be greater than zero".to_owned());
        }
        if inner != 1 {
            return Err("inner must equal one; one public call is one timed sample".to_owned());
        }
        if !values.is_empty() {
            return Err(format!(
                "unknown flags: {}",
                values.keys().cloned().collect::<Vec<_>>().join(", ")
            ));
        }

        Ok(Self {
            candidate,
            workload,
            pattern,
            order,
            len,
            updates,
            max_span,
            seed,
            warmup,
            samples,
            inner,
        })
    }
}

fn parse_flag_pairs() -> Result<BTreeMap<String, String>, String> {
    let mut arguments = std::env::args().skip(1);
    let mut values = BTreeMap::new();
    while let Some(flag) = arguments.next() {
        if flag == "--bench" {
            continue;
        }
        let name = flag
            .strip_prefix("--")
            .ok_or_else(|| format!("expected --flag, found {flag:?}"))?;
        if name.is_empty() {
            return Err("empty flag name".to_owned());
        }
        let value = arguments
            .next()
            .ok_or_else(|| format!("missing value for --{name}"))?;
        if values.insert(name.to_owned(), value).is_some() {
            return Err(format!("duplicate flag --{name}"));
        }
    }
    Ok(values)
}

fn take(values: &mut BTreeMap<String, String>, name: &str) -> Result<String, String> {
    values
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

fn generate_input(len: usize, rng: &mut Lcg) -> Vec<i64> {
    (0..len).map(|_| rng.mixed_i64()).collect()
}

fn generate_updates(config: &Config, rng: &mut Lcg) -> Result<Vec<RangeUpdate>, String> {
    let mut updates = Vec::with_capacity(config.updates);
    for update_index in 0..config.updates {
        let (start, end) = match config.pattern {
            Pattern::Point => {
                if config.len == 0 {
                    (0, 0)
                } else {
                    let start = rng.below(config.len);
                    (start, start + 1)
                }
            }
            Pattern::Full => (0, config.len),
            Pattern::Uniform => uniform_range(config.len, config.max_span, rng),
            Pattern::Clustered => clustered_range(config.len, config.max_span, rng),
        };
        updates.push(RangeUpdate {
            start,
            end,
            delta: rng.nonzero_mixed_i64(update_index),
        });
    }

    if updates.len() != config.updates {
        return Err("generator produced the wrong update count".to_owned());
    }
    Ok(updates)
}

fn uniform_range(len: usize, max_span: usize, rng: &mut Lcg) -> (usize, usize) {
    if len == 0 || max_span == 0 {
        let endpoint = rng.below(len + 1);
        return (endpoint, endpoint);
    }
    let start = rng.below(len);
    let available = (len - start).min(max_span);
    let width = 1 + rng.below(available);
    (start, start + width)
}

fn clustered_range(len: usize, max_span: usize, rng: &mut Lcg) -> (usize, usize) {
    if len == 0 || max_span == 0 {
        return (0, 0);
    }

    let hot_width = len.min(1_024);
    let hot_start = (len - hot_width) / 2;
    let bucket_count = hot_width.min(32) + 1;
    let first_bucket = rng.below(bucket_count);
    let mut second_bucket = rng.below(bucket_count);
    if second_bucket == first_bucket {
        second_bucket = if first_bucket + 1 < bucket_count {
            first_bucket + 1
        } else {
            first_bucket - 1
        };
    }
    let first = hot_start + first_bucket * hot_width / (bucket_count - 1);
    let second = hot_start + second_bucket * hot_width / (bucket_count - 1);
    let start = first.min(second);
    let mut end = first.max(second);
    end = end.min(start.saturating_add(max_span));
    (start, end)
}

fn apply_order(order: Order, updates: &mut [RangeUpdate], rng: &mut Lcg) {
    match order {
        Order::Sorted => updates.sort_by_key(|update| (update.start, update.end)),
        Order::Reverse => {
            updates.sort_by_key(|update| (update.start, update.end));
            updates.reverse();
        }
        Order::Shuffled => {
            for index in (1..updates.len()).rev() {
                updates.swap(index, rng.below(index + 1));
            }
        }
    }
}

#[derive(Clone, Copy)]
struct WorkCounts {
    validation_checks: u64,
    base_reads: u64,
    base_writes: u64,
    range_element_updates: u64,
    boundary_updates: u64,
    difference_steps: u64,
    scan_steps: u64,
    event_records: u64,
    sort_comparisons_count_pass: u64,
    vec_constructions: u64,
    allocated_scalar_slots: u64,
}

impl WorkCounts {
    fn for_call(candidate: Candidate, input: &[i64], updates: &[RangeUpdate]) -> Self {
        let len = to_u64(input.len());
        let update_count = to_u64(updates.len());
        let active_updates = updates
            .iter()
            .filter(|update| update.start != update.end && update.delta != 0)
            .count();
        let active_count = to_u64(active_updates);

        match candidate {
            Candidate::Reference => Self {
                validation_checks: update_count,
                base_reads: len,
                base_writes: len,
                range_element_updates: updates.iter().fold(0_u64, |total, update| {
                    total.saturating_add(to_u64(update.end - update.start))
                }),
                boundary_updates: 0,
                difference_steps: 0,
                scan_steps: 0,
                event_records: 0,
                sort_comparisons_count_pass: 0,
                vec_constructions: 1,
                allocated_scalar_slots: len,
            },
            Candidate::Dense => Self {
                validation_checks: update_count,
                base_reads: len,
                base_writes: len,
                range_element_updates: 0,
                boundary_updates: active_count.saturating_mul(2),
                difference_steps: 0,
                scan_steps: len,
                event_records: 0,
                sort_comparisons_count_pass: 0,
                vec_constructions: 2,
                allocated_scalar_slots: len.saturating_mul(2).saturating_add(1),
            },
            Candidate::InPlace => {
                let interior = len.saturating_sub(1);
                let boundary_updates = to_u64(
                    updates
                        .iter()
                        .filter(|update| update.start != update.end && update.delta != 0)
                        .map(|update| usize::from(update.end < input.len()) + 1)
                        .sum(),
                );
                Self {
                    validation_checks: update_count,
                    base_reads: len,
                    base_writes: len,
                    range_element_updates: 0,
                    boundary_updates,
                    difference_steps: interior,
                    scan_steps: interior,
                    event_records: 0,
                    sort_comparisons_count_pass: 0,
                    vec_constructions: 1,
                    allocated_scalar_slots: len,
                }
            }
            Candidate::SortedEvents => {
                let event_records = to_u64(sparse_events(input.len(), updates).len());
                Self {
                    validation_checks: update_count,
                    base_reads: len,
                    base_writes: len,
                    range_element_updates: 0,
                    boundary_updates: event_records,
                    difference_steps: 0,
                    scan_steps: len,
                    event_records,
                    sort_comparisons_count_pass: count_sort_comparisons(input.len(), updates),
                    vec_constructions: 2,
                    allocated_scalar_slots: len.saturating_add(update_count.saturating_mul(4)),
                }
            }
        }
    }
}

fn sparse_events(len: usize, updates: &[RangeUpdate]) -> Vec<(usize, i64)> {
    let mut events = Vec::with_capacity(updates.len().saturating_mul(2));
    for update in updates {
        if update.start == update.end || update.delta == 0 {
            continue;
        }
        events.push((update.start, update.delta));
        if update.end < len {
            events.push((update.end, 0_i64.wrapping_sub(update.delta)));
        }
    }
    events
}

fn count_sort_comparisons(len: usize, updates: &[RangeUpdate]) -> u64 {
    let mut events = sparse_events(len, updates);
    let mut comparisons = 0_u64;
    events.sort_unstable_by(|left, right| {
        comparisons = comparisons.saturating_add(1);
        left.0.cmp(&right.0)
    });
    comparisons
}

fn to_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn hash_i64_slice(values: &[i64]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in to_u64(values.len()).to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    for value in values {
        for byte in value.to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    hash
}

fn eager_canary(input: &[i64], updates: &[RangeUpdate]) -> Vec<i64> {
    let mut output = input.to_vec();
    for update in updates {
        for value in &mut output[update.start..update.end] {
            *value = value.wrapping_add(update.delta);
        }
    }
    output
}

fn hash_updates(updates: &[RangeUpdate]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in to_u64(updates.len()).to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    for update in updates {
        for byte in to_u64(update.start).to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        for byte in to_u64(update.end).to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        for byte in update.delta.to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    hash
}

fn print_record(
    config: &Config,
    sample_ns: &[u128],
    input_hash: u64,
    updates_hash: u64,
    output_hash: u64,
    work: WorkCounts,
) {
    let samples = sample_ns
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",");
    println!(
        concat!(
            "{{\"schema_version\":1,",
            "\"candidate\":\"{}\",\"workload\":\"{}\",",
            "\"pattern\":\"{}\",\"order\":\"{}\",",
            "\"len\":{},\"updates\":{},\"max_span\":{},\"seed\":{},",
            "\"warmup\":{},\"samples\":{},\"inner\":{},",
            "\"generator_version\":\"{}\",",
            "\"input_hash\":\"{:016x}\",\"updates_hash\":\"{:016x}\",",
            "\"sample_ns\":[{}],\"output_hash\":\"{:016x}\",",
            "\"work\":{{\"validation_checks\":{},\"base_reads\":{},",
            "\"base_writes\":{},\"range_element_updates\":{},",
            "\"boundary_updates\":{},\"difference_steps\":{},",
            "\"scan_steps\":{},\"event_records\":{},",
            "\"sort_comparisons_count_pass\":{},\"vec_constructions\":{},",
            "\"allocated_scalar_slots\":{}}}}}"
        ),
        config.candidate.label(),
        config.workload,
        config.pattern.label(),
        config.order.label(),
        config.len,
        config.updates,
        config.max_span,
        config.seed,
        config.warmup,
        config.samples,
        config.inner,
        GENERATOR_VERSION,
        input_hash,
        updates_hash,
        samples,
        output_hash,
        work.validation_checks,
        work.base_reads,
        work.base_writes,
        work.range_element_updates,
        work.boundary_updates,
        work.difference_steps,
        work.scan_steps,
        work.event_records,
        work.sort_comparisons_count_pass,
        work.vec_constructions,
        work.allocated_scalar_slots,
    );
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

    fn below(&mut self, upper: usize) -> usize {
        assert!(upper > 0);
        let upper_u64 = u64::try_from(upper).expect("benchmark bounds fit u64");
        usize::try_from(self.next_u64() % upper_u64).expect("remainder fits usize")
    }

    fn mixed_i64(&mut self) -> i64 {
        match self.next_u64() & 7 {
            0 => i64::MIN,
            1 => i64::MAX,
            2 => -1,
            3 => 0,
            4 => 1,
            _ => i64::from_ne_bytes(self.next_u64().to_ne_bytes()),
        }
    }

    fn nonzero_mixed_i64(&mut self, update_index: usize) -> i64 {
        let value = match update_index % 8 {
            0 => 1,
            1 => -1,
            2 => 7,
            3 => -13,
            4 => i64::MAX,
            5 => i64::MIN,
            _ => i64::from_ne_bytes(self.next_u64().to_ne_bytes()),
        };
        if value == 0 { 1 } else { value }
    }
}
