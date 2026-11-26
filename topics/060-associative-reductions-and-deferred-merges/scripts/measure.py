"""Run a fixed balanced schedule, retaining each attempt in JSON Lines."""

import itertools
import json
from pathlib import Path
import random
import subprocess
import sys

FIXTURES = ("tiny", "large-once", "large-read", "dense-cuts", "medium-stripes", "short-stripes")
ARMS = ("raw", "chunked", "compact")
SEED = 600032
KEYS = (
    "fixture", "arm", "values", "snapshots", "reads", "repeats", "visits",
    "buffers", "reserved_bytes", "digest", "build_ns", "read_ns", "drop_ns", "total_ns",
)


def schedule():
    rng = random.Random(SEED)
    for name in FIXTURES:
        for position, arm in enumerate(ARMS):
            yield name, "warmup", 0, position, arm
    cells = list(FIXTURES)
    rng.shuffle(cells)
    for name in cells:
        orders = list(itertools.permutations(ARMS)) * 2
        controls = [("raw-a", "raw-b"), ("raw-b", "raw-a")] * 6
        rng.shuffle(orders)
        rng.shuffle(controls)
        for block, (order, control) in enumerate(zip(orders, controls)):
            phases = [("main", order), ("control", control)]
            if rng.getrandbits(1):
                phases.reverse()
            for phase, arms in phases:
                for position, arm in enumerate(arms):
                    yield name, phase, block, position, arm


def main():
    binary, output = map(Path, sys.argv[1:])
    with output.open("x", encoding="utf-8", buffering=1) as records:
        for name, phase, block, position, arm in schedule():
            attempt = {"phase": phase, "block": block, "position": position,
                       "fixture": name, "arm": arm}
            try:
                result = subprocess.run([binary.resolve(), name, arm], text=True,
                                        capture_output=True, timeout=20, check=False)
                attempt.update(exit=result.returncode, stdout=result.stdout, stderr=result.stderr)
                values = result.stdout.split()
                if len(values) != len(KEYS):
                    raise ValueError("wrong number of process output fields")
                fields = dict(zip(KEYS, values))
                if result.returncode or (fields["fixture"], fields["arm"]) != (name, arm):
                    raise ValueError("process identity or status mismatch")
                for key in KEYS[2:]:
                    if key != "digest":
                        fields[key] = int(fields[key])
                if fields["build_ns"] + fields["read_ns"] + fields["drop_ns"] != fields["total_ns"]:
                    raise ValueError("phase times do not add to the total")
                attempt.update(fields)
            except (OSError, ValueError, subprocess.TimeoutExpired) as error:
                attempt["error"] = str(error)
            records.write(json.dumps(attempt, sort_keys=True) + "\n")


if __name__ == "__main__":
    main()
