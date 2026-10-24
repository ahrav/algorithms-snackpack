"""Smoke checks for run.py: stale campaigns, timeout retention, build identity, cargo bench arguments.

Each campaign scenario runs run.py in a scratch copy of the topic whose benchmark is replaced by a small stub, so a
campaign takes seconds. The cargo bench scenario runs the real benchmark for single cells. run.py requires Linux
(lscpu, sched_getaffinity), so these checks do too. Run from anywhere: python3 measurements/test_runner.py [scenario...]
"""

import contextlib
import json
import os
import pathlib
import shutil
import signal
import subprocess
import sys
import tempfile

TOPIC = pathlib.Path(__file__).resolve().parent.parent
CARGO_ROOT = TOPIC.parent.parent
PACKAGE = "topic027-ring-buffers-and-bounded-queues"
ROW = '{{\\"candidate\\":\\"{}\\",\\"case\\":\\"{}\\",\\"ns\\":1,\\"operations\\":1,\\"ns_per_op\\":1.0,\\"checksum\\":7}}'
CONSTANT_BENCH = (
    "fn main() {\n"
    "    let a: Vec<String> = std::env::args().collect();\n"
    '    println!("' + ROW + '", a[1], a[2]);\n'
    "}\n"
)
WEDGED_BENCH = "fn main() {\n    std::thread::sleep(std::time::Duration::from_secs(3600));\n}\n"


@contextlib.contextmanager
def scratch(bench, timeout=None):
    """Yield a topic copy with the benchmark source replaced and the per-command timeout optionally lowered."""
    with tempfile.TemporaryDirectory() as root:
        root = pathlib.Path(root)
        for part in ["src", "tests", "benches", "examples"]:
            shutil.copytree(TOPIC / part, root / part)
        shutil.copy(TOPIC / "run.py", root / "run.py")
        (root / "benches" / "portfolio.rs").write_text(bench)
        if timeout is not None:
            script = root / "run.py"
            script.write_text(script.read_text().replace("timeout=90", f"timeout={timeout}"))
        yield root


def run(root, timeout):
    """Run the campaign in its own process group so a wedged benchmark dies with it."""
    proc = subprocess.Popen([sys.executable, str(root / "run.py")], cwd=root, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True, start_new_session=True)
    try:
        _, stderr = proc.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        os.killpg(proc.pid, signal.SIGKILL)
        proc.wait()
        raise SystemExit(f"run.py still running after {timeout}s") from None
    return proc.returncode, stderr


def rerun_refuses_existing_campaign():
    with scratch(CONSTANT_BENCH) as root:
        code, stderr = run(root, 300)
        assert code == 0, stderr
        raw = root / "raw"
        before = {p.name: p.read_bytes() for p in raw.iterdir()}
        code, stderr = run(root, 300)
        assert code != 0, "run.py started a second campaign into a raw/ directory that already holds one"
        assert "raw" in stderr, stderr[-400:]
        after = {p.name: p.read_bytes() for p in raw.iterdir()}
        changed = sorted(set(before) ^ set(after) | {n for n in before if n in after and before[n] != after[n]})
        assert not changed, f"refused rerun changed retained files: {changed}"
    print(f"ok: a rerun over {len(before)} retained files refuses without touching any of them")


def identity_describes_the_optimized_build():
    with scratch(CONSTANT_BENCH) as root:
        code, stderr = run(root, 300)
        assert code == 0, stderr
        identity = json.loads((root / "raw" / "identity.json").read_text())
        assert "-O" in identity["flags"], identity["flags"]
        assert "debug_assertions" not in identity["features"], "identity.json features describe an unoptimized build"
        assert "target_arch=" in identity["features"], identity["features"][:200]
    print("ok: identity.json target cfg comes from the same -O build as the measured binaries")


def timed_out_command_is_recorded_and_aborts():
    with scratch(WEDGED_BENCH, timeout=20) as root:
        code, _ = run(root, 180)
        assert code != 0, "campaign reported success with a wedged benchmark"
        failures = list((root / "raw").glob("failure-*.txt"))
        assert len(failures) == 1, f"expected one retained failure record, found {[f.name for f in failures]}"
        text = failures[0].read_text()
        assert "portfolio" in text and "timeout" in text, text[:400]
        assert not (root / "raw" / "summary.json").exists(), "summary.json written despite a timed-out process"
    print("ok: a wedged benchmark is retained as a timeout failure and the campaign aborts")


def bench(*args):
    proc = subprocess.run(["cargo", "bench", "-p", PACKAGE, "--bench", "portfolio", "--", *args], cwd=CARGO_ROOT, text=True, capture_output=True, check=False)
    assert proc.returncode == 0, f"cargo bench -- {' '.join(args)} failed:\n{proc.stdout[-300:]}{proc.stderr[-400:]}"
    return proc.stdout


def cargo_bench_runs_default_and_requested_cells():
    out = bench()
    assert '"candidate":"deque","case":"tiny"' in out, "cargo bench with no arguments did not run deque/tiny:\n" + out[-300:]
    out = bench("mask")
    assert '"candidate":"mask","case":"tiny"' in out, "cargo bench -- mask did not default the case to tiny:\n" + out[-300:]
    out = bench("branch", "steady_odd")
    assert '"candidate":"branch","case":"steady_odd"' in out, "cargo bench -- branch steady_odd did not run the requested cell:\n" + out[-300:]
    print("ok: cargo bench --bench portfolio runs deque/tiny by default, -- mask keeps tiny, and -- branch steady_odd runs that cell")


for scenario in [rerun_refuses_existing_campaign, identity_describes_the_optimized_build, timed_out_command_is_recorded_and_aborts, cargo_bench_runs_default_and_requested_cells]:
    if len(sys.argv) == 1 or scenario.__name__ in sys.argv[1:]:
        scenario()
