#!/usr/bin/env python3
"""Fixed process schedule with independent complete-hit-stream and final-state oracles."""
import collections
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

POLICIES = ["scan", "linked", "clock", "segmented"]
CASES = [("tiny", 3, 9)] + [(w, c, 32768) for c in [4, 64, 1024]
                                    for w in ["hot", "scan", "loop", "phase", "uniform"]]
MASK = (1 << 64) - 1
EMPTY = MASK


def trace(name, cap, n):
    rng = 0x123456789ABCDEF0
    result = []
    hot = max(1, cap // 2)
    for i in range(n):
        rng = (rng ^ (rng << 13)) & MASK
        rng ^= rng >> 7
        rng = (rng ^ (rng << 17)) & MASK
        random = rng % (4 * cap + 1)
        if name == "tiny":
            key = [0, 1, 0, 1, 2, 3, 4, 0, 1][i % 9]
        elif name == "hot":
            key = cap + random if i % 16 == 15 else i % hot
        elif name == "scan":
            p = i % (8 * cap)
            key = p % hot if p < 4 * cap or p >= 6 * cap else cap + p
        elif name == "loop":
            key = i % (cap + 1)
        elif name == "phase":
            key = i % hot + (i // max(1, n // 4)) * cap
        else:
            key = random
        result.append(key)
    return result


def oracle(keys, cap, policy):
    probation, protected = collections.OrderedDict(), collections.OrderedDict()
    slots, bits, hand = [EMPTY] * cap, [False] * cap, 0
    hit_string, inspections, operations = [], 0, 0
    for k in keys:
        if cap == 0:
            hit_string.append("0")
            continue
        if policy == "clock":
            if k in slots:
                bits[slots.index(k)] = True
                hit = True
            else:
                hit = False
                while True:
                    inspections += 1
                    if slots[hand] == EMPTY or not bits[hand]:
                        slots[hand], bits[hand] = k, True
                        hand = (hand + 1) % cap
                        break
                    bits[hand] = False
                    hand = (hand + 1) % cap
        else:
            hit = k in probation or k in protected
            if k in protected:
                protected.move_to_end(k)
                operations += 2
            elif k in probation:
                del probation[k]
                operations += 2
                if policy == "segmented":
                    protected[k] = None
                    if len(protected) > cap // 2:
                        demoted, _ = protected.popitem(last=False)
                        probation[demoted] = None
                        operations += 2
                else:
                    probation[k] = None
            else:
                if len(probation) + len(protected) == cap:
                    probation.popitem(last=False)
                    inspections += cap
                    operations += 1
                probation[k] = None
                operations += 1
        hit_string.append("1" if hit else "0")
    return dict(hit_string="".join(hit_string), hits=hit_string.count("1"),
                queues=[] if policy == "clock" else ([list(probation), list(protected)]
                        if policy == "segmented" else [list(probation)]),
                slots=slots if policy == "clock" else [], bits=bits if policy == "clock" else [],
                hand=hand, inspections=inspections if policy in ["scan", "clock"] else 0,
                list_operations=operations if policy in ["linked", "segmented"] else 0)


def main():
    binary, output = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
    output.mkdir(parents=True, exist_ok=False)
    env = {"hostname": subprocess.check_output(["hostname"], text=True).strip(),
           "uname": subprocess.check_output(["uname", "-a"], text=True).strip(),
           "compiler": subprocess.check_output(["rustc", "-Vv"], text=True),
           "cpu": Path("/proc/cpuinfo").read_text(), "available_cpus": sorted(os.sched_getaffinity(0)),
           "flags": "-C opt-level=3 -C target-cpu=native --edition=2024",
           "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
           "target_features": subprocess.check_output(["rustc", "--print", "cfg", "-C", "target-cpu=native"], text=True)}
    (output / "environment.json").write_text(json.dumps(env, indent=2))
    schedule, cache = [], {}
    for case_id, (name, cap, n) in enumerate(CASES):
        keys = trace(name, cap, n)
        for p in POLICIES:
            cache[name, cap, n, p] = oracle(keys, cap, p)
            schedule.append(dict(case=case_id, phase="warmup", block=-1, label=p, policy=p))
        for block in range(6):
            rotation = block // 2
            order = POLICIES[rotation:] + POLICIES[:rotation]
            if block % 2:
                order = list(reversed(order))
            for p in order:
                schedule.append(dict(case=case_id, phase="main", block=block, label=p, policy=p))
    # Same linked image, two arm labels, balanced order; a mechanical control.
    for block in range(6):
        for label in (["A", "B"] if block % 2 == 0 else ["B", "A"]):
            schedule.append(dict(case=2, phase="aa", block=block, label=label, policy="linked"))
    (output / "schedule.json").write_text(json.dumps(schedule, indent=2))
    with (output / "records.jsonl").open("w") as stream:
        for index, task in enumerate(schedule):
            name, cap, n = CASES[task["case"]]
            args = [str(binary), task["policy"], name, str(cap), str(n)]
            process = subprocess.run(args, text=True, capture_output=True, timeout=60)
            (output / f"{index:04}.stdout").write_text(process.stdout)
            (output / f"{index:04}.stderr").write_text(process.stderr)
            assert process.returncode == 0, (index, process.stderr)
            record = json.loads(process.stdout)
            expected = cache[name, cap, n, task["policy"]]
            for field, value in expected.items():
                assert record[field] == value, (index, field, record[field], value)
            record.update(task)
            record["verified"] = True
            stream.write(json.dumps(record) + "\n")
            stream.flush()
    print(f"PASS {len(schedule)} processes; every hit and final policy state independently checked", flush=True)


if __name__ == "__main__":
    main()
