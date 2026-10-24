//! Contract tests for the safe scheduler models.

use topic028_work_stealing_deques::{Placement, global_fifo, locked_work_stealing, oracle};

#[test]
fn all_candidates_match_the_independent_oracle() {
    for workers in 1..=6 {
        for tasks in [0, 1, 2, 17, 256] {
            for work in [0, 1, 7] {
                let expected = oracle(tasks, work);
                assert_eq!(global_fifo(workers, tasks, work), expected);
                assert_eq!(
                    locked_work_stealing(workers, tasks, work, Placement::Skewed),
                    expected
                );
                assert_eq!(
                    locked_work_stealing(workers, tasks, work, Placement::Balanced),
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
