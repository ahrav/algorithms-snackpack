"""Run the predeclared fresh-process comparisons and write one JSON record per process."""

import itertools
import json
import os
import subprocess

CELLS = ("tiny", "equal", "skew", "weighted-wide")
POLICIES = ("rr", "drr", "wfq")
CPU = min(os.sched_getaffinity(0))


def run(cell, policy, mode, block, position):
    output = subprocess.check_output(
        ["taskset", "-c", str(CPU), "replay/bench_queues", cell, policy],
        text=True,
    )
    ns, packets, operations, digest = map(int, output.split())
    print(json.dumps(dict(
        cell=cell, policy=policy, mode=mode, block=block, position=position,
        cpu=CPU, ns=ns, packets=packets, operations=operations, digest=digest,
    )), flush=True)


for cell in CELLS:
    for position, policy in enumerate(POLICIES):
        run(cell, policy, "warmup", 0, position)
    for block, order in enumerate(itertools.permutations(POLICIES)):
        controls = range(2)
        if block % 2 == 0:
            for position in controls:
                run(cell, "drr", "control", block, position)
        for position, policy in enumerate(order):
            run(cell, policy, "timed", block, position)
        if block % 2 == 1:
            for position in controls:
                run(cell, "drr", "control", block, position)
