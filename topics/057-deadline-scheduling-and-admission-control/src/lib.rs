//! Single-processor deadline dispatch, periodic demand, and budget replenishment.

/// A ready-to-run job has a release, absolute deadline, execution cost, and fixed priority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Job {
    release: u32,
    deadline: u32,
    cost: u32,
    priority: u32,
}

impl Job {
    /// Builds a positive-cost job with a deadline strictly after its release.
    /// Smaller priority values run first under fixed priority.
    #[must_use]
    pub fn new(release: u32, deadline: u32, cost: u32, priority: u32) -> Option<Self> {
        (release < deadline && cost != 0).then_some(Self {
            release,
            deadline,
            cost,
            priority,
        })
    }
}

/// A dispatch rule for independent jobs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    /// Earliest absolute deadline first.
    Edf,
    /// Smallest declared fixed priority first.
    FixedPriority,
}

/// An implicit-deadline server whose request arrives at zero and never blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reservation {
    period: u32,
    budget: u32,
    work: u32,
    refill: Refill,
}

impl Reservation {
    /// Builds a server with a positive budget no greater than its period.
    #[must_use]
    pub fn new(period: u32, budget: u32, work: u32, refill: Refill) -> Option<Self> {
        (budget != 0 && budget <= period).then_some(Self {
            period,
            budget,
            work,
            refill,
        })
    }
}

/// A reservation's budget exhaustion rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refill {
    /// Refill immediately and move the server deadline ahead by a period.
    Immediate,
    /// Hold an exhausted server until its next fixed period boundary.
    PeriodBoundary,
}

/// Work executed in one unit-length slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    /// No runnable work.
    Idle,
    /// Job at the given input position.
    Job(usize),
    /// The reservation's request.
    Server,
}

/// A finite simulation result.
#[derive(Debug, PartialEq, Eq)]
pub struct Trace {
    /// One action per tick up to the horizon.
    pub slots: Vec<Slot>,
    /// Job positions with unsatisfied deadlines at or before the horizon.
    pub missed: Vec<usize>,
    /// Remaining units of reserved work.
    pub server_remaining: u32,
}

/// Simulates independent integer-time jobs. Late jobs remain eligible.
/// Input order breaks dispatch ties. The horizon limits time and trace allocation.
#[must_use]
pub fn schedule(jobs: &[Job], policy: Policy, horizon: u32) -> Trace {
    simulate(jobs, policy, horizon, None)
}

/// Simulates EDF with an implicit-deadline server. An ordinary job wins a deadline tie.
/// The boundary variant expires any unused budget at the end of its fixed period.
#[must_use]
pub fn schedule_with_server(jobs: &[Job], reservation: Reservation, horizon: u32) -> Trace {
    simulate(
        jobs,
        Policy::Edf,
        horizon,
        Some(Server {
            reservation,
            deadline: u64::from(reservation.period),
            budget: reservation.budget,
        }),
    )
}

struct Server {
    reservation: Reservation,
    deadline: u64,
    budget: u32,
}

impl Server {
    fn advance_period(&mut self) {
        self.deadline += u64::from(self.reservation.period);
        self.budget = self.reservation.budget;
    }

    fn eligible(&self) -> bool {
        self.reservation.work > 0 && self.budget > 0
    }

    fn spend(&mut self) {
        self.reservation.work -= 1;
        self.budget -= 1;
        if self.budget == 0 && self.reservation.refill == Refill::Immediate {
            self.advance_period();
        }
    }
}

fn simulate(jobs: &[Job], policy: Policy, horizon: u32, mut server: Option<Server>) -> Trace {
    let mut remaining: Vec<_> = jobs.iter().map(|job| job.cost).collect();
    let mut finished = vec![None; jobs.len()];
    let mut slots = Vec::with_capacity(horizon as usize);
    for now in 0..horizon {
        let ready = jobs
            .iter()
            .enumerate()
            .filter(|&(i, job)| job.release <= now && remaining[i] > 0)
            .min_by_key(|&(i, job)| {
                let priority = match policy {
                    Policy::Edf => job.deadline,
                    Policy::FixedPriority => job.priority,
                };
                (priority, i)
            });

        if let Some(server) = &mut server {
            if server.reservation.refill == Refill::PeriodBoundary
                && server.deadline == u64::from(now)
            {
                server.advance_period();
            }
            if server.eligible()
                && ready.is_none_or(|(_, job)| server.deadline < u64::from(job.deadline))
            {
                server.spend();
                slots.push(Slot::Server);
                continue;
            }
        }

        if let Some((i, _)) = ready {
            remaining[i] -= 1;
            if remaining[i] == 0 {
                finished[i] = Some(now + 1);
            }
            slots.push(Slot::Job(i));
        } else {
            slots.push(Slot::Idle);
        }
    }

    let missed = jobs
        .iter()
        .enumerate()
        .filter(|&(i, job)| job.deadline <= horizon && finished[i].is_none_or(|t| t > job.deadline))
        .map(|(i, _)| i)
        .collect();
    Trace {
        slots,
        missed,
        server_remaining: server.map_or(0, |server| server.reservation.work),
    }
}

/// A periodic or sporadic task with a constrained relative deadline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Task {
    cost: u32,
    deadline: u32,
    period: u32,
}

impl Task {
    /// Builds a positive-cost task with `0 < deadline <= period`.
    /// A cost larger than the deadline is representable and yields an overload witness.
    #[must_use]
    pub fn new(cost: u32, deadline: u32, period: u32) -> Option<Self> {
        (cost != 0 && deadline != 0 && deadline <= period).then_some(Self {
            cost,
            deadline,
            period,
        })
    }
}

/// Worst-case total execution due within an interval of the given length.
#[must_use]
pub fn demand(tasks: &[Task], length: u32) -> u128 {
    tasks
        .iter()
        .filter(|task| task.deadline <= length)
        .map(|task| {
            let jobs = 1 + (length - task.deadline) / task.period;
            u128::from(jobs) * u128::from(task.cost)
        })
        .sum()
}

/// Result of a bounded exact admission check.
#[derive(Debug, PartialEq, Eq)]
pub enum Admission {
    /// Every deadline is feasible under the stated task model.
    Feasible {
        /// The checked common period.
        hyperperiod: u32,
    },
    /// A window requires more execution than fits.
    Overloaded {
        /// Window length in ticks.
        length: u32,
        /// Execution ticks required within that window.
        required: u128,
    },
    /// All checked windows pass. The hyperperiod exceeds the probe limit.
    Inconclusive {
        /// Last window length checked.
        checked: u32,
    },
}

/// Checks all integer window lengths up to the smaller of the cap and hyperperiod.
/// A passing hyperperiod proves feasibility for independent, preemptible tasks on
/// one unit-speed CPU. A failed window proves overload even under a smaller cap.
#[must_use]
pub fn admit(tasks: &[Task], cap: u32) -> Admission {
    let hyperperiod = tasks
        .iter()
        .try_fold(1, |current, task| {
            let factor = task.period / gcd(current, task.period);
            current.checked_mul(factor).filter(|&period| period <= cap)
        })
        .filter(|&period| period <= cap);
    let limit = hyperperiod.unwrap_or(cap);
    for length in 1..=limit {
        let required = demand(tasks, length);
        if required > u128::from(length) {
            return Admission::Overloaded { length, required };
        }
    }
    match hyperperiod {
        Some(hyperperiod) => Admission::Feasible { hyperperiod },
        None => Admission::Inconclusive { checked: cap },
    }
}

fn gcd(mut left: u32, mut right: u32) -> u32 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left
}
