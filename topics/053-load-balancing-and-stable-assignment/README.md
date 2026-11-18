# Load balancing and stable assignment

A dispatcher chooses a backend for each arriving request. Queue-aware policies
react to unfinished work. Rendezvous hashing gives repeated keys a stable owner.
Locality can restrict eligibility or add a cost to a remote decision.

## Contract

The replay uses one dispatcher, two equally sized zones, identical single-server
backends, and exact knowledge of previously dispatched work. Arrivals are
nondecreasing, ties follow input order, and requests have positive service
durations. Each backend processes its queue in arrival order. Remote delay is
added to the response after service. Every backend is eligible in this model.
Each policy sees the same event stream, sample tokens, and initially empty fleet.

For arrival time `t` and backend tail `F`, waiting takes
`max(F - t, 0)`. Service of duration `s` advances the tail to `max(F, t) + s`.
The client observes wait plus service plus the remote penalty, if applicable.
At time 10, local B has a tail of 13 and remote C is idle. Service takes 4 ticks
and the remote penalty is 5. B responds in 7 ticks, C in 9. After assigning B,
its tail is 17. At time 11, B would respond in 10 ticks and idle C in 9.

| Policy | Mechanism | Cost per pick | Decision boundary |
|---|---|---|---|
| Round robin | Advance one position | Constant | Similar requests and backends |
| Uniform single choice | Reduce one random token into the pool | Constant | Simple randomized control |
| Power of two choices | Sample twice with replacement, pick least unfinished work | Two probes | Fresh comparable signal |
| Least-work scan | Read every tail and choose the minimum | One probe per backend | Small pools or an exact signal minimum |
| Rendezvous | Choose highest fixed key/backend score | One score per backend | Stable affinity and tolerable hot-key load |
| Local two choices with spill | Try two local draws, then two remote draws if they could win | Two or four probes | Soft locality with a comparable cost in time units |

A first-draw tie is deterministic. The scan uses the lowest ID on ties. The local
policy stays local when predicted response ties. Stable rendezvous IDs and a
consistent score preserve surviving owners on removal. On addition, only keys
won by the new ID move. The example's hash is an experimental score, and the
movement contract concerns fixed scores and IDs.

## Run

```sh
cargo run -p topic-053-stable-assignment --example focus
cargo test -p topic-053-stable-assignment
cargo build -p topic-053-stable-assignment --release --example compare
python3 topics/053-load-balancing-and-stable-assignment/measure.py \
  target/release/examples/compare /tmp/topic053-smoke --smoke
```

The focus prints B at time 10 and C at time 11. The smoke replay checks all six
policies against a queue model with explicit per-arrival service subtraction.

Start with round robin for homogeneous stateless requests. Try sampled work
when queue lengths matter. Use an eligible local pool when geography is
mandatory. For soft locality, check the network cost and remote capacity before
using spill. Use rendezvous for key ownership and measure request-weighted
concentration separately from the number of keys that move.

The catalog names the topic without a coverage array. The repository README
identifies P2C, rendezvous, and locality as its variants. Revisit independently
for delayed and partial load reports, uneven backend speeds, hard eligibility,
proportional locality, weighted rendezvous, and separate affinity/cache effects.
Topic 047 covers ring, rendezvous, modulo, and count-capped ownership movement.

## Primary sources

- Azar et al., [Balanced Allocations](https://cs.brown.edu/people/eupfal/papers/SICOMP29.pdf).
- Thaler and Ravishankar, [Using Name-Based Mappings to Increase Hit Rates](https://www.cs.ucr.edu/~ravi/Papers/Jrnl/HRW98.pdf).
- Google, [Load Balancing in the Datacenter](https://sre.google/sre-book/load-balancing-datacenter/).
