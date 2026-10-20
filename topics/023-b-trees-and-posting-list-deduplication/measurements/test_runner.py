"""Smoke checks for scripts/run.py: campaign reruns, test-output retention, wedged processes, checksum drift, cargo bench arguments.

Every scenario runs run.py against a scratch copy of the topic whose benchmark is replaced by a small stub, so a
campaign takes seconds. Only the cargo bench scenario runs the real benchmark, for one cell. run.py itself requires
Linux (lscpu, taskset, /proc/cpuinfo), so these checks do too.
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
ROW = '{{\\"ns\\":1,\\"checksum\\":{},\\"logical_bytes\\":0,\\"pairs\\":0,\\"distinct\\":0,\\"queries\\":64,\\"repeats\\":1,\\"build\\":false}}'
CONSTANT_BENCH = "fn main() {\n    println!(\"" + ROW + "\", 7);\n}\n"
# run.py schedules 9 cells x 7 labels = 63 processes per block, so n / 63 is the block index.
DRIFT_BENCH = (
    "fn main() {\n"
    "    let counter = std::env::current_exe().unwrap().with_file_name(\"counter\");\n"
    "    let n: u64 = std::fs::read_to_string(&counter).ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0);\n"
    "    std::fs::write(&counter, (n + 1).to_string()).unwrap();\n"
    "    println!(\"" + ROW + "\", n / 63);\n"
    "}\n"
)
WEDGED_BENCH = "fn main() {\n    std::thread::sleep(std::time::Duration::from_secs(3600));\n}\n"


@contextlib.contextmanager
def scratch(bench, timeout=None):
    """Yield (topic copy, output dir) with the benchmark source replaced and TIMEOUT optionally lowered; both are removed on exit."""
    with tempfile.TemporaryDirectory() as root, tempfile.TemporaryDirectory() as out:
        root = pathlib.Path(root)
        for part in ["src", "tests", "benches", "scripts"]:
            shutil.copytree(TOPIC / part, root / part)
        (root / "benches" / "postings.rs").write_text(bench)
        if timeout is not None:
            script = root / "scripts" / "run.py"
            script.write_text(script.read_text().replace("TIMEOUT = 600", f"TIMEOUT = {timeout}"))
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


def rerun_keeps_one_campaign():
    with scratch(CONSTANT_BENCH) as (root, out):
        for _ in range(2):
            code, stderr = run(root, out, 300)
            assert code == 0, stderr
        schedule = json.loads((out / "schedule.json").read_text())
        attempts = [json.loads(line) for line in (out / "attempts.jsonl").read_text().splitlines()]
        assert len((out / "raw.jsonl").read_text().splitlines()) == len(schedule)
        assert len(attempts) == len(schedule), f"attempts.jsonl holds {len(attempts)} records for a {len(schedule)}-process campaign"
        assert [{k: a[k] for k in ("block", "cell", "label", "candidate")} for a in attempts] == schedule
    print(f"ok: {len(attempts)} attempts match the current schedule after rerun")


def failing_test_output_is_retained():
    with scratch(CONSTANT_BENCH) as (root, out):
        with (root / "src" / "lib.rs").open("a") as lib:
            lib.write("\n#[cfg(test)]\nmod red {\n    #[test]\n    fn deliberate_failure() {\n        panic!(\"red\");\n    }\n}\n")
        code, stderr = run(root, out, 300)
        assert code != 0, "campaign continued past a failing unit test"
        assert (out / "tests.txt").exists(), f"tests.txt missing after unit-test failure; stderr tail: {stderr[-400:]}"
        text = (out / "tests.txt").read_text()
        assert "deliberate_failure" in text, text[-400:]
        assert "running_example_and_extremes" in text, "tests/contracts.rs did not run: " + text[-400:]
    print("ok: tests.txt retains unit and contract test output on failure")


def wedged_process_is_recorded_and_aborts():
    with scratch(WEDGED_BENCH, timeout=2) as (root, out):
        code, _ = run(root, out, 90)
        assert code != 0, "campaign reported success with a wedged benchmark"
        attempts = [json.loads(line) for line in (out / "attempts.jsonl").read_text().splitlines()]
        assert len(attempts) == 1 and attempts[0]["returncode"] == "timeout", attempts
    print("ok: wedged benchmark recorded as a timeout attempt and campaign aborted")


def cross_block_checksum_drift_is_rejected():
    with scratch(DRIFT_BENCH) as (root, out):
        code, stderr = run(root, out, 300)
        assert code != 0 and "checksum" in stderr, f"cross-block checksum drift accepted (exit {code}); stderr tail: {stderr[-300:]}"
        assert not (out / "summary.json").exists(), "summary.json written despite checksum drift"
    print("ok: cross-block checksum drift aborts the campaign")


def cargo_bench_accepts_user_arguments():
    proc = subprocess.run(["cargo", "bench", "-p", "topic023-b-trees-and-posting-list-deduplication", "--bench", "postings", "--", "tree", "tiny"], cwd=CARGO_ROOT, text=True, capture_output=True, check=True)
    assert '"ns":' in proc.stdout, "cargo bench -- tree tiny produced no measurement:\n" + proc.stdout[-300:] + proc.stderr[-300:]
    print("ok: cargo bench --bench postings -- tree tiny runs the requested cell")


for scenario in [rerun_keeps_one_campaign, failing_test_output_is_retained, wedged_process_is_recorded_and_aborts, cross_block_checksum_drift_is_rejected, cargo_bench_accepts_user_arguments]:
    if len(sys.argv) == 1 or scenario.__name__ in sys.argv[1:]:
        scenario()
