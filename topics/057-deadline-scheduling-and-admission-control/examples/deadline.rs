//! The two-task feasibility and five-tick reservation example.

use deadline_scheduling_and_admission_control::{
    Job, Policy, Refill, Reservation, Task, admit, schedule, schedule_with_server,
};

fn main() {
    let jobs = [Job::new(0, 5, 2, 5).unwrap(), Job::new(0, 3, 2, 7).unwrap()];
    for policy in [Policy::Edf, Policy::FixedPriority] {
        let trace = schedule(&jobs, policy, 5);
        println!("{policy:?}: {:?}, missed {:?}", trace.slots, trace.missed);
    }
    for a_deadline in [5, 3] {
        let tasks = [
            Task::new(2, a_deadline, 5).unwrap(),
            Task::new(2, 3, 7).unwrap(),
        ];
        println!("A deadline {a_deadline}: {:?}", admit(&tasks, 35));
    }

    let hard = [0, 5, 10].map(|r| Job::new(r, r + 5, 2, 5).unwrap());
    for refill in [Refill::Immediate, Refill::PeriodBoundary] {
        let reservation = Reservation::new(7, 2, 5, refill).unwrap();
        let trace = schedule_with_server(&hard, reservation, 15);
        let completion = trace
            .slots
            .iter()
            .rposition(|slot| *slot == deadline_scheduling_and_admission_control::Slot::Server)
            .unwrap()
            + 1;
        println!(
            "{refill:?}: server completed at {completion}, missed {:?}",
            trace.missed
        );
    }
}
