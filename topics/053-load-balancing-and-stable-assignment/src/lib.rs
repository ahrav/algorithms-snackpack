//! Deterministic replay of one dispatcher and two equal-sized backend zones.

/// Routing policy for a fixed set of eligible, homogeneous backends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    /// Rotate through all backend positions.
    RoundRobin,
    /// Use the first sample token alone.
    One,
    /// Sample two positions and prefer less unfinished work.
    Two,
    /// Inspect every backend's unfinished work.
    Scan,
    /// Assign repeat keys by highest score on stable IDs.
    Rendezvous,
    /// Sample two local positions and probe two remote positions when useful.
    Local,
}

/// One arrival. Sample tokens are supplied by the caller.
#[derive(Clone, Copy, Debug)]
pub struct Request {
    /// Nondecreasing arrival time in simulator ticks.
    pub at: u64,
    /// Positive service time in ticks, identical at each backend for this request.
    pub service: u64,
    /// Stable affinity key.
    pub key: u64,
    /// Source zone, zero or one.
    pub zone: usize,
    /// Two sample tokens reduced into the selected pool with replacement.
    pub sample: [usize; 2],
}

/// Timing and placement of one request.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Dispatch {
    /// Backend ID in the fixed fleet.
    pub backend: usize,
    /// Time spent waiting for earlier service.
    pub wait: u64,
    /// Total modeled response time including service and network penalty.
    pub response: u64,
}

/// State of a single dispatcher with exact visibility into its fleet's work.
pub struct Fleet {
    finish: Vec<u64>,
    ids: Vec<usize>,
    half: usize,
    penalty: u64,
    at: u64,
    next: usize,
}

impl Fleet {
    /// Create two equal zones with contiguous IDs and the given remote penalty.
    ///
    /// # Panics
    ///
    /// Panics if the backend count is zero or odd.
    #[must_use]
    pub fn new(backends: usize, penalty: u64) -> Self {
        assert!(backends != 0 && backends.is_multiple_of(2));
        Self {
            finish: vec![0; backends],
            ids: (0..backends).collect(),
            half: backends / 2,
            penalty,
            at: 0,
            next: 0,
        }
    }

    /// Reset queue tails and round-robin position without reallocating.
    pub fn reset(&mut self) {
        self.finish.fill(0);
        self.at = 0;
        self.next = 0;
    }

    /// Select a backend, append the request, and return its modeled response.
    ///
    /// Equal sampled waits prefer the first draw. A scan tie prefers the lowest
    /// ID. A locality tie stays in the source zone.
    ///
    /// # Panics
    ///
    /// Panics for decreasing arrivals, zero service time, an invalid zone, or
    /// arithmetic overflow in the modeled times.
    pub fn dispatch(&mut self, policy: Policy, request: Request) -> Dispatch {
        assert!(request.at >= self.at && request.service > 0 && request.zone < 2);
        let n = self.finish.len();
        let wait = |id: usize| self.finish[id].saturating_sub(request.at);
        let [a, b] = request.sample;
        let choose = |first, second| {
            if wait(first) <= wait(second) {
                first
            } else {
                second
            }
        };
        let backend = match policy {
            Policy::RoundRobin => self.next,
            Policy::One => a % n,
            Policy::Two => choose(a % n, b % n),
            Policy::Scan => {
                let mut best = 0;
                let mut least = wait(0);
                for id in 1..n {
                    let candidate = wait(id);
                    if candidate < least {
                        best = id;
                        least = candidate;
                    }
                }
                best
            }
            Policy::Rendezvous => rendezvous(request.key, &self.ids).expect("nonempty fleet"),
            Policy::Local => {
                let offset = request.zone * self.half;
                let near = choose(offset + a % self.half, offset + b % self.half);
                let near_wait = wait(near);
                if near_wait <= self.penalty {
                    near
                } else {
                    let other = self.half - offset;
                    let far = choose(other + a % self.half, other + b % self.half);
                    if wait(far) < near_wait - self.penalty {
                        far
                    } else {
                        near
                    }
                }
            }
        };
        let wait = wait(backend);
        let finish = self.finish[backend]
            .max(request.at)
            .checked_add(request.service)
            .expect("finish time overflow");
        let network = if backend / self.half == request.zone {
            0
        } else {
            self.penalty
        };
        let response = (finish - request.at)
            .checked_add(network)
            .expect("response time overflow");
        self.finish[backend] = finish;
        self.at = request.at;
        if policy == Policy::RoundRobin {
            self.next += 1;
            if self.next == n {
                self.next = 0;
            }
        }
        Dispatch {
            backend,
            wait,
            response,
        }
    }
}

/// Return the highest-scoring stable ID, breaking score ties by lowest ID.
///
/// IDs identify backends independently of their position in the supplied slice.
/// An empty view has no assignment. The mixer serves this reproducible exercise.
#[must_use]
pub fn rendezvous(key: u64, ids: &[usize]) -> Option<usize> {
    ids.iter().copied().max_by_key(|&id| {
        let mut score = key ^ (id as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        score = (score ^ (score >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        score = (score ^ (score >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        (score ^ (score >> 31), std::cmp::Reverse(id))
    })
}
