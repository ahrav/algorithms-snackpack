# Correctness and invariant audit

The independent oracle counts values in BTreeMap and emits keys in order with
multiplicity. It does not share partition, heap, buffer, or slice-sort logic.

Six integration tests:

1. Exhaust all 3280 arrays of lengths 0..7 over {-1,0,1}; check every K from
   0 through N+1 against all five candidates and input preservation. At every
   valid rank, check nth value, both side inequalities, and full multiset.
2. Running example, integer MIN/MAX, repeated extrema, and usize::MAX request.
3. Seeded random shapes over seven lengths and six K boundaries, 20 draws each.
4. Equal-band 1024 values: exactly one partition, 1024 classifications, no swaps.
5. Invalid rank returns None without mutation, including empty input.
6. Organ pipe: classification budget <=8*N and deterministic fallback actually
   executes; the selected rank agrees with the independent oracle.

Plausible violations rejected: selecting K instead of K-1; losing duplicate
multiplicity; advancing scan after a greater-than swap; returning an unordered
result; choosing a min-heap root; buffer insertion dropping the wrong endpoint;
mutating the caller; unsigned K-1 underflow; ignoring the scan budget.

Bounds are finite test evidence, not proof for all arrays or comparators.
The crate forbids unsafe code. Public APIs document the immutable integer
contract; generalized record/float comparators are not implemented here.
All per-process benchmark oracles run outside the measured interval.
