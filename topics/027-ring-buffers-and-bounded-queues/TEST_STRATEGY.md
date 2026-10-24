# Correctness and invariant audit

Scope: safe single-owner exact-capacity FIFO, optional-slot ownership and split spans.
Contract/failures: full returns the owned value unchanged; empty returns None; no eviction;
logical capacity is independent of mask padding. Construction is infallible-allocation style.
Primary technique: transition-by-transition differential histories against a BTreeMap keyed
by monotone insertion tickets, independently implementing admission and oldest-first removal.
Secondary: public integration test, exact running example, owning-drop counters and many laps.
Domain: 3^8=6561 histories at C=0,1,2,3,4,7,8 for all five candidates. Operation alphabet is
push(step), push(0), pop. Final committed tests include duplicate payloads. Additional checks
cover 10,000 three-element laps, zero-capacity owned strings and unit payloads.
Replay: cargo test; standalone rustc --test; run.py validates warm/timed checksums and final
state. A green run establishes these finite observations, not arbitrary histories or sharing.
Gaps: fallible allocation, panicking payload destruction, concurrent schedules/weak memory,
blocking cancellation/wakeup, overwrite and byte-bounded variable payloads. Those need a
separate protocol and reviewed implementation. No timing candidate implements sharing.

| Claim | Plausible violation | Smallest discriminating observation | Independence / comparator |
|---|---|---|---|
| Exact unchanged full rejection | Admit backing padding or report success | C=3 masked fourth push; C=0 first push | Map admission, exact Result, values and length |
| FIFO after wrap | Advance head by two or leave it fixed | C=2 push0,push1,pop,push2,pop,pop | Map pop_first, exact Option |
| Split spans preserve order | Skip/duplicate end slot | P=4 full, pop10, push14 | Literal expected slices plus public strings |
| Split spans track length | Expose capacity-sized spans at partial occupancy | C=3,4,17 varied push/pop rounds, wrapped partial spans in each mode | VecDeque oracle, exact lengths and Some payloads |
| Owning drops occur once | Lose ownership or destroy on rejected return | C=2, reject third, drop return/pop/container | External Rc<Cell> counter, exact counts |
| Duplicate payloads remain entries | Deduplicate values | push(0),push(0),pop,pop | Ticket identity independent of value |

Mutation backstops are compiled externally and retained in the evidence receipt. They
change full admission, report full success, corrupt read advancement and suppress returned
pops. Each must fail a named meaningful correctness check before the artifact is published.

measurements/test_runner.py checks run.py on Linux with a stub benchmark: refusal over a retained
campaign, timeout retention as a failure record, an identity whose target cfg comes from the -O
build, and `cargo bench` argument handling.
