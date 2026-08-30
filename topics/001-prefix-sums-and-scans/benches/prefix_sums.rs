//! One-position native benchmark harness for the prefix-scan experiment.

use prefix_sums_and_scans::{
    blocked_inclusive, linear_inclusive, parallel_inclusive, reference_inclusive,
};
use std::env;
use std::fmt::Write as _;
use std::hint::black_box;
use std::process;
use std::time::Instant;

const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy)]
enum Algorithm {
    Reference,
    Linear,
    Blocked,
    Parallel,
}

impl Algorithm {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "reference" => Ok(Self::Reference),
            "linear" => Ok(Self::Linear),
            "blocked" => Ok(Self::Blocked),
            "parallel" => Ok(Self::Parallel),
            _ => Err(format!("unknown algorithm: {value}")),
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Reference => "reference",
            Self::Linear => "linear",
            Self::Blocked => "blocked",
            Self::Parallel => "parallel",
        }
    }
}

#[derive(Clone, Copy)]
enum Pattern {
    Zero,
    Constant,
    Ascending,
    Alternating,
    Mixed,
}

impl Pattern {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "zero" => Ok(Self::Zero),
            "constant" => Ok(Self::Constant),
            "ascending" => Ok(Self::Ascending),
            "alternating" => Ok(Self::Alternating),
            "mixed" => Ok(Self::Mixed),
            _ => Err(format!("unknown pattern: {value}")),
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Zero => "zero",
            Self::Constant => "constant",
            Self::Ascending => "ascending",
            Self::Alternating => "alternating",
            Self::Mixed => "mixed",
        }
    }
}

struct Config {
    algorithm: Algorithm,
    label: String,
    attempt_id: String,
    contrast_id: String,
    phase: String,
    n: usize,
    pattern: Pattern,
    seed: u64,
    block_size: usize,
    workers: usize,
    warmups: usize,
    samples: usize,
}

struct WorkCounts {
    wrapping_adds: u128,
    input_elements_read: u128,
    output_element_writes: u128,
    output_element_reads: u128,
    auxiliary_element_writes: u128,
    auxiliary_element_reads: u128,
    explicit_vec_allocations: u128,
    requested_worker_threads: u128,
    effective_worker_threads: u128,
    scoped_thread_spawns: u128,
    scoped_thread_joins: u128,
}

fn usage() -> &'static str {
    "usage: prefix_sums \
--algorithm reference|linear|blocked|parallel \
--label LABEL --attempt-id ID --contrast-id ID --phase PHASE \
--n N --pattern zero|constant|ascending|alternating|mixed --seed SEED \
--block-size N --workers N --warmups N --samples N"
}

fn take_value(args: &[String], index: &mut usize, flag: &str) -> Result<String, String> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn parse_number<T>(value: &str, flag: &str) -> Result<T, String>
where
    T: std::str::FromStr,
{
    value
        .parse::<T>()
        .map_err(|_| format!("invalid value for {flag}: {value}"))
}

fn parse_args() -> Result<Config, String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", usage());
        process::exit(0);
    }

    let mut algorithm = None;
    let mut label = None;
    let mut attempt_id = None;
    let mut contrast_id = None;
    let mut phase = None;
    let mut n = None;
    let mut pattern = None;
    let mut seed = None;
    let mut block_size = None;
    let mut workers = None;
    let mut warmups = None;
    let mut samples = None;
    let mut index = 0;

    while index < args.len() {
        let flag = &args[index];
        match flag.as_str() {
            "--algorithm" => {
                let value = take_value(&args, &mut index, flag)?;
                algorithm = Some(Algorithm::parse(&value)?);
            }
            "--label" => label = Some(take_value(&args, &mut index, flag)?),
            "--attempt-id" => attempt_id = Some(take_value(&args, &mut index, flag)?),
            "--contrast-id" => contrast_id = Some(take_value(&args, &mut index, flag)?),
            "--phase" => phase = Some(take_value(&args, &mut index, flag)?),
            "--n" => {
                let value = take_value(&args, &mut index, flag)?;
                n = Some(parse_number::<usize>(&value, flag)?);
            }
            "--pattern" => {
                let value = take_value(&args, &mut index, flag)?;
                pattern = Some(Pattern::parse(&value)?);
            }
            "--seed" => {
                let value = take_value(&args, &mut index, flag)?;
                seed = Some(parse_number::<u64>(&value, flag)?);
            }
            "--block-size" => {
                let value = take_value(&args, &mut index, flag)?;
                block_size = Some(parse_number::<usize>(&value, flag)?);
            }
            "--workers" => {
                let value = take_value(&args, &mut index, flag)?;
                workers = Some(parse_number::<usize>(&value, flag)?);
            }
            "--warmups" => {
                let value = take_value(&args, &mut index, flag)?;
                warmups = Some(parse_number::<usize>(&value, flag)?);
            }
            "--samples" => {
                let value = take_value(&args, &mut index, flag)?;
                samples = Some(parse_number::<usize>(&value, flag)?);
            }
            // Accept Cargo's injected `--bench` argument so `cargo bench -- ...` works.
            "--bench" => {}
            _ => return Err(format!("unknown argument: {flag}\n{}", usage())),
        }
        index += 1;
    }

    let config = Config {
        algorithm: algorithm.ok_or_else(|| "missing --algorithm".to_owned())?,
        label: label.ok_or_else(|| "missing --label".to_owned())?,
        attempt_id: attempt_id.ok_or_else(|| "missing --attempt-id".to_owned())?,
        contrast_id: contrast_id.ok_or_else(|| "missing --contrast-id".to_owned())?,
        phase: phase.ok_or_else(|| "missing --phase".to_owned())?,
        n: n.ok_or_else(|| "missing --n".to_owned())?,
        pattern: pattern.ok_or_else(|| "missing --pattern".to_owned())?,
        seed: seed.ok_or_else(|| "missing --seed".to_owned())?,
        block_size: block_size.ok_or_else(|| "missing --block-size".to_owned())?,
        workers: workers.ok_or_else(|| "missing --workers".to_owned())?,
        warmups: warmups.ok_or_else(|| "missing --warmups".to_owned())?,
        samples: samples.ok_or_else(|| "missing --samples".to_owned())?,
    };

    if config.samples == 0 {
        return Err("--samples must be greater than zero".to_owned());
    }
    if matches!(config.algorithm, Algorithm::Blocked) && config.block_size == 0 {
        return Err("blocked requires --block-size greater than zero".to_owned());
    }
    if matches!(config.algorithm, Algorithm::Parallel) && config.workers == 0 {
        return Err("parallel requires --workers greater than zero".to_owned());
    }

    Ok(config)
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut value = *state;
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn bits_as_i64(value: u64) -> i64 {
    i64::from_ne_bytes(value.to_ne_bytes())
}

fn generate_input(n: usize, pattern: Pattern, seed: u64) -> Result<Vec<i64>, String> {
    let mut input = Vec::with_capacity(n);
    let mut state = seed ^ u64::try_from(n).map_err(|_| "length does not fit u64")?;

    for index in 0..n {
        let index_i64 = i64::try_from(index).map_err(|_| "index does not fit i64")?;
        let value = match pattern {
            Pattern::Zero => 0,
            Pattern::Constant => 7,
            Pattern::Ascending => index_i64,
            Pattern::Alternating => match index % 4 {
                0 => i64::MAX,
                1 => 1,
                2 => i64::MIN,
                _ => -1,
            },
            Pattern::Mixed => {
                let random = splitmix64(&mut state);
                match random & 7 {
                    0 => 0,
                    1 => 1,
                    2 => -1,
                    3 => index_i64 & 1023,
                    4 => -(index_i64 & 1023),
                    5 => bits_as_i64(random),
                    6 => i64::MAX - i64::try_from(random & 0xffff).expect("masked value fits"),
                    _ => i64::MIN + i64::try_from(random & 0xffff).expect("masked value fits"),
                }
            }
        };
        input.push(value);
    }
    Ok(input)
}

fn run_candidate(config: &Config, input: &[i64]) -> Result<Vec<i64>, String> {
    match config.algorithm {
        Algorithm::Reference => Ok(reference_inclusive(input)),
        Algorithm::Linear => Ok(linear_inclusive(input)),
        Algorithm::Blocked => {
            blocked_inclusive(input, config.block_size).map_err(|error| error.to_string())
        }
        Algorithm::Parallel => {
            parallel_inclusive(input, config.workers).map_err(|error| error.to_string())
        }
    }
}

fn validate_output(input: &[i64], output: &[i64]) -> bool {
    if input.len() != output.len() {
        return false;
    }

    let mut expected = 0_i64;
    input.iter().zip(output).all(|(&value, &observed)| {
        expected = expected.wrapping_add(value);
        observed == expected
    })
}

fn checksum(values: &[i64]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in u64::try_from(values.len())
        .expect("length fits u64")
        .to_le_bytes()
        .into_iter()
        .chain(values.iter().flat_map(|value| value.to_le_bytes()))
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn div_ceil(numerator: usize, denominator: usize) -> usize {
    if numerator == 0 {
        0
    } else {
        1 + (numerator - 1) / denominator
    }
}

#[allow(clippy::too_many_lines)]
fn work_counts(config: &Config) -> WorkCounts {
    let n = config.n as u128;
    match config.algorithm {
        Algorithm::Reference => WorkCounts {
            wrapping_adds: n.saturating_mul(n + 1) / 2,
            input_elements_read: n.saturating_mul(n + 1) / 2,
            output_element_writes: n,
            output_element_reads: 0,
            auxiliary_element_writes: 0,
            auxiliary_element_reads: 0,
            explicit_vec_allocations: u128::from(config.n > 0),
            requested_worker_threads: 0,
            effective_worker_threads: 0,
            scoped_thread_spawns: 0,
            scoped_thread_joins: 0,
        },
        Algorithm::Linear => WorkCounts {
            wrapping_adds: config.n.saturating_sub(1) as u128,
            input_elements_read: n,
            output_element_writes: n,
            output_element_reads: 0,
            auxiliary_element_writes: 0,
            auxiliary_element_reads: 0,
            explicit_vec_allocations: u128::from(config.n > 0),
            requested_worker_threads: 0,
            effective_worker_threads: 0,
            scoped_thread_spawns: 0,
            scoped_thread_joins: 0,
        },
        Algorithm::Blocked => {
            let block_count = div_ceil(config.n, config.block_size);
            let first_block = config.n.min(config.block_size);
            let adjusted = config.n - first_block;
            let wrapping_adds = match block_count {
                0 => 0,
                1 => config.n.saturating_sub(1) as u128,
                _ => 2 * n - first_block as u128 - 2,
            };
            WorkCounts {
                wrapping_adds,
                input_elements_read: n,
                output_element_writes: n + adjusted as u128,
                output_element_reads: adjusted as u128,
                auxiliary_element_writes: block_count as u128,
                auxiliary_element_reads: block_count as u128,
                explicit_vec_allocations: 2 * u128::from(config.n > 0),
                requested_worker_threads: 0,
                effective_worker_threads: 0,
                scoped_thread_spawns: 0,
                scoped_thread_joins: 0,
            }
        }
        Algorithm::Parallel => {
            let effective_workers = config.workers.min(config.n);
            if effective_workers == 0 {
                return WorkCounts {
                    wrapping_adds: 0,
                    input_elements_read: 0,
                    output_element_writes: 0,
                    output_element_reads: 0,
                    auxiliary_element_writes: 0,
                    auxiliary_element_reads: 0,
                    explicit_vec_allocations: 0,
                    requested_worker_threads: config.workers as u128,
                    effective_worker_threads: 0,
                    scoped_thread_spawns: 0,
                    scoped_thread_joins: 0,
                };
            }
            if effective_workers == 1 {
                return WorkCounts {
                    wrapping_adds: config.n.saturating_sub(1) as u128,
                    input_elements_read: n,
                    output_element_writes: n,
                    output_element_reads: 0,
                    auxiliary_element_writes: 0,
                    auxiliary_element_reads: 0,
                    explicit_vec_allocations: 1,
                    requested_worker_threads: config.workers as u128,
                    effective_worker_threads: 1,
                    scoped_thread_spawns: 0,
                    scoped_thread_joins: 0,
                };
            }

            let first_chunk = config.n / effective_workers
                + usize::from(!config.n.is_multiple_of(effective_workers));
            let adjusted = config.n - first_chunk;
            WorkCounts {
                wrapping_adds: 2 * n - first_chunk as u128 - 2,
                input_elements_read: n,
                output_element_writes: 2 * n + adjusted as u128,
                output_element_reads: n + adjusted as u128,
                auxiliary_element_writes: (2 * effective_workers - 1) as u128,
                auxiliary_element_reads: (2 * effective_workers - 1) as u128,
                explicit_vec_allocations: (effective_workers + 6) as u128,
                requested_worker_threads: config.workers as u128,
                effective_worker_threads: effective_workers as u128,
                scoped_thread_spawns: (2 * effective_workers - 1) as u128,
                scoped_thread_joins: (2 * effective_workers - 1) as u128,
            }
        }
    }
}

fn median(samples: &[u128]) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let middle = ordered.len() / 2;
    if ordered.len().is_multiple_of(2) {
        u128::midpoint(ordered[middle - 1], ordered[middle])
    } else {
        ordered[middle]
    }
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character.is_control() => {
                write!(escaped, "\\u{:04x}", u32::from(character))
                    .expect("String writes cannot fail");
            }
            character => escaped.push(character),
        }
    }
    escaped
}

fn observed_affinity() -> Option<String> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    status.lines().find_map(|line| {
        line.strip_prefix("Cpus_allowed_list:")
            .map(str::trim)
            .map(str::to_owned)
    })
}

fn real_main() -> Result<(), String> {
    let config = parse_args()?;
    let input = generate_input(config.n, config.pattern, config.seed)?;
    let input_checksum = checksum(&input);
    let counts = work_counts(&config);

    let mut expected_output_checksum = None;
    for _ in 0..config.warmups {
        let output = run_candidate(&config, black_box(&input))?;
        if !validate_output(&input, &output) {
            return Err("semantic canary failed during warmup".to_owned());
        }
        expected_output_checksum = Some(checksum(black_box(&output)));
    }

    let mut sample_ns = Vec::with_capacity(config.samples);
    for _ in 0..config.samples {
        let start = Instant::now();
        let output = black_box(run_candidate(&config, black_box(&input))?);
        let elapsed_ns = start.elapsed().as_nanos();
        if elapsed_ns == 0 {
            return Err("timer returned zero nanoseconds".to_owned());
        }
        if !validate_output(&input, &output) {
            return Err("semantic canary failed after timed call".to_owned());
        }
        let output_checksum = checksum(black_box(&output));
        if let Some(expected) = expected_output_checksum {
            if output_checksum != expected {
                return Err("output checksum changed across calls".to_owned());
            }
        } else {
            expected_output_checksum = Some(output_checksum);
        }
        sample_ns.push(elapsed_ns);
    }

    let output_checksum = expected_output_checksum.ok_or_else(|| "missing checksum".to_owned())?;
    let samples_json = sample_ns
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let affinity_json = observed_affinity().map_or_else(
        || "null".to_owned(),
        |value| format!("\"{}\"", json_escape(&value)),
    );

    println!(
        concat!(
            "{{\"schema_version\":{},\"attempt_id\":\"{}\",",
            "\"contrast_id\":\"{}\",\"phase\":\"{}\",\"label\":\"{}\",",
            "\"algorithm\":\"{}\",\"n\":{},\"pattern\":\"{}\",\"seed\":{},",
            "\"block_size\":{},\"workers\":{},\"warmups\":{},\"samples\":{},",
            "\"pid\":{},\"observed_affinity\":{},\"semantic_ok\":true,",
            "\"input_checksum\":\"{:016x}\",\"output_checksum\":\"{:016x}\",",
            "\"sample_ns\":[{}],\"position_ns\":{},\"work\":{{",
            "\"wrapping_adds\":{},\"input_elements_read\":{},",
            "\"output_element_writes\":{},\"output_element_reads\":{},",
            "\"auxiliary_element_writes\":{},\"auxiliary_element_reads\":{},",
            "\"explicit_vec_allocations\":{},\"requested_worker_threads\":{},",
            "\"effective_worker_threads\":{},\"scoped_thread_spawns\":{},",
            "\"scoped_thread_joins\":{},\"thread_runtime_allocations_counted\":false",
            "}}}}"
        ),
        SCHEMA_VERSION,
        json_escape(&config.attempt_id),
        json_escape(&config.contrast_id),
        json_escape(&config.phase),
        json_escape(&config.label),
        config.algorithm.name(),
        config.n,
        config.pattern.name(),
        config.seed,
        config.block_size,
        config.workers,
        config.warmups,
        config.samples,
        process::id(),
        affinity_json,
        input_checksum,
        output_checksum,
        samples_json,
        median(&sample_ns),
        counts.wrapping_adds,
        counts.input_elements_read,
        counts.output_element_writes,
        counts.output_element_reads,
        counts.auxiliary_element_writes,
        counts.auxiliary_element_reads,
        counts.explicit_vec_allocations,
        counts.requested_worker_threads,
        counts.effective_worker_threads,
        counts.scoped_thread_spawns,
        counts.scoped_thread_joins,
    );
    Ok(())
}

fn main() {
    if env::args_os().len() == 1 {
        return;
    }
    if let Err(error) = real_main() {
        eprintln!("prefix_sums benchmark error: {error}");
        process::exit(2);
    }
}
