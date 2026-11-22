//! Snapshot algorithms for mandatory waits and reusable resource pools.

use std::collections::VecDeque;

/// A malformed graph, resource snapshot, or recovery policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputError {
    /// A wait edge names a task outside the graph.
    Endpoint,
    /// Resource vectors have different widths or task counts.
    Shape,
    /// Existing allocations exceed capacity.
    Capacity,
    /// The task's own allocation plus its request exceeds capacity.
    ImpossibleRequest,
    /// The recovery policy lacks one cost per task.
    Costs,
}

/// Result of peeling mandatory wait edges.
#[derive(Debug, Eq, PartialEq)]
pub struct WaitReport {
    /// Tasks that can finish, in dependency-first order.
    pub order: Vec<usize>,
    /// Tasks with an unresolved mandatory wait, sorted by task ID.
    pub blocked: Vec<usize>,
    /// One directed cycle, with its first task repeated at the end.
    pub cycle: Option<Vec<usize>>,
}

/// Peel mandatory edges from a single coherent snapshot.
///
/// `waits[task]` lists tasks whose completion is required. When a task finishes
/// it releases all such dependencies. Parallel edges and self-waits are allowed.
///
/// # Errors
///
/// Returns [`InputError::Endpoint`] if an edge names a task outside the graph.
pub fn mandatory_waits(waits: &[Vec<usize>]) -> Result<WaitReport, InputError> {
    let mut dependents = vec![Vec::new(); waits.len()];
    let mut counts = Vec::with_capacity(waits.len());
    let mut ready = VecDeque::new();
    for (task, edges) in waits.iter().enumerate() {
        counts.push(edges.len());
        if edges.is_empty() {
            ready.push_back(task);
        }
        for &dependency in edges {
            dependents
                .get_mut(dependency)
                .ok_or(InputError::Endpoint)?
                .push(task);
        }
    }

    let mut order = Vec::with_capacity(waits.len());
    while let Some(finished) = ready.pop_front() {
        order.push(finished);
        for &task in &dependents[finished] {
            counts[task] -= 1;
            if counts[task] == 0 {
                ready.push_back(task);
            }
        }
    }
    let blocked: Vec<_> = (0..waits.len()).filter(|&task| counts[task] > 0).collect();
    let cycle = blocked
        .first()
        .map(|&task| residual_cycle(waits, &counts, task));
    Ok(WaitReport {
        order,
        blocked,
        cycle,
    })
}

fn residual_cycle(waits: &[Vec<usize>], counts: &[usize], mut task: usize) -> Vec<usize> {
    let mut seen = vec![None; waits.len()];
    let mut path = Vec::new();
    while seen[task].is_none() {
        seen[task] = Some(path.len());
        path.push(task);
        task = *waits[task]
            .iter()
            .find(|&&dependency| counts[dependency] > 0)
            .expect("every residual task has a residual dependency");
    }
    path.push(task);
    path[seen[task].expect("task appears in path")..].to_vec()
}

/// Fixed capacities, allocations, and outstanding requests.
#[derive(Debug)]
pub struct Snapshot<'a> {
    held: &'a [Vec<u32>],
    requested: &'a [Vec<u32>],
    free: Vec<u32>,
}

/// Result of simulating completion and release.
#[derive(Debug, Eq, PartialEq)]
pub struct Reduction {
    /// Tasks that can finish, in simulated order.
    pub order: Vec<usize>,
    /// Tasks still waiting, sorted by task ID, including zero-allocation waiters.
    pub stalled: Vec<usize>,
    /// Available units at the end of the simulation.
    pub free: Vec<u32>,
}

/// A single rollback that allows every other task to finish.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Victim {
    /// ID of the task to roll back.
    pub task: usize,
    /// Cost supplied by the recovery policy.
    pub cost: u64,
}

impl<'a> Snapshot<'a> {
    /// Validate a snapshot and derive free units from the allocated units.
    ///
    /// The model grants each task's whole current request before it can finish.
    /// A finish or rollback releases that task's allocation. Inputs describe
    /// reusable, interchangeable units within each resource class.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::Shape`] on mismatched dimensions,
    /// [`InputError::Capacity`] on over-allocation, and
    /// [`InputError::ImpossibleRequest`] when a task's request exceeds the
    /// capacity outside its own holdings.
    pub fn new(
        capacity: &[u32],
        held: &'a [Vec<u32>],
        requested: &'a [Vec<u32>],
    ) -> Result<Self, InputError> {
        if held.len() != requested.len()
            || held
                .iter()
                .chain(requested)
                .any(|row| row.len() != capacity.len())
        {
            return Err(InputError::Shape);
        }
        let mut free = capacity.to_vec();
        for allocation in held {
            for (available, &units) in free.iter_mut().zip(allocation) {
                *available = available.checked_sub(units).ok_or(InputError::Capacity)?;
            }
        }
        for (allocation, request) in held.iter().zip(requested) {
            if capacity
                .iter()
                .zip(allocation)
                .zip(request)
                .any(|((&total, &own), &need)| need > total - own)
            {
                return Err(InputError::ImpossibleRequest);
            }
        }
        Ok(Self {
            held,
            requested,
            free,
        })
    }

    /// Repeatedly grant each eligible request and release its existing holdings.
    #[must_use]
    pub fn reduce(&self) -> Reduction {
        self.reduce_after(None)
    }

    /// Return the cheapest feasible single rollback, breaking ties by task ID.
    ///
    /// Returns `None` if all tasks can finish or every single rollback leaves
    /// at least one stalled task. Multi-victim cost optimization is a separate
    /// problem. This function simulates a decision on an immutable snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::Costs`] if the policy has a different task count.
    pub fn cheapest_single_victim(&self, costs: &[u64]) -> Result<Option<Victim>, InputError> {
        if costs.len() != self.held.len() {
            return Err(InputError::Costs);
        }

        let mut best = None;
        for task in self.reduce().stalled {
            let cost = costs[task];
            if best.is_some_and(|victim: Victim| victim.cost <= cost) {
                continue;
            }
            if self.reduce_after(Some(task)).stalled.is_empty() {
                best = Some(Victim { task, cost });
            }
        }
        Ok(best)
    }

    fn reduce_after(&self, rollback: Option<usize>) -> Reduction {
        let mut free = self.free.clone();
        let mut finished = vec![false; self.held.len()];
        let mut remaining = self.held.len();
        if let Some(task) = rollback {
            finished[task] = true;
            remaining -= 1;
            for (available, &release) in free.iter_mut().zip(&self.held[task]) {
                *available += release;
            }
        }

        let mut order = Vec::with_capacity(remaining);
        while remaining > 0 {
            let previous_len = order.len();
            for (task, done) in finished.iter_mut().enumerate() {
                if *done
                    || self.requested[task]
                        .iter()
                        .zip(&free)
                        .any(|(need, have)| need > have)
                {
                    continue;
                }
                for (available, &release) in free.iter_mut().zip(&self.held[task]) {
                    *available += release;
                }
                *done = true;
                remaining -= 1;
                order.push(task);
            }
            if order.len() == previous_len {
                break;
            }
        }
        let stalled = (0..finished.len())
            .filter(|&task| !finished[task])
            .collect();
        Reduction {
            order,
            stalled,
            free,
        }
    }
}
