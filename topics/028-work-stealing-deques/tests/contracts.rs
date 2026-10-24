//! Contract tests for the safe scheduler models.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use topic028_work_stealing_deques::{
    Outcome, Placement, global_fifo, locked_work_stealing, oracle,
};

/// Termination is part of the scheduler contract.
fn within_deadline(run: impl FnOnce() -> Outcome + Send + 'static) -> Outcome {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let _ = sender.send(run());
    });
    receiver
        .recv_timeout(Duration::from_secs(30))
        .expect("scheduler did not terminate within 30 seconds")
}

#[test]
fn all_candidates_match_the_independent_oracle() {
    for workers in 1..=6 {
        for tasks in [0, 1, 2, 17, 256] {
            for work in [0, 1, 7] {
                let expected = oracle(tasks, work);
                assert_eq!(
                    within_deadline(move || global_fifo(workers, tasks, work)),
                    expected
                );
                assert_eq!(
                    within_deadline(move || locked_work_stealing(
                        workers,
                        tasks,
                        work,
                        Placement::Skewed
                    )),
                    expected
                );
                assert_eq!(
                    within_deadline(move || locked_work_stealing(
                        workers,
                        tasks,
                        work,
                        Placement::Balanced
                    )),
                    expected
                );
            }
        }
    }
}

fn panic_message(run: impl FnOnce() -> Outcome + std::panic::UnwindSafe) -> String {
    let payload = std::panic::catch_unwind(run).expect_err("zero workers must panic");
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_owned()))
        .expect("panic payload must be a string")
}

#[test]
fn zero_workers_is_rejected() {
    assert_eq!(
        panic_message(|| global_fifo(0, 1, 1)),
        "workers must be positive"
    );
}

#[test]
fn zero_workers_is_rejected_by_work_stealing() {
    assert_eq!(
        panic_message(|| locked_work_stealing(0, 1, 1, Placement::Skewed)),
        "workers must be positive"
    );
}
