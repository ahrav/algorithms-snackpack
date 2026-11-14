//! Sequential timer replay with replace-on-arm and never-early expiry.
//!
//! ```
//! use topic_049_timers::{replay, Candidate, Event};
//! let events = [Event::Arm(0, 5), Event::Arm(0, 9), Event::Advance(5), Event::Advance(9)];
//! let result = replay(Candidate::Heap, 1, &events).unwrap();
//! assert_eq!(result.fired, [(9, 9, 0)]);
//! ```

use std::{
    cmp::Reverse,
    collections::{BTreeSet, BinaryHeap},
};

/// Maximum supported clock/deadline, in integer ticks.
pub const MAX_TICK: u32 = 1_000_000;
/// Implemented candidates under the same contract.
#[derive(Clone, Copy, Debug)]
pub enum Candidate {
    /// Dense ID-array scan on each advance.
    Scan,
    /// Immutable records with lazy serial validation.
    Heap,
    /// Ordered live keys with immediate removal.
    Tree,
    /// 64 slots with absolute-deadline and serial checks.
    Wheel,
}
/// A sequential operation. Time starts at zero.
#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// Replace this ID's timer; deadline must be strictly in the future.
    Arm(usize, u32),
    /// Cancel this ID; absent cancellation is idempotent.
    Cancel(usize),
    /// Return all due timers; time must be nondecreasing and bounded.
    Advance(u32),
}
/// Deterministic mechanical counters, separate from timing.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Work {
    /// Scan state slots or records inspected during advances.
    pub inspections: usize,
    /// Wheel ticks crossed.
    pub ticks: usize,
    /// Largest physical queue record count, or scan ID-slot count.
    pub peak_records: usize,
}
/// Complete observable result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Replay {
    /// Expiries ordered by advance, then deadline, then ID: (poll, deadline, ID).
    pub fired: Vec<(u32, u32, usize)>,
    /// Final live (ID, deadline) records, ordered by ID.
    pub live: Vec<(usize, u32)>,
    /// Candidate-specific work counters.
    pub work: Work,
}

type Record = (u32, usize, usize);
type Live = Vec<Option<(u32, usize)>>;

/// Replay a validated, finite sequential workload.
///
/// Equal deadlines use ascending ID. Rearming replaces the prior arm; stale queue
/// records never fire a replacement. Every expiry appears once at the first
/// advance at or after its deadline. The full input is checked before execution.
///
/// # Errors
/// Returns an error for more than 65,536 IDs, an out-of-range ID, a backward or
/// oversized advance, or an arm outside `(current_time, MAX_TICK]`.
pub fn replay(candidate: Candidate, ids: usize, events: &[Event]) -> Result<Replay, &'static str> {
    if ids > 65_536 {
        return Err("ID capacity exceeds 65536");
    }
    let mut now = 0;
    for event in events {
        match *event {
            Event::Arm(id, d) => {
                if id >= ids {
                    return Err("ID outside capacity");
                }
                if d <= now || d > MAX_TICK {
                    return Err("deadline outside future horizon");
                }
            }
            Event::Cancel(id) => {
                if id >= ids {
                    return Err("ID outside capacity");
                }
            }
            Event::Advance(t) => {
                if t < now || t > MAX_TICK {
                    return Err("invalid clock advance");
                }
                now = t;
            }
        }
    }
    Ok(match candidate {
        Candidate::Scan => scan(ids, events),
        Candidate::Heap => heap(ids, events),
        Candidate::Tree => tree(ids, events),
        Candidate::Wheel => wheel(ids, events),
    })
}

fn finish(live: Live, fired: Vec<(u32, u32, usize)>, work: Work) -> Replay {
    Replay {
        fired,
        live: live
            .into_iter()
            .enumerate()
            .filter_map(|(id, x)| x.map(|(d, _)| (id, d)))
            .collect(),
        work,
    }
}

fn scan(ids: usize, events: &[Event]) -> Replay {
    let mut live = vec![None; ids];
    let mut fired = Vec::new();
    let mut work = Work {
        peak_records: ids,
        ..Work::default()
    };
    for (serial, event) in events.iter().enumerate() {
        match *event {
            Event::Arm(id, d) => live[id] = Some((d, serial)),
            Event::Cancel(id) => live[id] = None,
            Event::Advance(t) => {
                let start = fired.len();
                for (id, value) in live.iter_mut().enumerate() {
                    work.inspections += 1;
                    if let Some((d, _)) = *value
                        && d <= t
                    {
                        fired.push((t, d, id));
                        *value = None;
                    }
                }
                fired[start..].sort_unstable();
            }
        }
    }
    finish(live, fired, work)
}

fn heap(ids: usize, events: &[Event]) -> Replay {
    let mut live = vec![None; ids];
    let mut queue = BinaryHeap::<Reverse<Record>>::new();
    let mut fired = Vec::new();
    let mut work = Work::default();
    for (serial, event) in events.iter().enumerate() {
        match *event {
            Event::Arm(id, d) => {
                live[id] = Some((d, serial));
                queue.push(Reverse((d, id, serial)));
                work.peak_records = work.peak_records.max(queue.len());
            }
            Event::Cancel(id) => live[id] = None,
            Event::Advance(t) => {
                while let Some(&Reverse((d, id, s))) = queue.peek() {
                    if d > t {
                        break;
                    }
                    queue.pop();
                    work.inspections += 1;
                    if live[id] == Some((d, s)) {
                        fired.push((t, d, id));
                        live[id] = None;
                    }
                }
            }
        }
    }
    finish(live, fired, work)
}

fn tree(ids: usize, events: &[Event]) -> Replay {
    let mut live = vec![None; ids];
    let mut queue = BTreeSet::new();
    let mut fired = Vec::new();
    let mut work = Work::default();
    for (serial, event) in events.iter().enumerate() {
        match *event {
            Event::Arm(id, d) => {
                if let Some((old, _)) = live[id] {
                    queue.remove(&(old, id));
                }
                live[id] = Some((d, serial));
                queue.insert((d, id));
                work.peak_records = work.peak_records.max(queue.len());
            }
            Event::Cancel(id) => {
                if let Some((old, _)) = live[id].take() {
                    queue.remove(&(old, id));
                }
            }
            Event::Advance(t) => {
                while let Some(&(d, id)) = queue.first() {
                    if d > t {
                        break;
                    }
                    queue.pop_first();
                    work.inspections += 1;
                    fired.push((t, d, id));
                    live[id] = None;
                }
            }
        }
    }
    finish(live, fired, work)
}

fn wheel(ids: usize, events: &[Event]) -> Replay {
    let mut live = vec![None; ids];
    let mut buckets: [Vec<Record>; 64] = std::array::from_fn(|_| Vec::new());
    let mut fired = Vec::new();
    let mut work = Work::default();
    let mut now = 0;
    let mut records = 0;
    for (serial, event) in events.iter().enumerate() {
        match *event {
            Event::Arm(id, d) => {
                live[id] = Some((d, serial));
                buckets[(d % 64) as usize].push((d, id, serial));
                records += 1;
                work.peak_records = work.peak_records.max(records);
            }
            Event::Cancel(id) => live[id] = None,
            Event::Advance(t) => {
                let start = fired.len();
                while now < t {
                    now += 1;
                    work.ticks += 1;
                    buckets[(now % 64) as usize].retain(|&(d, id, s)| {
                        work.inspections += 1;
                        if live[id] != Some((d, s)) {
                            records -= 1;
                            return false;
                        }
                        if d <= now {
                            fired.push((t, d, id));
                            live[id] = None;
                            records -= 1;
                            return false;
                        }
                        true
                    });
                }
                fired[start..].sort_unstable();
            }
        }
    }
    finish(live, fired, work)
}

/// Deterministic workload shared by the runnable example and smoke benchmark.
///
/// # Errors
/// Rejects unknown shapes, zero/oversized ID counts and cancellation percentages
/// other than 0 or 90. Shapes are `near`, `far`, and `burst`.
pub fn workload(ids: usize, shape: &str, cancel: usize) -> Result<Vec<Event>, &'static str> {
    if ids == 0 || ids > 65_536 || ![0, 90].contains(&cancel) {
        return Err("invalid workload dimensions");
    }
    let horizon = match shape {
        "near" => 256,
        "far" | "burst" => 65_536,
        _ => return Err("unknown shape"),
    };
    let deadline = |id: usize, offset: usize| -> u32 {
        if shape == "burst" {
            horizon
        } else {
            1 + u32::try_from((id * 73 + offset) % horizon as usize).unwrap_or(0)
        }
    };
    let mut events = Vec::new();
    for id in 0..ids {
        events.push(Event::Arm(id, deadline(id, 0)));
    }
    for id in (0..ids).step_by(4) {
        events.push(Event::Arm(id, deadline(id, 17)));
    }
    for id in 0..ids {
        if id % 10 < cancel / 10 {
            events.push(Event::Cancel(id));
        }
    }
    if shape == "burst" {
        events.push(Event::Advance(horizon));
    } else {
        for i in 1..=256 {
            events.push(Event::Advance(i * (horizon / 256)));
        }
    }
    Ok(events)
}
