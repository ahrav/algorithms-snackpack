# Catalog provenance

Source conversation:
<https://chatgpt.com/c/6984b0e8-e0f8-8323-a05e-73eca3e15849>

The bootstrap audit read all 31 historical response items available in the
conversation on 2026-08-29. The early installments repeatedly covered scans,
deferred range updates, bitmaps, interval sets, windows, union-find, codecs, and
selection. Later installments added searching, sorting, hashes, trees, tries,
queues, graph algorithms, sampling, probabilistic structures, scheduling,
eviction, compression, production hybrids, and incremental indexes.

The catalog is a semantic deduplication, not a transcript index:

- synonymous headings were merged;
- a broad heading was split when its variants have distinct invariants or cost
  models, such as open addressing, SwissTable, Robin Hood, and cuckoo hashing;
- production combinations were retained when composition changes the algorithm,
  such as hash joins with recursive spill or edit distance with dictionary
  automata;
- cross-cutting advice became methodology rather than duplicate topics;
- topics can be appended, but published numbers are never reassigned.

The source conversation remains useful context, but `curriculum.toml` is the
durable topic authority for the scheduled curriculum.

## 2026-11-20 expansion

Topics 055–064 come from the saved daily rotation. Topics 065–067 retain the
earlier catalog review's three distinct proposals. The remaining additions
cover static and succinct indexes, substring data structures, geometry, graph
optimization, arithmetic, numerical methods and combinatorial search.

Each addition has a distinct question, a primary source and a runnable
experiment plan with a correctness check. Coverage keeps variants under
their closest topic. Topic 003 covers bitmap set operations. Topic 070 covers
positional rank and select. Topic 029 covers offline rollback. Topic 088 covers
online deletion and replacement edges. Topic 087 covers weight-optimal forests.
Topic 100 covers sampling spanning trees from a specified distribution.

## 2026-11-21 placement

Topic 115 places isotonic regression after multilevel graph partitioning.
This lesson fits an ordered signal and can revise previously fitted blocks.
Topic 006 covers candidate stacks and deques for extrema and boundary queries.
