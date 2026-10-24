//! Run one safe scheduler model and check it against a sequential oracle.
use std::time::Instant;
use topic028_work_stealing_deques::{Placement, global_fifo, locked_work_stealing, oracle};

fn value_after(flag: &str) -> String {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == flag {
            return args
                .next()
                .unwrap_or_else(|| panic!("{flag} needs a value"));
        }
    }
    panic!("missing {flag}");
}

fn positive_usize(flag: &str) -> usize {
    value_after(flag)
        .parse()
        .unwrap_or_else(|_| panic!("{flag} must be an unsigned integer"))
}

fn main() {
    let candidate = value_after("--candidate");
    let workers = positive_usize("--workers");
    let tasks = value_after("--tasks")
        .parse()
        .expect("--tasks must be a u32");
    let leaf_work = value_after("--leaf-work")
        .parse()
        .expect("--leaf-work must be a u32");
    let expected = oracle(tasks, leaf_work);
    let started = Instant::now();
    let result = match candidate.as_str() {
        "global_fifo" => global_fifo(workers, tasks, leaf_work),
        "locked_steal_skewed" => locked_work_stealing(workers, tasks, leaf_work, Placement::Skewed),
        "locked_steal_balanced" => {
            locked_work_stealing(workers, tasks, leaf_work, Placement::Balanced)
        }
        _ => panic!("unknown --candidate {candidate}"),
    };
    assert_eq!(result, expected, "{candidate} violated the task contract");
    println!(
        "candidate={candidate} workers={workers} tasks={tasks} leaf_work={leaf_work} elapsed_ns={}",
        started.elapsed().as_nanos()
    );
}
