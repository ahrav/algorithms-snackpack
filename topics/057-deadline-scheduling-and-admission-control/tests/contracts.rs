//! Independent feasibility oracles and deadline boundary checks.

use deadline_scheduling_and_admission_control::{
    Admission, Job, Policy, Refill, Reservation, Slot, Task, admit, demand, schedule,
    schedule_with_server,
};

type JobSpec = (u32, u32, u32);

fn can_assign(time: u32, horizon: u32, specs: &[JobSpec], work: &mut [u32]) -> bool {
    if work.iter().all(|&left| left == 0) {
        return true;
    }
    if time == horizon
        || specs
            .iter()
            .zip(&*work)
            .any(|(&(_, due, _), &left)| left > due.saturating_sub(time))
    {
        return false;
    }
    for (i, &(release, _, _)) in specs.iter().enumerate() {
        if release <= time && work[i] > 0 {
            work[i] -= 1;
            let feasible = can_assign(time + 1, horizon, specs, work);
            work[i] += 1;
            if feasible {
                return true;
            }
        }
    }
    can_assign(time + 1, horizon, specs, work)
}

#[test]
fn edf_feasibility_agrees_with_exhaustive_schedules() {
    let mut menu = Vec::new();
    for release in 0..3 {
        for width in 1..=3 {
            for cost in 1..=width.min(2) {
                menu.push((release, release + width, cost));
            }
        }
    }
    let mut cases = 0;
    for &a in &menu {
        for &b in &menu {
            for &c in &menu {
                let specs = [a, b, c];
                let jobs = specs.map(|(r, d, c)| Job::new(r, d, c, 0).unwrap());
                let mut work = specs.map(|(_, _, c)| c);
                assert_eq!(
                    schedule(&jobs, Policy::Edf, 5).missed.is_empty(),
                    can_assign(0, 5, &specs, &mut work),
                    "{specs:?}"
                );
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 3375);
}

#[test]
fn dispatch_uses_absolute_deadlines_and_accepts_completion_at_deadline() {
    let jobs = [Job::new(0, 5, 2, 5).unwrap(), Job::new(0, 3, 2, 7).unwrap()];
    let edf = schedule(&jobs, Policy::Edf, 5);
    assert_eq!(
        edf.slots,
        [
            Slot::Job(1),
            Slot::Job(1),
            Slot::Job(0),
            Slot::Job(0),
            Slot::Idle
        ]
    );
    assert!(edf.missed.is_empty());
    assert_eq!(schedule(&jobs, Policy::FixedPriority, 5).missed, [1]);

    let shifted = [Job::new(0, 4, 4, 0).unwrap(), Job::new(2, 5, 1, 0).unwrap()];
    assert_eq!(
        schedule(&shifted, Policy::Edf, 5).slots,
        [
            Slot::Job(0),
            Slot::Job(0),
            Slot::Job(0),
            Slot::Job(0),
            Slot::Job(1)
        ]
    );
    assert!(schedule(&shifted, Policy::Edf, 5).missed.is_empty());
    assert!(schedule(&shifted, Policy::Edf, 3).missed.is_empty());

    let preempt = [Job::new(1, 4, 2, 0).unwrap(), Job::new(2, 3, 1, 0).unwrap()];
    assert_eq!(
        schedule(&preempt, Policy::Edf, 4).slots,
        [Slot::Idle, Slot::Job(0), Slot::Job(1), Slot::Job(0)]
    );
}

fn enumerated_demand(tasks: [(u32, u32, u32); 2], length: u32) -> u128 {
    tasks
        .into_iter()
        .flat_map(|(cost, due, period)| {
            (0..length)
                .filter(move |&release| release % period == 0 && release + due <= length)
                .map(move |_| u128::from(cost))
        })
        .sum()
}

#[test]
fn demand_and_admission_agree_with_enumerated_periodic_jobs() {
    let mut menu = Vec::new();
    for period in 1..=5 {
        for due in 1..=period {
            for cost in 1..=2 {
                menu.push((cost, due, period));
            }
        }
    }
    let mut cases = 0;
    for &a in &menu {
        for &b in &menu {
            let specs = [a, b];
            let tasks = specs.map(|(c, d, p)| Task::new(c, d, p).unwrap());
            let hyperperiod = (1..=20).find(|h| h % a.2 == 0 && h % b.2 == 0).unwrap();
            let mut expected = Admission::Feasible { hyperperiod };
            for length in 0..=hyperperiod {
                let required = enumerated_demand(specs, length);
                assert_eq!(demand(&tasks, length), required);
                if required > u128::from(length) && matches!(expected, Admission::Feasible { .. }) {
                    expected = Admission::Overloaded { length, required };
                }
            }
            assert_eq!(admit(&tasks, 20), expected, "{specs:?}");
            cases += 1;
        }
    }
    assert_eq!(cases, 900);
}

#[test]
fn admission_distinguishes_overload_from_a_probe_limit() {
    let b = Task::new(2, 3, 7).unwrap();
    let tasks = [Task::new(2, 5, 5).unwrap(), b];
    assert_eq!(admit(&tasks, 35), Admission::Feasible { hyperperiod: 35 });
    assert_eq!(admit(&tasks, 34), Admission::Inconclusive { checked: 34 });
    let tight = [Task::new(2, 3, 5).unwrap(), b];
    assert_eq!(admit(&tight, 2), Admission::Inconclusive { checked: 2 });
    assert_eq!(
        admit(&tight, 3),
        Admission::Overloaded {
            length: 3,
            required: 4
        }
    );
    assert_eq!(admit(&[], 1), Admission::Feasible { hyperperiod: 1 });
    let overlarge = [
        Task::new(1, u32::MAX, u32::MAX).unwrap(),
        Task::new(1, u32::MAX - 1, u32::MAX - 1).unwrap(),
    ];
    assert_eq!(
        admit(&overlarge, 10),
        Admission::Inconclusive { checked: 10 }
    );
    assert_eq!(
        demand(&[Task::new(u32::MAX, 1, 1).unwrap(); 2], u32::MAX),
        2 * u128::from(u32::MAX).pow(2)
    );
    assert!(Task::new(1, 2, 1).is_none());
    assert!(Job::new(4, 4, 1, 0).is_none());
    assert!(Reservation::new(2, 3, 4, Refill::Immediate).is_none());
}

#[test]
fn budget_policy_preserves_deadline_order_and_refill_boundaries() {
    let hard = [0, 5, 10].map(|r| Job::new(r, r + 5, 2, 5).unwrap());
    for (refill, expected) in [
        (Refill::Immediate, [2, 3, 4, 7, 8]),
        (Refill::PeriodBoundary, [2, 3, 7, 8, 14]),
    ] {
        let reservation = Reservation::new(7, 2, 5, refill).unwrap();
        let trace = schedule_with_server(&hard, reservation, 15);
        let server_ticks: Vec<_> = trace
            .slots
            .iter()
            .enumerate()
            .filter_map(|(t, &slot)| (slot == Slot::Server).then_some(t))
            .collect();
        assert_eq!(server_ticks, expected);
        assert_eq!(trace.server_remaining, 0);
        assert!(trace.missed.is_empty());
    }
    // A hard-refill server is equivalent on this admitted workload to EDF
    // jobs released at fixed boundaries, each with the next boundary as its deadline.
    let mut combined = hard.to_vec();
    combined
        .extend([0, 7, 14].map(|r| Job::new(r, r + 7, if r == 14 { 1 } else { 2 }, 0).unwrap()));
    let oracle: Vec<_> = schedule(&combined, Policy::Edf, 15)
        .slots
        .into_iter()
        .map(|slot| match slot {
            Slot::Job(i) if i >= hard.len() => Slot::Server,
            other => other,
        })
        .collect();
    let reservation = Reservation::new(7, 2, 5, Refill::PeriodBoundary).unwrap();
    assert_eq!(schedule_with_server(&hard, reservation, 15).slots, oracle);
    assert_eq!(
        schedule_with_server(&hard, reservation, 14).server_remaining,
        1
    );
}
