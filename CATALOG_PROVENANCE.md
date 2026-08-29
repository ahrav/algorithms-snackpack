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
