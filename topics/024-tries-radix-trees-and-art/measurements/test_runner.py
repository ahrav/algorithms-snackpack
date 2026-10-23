"""Smoke checks for scripts/run.py: stale artifacts, test-output retention, build-artifact placement, wedged processes, cargo bench arguments.

Each scenario runs run.py against a scratch copy of the topic whose benchmark is replaced by a small stub, so a campaign
takes seconds. Only the cargo bench scenario runs the real benchmark, for one cell. run.py requires Linux (lscpu), so these do too.
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
ROW = '{{\\"ns\\":1,\\"checksum\\":7,\\"loops\\":1,\\"queries\\":64,\\"n\\":32,\\"nodes\\":0,\\"capacity_bytes\\":0,\\"families\\":[0,0,0,0],\\"oracle\\":true}}'
CONSTANT_BENCH = 'fn main() {\n    println!("' + ROW + '");\n}\n'
WEDGED_BENCH = "fn main() {\n    std::thread::sleep(std::time::Duration::from_secs(3600));\n}\n"
RED_TEST = '\n#[cfg(test)]\nmod red {\n    #[test]\n    fn deliberate_failure() {\n        panic!("red");\n    }\n}\n'
STATE = pathlib.Path(os.environ.get("XDG_STATE_HOME", pathlib.Path.home() / ".local" / "state")) / "topic024-test-runner"


@contextlib.contextmanager
def scratch(bench, timeout=None):
    """Yield (topic copy, output dir) with the benchmark replaced and TIMEOUT optionally lowered; both are removed on exit."""
    STATE.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=STATE) as root, tempfile.TemporaryDirectory(dir=STATE) as out:
        root = pathlib.Path(root)
        for part in ["src", "tests", "benches", "scripts"]:
            shutil.copytree(TOPIC / part, root / part)
        (root / "benches" / "dictionary.rs").write_text(bench)
        if timeout is not None:
            script = root / "scripts" / "run.py"
            script.write_text(script.read_text().replace("TIMEOUT=600", f"TIMEOUT={timeout}"))
        yield root, pathlib.Path(out)


def run(root, out, timeout):
    """Run the campaign in its own process group so a wedged benchmark dies with it."""
    proc = subprocess.Popen([sys.executable, str(root / "scripts" / "run.py"), str(out)], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True, start_new_session=True)
    try:
        _, stderr = proc.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        os.killpg(proc.pid, signal.SIGKILL)
        proc.wait()
        raise SystemExit(f"run.py still running after {timeout}s") from None
    return proc.returncode, stderr


def build_artifacts_stay_in_the_output_directory():
    with scratch(CONSTANT_BENCH) as (root, out):
        code, stderr = run(root, out, 300)
        assert code == 0, stderr
        leaked = sorted(p.name for p in root.iterdir() if p.is_file())
        assert not leaked, f"run.py wrote into the crate root: {leaked}"
        assert (out / "dictionary").exists() and (out / "libtopic024.rlib").exists(), sorted(p.name for p in out.iterdir())
    print("ok: compiled artifacts land in the output directory, not the crate root")


def failing_test_output_is_retained():
    with scratch(CONSTANT_BENCH) as (root, out):
        with (root / "src" / "lib.rs").open("a") as lib:
            lib.write(RED_TEST)
        code, stderr = run(root, out, 300)
        assert code != 0, "campaign continued past a failing unit test"
        assert (out / "tests.txt").exists(), f"tests.txt missing after unit-test failure; stderr tail: {stderr[-400:]}"
        text = (out / "tests.txt").read_text()
        assert "deliberate_failure" in text, text[-400:]
        assert "zero_is_a_value_and_ff_prefix_needs_no_successor_key" in text, "tests/contracts.rs did not run: " + text[-400:]
    print("ok: tests.txt retains unit and contract test output on failure")


def failed_rerun_leaves_no_stale_results():
    with scratch(CONSTANT_BENCH) as (root, out):
        code, stderr = run(root, out, 300)
        assert code == 0, stderr
        with (root / "src" / "lib.rs").open("a") as lib:
            lib.write(RED_TEST)
        code, _ = run(root, out, 300)
        assert code != 0
        stale = [name for name in ["schedule.json", "raw.jsonl", "attempts.jsonl", "summary.json"] if (out / name).exists()]
        assert not stale, f"previous campaign artifacts survive a failed rerun: {stale}"
    print("ok: a failed rerun leaves no results from the previous campaign")


def wedged_process_is_recorded_and_aborts():
    with scratch(WEDGED_BENCH, timeout=2) as (root, out):
        code, _ = run(root, out, 90)
        assert code != 0, "campaign reported success with a wedged benchmark"
        attempts = [json.loads(line) for line in (out / "attempts.jsonl").read_text().splitlines()]
        assert len(attempts) == 1 and attempts[0]["returncode"] == "timeout", attempts
    print("ok: wedged benchmark recorded as a timeout attempt and campaign aborted")


def cargo_bench_accepts_user_arguments():
    proc = subprocess.run(["cargo", "bench", "-p", "topic024-tries-radix-trees-and-art", "--bench", "dictionary", "--", "radix", "tiny"], cwd=CARGO_ROOT, text=True, capture_output=True, check=True)
    assert '"case":"tiny"' in proc.stdout, "cargo bench -- radix tiny produced no measurement:\n" + proc.stdout[-300:] + proc.stderr[-300:]
    print("ok: cargo bench --bench dictionary -- radix tiny runs the requested cell")


for scenario in [build_artifacts_stay_in_the_output_directory, failing_test_output_is_retained, failed_rerun_leaves_no_stale_results, wedged_process_is_recorded_and_aborts, cargo_bench_accepts_user_arguments]:
    if len(sys.argv) == 1 or scenario.__name__ in sys.argv[1:]:
        scenario()
