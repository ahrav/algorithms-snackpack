# Local smoke campaign

The local campaign ran seven fresh processes per candidate and workload on
macOS under an earlier revision of the runner, which accepted odd repetition
counts and ran no warmups. It rotated candidate order and measured 70
processes in total. The current `run_campaign.py` requires an even count of
at least four and adds warmups and A/B controls, so this table is retained
as a record and is not reproducible with the committed runner.

| Workload | Local median winner | Global FIFO median ns | Locked steal median ns | Range warning |
| --- | --- | ---: | ---: | --- |
| tiny balanced | locked balanced | 77,541 | 49,875 | 29,084 to 707,000 ns |
| tiny skewed | locked skewed | 50,417 | 38,083 | 30,667 to 693,875 ns |
| medium balanced | global FIFO | 672,208 | 872,959 | 214,667 to 1,672,667 ns |
| medium skewed | global FIFO | 481,583 | 1,288,417 | 190,417 to 2,160,500 ns |
| coarse skewed | global FIFO | 1,038,791 | 1,526,291 | 808,458 to 1,784,708 ns |

Each process passed the independent oracle. The observed ranges overlap within
several workloads, and the host was not pinned. These medians served as the
pilot variance for the Linux protocol in `../BENCHMARK.md` and establish no
general winner.
