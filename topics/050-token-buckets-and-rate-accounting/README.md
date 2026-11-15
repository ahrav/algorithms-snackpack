# Token buckets and rate accounting

Limit admitted cost with a sustained rate and a bounded initial burst.
Each flow starts full at tick zero. Serial attempts use nondecreasing `u64`
time, integer credits per tick, and weighted `u64` costs. A backward time
returns an error and preserves state. Zero cost succeeds; cost above capacity
fails; zero rate spends only the initial credit. The caller synchronizes access.

## Running example

Capacity 10, rate 3 credits/tick:

```text
BEFORE t=0       DURING t=1             AFTER t=2
10 credits       2+3 = 5 credits        5+3 = 8 credits
cost 8: accept   cost 6: reject         cost 6: accept
2 remain         5 remain              2 remain
```

```sh
cargo run -p topic050-rate-accounting --example running
cargo test -p topic050-rate-accounting --all-targets
```

Expected decisions and remaining credits: `(true,2)`, `(false,5)`, `(true,2)`.

## Representations

`Wide` computes `min(B, credit + elapsed*R)` with a `u128` intermediate.
`Bounded` derives `ceil((B-credit)/R)` using quotient and remainder. Enough
elapsed time sets credit to capacity. Shorter elapsed time permits safe narrow
multiplication; zero rate skips refill. `Virtual` stores a credit-scaled finish
value V: start=max(V,tR), admit if start+cost <= tR+B, then advance V on success.
These three implement the same integer contract. A rejected attempt refills
credit while preserving it for later attempts.

B is capacity, R credits per tick, t current time, and elapsed ticks since the
previous attempt. For B=10, R=3, credit=2, elapsed=1, ready=min(10,2+3)=5.
A six-credit request fails. Virtual state V=8 at t=1 tests 8+6 <= 3+10 and fails.
Accepted cost over an interval of D ticks is at most B+RD. The example accepts
14 credits by tick 2, within 10+3*2=16. Each candidate uses O(1) work per attempt
and O(F) state for F flows; N attempts plus initialization cost O(N+F).

Fixed-window counters allow adjacent-window bursts. Exact sliding-window quotas
retain expiring accepted events. A shaper queues until eligible and requires
queue/delay bounds. Three-color single-rate and two-rate markers add distinct
classification contracts. Rate accounting needs separate concurrent-work bounds
and a scheduler when the requirement includes fair service. Composed checks
require one atomic admission decision. Independent full buckets multiply a
fleet's initial allowance by the number of copies.

## Selection and coverage

Start with wide arithmetic and a clear invariant. Use measured workload cells
to select a representation; see measurements/README.md. Weighted byte/work
charging and per-flow state belong to the contract. Fractional rates need
preserved remainders or exact scaling. Cold CPU caches, concurrent/distributed
admission, live clock acquisition, fixed/sliding window implementations, shaping,
GCRA variants, fractional rates, and RFC 2697/2698 color markers are revisit scope.

The authoritative catalog has no coverage arrays and ALGORITHM_INDEX.md is
absent at this base. This gap remains tracked separately. Named variants above
are explicit scope boundaries.

## Sources

- [RFC 3290 section 5.1.3 and appendix A.4](https://www.rfc-editor.org/rfc/rfc3290.html#appendix-A.4): strict accounting.
- [RFC 2697](https://www.rfc-editor.org/rfc/rfc2697.html): single-rate three-color marking and byte units.
- [RFC 2698](https://www.rfc-editor.org/rfc/rfc2698.html): peak and committed rate profiles.

Virtual-debt algebra and the integer error contract are derived for this lesson.
