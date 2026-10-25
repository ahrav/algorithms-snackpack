# Work-stealing deques

One owner removes recent tasks from a deque bottom. Idle workers steal older
tasks from a deque top. This crate compares a global locked FIFO with a safe
locked work-stealing model. The latter exposes ownership and steal direction.
Its mutex claim keeps the code safe and does not represent Chase-Lev throughput.

Run the checked example:

```bash
cargo run --release -p topic028-work-stealing-deques --example compare -- \
  --candidate locked_steal_skewed --workers 4 --tasks 4096 --leaf-work 512
```

The contract requires every task exactly once. `oracle` is independent of both
schedulers. Tests sweep worker, task, and leaf-work counts.

The declared selection rule and exact-source Linux evidence are retained in
[BENCHMARK.md](BENCHMARK.md) and [measurements](measurements/README.md).
Lock-free implementations also require atomic publication and retired-array
lifetime arguments.

Sources: [Chase and Lev, Dynamic Circular Work-Stealing Deque](https://www.cs.wm.edu/~dcschmidt/PDF/work-stealing-dequeue.pdf), and [Blumofe and Leiserson, Scheduling Multithreaded Computations by Work Stealing](https://www.cs.cornell.edu/courses/cs612/2006sp/papers/blumofe94.pdf).

## Claim and termination

Each ID is inserted once. A mutex serializes each deque removal, so a successful
removal transfers one unique task to one worker. Workers release the queue lock
before running the task. The `remaining` counter includes queued and executing
tasks and decreases after execution. For this closed, preloaded task set, zero
means every task has completed. Count and checksum checks provide bounded
regression evidence; the ownership argument supplies uniqueness.

Dynamic spawning, cancellation, parking, Chase-Lev atomic operations, batch
stealing and safe retired-array reclamation are explicit revisit scope. The
current catalog supplies Topic028's ID, slug and title. It has no coverage array,
and the repository has no ALGORITHM_INDEX.md.

## Measured results

The exact-source Linux campaign ran under the frozen paired 3 percent rule
without the identical-artifact A/A control the protocol now requires, so its
cells are reported as provisional rather than selected. Under that rule the
paired ratios favour balanced per-worker deques for medium balanced work on
both hosts and global FIFO for skewed medium and coarse work on Arm; tiny
cases and the remaining x86 cells are unresolved. These results become
selections once the campaign is rerun with the current runner and its A/A
ratios pass. See the measurement table for descriptive median ranks and full
observed ranges.
