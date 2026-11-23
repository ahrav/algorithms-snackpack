//! Compare graph and resource decisions with independent completion searches.

use cycle_detection_and_deadlock_resolution::{InputError, Snapshot, Victim, mandatory_waits};

fn graph_completion_search(waits: &[Vec<usize>], done: usize) -> usize {
    let mut union = done;
    for (task, dependencies) in waits.iter().enumerate() {
        if done & (1 << task) == 0 && dependencies.iter().all(|&other| done & (1 << other) != 0) {
            union |= graph_completion_search(waits, done | (1 << task));
        }
    }
    union
}

fn resource_completion_search(
    free: &[u32],
    held: &[Vec<u32>],
    requests: &[Vec<u32>],
    done: usize,
) -> usize {
    let mut union = done;
    for (task, (allocation, request)) in held.iter().zip(requests).enumerate() {
        if done & (1 << task) != 0 {
            continue;
        }
        let after_grant: Option<Vec<_>> = free
            .iter()
            .zip(request)
            .map(|(available, demand)| available.checked_sub(*demand))
            .collect();
        if let Some(mut after_grant) = after_grant {
            for ((available, &used), &previously_held) in
                after_grant.iter_mut().zip(request).zip(allocation)
            {
                *available = (*available + used) + previously_held;
            }
            union |= resource_completion_search(&after_grant, held, requests, done | (1 << task));
        }
    }
    union
}

#[test]
fn all_three_task_graphs_match_exhaustive_completion() {
    for bits in 0..1 << 9 {
        let waits: Vec<Vec<_>> = (0..3)
            .map(|task| {
                (0..3)
                    .filter(|&dep| bits & (1 << (task * 3 + dep)) != 0)
                    .collect()
            })
            .collect();
        let report = mandatory_waits(&waits).unwrap();
        let found = graph_completion_search(&waits, 0);
        let expected_blocked: Vec<_> = (0..3).filter(|task| found & (1 << task) == 0).collect();
        assert_eq!(report.blocked, expected_blocked, "{waits:?}");
        let mut emitted = 0;
        for task in report.order {
            assert_eq!(emitted & (1 << task), 0);
            assert!(waits[task].iter().all(|&dep| emitted & (1 << dep) != 0));
            emitted |= 1 << task;
        }
        assert_eq!(emitted, found);
        if let Some(cycle) = report.cycle {
            assert!(cycle.len() >= 2);
            assert_eq!(cycle.first(), cycle.last());
            let mut interior = cycle[..cycle.len() - 1].to_vec();
            interior.sort_unstable();
            interior.dedup();
            assert_eq!(interior.len() + 1, cycle.len());
            assert!(
                cycle
                    .windows(2)
                    .all(|edge| waits[edge[0]].contains(&edge[1]))
            );
        } else {
            assert!(expected_blocked.is_empty());
        }
    }
    println!("512 directed graphs checked");
}

fn resource_columns() -> Vec<(u32, [u32; 2], [u32; 2])> {
    let mut columns = Vec::new();
    for capacity in 0..=2 {
        for a_held in 0..=capacity {
            for b_held in 0..=capacity - a_held {
                for a_need in 0..=capacity - a_held {
                    for b_need in 0..=capacity - b_held {
                        columns.push((capacity, [a_held, b_held], [a_need, b_need]));
                    }
                }
            }
        }
    }
    columns
}

#[test]
fn all_two_task_two_resource_snapshots_match_exhaustive_grants() {
    let columns = resource_columns();
    for &(x, x_held, x_need) in &columns {
        for &(y, y_held, y_need) in &columns {
            let held = [vec![x_held[0], y_held[0]], vec![x_held[1], y_held[1]]];
            let need = [vec![x_need[0], y_need[0]], vec![x_need[1], y_need[1]]];
            let free = [
                x - x_held.iter().sum::<u32>(),
                y - y_held.iter().sum::<u32>(),
            ];
            let snapshot = Snapshot::new(&[x, y], &held, &need).unwrap();
            let result = snapshot.reduce();
            let reachable = resource_completion_search(&free, &held, &need, 0);
            let stalled: Vec<_> = (0..2).filter(|task| reachable & (1 << task) == 0).collect();
            assert_eq!(result.stalled, stalled, "{x} {y} {held:?} {need:?}");

            let mut trial_free = free;
            let mut completed = 0;
            for task in result.order {
                assert_eq!(completed & (1 << task), 0);
                completed |= 1 << task;
                for ((units, &grant), &release) in
                    trial_free.iter_mut().zip(&need[task]).zip(&held[task])
                {
                    *units = units.checked_sub(grant).unwrap();
                    *units += grant + release;
                }
            }
            assert_eq!(completed, reachable);
            assert_eq!(result.free, trial_free);

            for costs in [[4, 1], [1, 4], [2, 2]] {
                let best = if reachable == 3 {
                    None
                } else {
                    (0..2)
                        .filter(|&task| {
                            let after = [free[0] + held[task][0], free[1] + held[task][1]];
                            resource_completion_search(&after, &held, &need, 1 << task) == 3
                        })
                        .min_by_key(|&task| (costs[task], task))
                        .map(|task| Victim {
                            task,
                            cost: costs[task],
                        })
                };
                assert_eq!(snapshot.cheapest_single_victim(&costs), Ok(best));
            }
        }
    }
    println!(
        "{} resource snapshots and {} recovery policies checked",
        columns.len().pow(2),
        3 * columns.len().pow(2)
    );
}

#[test]
fn alternative_holders_dependents_and_disjoint_cycles() {
    let graph = mandatory_waits(&[vec![1, 1], vec![0], vec![0], vec![]]).unwrap();
    assert_eq!(graph.order, [3]);
    assert_eq!(graph.blocked, [0, 1, 2]);
    assert_eq!(graph.cycle.unwrap(), [0, 1, 0]);
    assert_eq!(mandatory_waits(&[vec![0]]).unwrap().cycle, Some(vec![0, 0]));
    assert_eq!(
        mandatory_waits(&[vec![1, 1], vec![]]).unwrap().order,
        [1, 0]
    );

    let held = [vec![1, 0], vec![0, 1], vec![0, 1]];
    let need = [vec![0, 1], vec![1, 0], vec![0, 0]];
    let pools = Snapshot::new(&[1, 2], &held, &need).unwrap();
    assert_eq!(pools.reduce().order, [2, 0, 1]);

    let held = [vec![1, 0], vec![0, 1], vec![0, 0]];
    let need = [vec![0, 1], vec![1, 0], vec![1, 0]];
    let pools = Snapshot::new(&[1, 1], &held, &need).unwrap();
    assert_eq!(pools.reduce().stalled, [0, 1, 2]);
    assert_eq!(
        pools.cheapest_single_victim(&[7, 3, 0]).unwrap(),
        Some(Victim { task: 1, cost: 3 })
    );

    let held = [
        vec![1, 0, 0, 0],
        vec![0, 1, 0, 0],
        vec![0, 0, 1, 0],
        vec![0, 0, 0, 1],
    ];
    let need = [
        held[1].clone(),
        held[0].clone(),
        held[3].clone(),
        held[2].clone(),
    ];
    let pools = Snapshot::new(&[1, 1, 1, 1], &held, &need).unwrap();
    assert_eq!(pools.reduce().stalled, [0, 1, 2, 3]);
    assert_eq!(pools.cheapest_single_victim(&[1; 4]).unwrap(), None);
}

#[test]
fn input_boundaries_and_maximum_counts() {
    assert_eq!(mandatory_waits(&[vec![1]]), Err(InputError::Endpoint));
    assert!(mandatory_waits(&[]).unwrap().order.is_empty());
    assert!(
        Snapshot::new(&[1], &[], &[])
            .unwrap()
            .reduce()
            .order
            .is_empty()
    );
    assert!(matches!(
        Snapshot::new(&[1], &[vec![]], &[vec![]]),
        Err(InputError::Shape)
    ));
    assert!(matches!(
        Snapshot::new(&[u32::MAX], &[vec![u32::MAX], vec![1]], &[vec![0], vec![0]]),
        Err(InputError::Capacity)
    ));
    assert!(matches!(
        Snapshot::new(&[u32::MAX], &[vec![u32::MAX]], &[vec![1]]),
        Err(InputError::ImpossibleRequest)
    ));
    let held = [vec![u32::MAX]];
    let need = [vec![0]];
    let snapshot = Snapshot::new(&[u32::MAX], &held, &need).unwrap();
    assert_eq!(snapshot.reduce().free, [u32::MAX]);
    assert_eq!(snapshot.cheapest_single_victim(&[]), Err(InputError::Costs));
}
