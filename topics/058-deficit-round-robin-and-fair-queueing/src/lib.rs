//! Byte service and queue feedback on one deterministic, non-preemptive link.

use std::{
    cmp::{Ordering, Reverse},
    collections::{BinaryHeap, VecDeque},
};

/// A batch of packets that all arrive at time zero in FIFO order.
pub struct Flow<'a> {
    /// Positive packet lengths in bytes.
    pub lengths: &'a [u32],
    /// Positive relative bandwidth weight.
    pub weight: u32,
}

/// A packet selection policy for simultaneous arrivals.
#[derive(Clone, Copy)]
pub enum Policy {
    /// One packet per active flow visit.
    PacketRoundRobin,
    /// Byte credit per visit, proportional to the flow weight.
    DeficitRoundRobin {
        /// Positive quantum in bytes for a flow of weight one.
        quantum: u32,
    },
    /// Earliest ideal fluid finish tag, with ties by flow index.
    WeightedFairQueueing,
}

#[derive(Eq, PartialEq)]
struct Head {
    bytes: u64,
    weight: u32,
    flow: usize,
    packet: usize,
}

impl Ord for Head {
    fn cmp(&self, other: &Self) -> Ordering {
        // Cross products compare the rational finish tags exactly.
        (u128::from(self.bytes) * u128::from(other.weight))
            .cmp(&(u128::from(other.bytes) * u128::from(self.weight)))
            .then_with(|| self.flow.cmp(&other.flow))
            .then_with(|| self.packet.cmp(&other.packet))
            .then_with(|| self.weight.cmp(&other.weight))
    }
}

impl PartialOrd for Head {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Validate a batch once before replaying it under several policies.
///
/// # Panics
///
/// Panics for zero lengths, zero weights, or a flow totaling over `u64::MAX` bytes.
pub fn validate(flows: &[Flow<'_>]) {
    for flow in flows {
        assert!(flow.weight > 0);
        let mut total = 0_u64;
        for &length in flow.lengths {
            assert!(length > 0);
            total = total
                .checked_add(u64::from(length))
                .expect("flow bytes overflow");
        }
    }
}

/// Emit each packet in service order as `(flow index, packet index)`.
///
/// Returns the number of flow visits for ring schedulers, or heap selections
/// for weighted fair queueing. Call [`validate`] once per input batch.
///
/// # Panics
///
/// Panics for a zero deficit quantum. Invalid batches may also panic.
pub fn dispatch(flows: &[Flow<'_>], policy: Policy, mut emit: impl FnMut(usize, usize)) -> usize {
    if let Policy::WeightedFairQueueing = policy {
        return dispatch_by_finish(flows, emit);
    }
    if let Policy::DeficitRoundRobin { quantum } = policy {
        assert!(quantum > 0);
    }
    let mut active: VecDeque<_> = flows
        .iter()
        .enumerate()
        .filter(|(_, flow)| !flow.lengths.is_empty())
        .map(|(flow, _)| (flow, 0, 0_u64))
        .collect();
    let mut visits = 0;
    while let Some((flow, mut packet, mut credit)) = active.pop_front() {
        visits += 1;
        let queue = &flows[flow];
        match policy {
            Policy::PacketRoundRobin => {
                emit(flow, packet);
                packet += 1;
            }
            Policy::DeficitRoundRobin { quantum } => {
                credit += u64::from(quantum) * u64::from(queue.weight);
                while let Some(&length) = queue.lengths.get(packet) {
                    if u64::from(length) > credit {
                        break;
                    }
                    credit -= u64::from(length);
                    emit(flow, packet);
                    packet += 1;
                }
            }
            Policy::WeightedFairQueueing => unreachable!(),
        }
        if packet < queue.lengths.len() {
            active.push_back((flow, packet, credit));
        }
    }
    visits
}

fn dispatch_by_finish(flows: &[Flow<'_>], mut emit: impl FnMut(usize, usize)) -> usize {
    let heads: Vec<_> = flows
        .iter()
        .enumerate()
        .filter_map(|(flow, queue)| {
            Some(Reverse(Head {
                bytes: u64::from(*queue.lengths.first()?),
                weight: queue.weight,
                flow,
                packet: 0,
            }))
        })
        .collect();
    let mut active = BinaryHeap::from(heads);
    let mut selections = 0;
    while let Some(Reverse(mut head)) = active.pop() {
        selections += 1;
        emit(head.flow, head.packet);
        head.packet += 1;
        if let Some(&length) = flows[head.flow].lengths.get(head.packet) {
            head.bytes += u64::from(length);
            active.push(Reverse(head));
        }
    }
    selections
}

/// A packet offered to the finite FIFO link.
#[derive(Clone, Copy)]
pub struct Arrival {
    /// Offer time in ticks.
    pub tick: u64,
    /// Positive packet length. The link serves one byte per tick.
    pub bytes: u32,
}

/// A teaching controller that marks after a run of high-sojourn dequeues.
#[derive(Clone, Copy)]
pub struct PersistentMark {
    /// Positive sojourn threshold in ticks, inclusive.
    pub threshold: u64,
    /// Minimum ticks since the first high observation, inclusive.
    pub interval: u64,
}

/// The terminal outcome for one offered packet.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// The queued-byte capacity could not fit the packet.
    TailDrop,
    /// The packet began transmission at the given tick.
    Sent {
        /// The tick at which the packet left the waiting queue.
        start: u64,
        /// Whether the controller marked this packet.
        marked: bool,
    },
}

/// Replay ordered arrivals with a waiting-byte cap and optional persistent marks.
///
/// Arrivals at a service-start tick join before selection. Capacity excludes
/// the packet in service. Marked packets still transmit. The open-loop sender
/// keeps the offered arrivals fixed. Outcomes are indexed by arrival order.
///
/// # Panics
///
/// Panics for unordered arrivals, zero lengths, zero threshold, or clock overflow.
#[must_use]
pub fn replay_fifo(
    arrivals: &[Arrival],
    capacity: u64,
    mark: Option<PersistentMark>,
) -> Vec<Outcome> {
    assert!(arrivals.windows(2).all(|pair| pair[0].tick <= pair[1].tick));
    assert!(arrivals.iter().all(|arrival| arrival.bytes > 0));
    assert!(mark.is_none_or(|policy| policy.threshold > 0));

    let mut outcomes = vec![Outcome::TailDrop; arrivals.len()];
    let mut waiting = VecDeque::new();
    let mut queued_bytes = 0;
    let mut input = 0;
    let mut now = 0;
    let mut high_since = None;
    while input < arrivals.len() || !waiting.is_empty() {
        if waiting.is_empty() && arrivals[input].tick > now {
            now = arrivals[input].tick;
            high_since = None;
        }
        while let Some(arrival) = arrivals.get(input).filter(|a| a.tick <= now) {
            let bytes = u64::from(arrival.bytes);
            if bytes <= capacity - queued_bytes {
                waiting.push_back(input);
                queued_bytes += bytes;
            }
            input += 1;
        }
        if let Some(index) = waiting.pop_front() {
            let arrival = arrivals[index];
            let bytes = u64::from(arrival.bytes);
            queued_bytes -= bytes;
            let marked = mark.is_some_and(|policy| {
                if now - arrival.tick < policy.threshold {
                    high_since = None;
                    return false;
                }
                let first = *high_since.get_or_insert(now);
                now - first >= policy.interval
            });
            outcomes[index] = Outcome::Sent { start: now, marked };
            now = now.checked_add(bytes).expect("link clock overflow");
        }
    }
    outcomes
}
