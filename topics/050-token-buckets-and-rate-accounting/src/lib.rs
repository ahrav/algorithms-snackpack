//! Serial strict rate accounting with integer credits and ticks.
//! Each flow starts full at tick zero. Backward time preserves state.
#![forbid(unsafe_code)]

/// Admission result and remaining credit.
pub type Decision = Result<(bool, u64), &'static str>;

/// A serial rate meter. The caller provides synchronization.
pub trait Meter {
    /// Start full with `capacity` credits and `rate` credits per tick.
    fn new(capacity: u64, rate: u64) -> Self;
    /// Refill and charge an attempt.
    ///
    /// # Errors
    /// Returns an error for backward time, preserving the state.
    fn admit(&mut self, now: u64, cost: u64) -> Decision;
}

/// Direct wide-intermediate credit accounting.
pub struct Wide {
    capacity: u64,
    rate: u64,
    credit: u64,
    last: u64,
}
impl Meter for Wide {
    fn new(capacity: u64, rate: u64) -> Self {
        Self {
            capacity,
            rate,
            credit: capacity,
            last: 0,
        }
    }
    fn admit(&mut self, now: u64, cost: u64) -> Decision {
        let elapsed = now.checked_sub(self.last).ok_or("backward time")?;
        let ready = u128::from(self.credit) + u128::from(elapsed) * u128::from(self.rate);
        self.credit =
            u64::try_from(ready.min(u128::from(self.capacity))).map_err(|_| "credit range")?;
        self.last = now;
        let accepted = cost <= self.credit;
        if accepted {
            self.credit -= cost;
        }
        Ok((accepted, self.credit))
    }
}

/// Narrow arithmetic guarded by the time needed to fill the credit gap.
pub struct Bounded {
    capacity: u64,
    rate: u64,
    credit: u64,
    last: u64,
}
impl Meter for Bounded {
    fn new(capacity: u64, rate: u64) -> Self {
        Self {
            capacity,
            rate,
            credit: capacity,
            last: 0,
        }
    }
    fn admit(&mut self, now: u64, cost: u64) -> Decision {
        let elapsed = now.checked_sub(self.last).ok_or("backward time")?;
        let gap = self.capacity - self.credit;
        if let Some(whole) = gap.checked_div(self.rate) {
            let fill_ticks = whole + u64::from(!gap.is_multiple_of(self.rate));
            if elapsed >= fill_ticks {
                self.credit = self.capacity;
            } else {
                self.credit += elapsed * self.rate;
            }
        }
        self.last = now;
        let accepted = cost <= self.credit;
        if accepted {
            self.credit -= cost;
        }
        Ok((accepted, self.credit))
    }
}

/// Algebraically equivalent virtual debt in credit-scaled time.
pub struct Virtual {
    capacity: u64,
    rate: u64,
    last: u64,
    finish: u128,
}
impl Meter for Virtual {
    fn new(capacity: u64, rate: u64) -> Self {
        Self {
            capacity,
            rate,
            last: 0,
            finish: 0,
        }
    }
    fn admit(&mut self, now: u64, cost: u64) -> Decision {
        if now < self.last {
            return Err("backward time");
        }
        let scaled = u128::from(now) * u128::from(self.rate);
        let start = self.finish.max(scaled);
        let accepted = start + u128::from(cost) <= scaled + u128::from(self.capacity);
        self.finish = start + if accepted { u128::from(cost) } else { 0 };
        self.last = now;
        let credit = u128::from(self.capacity) - (self.finish - scaled);
        Ok((accepted, u64::try_from(credit).map_err(|_| "credit range")?))
    }
}
