# Native routing replay

Frozen executable source `7a5786c11202312dc005918b58a6dd03a91d7909`.
Both Linux hosts passed the focused example, four Rust regressions, and
348 fresh-process checks each. Each process's complete decision, wait, and
response vector matched the Python oracle. The two hosts matched on all
traces, vectors and modeled quality. Six paired blocks per cell use the
selection rule in [BENCHMARK.md](../BENCHMARK.md).

The Arm host exposed ARM implementer 0x41, part 0xd40, 64 available CPUs
and Rust 1.98.1. The x86 host exposed Intel Xeon Platinum 8488C, 192 available
CPUs and Rust 1.99.0. Both runs pinned CPU 0 and compiled with
`--edition=2024 -C opt-level=3 -C target-cpu=native`. Kernel, target features,
archive and binary identities are recorded in the raw evidence and receipt.

## Dispatch cost in microseconds per trace

Median [minimum, maximum] from six process samples. Setup parses and
allocates the input and output separately. Per-trace setup medians are
available in the retained JSON. Use the other table for modeled response.

### arm

| Cell | RR | One | Two | Scan | HRW | Local | Selection |
|---|---:|---:|---:|---:|---:|---:|---|
| tiny | 0.126 [0.124, 0.130] | 0.133 [0.133, 0.135] | 0.190 [0.187, 0.270] | 0.202 [0.200, 0.203] | 0.278 [0.269, 0.279] | 0.201 [0.196, 0.250] | unresolved |
| regular | 25.281 [24.477, 26.395] | 25.695 [25.343, 26.226] | 33.687 [33.508, 35.527] | 141.405 [139.965, 156.782] | 316.441 [314.579, 318.614] | 71.316 [70.732, 71.821] | unresolved |
| variable | 25.378 [24.590, 26.009] | 25.710 [25.386, 26.129] | 33.774 [33.564, 35.247] | 141.181 [140.786, 142.648] | 316.643 [314.656, 348.916] | 55.221 [55.108, 56.180] | unresolved |
| burst | 25.396 [24.385, 26.373] | 25.803 [25.648, 26.082] | 33.916 [33.346, 35.087] | 139.869 [139.097, 140.764] | 316.405 [316.164, 316.872] | 66.945 [66.769, 68.094] | unresolved |
| hot | 25.360 [24.578, 26.141] | 25.848 [25.652, 26.142] | 33.570 [33.405, 34.119] | 142.759 [142.383, 145.044] | 316.252 [314.578, 347.431] | 52.654 [52.374, 58.248] | unresolved |
| local-skew | 26.043 [24.557, 27.128] | 25.410 [25.223, 25.686] | 33.750 [33.244, 34.542] | 141.074 [140.903, 142.390] | 317.353 [316.037, 340.701] | 71.268 [71.097, 72.856] | unresolved |
| expensive-remote | 25.326 [24.460, 27.114] | 25.837 [25.336, 26.979] | 33.437 [33.279, 33.925] | 141.834 [140.858, 142.824] | 316.867 [314.468, 318.084] | 57.689 [57.244, 58.959] | unresolved |
| wide | 52.702 [48.953, 61.202] | 51.418 [50.760, 55.839] | 67.269 [66.306, 69.000] | 2036.228 [2033.886, 2050.135] | 4825.742 [4811.131, 4854.716] | 136.537 [133.382, 138.949] | unresolved |

Regular RR A/A first-to-second time ratios ranged from 0.991 to 1.019.

### x86

| Cell | RR | One | Two | Scan | HRW | Local | Selection |
|---|---:|---:|---:|---:|---:|---:|---|
| tiny | 0.175 [0.175, 0.185] | 0.182 [0.182, 0.207] | 0.201 [0.201, 0.202] | 0.204 [0.199, 0.209] | 0.268 [0.258, 0.314] | 0.211 [0.211, 0.477] | unresolved |
| regular | 41.670 [40.743, 89.028] | 58.309 [54.489, 60.096] | 70.910 [67.764, 154.084] | 159.621 [158.516, 161.553] | 438.148 [435.767, 461.066] | 80.584 [80.404, 81.917] | unresolved |
| variable | 41.269 [40.869, 43.256] | 58.495 [57.175, 59.786] | 69.601 [68.931, 70.334] | 159.196 [157.393, 165.870] | 440.550 [429.654, 447.549] | 56.789 [55.991, 57.469] | rr |
| burst | 41.207 [40.905, 43.093] | 58.687 [56.562, 59.650] | 70.887 [69.102, 74.544] | 158.757 [157.600, 161.028] | 435.469 [431.852, 464.739] | 70.689 [67.339, 71.356] | rr |
| hot | 41.237 [40.754, 42.880] | 58.463 [56.438, 60.751] | 69.343 [68.425, 71.120] | 159.211 [158.553, 160.150] | 298.572 [295.344, 301.791] | 53.996 [53.178, 56.004] | rr |
| local-skew | 38.842 [38.266, 41.491] | 58.547 [56.019, 79.825] | 71.526 [68.937, 73.816] | 144.992 [144.644, 158.836] | 412.598 [410.239, 415.653] | 74.197 [73.615, 76.271] | rr |
| expensive-remote | 38.369 [38.289, 40.270] | 58.269 [56.072, 58.980] | 71.465 [69.446, 72.046] | 145.270 [144.375, 177.880] | 413.934 [410.620, 419.360] | 58.385 [57.705, 117.214] | rr |
| wide | 87.104 [86.201, 88.945] | 120.023 [118.478, 121.775] | 146.516 [145.076, 147.331] | 715.924 [713.509, 723.101] | 4072.160 [4066.200, 4078.603] | 175.567 [174.383, 192.968] | rr |

Regular RR A/A first-to-second time ratios ranged from 0.994 to 1.086.

## Modeled response and locality

Each cell uses identical inputs on both hosts. Values are P95 in ticks.
Remote counts and logical decision probes are available in the retained JSON.

| Cell | RR | One | Two | Scan | HRW | Local | Best P95 |
|---|---:|---:|---:|---:|---:|---:|---|
| tiny | 9 | 13 | 11 | 9 | 13 | 8 | local |
| regular | 33 | 330 | 69 | 33 | 265 | 49 | rr/scan |
| variable | 248 | 273 | 125 | 125 | 251 | 120 | local |
| burst | 35 | 195 | 70 | 35 | 190 | 60 | rr/scan |
| hot | 25 | 78 | 38 | 25 | 58282 | 28 | rr/scan |
| local-skew | 29 | 152 | 50 | 29 | 146 | 37 | rr/scan |
| expensive-remote | 84 | 198 | 103 | 84 | 179 | 87 | rr/scan |
| wide | 213 | 1328 | 451 | 213 | 1463 | 328 | rr/scan |

Round robin and scan tie on modeled P95 in six cells, including the
equal-size regular and burst traces. The local policy wins tiny and
variable. In the expensive-remote trace it cuts remote traffic from
2,056 to 938 of 4,096 requests compared with round robin, with P95
rising from 84 to 87 ticks. Global two-choice reduces modeled P95 relative
to one-choice in every cell. HRW concentrates repeated work at one owner
by design. The hot workload models no reduction in service from affinity.

These are bounded simulations with a single exact load view. Dispatch
timings exclude process startup, parsing, resets and output. P95 values
are simulator ticks. Models with stale reports, heterogeneous speeds,
admission, affinity savings or a real network need separate evidence.

## Reproduce

```sh
cargo build -p topic-053-stable-assignment --release --example compare
python3 topics/053-load-balancing-and-stable-assignment/measure.py \
  target/release/examples/compare /tmp/topic053-replay
```

The external evidence retains the exact native driver, frozen source,
binaries, environments, assembly, vectors, process samples and checks.
See [the receipt](EVIDENCE_RECEIPT.json).
