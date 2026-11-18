//! Discriminating routing and membership regressions.

use topic_053_stable_assignment::{Dispatch, Fleet, Policy, Request, rendezvous};

fn job(at: u64, service: u64, zone: usize, a: usize, b: usize) -> Request {
    Request {
        at,
        service,
        key: 11,
        zone,
        sample: [a, b],
    }
}

#[test]
fn local_wait_crosses_remote_threshold() {
    for (penalty, expected) in [(5, 2), (6, 1)] {
        let mut fleet = Fleet::new(4, penalty);
        for (service, backend) in [(18, 0), (13, 1), (10, 2), (15, 3)] {
            assert_eq!(
                fleet
                    .dispatch(Policy::RoundRobin, job(0, service, 0, 0, 0))
                    .backend,
                backend
            );
        }
        assert_eq!(
            fleet.dispatch(Policy::Local, job(10, 4, 0, 0, 1)),
            Dispatch {
                backend: 1,
                wait: 3,
                response: 7
            }
        );
        assert_eq!(
            fleet.dispatch(Policy::Local, job(11, 4, 0, 0, 1)).backend,
            expected
        );
    }
}

#[test]
fn sampling_repeats_and_queue_ties() {
    let mut fleet = Fleet::new(2, 4);
    assert_eq!(fleet.dispatch(Policy::Two, job(0, 7, 0, 1, 0)).backend, 1);
    let repeated = fleet.dispatch(Policy::Two, job(0, 1, 0, 1, 3));
    assert_eq!(
        repeated,
        Dispatch {
            backend: 1,
            wait: 7,
            response: 12
        }
    );
    assert_eq!(fleet.dispatch(Policy::Scan, job(8, 1, 1, 0, 1)).backend, 0);
    fleet.reset();
    assert_eq!(fleet.dispatch(Policy::Local, job(0, 1, 1, 0, 0)).backend, 1);
}

#[test]
fn membership_edit_moves_only_affected_keys() {
    let original = [83, 10, 25, 47];
    assert_eq!(rendezvous(0, &[]), None);
    let mut removed_owner = 0;
    let mut added_owner = 0;
    for key in 0..4096 {
        let old = rendezvous(key, &original).unwrap();
        let permuted = rendezvous(key, &[47, 25, 10, 83]).unwrap();
        let removed = rendezvous(key, &[47, 25, 83]).unwrap();
        let added = rendezvous(key, &[47, 25, 10, 83, 91]).unwrap();
        assert_eq!(old, permuted);
        if old == 10 {
            assert_ne!(removed, old);
            removed_owner += 1;
        } else {
            assert_eq!(removed, old);
        }
        if added != old {
            assert_eq!(added, 91);
            added_owner += 1;
        }
    }
    assert!(removed_owner > 0 && added_owner > 0);
}

#[test]
#[should_panic(expected = "finish time overflow")]
fn modeled_times_fail_before_wrapping() {
    Fleet::new(2, 0).dispatch(Policy::Scan, job(u64::MAX, 1, 0, 0, 0));
}
