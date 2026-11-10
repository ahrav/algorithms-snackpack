#!/usr/bin/env python3
"""Frozen six-process screen with an independent arithmetic/byte oracle."""
import hashlib, json, math, os, pathlib, platform, shutil, subprocess, tempfile, time

# The oracle checks frames with assert statements, which -O and PYTHONOPTIMIZE compile out.
if not __debug__:
    raise SystemExit("run.py needs assertions enabled: run python3 measurements/run.py without -O and with PYTHONOPTIMIZE unset")

source = pathlib.Path(__file__).resolve().parent
root = pathlib.Path(tempfile.mkdtemp(prefix="topic045-run-"))
lib = source / "lib.rs" if (source / "lib.rs").exists() else source.parent / "src/lib.rs"
example = source / "experiment.rs" if (source / "experiment.rs").exists() else source.parent / "examples/experiment.rs"
for src, name in [(lib, "lib.rs"), (example, "experiment.rs"), (source / "run.py", "run.py")]:
    shutil.copyfile(src, root / name)
os.chdir(root)
print("Evidence directory: " + str(root), flush=True)

def call(args):
    return subprocess.check_output(args, text=True, stderr=subprocess.STDOUT)

TIMEOUT = float(os.environ.get("TOPIC_045_PROCESS_TIMEOUT_S", 60))
REQUIRED = {"case", "mode", "operation", "n", "bytes", "reps", "ns"}
sizes = {}  # (case, mode) -> word count and frame length of the dump the oracle checked

def measure(out, args, **meta):
    """Run one timed process and write its attempt row before any failure halts the run."""
    row: dict = {"status": "ok"}
    def text(captured):
        return (captured or b"").decode(errors="replace")
    try:
        output = text(subprocess.check_output(args, stderr=subprocess.STDOUT, timeout=TIMEOUT))
    except subprocess.CalledProcessError as failed:
        row.update(status="failed", returncode=failed.returncode, output=text(failed.output))
    except subprocess.TimeoutExpired as expired:
        row.update(status="timeout", timeout_s=TIMEOUT, output=text(expired.output))
    else:
        try:
            decoded = json.loads(output)
            if not isinstance(decoded, dict) or not decoded.keys() >= REQUIRED:
                raise ValueError("measurement object missing " + ",".join(sorted(REQUIRED)))
            invocation = dict({k: meta[k] for k in ("case", "mode", "operation")}, **sizes[meta["case"], meta["mode"]])
            printed = {k: decoded[k] for k in invocation}
            if printed != invocation:
                raise ValueError("measurement labels " + json.dumps(printed) + " differ from invocation " + json.dumps(invocation))
            reps, ns = decoded["reps"], decoded["ns"]
            if type(reps) is not int or reps <= 0 or type(ns) not in (int, float) or not math.isfinite(ns) or ns <= 0:
                raise ValueError("measurement values reps=" + repr(reps) + " ns=" + repr(ns) + " must be a positive count and a finite positive time")
            row.update(decoded)
        except ValueError as malformed:
            row.update(status="malformed", output=output, error=repr(malformed))
    row.update(args=args, **meta)
    out.write(json.dumps(row) + "\n")
    out.flush()
    if row["status"] != "ok":
        raise SystemExit("attempt " + row["status"] + "; retained in raw.jsonl: " + json.dumps(row))

def sha(p):
    return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()

flags = ["--edition=2024", "-C", "opt-level=3", "-C", "target-cpu=native"]
subprocess.run(["rustc", *flags, "--crate-name", "topic045", "--crate-type", "rlib", "lib.rs", "-o", "libtopic045.rlib"], check=True)
subprocess.run(["rustc", *flags, "experiment.rs", "--extern", "topic045=libtopic045.rlib", "-o", "experiment"], check=True)
subprocess.run(["rustc", *flags, "--test", "lib.rs", "-o", "contracts"], check=True)
contracts = call(["./contracts"])
example_result = call(["./experiment", "example"])

def pack(data):
    """Packet coding from the README rule: runs of at least three, literals of at most 128 bytes."""
    out, literal, i = [], [], 0
    while i < len(data):
        if data[i:i+3] == [data[i]] * 3:
            n = 3
            while n < 128 and data[i+n:i+n+1] == [data[i]]:
                n += 1
            if literal:
                out += [len(literal) - 1, *literal]; literal = []
            out += [128 + n - 1, data[i]]; i += n
        else:
            literal.append(data[i]); i += 1
            if len(literal) == 128:
                out += [127, *literal]; literal = []
    if literal:
        out += [len(literal) - 1, *literal]
    return out

def oracle(frame, values, mode):
    assert frame[0] == mode and int.from_bytes(bytes(frame[1:5]), "little") == len(values)
    expected = []
    if mode == 5:
        expected = [(v // 256**b) % 256 for b in range(4) for v in values]
    else:
        for i, v in enumerate(values):
            previous = values[i-1] if i else 0
            if mode == 2:
                v -= previous
            elif mode == 3:
                older = values[i-2] if i > 1 else 0
                v = v - 2*previous + older
            elif mode == 4:
                v ^= previous
            v %= 2**32
            expected += [(v // 256**b) % 256 for b in range(4)]
    # The complete frame, header and packet choice included, equals the Python encoding.
    assert frame == [mode, *len(values).to_bytes(4, "little"), *(expected if mode == 0 else pack(expected))]
    data = frame[5:]
    unpacked = []
    if mode == 0:
        unpacked = data
    else:
        i = 0
        while i < len(data):
            tag = data[i]; i += 1
            n = tag % 128 + 1
            if tag >= 128:
                unpacked += [data[i]] * n; i += 1
            else:
                assert i + n <= len(data)
                unpacked += data[i:i+n]; i += n
    assert unpacked == expected
    # Reconstruct independently using whole-prefix sums rather than the Rust loop.
    if mode == 5:
        words = [sum(unpacked[b*len(values)+i] * 256**b for b in range(4)) for i in range(len(values))]
    else:
        words = [int.from_bytes(bytes(unpacked[i:i+4]), "little") for i in range(0, len(unpacked), 4)]
        if mode in (2,3):
            import itertools
            words = [x % 2**32 for x in itertools.accumulate(words)]
            if mode == 3:
                words = [x % 2**32 for x in itertools.accumulate(words)]
        elif mode == 4:
            import itertools, operator
            words = list(itertools.accumulate(words, operator.xor))
    assert words == values

for case in range(12):
    for mode in range(6):
        row = json.loads(call(["./experiment", "dump", str(case), str(mode)]))
        oracle(row["frame"], row["values"], mode)
        sizes[case, mode] = {"n": len(row["values"]), "bytes": len(row["frame"])}

env = {"hostname": platform.node(), "architecture": platform.machine(), "uname": call(["uname", "-a"]),
       "cpu": call(["lscpu"]), "available_cpus": len(os.sched_getaffinity(0)), "affinity": sorted(os.sched_getaffinity(0)),
       "toolchain": call(["rustc", "-vV"]), "features": call(["rustc", "--print", "cfg", "-C", "target-cpu=native"]),
       "flags": flags, "process_timeout_s": TIMEOUT, "inputs": {p: sha(p) for p in ["lib.rs", "experiment.rs", "run.py"]},
       "binary_sha256": sha("experiment"), "contracts": contracts, "example": example_result,
       "python_oracle": "72 complete workload/candidate frames equal the Python encoding and reconstruct",
       "started_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}
pathlib.Path("environment.json").write_text(json.dumps(env, indent=2) + "\n")
cpu = min(os.sched_getaffinity(0))
with open("raw.jsonl", "w") as out:
    for case in range(12):
        for operation in ["encode", "decode"]:
            for block in range(-1, 6):
                shift = 0 if block < 0 else 2*(block//2)
                order = list(range(6))[shift:] + list(range(6))[:shift]
                if block >= 0 and block % 2:
                    order.reverse()
                for position, mode in enumerate(order):
                    measure(out, ["taskset", "-c", str(cpu), "./experiment", operation, str(case), str(mode)],
                            case=case, mode=mode, operation=operation, block=block, position=position, warmup=block < 0, cpu=cpu)
    for block in range(6):
        for label in (["A", "B"] if block % 2 == 0 else ["B", "A"]):
            measure(out, ["taskset", "-c", str(cpu), "./experiment", "encode", "10", "0"],
                    case=10, mode=0, operation="encode", block=block, label=label, control="AA", warmup=False, cpu=cpu)
subprocess.run(["rustc", *flags, "--crate-name", "topic045", "--crate-type", "rlib", "--emit=asm", "lib.rs", "-o", "lib.s"], check=True)
print(json.dumps({"status": "complete", "records": 1020, "raw_sha256": sha("raw.jsonl"), "binary_sha256": sha("experiment")}))
