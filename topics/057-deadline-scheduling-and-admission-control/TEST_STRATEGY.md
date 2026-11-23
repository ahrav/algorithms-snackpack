# Test strategy

| Invariant | Independent check | Violation exposed |
| --- | --- | --- |
| EDF meets feasible finite sets | Exhaustively search assignments of idle or ready work for 3,375 three-job sets | Order by fixed priority or ignore release times |
| Jobs may finish at their deadline | Explicit absolute-deadline, shifted-release and preemption traces | Compare completion with a strict deadline or use relative deadlines to dispatch |
| Only jobs with both endpoints in the interval contribute demand | Enumerate synchronous releases for 900 task pairs | Round up before the first due job or skip work due exactly at the boundary |
| The probe cap never proves a truncated check | Known overload and truncated hyperperiod examples, including large periods | Admit from utilization or from a passing strict prefix |
| Budget exhaustion changes server eligibility or its deadline | Explicit server slots, plus ordinary periodic jobs for the fixed-boundary example | Reuse the old deadline after immediate refill or run a throttled server early |

The test oracle searches time slots instead of using a dispatch priority. The
periodic oracle generates releases instead of using the demand formula.
The server checks cover finite, continuously backlogged traces. They do not
model kernel wakeups or claim general kernel equivalence.
