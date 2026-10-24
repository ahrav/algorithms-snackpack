"""Smoke checks for run.py: stale campaigns, timeout retention, contract tests, artifact placement, cargo bench arguments.

Each campaign scenario runs run.py in a scratch copy of the topic whose benchmark is replaced by a small stub, so a
campaign takes seconds. The cargo bench scenarios run the real benchmark for one cell. run.py requires Linux
(lscpu, sched_getaffinity), so these checks do too. Run from anywhere: python3 measurements/test_runner.py [scenario...]
"""

import contextlib
import os
import pathlib
import shutil
import signal
import subprocess
import sys
import tempfile

TOPIC = pathlib.Path(__file__).resolve().parent.parent
CARGO_ROOT = TOPIC.parent.parent
PACKAGE = "topic025-finite-state-transducers"
ROOT_ARTIFACTS = ["tests-bin", "contract-bin", "libalgorithms_fst.rlib", "portfolio"]
ROW = (
    '{{\\"candidate\\":\\"{}\\",\\"case\\":\\"{}\\",\\"ns_per_unit\\":1.0,\\"checksum\\":7,'
    '\\"states\\":0,\\"arcs\\":0,\\"heap_capacity_bytes\\":0,\\"units\\":1}}'
)
CONSTANT_BENCH = (
    "fn main() {\n"
    "    let a: Vec<String> = std::env::args().collect();\n"
    '    println!("' + ROW + '", a[1], a[2]);\n'
    "}\n"
)
WEDGED_BENCH = "fn main() {\n    std::thread::sleep(std::time::Duration::from_secs(3600));\n}\n"
RED_TEST = "\n#[test]\nfn deliberate_failure() {\n    panic!(\"red\");\n}\n"


@contextlib.contextmanager
def scratch(bench, timeout=None):
    """Yield a topic copy with the benchmark source replaced and the per-command timeout optionally lowered."""
    with tempfile.TemporaryDirectory() as root:
        root = pathlib.Path(root)
        for part in ["src", "tests", "benches"]:
            shutil.copytree(TOPIC / part, root / part)
        shutil.copy(TOPIC / "run.py", root / "run.py")
        (root / "benches" / "portfolio.rs").write_text(bench)
        if timeout is not None:
            script = root / "run.py"
            script.write_text(script.read_text().replace("timeout=60", f"timeout={timeout}"))
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
        before = sorted(p.name for p in raw.iterdir())
        summary = (raw / "summary.json").read_bytes()
        code, stderr = run(root, 300)
        assert code != 0, "run.py started a second campaign into a raw/ directory that already holds one"
        assert "raw" in stderr, stderr[-400:]
        assert sorted(p.name for p in raw.iterdir()) == before, "refused rerun changed raw/ contents"
        assert (raw / "summary.json").read_bytes() == summary, "refused rerun rewrote summary.json"
    print(f"ok: a rerun over {len(before)} retained files refuses without touching them")


def build_artifacts_stay_inside_raw():
    with scratch(CONSTANT_BENCH) as root:
        code, stderr = run(root, 300)
        assert code == 0, stderr
        leaked = [name for name in ROOT_ARTIFACTS if (root / name).exists()]
        assert not leaked, f"build artifacts written beside the sources: {leaked}"
        missing = [name for name in ROOT_ARTIFACTS if not (root / "raw" / name).exists()]
        assert not missing, f"build artifacts absent from raw/: {missing}"
    print("ok: compiled test, library and benchmark artifacts live in raw/")


def failing_contract_test_is_run_and_retained():
    with scratch(CONSTANT_BENCH) as root:
        with (root / "tests" / "contract.rs").open("a") as contract:
            contract.write(RED_TEST)
        code, stderr = run(root, 300)
        assert code != 0, "campaign continued past a failing contract test"
        tests = root / "raw" / "tests.txt"
        assert tests.exists(), f"tests.txt missing after contract-test failure; stderr tail: {stderr[-400:]}"
        assert "generated_differential_and_arrival_order" in tests.read_text(), "unit tests did not run: " + tests.read_text()[-400:]
        failures = [f.read_text() for f in (root / "raw").glob("failure-*.txt")]
        assert any("contract-bin" in f and "deliberate_failure" in f for f in failures), f"contract-test failure not retained; stderr tail: {stderr[-400:]}"
    print("ok: tests.txt retains unit test output, the contract failure is retained, and the campaign aborts")


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


def cargo_bench_defaults_candidate_and_case():
    out = bench()
    assert '"candidate":"pushed","case":"tiny"' in out, "cargo bench with no arguments did not run the default cell:\n" + out[-300:]
    out = bench("flat")
    assert '"candidate":"flat","case":"tiny"' in out, "cargo bench -- flat did not default the case:\n" + out[-300:]
    print("ok: cargo bench --bench portfolio runs pushed/tiny by default and -- flat runs flat/tiny")


def build_checksum_discriminates():
    for name in ["flat", "pushed"]:
        out = bench(name, "build")
        assert f'"candidate":"{name}","case":"build"' in out, out[-300:]
        assert '"checksum":0,' not in out, f"{name} build checksum is 0, so cross-candidate agreement is vacuous:\n{out[-300:]}"
    print("ok: build probe returns a nonzero output for flat and pushed")


for scenario in [rerun_refuses_existing_campaign, build_artifacts_stay_inside_raw, failing_contract_test_is_run_and_retained, timed_out_command_is_recorded_and_aborts, cargo_bench_defaults_candidate_and_case, build_checksum_discriminates]:
    if len(sys.argv) == 1 or scenario.__name__ in sys.argv[1:]:
        scenario()
