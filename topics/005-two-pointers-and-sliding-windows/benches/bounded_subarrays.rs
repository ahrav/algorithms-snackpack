//! Fresh-process native benchmark harness for Topic 005 bounded-subarray counters.

#![allow(clippy::too_many_lines)]

use std::collections::BTreeMap;
use std::hint::black_box;
use std::process::ExitCode;
use std::time::Instant;

use topic_005_two_pointers_and_sliding_windows::{
    direct_sliding, oversized_reset_sliding, prefix_binary_search, quadratic_early_exit,
    recompute_reference,
};

const SCHEMA_VERSION: u32 = 1;
const GENERATOR_VERSION: &str = "topic005-fixture-v1";
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
            eprintln!("bounded_subarrays benchmark: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let config = Config::parse()?;
    let fixture = Fixture::generate(config.workload);
    if !config.candidate.supports(config.workload) {
        return Err(format!(
            "candidate {} is intentionally unsupported for workload {}",
            config.candidate.label(),
            config.workload.label()
        ));
    }
    let canaries = verify_candidate(config.candidate, &fixture)?;

    for _ in 0..config.warmup {
        black_box(topic005_timed_loop(
            config.candidate,
            &fixture,
            config.inner,
        ));
    }

    let mut sample_ns = Vec::with_capacity(config.samples);
    let mut timed_checksum = 0_u64;
    for sample_index in 0..config.samples {
        let started = topic005_clock_now();
        let checksum = topic005_timed_loop(config.candidate, &fixture, config.inner);
        let elapsed = started.elapsed().as_nanos();
        if elapsed == 0 {
            return Err(format!(
                "sample {sample_index} had zero elapsed nanoseconds"
            ));
        }
        sample_ns.push(elapsed);
        timed_checksum ^=
            checksum.rotate_left(u32::try_from(sample_index % 64).expect("rotation is below 64"));
    }
    black_box(timed_checksum);

    let work = Work::count(config.candidate, &fixture, config.inner);
    print_record(&config, &fixture, &canaries, &sample_ns, &work);
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Candidate {
    Reference,
    Direct,
    Quadratic,
    Reset,
    Prefix,
}

impl Candidate {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "reference" => Ok(Self::Reference),
            "direct" => Ok(Self::Direct),
            "quadratic" => Ok(Self::Quadratic),
            "reset" => Ok(Self::Reset),
            "prefix" => Ok(Self::Prefix),
            _ => Err(format!("unsupported candidate {value:?}")),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Reference => "reference",
            Self::Direct => "direct",
            Self::Quadratic => "quadratic",
            Self::Reset => "reset",
            Self::Prefix => "prefix",
        }
    }

    const fn supports(self, workload: Workload) -> bool {
        match self {
            Self::Direct => true,
            Self::Reference => {
                matches!(workload, Workload::N8ZeroHeavyBudget0 | Workload::N8AllFit)
            }
            Self::Quadratic => matches!(
                workload,
                Workload::N8AllFit
                    | Workload::N64ImmediateReject
                    | Workload::N4096ImmediateReject
                    | Workload::N4096AllFit
            ),
            Self::Reset => matches!(
                workload,
                Workload::N65536AllFit
                    | Workload::N65536OversizedEvery64
                    | Workload::N65536HalfOversizedAlternatingZero
            ),
            Self::Prefix => matches!(
                workload,
                Workload::N64ZeroHeavyBudget0
                    | Workload::N4096UniformModerate
                    | Workload::N65536AllFit
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Workload {
    N8ZeroHeavyBudget0,
    N8AllFit,
    N64ImmediateReject,
    N4096ImmediateReject,
    N4096AllFit,
    N65536AllFit,
    N65536OversizedEvery64,
    N65536HalfOversizedAlternatingZero,
    N64ZeroHeavyBudget0,
    N4096UniformModerate,
}

impl Workload {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "n8_zero_heavy_budget0" => Ok(Self::N8ZeroHeavyBudget0),
            "n8_all_fit" => Ok(Self::N8AllFit),
            "n64_immediate_reject" => Ok(Self::N64ImmediateReject),
            "n4096_immediate_reject" => Ok(Self::N4096ImmediateReject),
            "n4096_all_fit" => Ok(Self::N4096AllFit),
            "n65536_all_fit" => Ok(Self::N65536AllFit),
            "n65536_oversized_every64" => Ok(Self::N65536OversizedEvery64),
            "n65536_half_oversized_alternating_zero" => {
                Ok(Self::N65536HalfOversizedAlternatingZero)
            }
            "n64_zero_heavy_budget0" => Ok(Self::N64ZeroHeavyBudget0),
            "n4096_uniform_moderate" => Ok(Self::N4096UniformModerate),
            _ => Err(format!("unsupported workload {value:?}")),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::N8ZeroHeavyBudget0 => "n8_zero_heavy_budget0",
            Self::N8AllFit => "n8_all_fit",
            Self::N64ImmediateReject => "n64_immediate_reject",
            Self::N4096ImmediateReject => "n4096_immediate_reject",
            Self::N4096AllFit => "n4096_all_fit",
            Self::N65536AllFit => "n65536_all_fit",
            Self::N65536OversizedEvery64 => "n65536_oversized_every64",
            Self::N65536HalfOversizedAlternatingZero => "n65536_half_oversized_alternating_zero",
            Self::N64ZeroHeavyBudget0 => "n64_zero_heavy_budget0",
            Self::N4096UniformModerate => "n4096_uniform_moderate",
        }
    }
}

struct Config {
    candidate: Candidate,
    workload: Workload,
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
        let phase = take(&mut flags, "phase")?;
        if phase != "count" {
            return Err(format!("unsupported phase {phase:?}"));
        }
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
    values: Vec<u64>,
    budget: u128,
    expected: u128,
    formula: &'static str,
    fingerprint: u64,
}

impl Fixture {
    fn generate(workload: Workload) -> Self {
        let (values, budget, expected, formula) = match workload {
            Workload::N8ZeroHeavyBudget0 => (
                vec![0, 2, 0, 0, 5, 0, 1, 0],
                0,
                6,
                "sum of zero-run triangular counts: 1+3+1+1",
            ),
            Workload::N8AllFit => (
                vec![2, 0, 3, 2, 1, 4, 0, 1],
                13,
                triangular(8),
                "all subarrays fit: n(n+1)/2",
            ),
            Workload::N64ImmediateReject => (
                vec![1; 64],
                0,
                0,
                "every positive singleton exceeds a zero budget",
            ),
            Workload::N4096ImmediateReject => (
                vec![1; 4_096],
                0,
                0,
                "every positive singleton exceeds a zero budget",
            ),
            Workload::N4096AllFit => (
                vec![1; 4_096],
                4_096,
                triangular(4_096),
                "all subarrays fit: n(n+1)/2",
            ),
            Workload::N65536AllFit => (
                vec![1; 65_536],
                65_536,
                triangular(65_536),
                "all subarrays fit: n(n+1)/2",
            ),
            Workload::N65536OversizedEvery64 => {
                let values = (0..65_536)
                    .map(|index| if index % 64 == 0 { 64 } else { 1 })
                    .collect();
                (
                    values,
                    63,
                    1_024 * triangular(63),
                    "1024 independent runs of 63 ones separated by oversized values",
                )
            }
            Workload::N65536HalfOversizedAlternatingZero => {
                let values = (0..65_536).map(|index| u64::from(index % 2 == 0)).collect();
                (
                    values,
                    0,
                    32_768,
                    "32768 isolated zero singletons between oversized values",
                )
            }
            Workload::N64ZeroHeavyBudget0 => {
                let values = (0..64).map(|index| u64::from(index % 4 == 3)).collect();
                (values, 0, 16 * triangular(3), "16 zero runs of length 3")
            }
            Workload::N4096UniformModerate => (
                vec![1; 4_096],
                32,
                32 * 4_096 - triangular(31),
                "all subarrays of lengths 1 through 32",
            ),
        };
        let fingerprint = fixture_hash(workload, &values, budget, expected);
        Self {
            values,
            budget,
            expected,
            formula,
            fingerprint,
        }
    }
}

const fn triangular(length: u128) -> u128 {
    length * (length + 1) / 2
}

#[derive(Debug)]
struct Canaries {
    exact_match: bool,
    expected_count: u128,
    observed_count: u128,
    mathematical_u128: bool,
}

fn verify_candidate(candidate: Candidate, fixture: &Fixture) -> Result<Canaries, String> {
    let observed = invoke(candidate, fixture);
    if observed != fixture.expected {
        return Err(format!(
            "candidate {} returned {observed}, expected {} ({})",
            candidate.label(),
            fixture.expected,
            fixture.formula
        ));
    }
    Ok(Canaries {
        exact_match: true,
        expected_count: fixture.expected,
        observed_count: observed,
        mathematical_u128: true,
    })
}

#[inline(never)]
fn topic005_reference_workload(values: &[u64], budget: u128) -> u128 {
    black_box(recompute_reference(black_box(values), black_box(budget)))
}

#[inline(never)]
fn topic005_direct_workload(values: &[u64], budget: u128) -> u128 {
    black_box(direct_sliding(black_box(values), black_box(budget)))
}

#[inline(never)]
fn topic005_quadratic_workload(values: &[u64], budget: u128) -> u128 {
    black_box(quadratic_early_exit(black_box(values), black_box(budget)))
}

#[inline(never)]
fn topic005_reset_workload(values: &[u64], budget: u128) -> u128 {
    black_box(oversized_reset_sliding(
        black_box(values),
        black_box(budget),
    ))
}

#[inline(never)]
fn topic005_prefix_workload(values: &[u64], budget: u128) -> u128 {
    black_box(prefix_binary_search(black_box(values), black_box(budget)))
}

fn invoke(candidate: Candidate, fixture: &Fixture) -> u128 {
    match candidate {
        Candidate::Reference => topic005_reference_workload(&fixture.values, fixture.budget),
        Candidate::Direct => topic005_direct_workload(&fixture.values, fixture.budget),
        Candidate::Quadratic => topic005_quadratic_workload(&fixture.values, fixture.budget),
        Candidate::Reset => topic005_reset_workload(&fixture.values, fixture.budget),
        Candidate::Prefix => topic005_prefix_workload(&fixture.values, fixture.budget),
    }
}

#[inline(never)]
fn topic005_consume_result(value: u128, iteration: usize) -> u64 {
    let bytes = value.to_le_bytes();
    let low = u64::from_le_bytes(bytes[..8].try_into().expect("low half has eight bytes"));
    let high = u64::from_le_bytes(bytes[8..].try_into().expect("high half has eight bytes"));
    let folded = low ^ high;
    folded.rotate_left(u32::try_from(iteration % 64).expect("rotation is below 64"))
}

#[inline(never)]
fn topic005_timed_loop(candidate: Candidate, fixture: &Fixture, inner: usize) -> u64 {
    let mut checksum = FNV_OFFSET;
    for iteration in 0..inner {
        checksum ^= topic005_consume_result(invoke(candidate, fixture), iteration);
        checksum = checksum.wrapping_mul(FNV_PRIME);
    }
    black_box(checksum)
}

#[inline(never)]
fn topic005_clock_now() -> Instant {
    Instant::now()
}

#[derive(Debug)]
struct Work {
    input_values: u128,
    mathematical_subarrays: u128,
    zero_values: u128,
    oversized_values: u128,
    candidate_calls: u128,
    candidate_value_visits: u128,
    candidate_prefix_slots: u128,
    result_scalar_slots: u128,
}

impl Work {
    fn count(candidate: Candidate, fixture: &Fixture, inner: usize) -> Self {
        let multiplier = inner as u128;
        let length = fixture.values.len() as u128;
        let zero_values = fixture.values.iter().filter(|&&value| value == 0).count() as u128;
        let oversized_values = fixture
            .values
            .iter()
            .filter(|&&value| u128::from(value) > fixture.budget)
            .count() as u128;
        let candidate_value_visits = match candidate {
            Candidate::Reference => length * (length + 1) * (length + 2) / 6,
            Candidate::Quadratic => quadratic_visits(&fixture.values, fixture.budget),
            Candidate::Direct => direct_visits(&fixture.values, fixture.budget, false),
            Candidate::Reset => direct_visits(&fixture.values, fixture.budget, true),
            Candidate::Prefix => {
                length + prefix_search_comparisons(&fixture.values, fixture.budget)
            }
        };
        Self {
            input_values: length * multiplier,
            mathematical_subarrays: triangular(length) * multiplier,
            zero_values: zero_values * multiplier,
            oversized_values: oversized_values * multiplier,
            candidate_calls: multiplier,
            candidate_value_visits: candidate_value_visits * multiplier,
            candidate_prefix_slots: if candidate == Candidate::Prefix {
                (length + 1) * multiplier
            } else {
                0
            },
            result_scalar_slots: multiplier,
        }
    }
}

fn quadratic_visits(values: &[u64], budget: u128) -> u128 {
    let mut visits = 0_u128;
    for start in 0..values.len() {
        let mut sum = 0_u128;
        for &value in &values[start..] {
            visits += 1;
            sum += u128::from(value);
            if sum > budget {
                break;
            }
        }
    }
    visits
}

fn direct_visits(values: &[u64], budget: u128, resets_oversized: bool) -> u128 {
    let mut left = 0_usize;
    let mut sum = 0_u128;
    let mut visits = 0_u128;
    for (right, &value) in values.iter().enumerate() {
        visits += 1;
        if resets_oversized && u128::from(value) > budget {
            left = right + 1;
            sum = 0;
            continue;
        }
        sum += u128::from(value);
        while sum > budget {
            visits += 1;
            sum -= u128::from(values[left]);
            left += 1;
        }
    }
    visits
}

fn prefix_search_comparisons(values: &[u64], budget: u128) -> u128 {
    let mut prefix = Vec::with_capacity(values.len() + 1);
    prefix.push(0_u128);
    for &value in values {
        prefix.push(prefix.last().copied().expect("prefix is nonempty") + u128::from(value));
    }
    let mut comparisons = 0_u128;
    for end in 1..prefix.len() {
        let threshold = prefix[end].saturating_sub(budget);
        let mut size = end;
        let mut base = 0_usize;
        while size > 0 {
            let half = size / 2;
            let middle = base + half;
            comparisons += 1;
            if prefix[middle] < threshold {
                base = middle + 1;
                size -= half + 1;
            } else {
                size = half;
            }
        }
    }
    comparisons
}

fn hash_byte(state: &mut u64, value: u8) {
    *state ^= u64::from(value);
    *state = state.wrapping_mul(FNV_PRIME);
}

fn hash_bytes(state: &mut u64, bytes: &[u8]) {
    for &byte in bytes {
        hash_byte(state, byte);
    }
}

fn fixture_hash(workload: Workload, values: &[u64], budget: u128, expected: u128) -> u64 {
    let mut state = FNV_OFFSET;
    hash_bytes(&mut state, GENERATOR_VERSION.as_bytes());
    hash_bytes(&mut state, workload.label().as_bytes());
    hash_bytes(&mut state, &(values.len() as u64).to_le_bytes());
    for value in values {
        hash_bytes(&mut state, &value.to_le_bytes());
    }
    hash_bytes(&mut state, &budget.to_le_bytes());
    hash_bytes(&mut state, &expected.to_le_bytes());
    state
}

fn output_hash(value: u128) -> u64 {
    let mut state = FNV_OFFSET;
    hash_bytes(&mut state, &value.to_le_bytes());
    state
}

fn print_record(
    config: &Config,
    fixture: &Fixture,
    canaries: &Canaries,
    sample_ns: &[u128],
    work: &Work,
) {
    let samples = sample_ns
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",");
    println!(
        concat!(
            "{{\"schema_version\":{},",
            "\"candidate\":\"{}\",",
            "\"workload\":\"{}\",",
            "\"phase\":\"count\",",
            "\"seed\":{},\"warmup\":{},\"samples\":{},\"inner\":{},",
            "\"generator_version\":\"{}\",",
            "\"fixture_hash\":\"{:016x}\",",
            "\"sample_ns\":[{}],",
            "\"output_hash\":\"{:016x}\",",
            "\"work\":{{",
            "\"input_values\":{},",
            "\"mathematical_subarrays\":{},",
            "\"zero_values\":{},",
            "\"oversized_values\":{},",
            "\"candidate_calls\":{},",
            "\"candidate_value_visits\":{},",
            "\"candidate_prefix_slots\":{},",
            "\"result_scalar_slots\":{}",
            "}},",
            "\"canaries\":{{",
            "\"exact_match\":{},",
            "\"expected_count\":{},",
            "\"observed_count\":{},",
            "\"mathematical_u128\":{}",
            "}}}}"
        ),
        SCHEMA_VERSION,
        config.candidate.label(),
        config.workload.label(),
        config.seed,
        config.warmup,
        config.samples,
        config.inner,
        GENERATOR_VERSION,
        fixture.fingerprint,
        samples,
        output_hash(fixture.expected),
        work.input_values,
        work.mathematical_subarrays,
        work.zero_values,
        work.oversized_values,
        work.candidate_calls,
        work.candidate_value_visits,
        work.candidate_prefix_slots,
        work.result_scalar_slots,
        canaries.exact_match,
        canaries.expected_count,
        canaries.observed_count,
        canaries.mathematical_u128,
    );
}
