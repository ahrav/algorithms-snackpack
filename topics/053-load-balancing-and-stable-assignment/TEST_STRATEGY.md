# Test strategy

The Python oracle explicitly advances every queue by the elapsed time before
each arrival, selects using a separate formulation, and checks complete
decision/wait/response vectors. Oracle work runs outside the timed native code.
Rendezvous uses the specified score in Python and a separate stable-ID ranking.

| Invariant | Detected violation |
|---|---|
| Locality compares saved queue time to remote delay | Never spill after local work rises, or spill on an equal response |
| Sample ties prefer the first draw and duplicate draws stay duplicated | Redraw a duplicate, reverse a tie, or replace the ordered sample by a set |
| Queue time decays through idle periods | Treat an old absolute tail as current unfinished work |
| Membership edits preserve unrelated rendezvous owners | Score slice positions instead of stable identities |
| Overflow fails before mutation | Wrap a tail at the end of the tick range |

The full replay covers zones with different traffic, equal arrivals, variable
service, indivisible affinity keys, and different fleet sizes. These are bounded
tests. The stable-ID movement claim follows from retaining the scores of
remaining owners under a fixed tie rule.
