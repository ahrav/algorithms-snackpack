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

#[test]
#[should_panic(expected = "workers must be positive")]
fn zero_workers_is_rejected() {
    let _ = global_fifo(0, 1, 1);
}

#[test]
#[should_panic(expected = "workers must be positive")]
fn zero_workers_is_rejected_by_work_stealing() {
    let _ = locked_work_stealing(0, 1, 1, Placement::Skewed);
}
