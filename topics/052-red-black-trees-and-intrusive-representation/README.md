# Red-black trees and intrusive representations

Store unique immutable `u64` keys. Insert returns whether membership changed;
contains returns exact membership. Duplicate insertion preserves the existing
record. This visit implements insertion and lookup. Deletion, range APIs,
augmented summaries, cached minima and pointer-based intrusive ownership are
explicit revisit work.

## Balance and record identity

Missing children are black. The root is black; red links have black children;
every path to a missing child has equal black height. Left-leaning trees also
forbid right-red links. Repair insertion by rotating a lone right-red link left,
rotating consecutive left-red links right, then splitting two red children.

```text
before insert20      attached20          rotate left
    10B                  10B                 20B
                            \               /
                            20R            10R
```

Inserting30 next flips the two child colors and blackens the root. The set is
10,20,30; repeating20 changes nothing. Rotations relink records and preserve keys.
Arena birth-slot order is tested separately from sorted tree traversal.

The arena embeds links in records and uses stable slot numbers. Its owning vector
can reallocate. Moving/reusing individual slots requires another identity protocol.
Linux-style intrusive pointers instead require externally owned, address-stable
records, unlink-before-drop, separate links per membership and synchronization.

## Cost and choice

For distinct cardinality n, black height b and height h, equal black height gives
n >= 2^b-1; separated red links give h <= 2b <= 2 log2(n+1). At n=3 this bound
is4; the example has height2. For m=4 insert attempts and q=4 queries, the
O((m+q)log2(n+1)) model substitutes O(8*2), rather than an exact operation count.

Sorted-vector lookup is logarithmic, while descending distinct insertion moves
n(n-1)/2 elements:3 at n=3,8,386,560 at n=4096. Ascending insertion moves zero.
The boxed tree creates one Box per distinct key; the reserved arena allocates one
record buffer when capacity is positive. These are source-level allocation
contracts; allocator bytes, resident memory and cache effects remain unmeasured.

| Candidate | Useful workload | Deciding cost or contract |
|---|---|---|
| Sorted vector | small or mostly static sets | tail movement during insertion |
| Boxed red-black | individual owning nodes | allocation per distinct key |
| Arena red-black | embedded links and stable slot identity | reservation and immutable slot mapping |
| BTreeSet | general ordered-set operations | multi-key node management |

Read [measurement results](measurements/README.md) before making a speed choice.
The experiment separates build/query/drop and tests both small and larger inputs,
ascending/descending/duplicate-heavy order, dispersed/hot lookup and hits/misses.

## Run

```sh
cargo test -p topic-052-red-black
cargo run --release -p topic-052-red-black --example compare -- arena 32 ascending dispersed check
cargo build --release -p topic-052-red-black --example compare
python3 topics/052-red-black-trees-and-intrusive-representation/measure.py target/release/examples/compare results/processes.jsonl
```

Check mode emits full input, insertion booleans, sorted keys, birth slots and hit
strings. The independent Python set/sort oracle checks every result. Check-mode
timings contain validation/output preparation and are excluded from selection.

## Primary sources

- [Princeton left-leaning insertion and deletion](https://raw.githubusercontent.com/kevin-wayne/algs4/master/src/main/java/edu/princeton/cs/algs4/RedBlackBST.java)
- [Sedgewick, left-leaning red-black trees](https://sedgewick.io/wp-content/themes/sedgewick/papers/2008LLRB.pdf)
- [Linux embedded rbtree API, erasure, augmentation and cached minimum](https://docs.kernel.org/core-api/rbtree.html)
- [Rust BTreeSet contract](https://doc.rust-lang.org/std/collections/struct.BTreeSet.html)

The general red-black rotation-count bounds in Linux documentation apply to its
algorithm. The left-leaning variant here has its own repair sequence.
