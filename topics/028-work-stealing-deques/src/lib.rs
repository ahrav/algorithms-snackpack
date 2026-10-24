//! Safe locked scheduler models for studying work-stealing policy.
//!
//! The work-stealing candidate makes task ownership and steal direction explicit.
//! It uses mutexes so the experiment isolates policy without claiming a lock-free deque.
#![forbid(unsafe_code)]

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

/// Distribution of initial tasks among workers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Placement {
    /// Give every task to worker zero so idle workers must steal.
    Skewed,
    /// Round-robin tasks among workers.
    Balanced,
}

/// Checked result of one scheduler run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Outcome {
    /// Number of executed tasks.
    pub completed: usize,
    /// Wrapping sum of deterministic per-task values.
    pub checksum: u64,
}

fn task_value(id: u32, leaf_work: u32) -> u64 {
    let mut x = u64::from(id).wrapping_mul(0x9e37_79b9);
    for _ in 0..leaf_work {
        x = x
            .rotate_left(7)
            .wrapping_mul(0x9e37_79b9_7f4a_7c15)
            .wrapping_add(17);
    }
    x
}

/// Independent sequential oracle for the exact task set.
#[must_use]
pub fn oracle(task_count: u32, leaf_work: u32) -> Outcome {
    Outcome {
        completed: task_count as usize,
        checksum: (0..task_count)
            .fold(0_u64, |sum, id| sum.wrapping_add(task_value(id, leaf_work))),
    }
}

fn checked_args(workers: usize) {
    assert!(workers > 0, "workers must be positive");
}

/// Execute tasks with one globally locked FIFO queue.
///
/// # Panics
///
/// Panics when `workers` is zero or a worker mutex is poisoned.
#[must_use]
pub fn global_fifo(workers: usize, task_count: u32, leaf_work: u32) -> Outcome {
    checked_args(workers);
    let queue = Arc::new(Mutex::new((0..task_count).collect::<VecDeque<_>>()));
    let completed = Arc::new(AtomicUsize::new(0));
    let checksum = Arc::new(AtomicU64::new(0));
    thread::scope(|scope| {
        for _ in 0..workers {
            let queue = Arc::clone(&queue);
            let completed = Arc::clone(&completed);
            let checksum = Arc::clone(&checksum);
            scope.spawn(move || {
                loop {
                    let task = queue.lock().expect("queue mutex poisoned").pop_front();
                    let Some(id) = task else { break };
                    checksum.fetch_add(task_value(id, leaf_work), Ordering::Relaxed);
                    completed.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });
    Outcome {
        completed: completed.load(Ordering::Relaxed),
        checksum: checksum.load(Ordering::Relaxed),
    }
}

/// Execute tasks with owner-bottom pops and thief-top steals.
///
/// Each worker owns one deque. The owner removes from the back. A thief visits
/// victims cyclically and removes from the front. Mutexes provide the claim for
/// this safe teaching implementation. A Chase-Lev deque replaces that claim
/// with carefully ordered atomics and safe retired-array handling.
///
/// # Panics
///
/// Panics when `workers` is zero or a worker mutex is poisoned.
#[must_use]
pub fn locked_work_stealing(
    workers: usize,
    task_count: u32,
    leaf_work: u32,
    placement: Placement,
) -> Outcome {
    checked_args(workers);
    let queues = Arc::new(
        (0..workers)
            .map(|_| Mutex::new(VecDeque::new()))
            .collect::<Vec<_>>(),
    );
    for id in 0..task_count {
        let target = match placement {
            Placement::Skewed => 0,
            Placement::Balanced => id as usize % workers,
        };
        queues[target]
            .lock()
            .expect("queue mutex poisoned")
            .push_back(id);
    }
    let remaining = Arc::new(AtomicUsize::new(task_count as usize));
    let completed = Arc::new(AtomicUsize::new(0));
    let checksum = Arc::new(AtomicU64::new(0));
    thread::scope(|scope| {
        for worker in 0..workers {
            let queues = Arc::clone(&queues);
            let remaining = Arc::clone(&remaining);
            let completed = Arc::clone(&completed);
            let checksum = Arc::clone(&checksum);
            scope.spawn(move || {
                loop {
                    let own = queues[worker]
                        .lock()
                        .expect("queue mutex poisoned")
                        .pop_back();
                    let task = own.or_else(|| {
                        (1..workers).find_map(|offset| {
                            let victim = (worker + offset) % workers;
                            queues[victim]
                                .lock()
                                .expect("queue mutex poisoned")
                                .pop_front()
                        })
                    });
                    if let Some(id) = task {
                        checksum.fetch_add(task_value(id, leaf_work), Ordering::Relaxed);
                        completed.fetch_add(1, Ordering::Relaxed);
                        remaining.fetch_sub(1, Ordering::Release);
                    } else if remaining.load(Ordering::Acquire) == 0 {
                        break;
                    } else {
                        thread::yield_now();
                    }
                }
            });
        }
    });
    Outcome {
        completed: completed.load(Ordering::Relaxed),
        checksum: checksum.load(Ordering::Relaxed),
    }
}
