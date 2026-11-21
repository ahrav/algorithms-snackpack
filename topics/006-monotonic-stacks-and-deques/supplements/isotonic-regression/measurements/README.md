# Native measurements

Frozen executable input commit `733d121e09e03accf01ccdbd99efad68c94badd7`.

Build the library, binary and contract tests with Rust 2024, opt level 3 and
native target features. Exact compiler and host identities are in the external
archive. The runner pins the first allowed CPU (CPU 0 on both hosts).

The independent oracle and residual certificates checked 1,190 inputs each
for all three candidates on both hosts. The two Rust contract tests also passed
per host. Each host ran 330 fresh processes, including 180 candidate samples,
120 stack A/A controls and 30 discarded warmup processes.

Times are median microseconds per fit in shift / stack / hull order. The
A/A column gives the range of identical-run control ratios. Selection requires
the median winner to beat both alternatives by a 1.05 time ratio in all six
paired blocks and the A/A controls to stay in [1/1.05, 1.05].

| Input | Arm µs | A/A | Selection | x86-64 µs | A/A | Selection |
|---|---:|---:|---|---:|---:|---|
| 16 up | 0.034 / 0.036 / 0.076 | 0.994–1.036 | shift | 0.039 / 0.037 / 0.055 | 0.909–1.125 | Unresolved |
| 16 noise | 0.080 / 0.049 / 0.077 | 0.978–1.003 | stack | 0.208 / 0.046 / 0.058 | 0.958–1.056 | Unresolved |
| 512 up | 0.592 / 0.643 / 1.471 | 0.993–1.039 | Unresolved | 0.584 / 0.832 / 1.127 | 0.856–1.173 | Unresolved |
| 512 plateau | 29.027 / 1.366 / 1.756 | 0.908–1.017 | Unresolved | 17.459 / 1.067 / 1.269 | 0.999–1.022 | stack |
| 512 fall | 29.391 / 1.224 / 1.170 | 0.972–1.022 | Unresolved | 17.142 / 1.191 / 0.985 | 0.999–1.017 | hull |
| 512 noise | 29.081 / 1.167 / 1.588 | 0.970–1.006 | stack | 17.794 / 1.227 / 1.341 | 0.975–1.021 | stack |
| 4,096 up | 4.718 / 4.858 / 11.043 | 0.946–1.033 | Unresolved | 5.289 / 6.488 / 9.509 | 0.869–1.002 | Unresolved |
| 4,096 plateau | 1854.292 / 10.871 / 13.981 | 0.978–1.037 | stack | 38271.961 / 8.365 / 9.886 | 0.962–1.073 | Unresolved |
| 4,096 fall | 1914.136 / 9.594 / 8.955 | 0.998–1.021 | Unresolved | 39485.620 / 9.531 / 7.678 | 0.965–1.000 | hull |
| 4,096 noise | 1915.291 / 14.411 / 16.048 | 0.964–1.035 | stack | 39067.875 / 13.197 / 11.519 | 0.593–1.066 | Unresolved |

The selection applies to warm serial fits with compressed output on these
hosts. Materialized vectors add output writes. The test did not vary concurrency
or model production input. A result can be unresolved when identical runs
vary more than the declared control bound or the paired margin is too small.

The [receipt](EVIDENCE_RECEIPT.json) identifies both archives, which retain
every measured process sample and the hash-verified transferred source.
