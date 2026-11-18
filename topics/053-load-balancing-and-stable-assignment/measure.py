"""Generate deterministic inputs, verify each replay, and save process samples."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import random
import subprocess

POLICIES = ("rr", "one", "two", "scan", "hrw", "local")
# name, fleet size, trace length, remote penalty
CELLS = (
    ("tiny", 4, 16, 5),
    ("regular", 32, 4096, 5),
    ("variable", 32, 4096, 5),
    ("burst", 32, 4096, 5),
    ("hot", 32, 4096, 5),
    ("local-skew", 32, 4096, 5),
    ("expensive-remote", 32, 4096, 60),
    ("wide", 256, 8192, 5),
)
MASK = (1 << 64) - 1


def events(name, length):
    rng = random.Random(53011)
    for i in range(length):
        zone = rng.randrange(10)
        key = rng.getrandbits(64)
        a, b = rng.getrandbits(32), rng.getrandbits(32)
        at, service = i, 28
        if name == "tiny":
            at, service = 2 * i, 4
        elif name == "variable":
            service = 120 if rng.randrange(8) == 0 else 8
        elif name == "burst":
            at, service = i // 32 * 40, 30
        elif name == "hot":
            key, service = (0 if i % 5 else key), 20
        elif name in ("local-skew", "expensive-remote"):
            service = 24
        elif name == "wide":
            service = 208
        yield (at, service, key, int(zone == 0 if name in
               ("local-skew", "expensive-remote") else zone < 5), a, b)


def score(key, backend):
    # Independent spelling of the specified unsigned mixer. Python integers
    # need explicit reduction at the multiplication boundaries.
    value = key ^ ((backend * 11400714819323198485) & MASK)
    for shift, factor in [(30, 13787848793156543929), (27, 10723151780598845931)]:
        value = ((value ^ (value >> shift)) * factor) & MASK
    return value ^ (value >> 31)


def oracle(policy, n, penalty, trace):
    # Advance residual service for every queue at each arrival. Compare this
    # explicit time-step model with the implementation's lazy absolute tails.
    remaining = [0] * n
    half = n // 2
    last = 0
    rows = []
    probes = 0
    remote = 0
    repeated_keys = {}
    for i, (at, service, key, zone, a, b) in enumerate(trace):
        elapsed = at - last
        remaining = [max(work - elapsed, 0) for work in remaining]
        last = at
        if policy == "rr":
            owner = i % n
            probes += 1
        elif policy == "one":
            owner = a % n
            probes += 1
        elif policy == "two":
            owner = sorted((a % n, b % n), key=remaining.__getitem__)[0]
            probes += 2
        elif policy == "scan":
            owner = min(range(n), key=remaining.__getitem__)
            probes += n
        elif policy == "hrw":
            owner = max(range(n), key=lambda j: (score(key, j), -j))
            probes += n
            assert repeated_keys.setdefault(key, owner) == owner
        else:
            local = [zone * half + a % half, zone * half + b % half]
            far = [(1 - zone) * half + a % half, (1 - zone) * half + b % half]
            local_best = min(remaining[j] for j in local)
            probes += 2 + 2 * (local_best > penalty)
            owner = min(local + far, key=lambda j: remaining[j] +
                        penalty * (j // half != zone))
        wait = remaining[owner]
        remaining[owner] += service
        crossing = owner // half != zone
        remote += crossing
        rows.append((owner, wait, wait + service + penalty * crossing))
    responses = sorted(row[2] for row in rows)
    digest = hashlib.sha256(encode(rows)).hexdigest()
    quality = {
        "response_p95": responses[(95 * len(rows) + 99) // 100 - 1],
        "response_max": responses[-1], "remote": remote, "decision_probes": probes,
        "vector_sha256": digest,
    }
    return rows, quality


def encode(rows):
    return "".join(" ".join(map(str, row)) + "\n" for row in rows).encode()


def run(binary, out, smoke):
    out.mkdir(parents=True, exist_ok=False)
    cpu = min(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else None
    if cpu is not None:
        os.sched_setaffinity(0, {cpu})
    quality = {}
    with (out / "processes.jsonl").open("w") as samples:
        for cell, n, length, penalty in CELLS[:1] if smoke else CELLS:
            trace = list(events(cell, length))
            trace_path = out / f"{cell}.trace"
            trace_path.write_bytes(encode(trace))
            expected = {p: oracle(p, n, penalty, trace) for p in POLICIES}
            quality[cell] = {
                "requests": length, "backends": n, "penalty": penalty,
                "trace_sha256": hashlib.sha256(trace_path.read_bytes()).hexdigest(),
                "policies": {p: metrics for p, (_, metrics) in expected.items()},
            }
            for p, (rows, _) in expected.items():
                (out / f"{cell}.{p}.vector").write_bytes(encode(rows))
            reps = 256 if cell == "tiny" else 8
            # Six measured blocks, one launch per position per policy.
            schedule = [("warmup", POLICIES)] + [
                ("measured", POLICIES[i:] + POLICIES[:i]) for i in range(6)
            ]
            if cell == "regular":
                schedule.extend([("aa", ("rr", "rr"))] * 6)
            for block, (role, order) in enumerate(schedule):
                for position, policy in enumerate(order):
                    raw = subprocess.check_output([
                        str(binary), policy, str(n), str(penalty), str(reps),
                        str(trace_path),
                    ])
                    timer, vector = raw.split(b"\n", 1)
                    setup_ns, elapsed_ns = map(int, timer.split())
                    want = expected[policy][1]["vector_sha256"]
                    actual = hashlib.sha256(vector).hexdigest()
                    if actual != want:
                        raise ValueError(f"{cell}/{policy}/{block}: incorrect vector")
                    print(json.dumps({
                        "cell": cell, "block": block, "position": position,
                        "role": role, "policy": policy, "reps": reps, "cpu": cpu,
                        "setup_ns": setup_ns, "elapsed_ns": elapsed_ns,
                        "vector_sha256": actual,
                    }), file=samples, flush=True)
    (out / "quality.json").write_text(json.dumps(quality, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--smoke", action="store_true")
    args = parser.parse_args()
    run(args.binary.resolve(), args.output, args.smoke)


if __name__ == "__main__":
    main()
