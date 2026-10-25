#!/usr/bin/env python3
"""Behavioral checks for run_campaign.py against a planned stub binary.

The stub prints the same `key=value` line as the compare example. A plan maps
`<candidate>#<occurrence>` (occurrence 0 is the warmup) to an action: report a
specific timing, exit non-zero, or hang.
Run from anywhere: python3 topics/028-work-stealing-deques/scripts/test_run_campaign.py
"""
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import time
import unittest

RUNNER = pathlib.Path(__file__).resolve().with_name("run_campaign.py")
STUB = """#!/usr/bin/env python3
import json, os, sys, time
args = dict(zip(sys.argv[1::2], sys.argv[2::2]))
candidate = args["--candidate"]
with open(os.environ["STUB_LOG"], "a+") as log:
    log.seek(0)
    occurrence = sum(1 for line in log if line.split()[0] == candidate)
    log.write(candidate + " " + " ".join(sys.argv[1:]) + "\\n")
action = json.load(open(os.environ["STUB_PLAN"])).get(candidate + "#" + str(occurrence), {})
if action.get("replace_binary"):
    lines = ["#!/usr/bin/env python3", "import sys", "a = dict(zip(sys.argv[1::2], sys.argv[2::2]))",
             "print('candidate=%s workers=%s tasks=%s leaf_work=%s elapsed_ns=5' % (a['--candidate'], a['--workers'], a['--tasks'], a['--leaf-work']))"]
    with open(os.environ["STUB_BINARY"], "w") as replacement:
        replacement.write(chr(10).join(lines) + chr(10))
if action.get("rewrite_file"):
    with open(action["rewrite_file"][0], "w") as target:
        target.write(action["rewrite_file"][1])
if action.get("hang"):
    print("partial output before hang", flush=True)
    print("partial stderr", file=sys.stderr, flush=True)
    time.sleep(30)
if action.get("fail"):
    print("stub failure", file=sys.stderr)
    sys.exit(3)
if action.get("garbage"):
    sys.stdout.buffer.write(bytes([0xff, 0xfe]) + b"garbage")
    sys.stdout.flush()
    sys.exit(2)
base = {"global_fifo": 1000, "locked_steal_balanced": 900, "locked_steal_skewed": 1100}[candidate]
print("candidate=%s workers=%s tasks=%s leaf_work=%s elapsed_ns=%s" % (
    action.get("echo_candidate", candidate), args["--workers"], args["--tasks"], args["--leaf-work"], action.get("elapsed_ns", base)))
"""


SOURCE_FILES = [
    "Cargo.lock", "Cargo.toml", "topics/028-work-stealing-deques/Cargo.toml",
    "topics/028-work-stealing-deques/examples/compare.rs", "topics/028-work-stealing-deques/scripts/run_campaign.py",
    "topics/028-work-stealing-deques/src/lib.rs", "topics/028-work-stealing-deques/tests/contracts.rs",
]


def clean_repository(root):
    """Commit a copy of the runner and placeholder frozen inputs so the runner sees a clean, identified source tree."""
    repo = root / "repo"
    for name in SOURCE_FILES:
        path = repo / name
        path.parent.mkdir(parents=True, exist_ok=True)
        if name.endswith("run_campaign.py"):
            path.write_text(RUNNER.read_text())
        elif name == "Cargo.toml":
            path.write_text("[workspace]" + chr(10) + "members = []" + chr(10) + "resolver = \"2\"" + chr(10))
        else:
            path.write_text(name + chr(10))
    (repo / ".gitignore").write_text("/target/" + chr(10))
    subprocess.run(["git", "init", "-q", repo], check=True)
    subprocess.run(["git", "-C", str(repo), "add", "."], check=True)
    subprocess.run(["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "frozen"], check=True)
    return repo


FAKE_RUSTC = """#!/bin/sh
case "$1" in
  -Vv) echo 'rustc 0.0.0-stub'; echo 'release: 0.0.0-stub' ;;
  --print) printf 'aarch64-unknown-linux-gnu\\nx86_64-unknown-linux-gnu\\n' ;;
esac
"""


def fake_rustc(root):
    path = root / "fake-rustc"
    path.write_text(FAKE_RUSTC)
    path.chmod(0o755)
    return path


def stub_build(stub, executable, running_args="", compiler=None, env_prefix="", ansi=False):
    """A stub build command: install the stub as the example, report it as Cargo would, and print the verbose
    Running line Cargo prints, naming the compiler an override would select; `ansi` colours it as `--color always` does."""
    compiler = compiler or '${RUSTC:-${CARGO_BUILD_RUSTC:-$FAKE_RUSTC}}'
    compiler = f"{env_prefix} {compiler}".strip()
    message = json.dumps({"reason": "compiler-artifact", "target": {"name": "compare", "kind": ["example"]}, "executable": executable})
    label = "\\033[1m\\033[32m     Running\\033[0m " if ansi else "     Running "
    running = label + "\\`" + f"{compiler} --crate-name compare --edition=2024 examples/compare.rs {running_args} -C opt-level=3" + "\\`"
    return f"mkdir -p \"$(dirname {executable})\" && cp {stub} {executable} && echo '{message}' && printf '%b\\n' \"{running}\" >&2"


def run_campaign_module():
    """Import the runner under test as a module."""
    import importlib
    sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
    try:
        return importlib.import_module("run_campaign")
    finally:
        sys.path.pop(0)


def start_campaign(plan, timeout=None, repetitions=8, toolchain=None, dirty=False, extra_env=None, build=None, skip_build=False):
    """Prepare a scratch repository whose stub build command installs the planned stub as the built example.

    Returns (command, env, output path, built binary path, stub log). `build` overrides the stub build command.
    """
    root = pathlib.Path(tempfile.mkdtemp(prefix="topic028-campaign-"))
    repo = clean_repository(root)
    if dirty:
        (repo / "topics/028-work-stealing-deques/build.rs").write_text("fn main() {}" + chr(10))
    runner = repo / "topics/028-work-stealing-deques/scripts/run_campaign.py"
    stub = root / "stub.py"
    stub.write_text(STUB)
    stub.chmod(0o755)
    rustc_stub = fake_rustc(root)
    binary = repo / "target/release/examples/compare"
    if build is None:
        build = stub_build(stub, "target/release/examples/compare")
    (root / "plan.json").write_text(json.dumps(plan))
    log = root / "stub.log"
    out = root / "campaign.json"
    command = [sys.executable, str(runner), "--source", str(repo), "--build-command", build, "--binary", "target/release/examples/compare", "--repetitions", str(repetitions), "--out", str(out)]
    if skip_build:
        binary.parent.mkdir(parents=True, exist_ok=True)
        binary.write_text(STUB)
        binary.chmod(0o755)
        command += ["--skip-build"]
    if timeout is not None:
        command += ["--timeout", str(timeout)]
    if toolchain is not None:
        command += ["--toolchain", toolchain]
    env = dict(os.environ, STUB_PLAN=str(root / "plan.json"), STUB_LOG=str(log), STUB_BINARY=str(binary), FAKE_RUSTC=str(rustc_stub), **(extra_env or {}))
    return command, env, out, binary, log


def run_campaign(plan, timeout=None, repetitions=8, toolchain=None, dirty=False, extra_env=None, build=None, skip_build=False):
    """Run the runner to completion; return (completed process, output path, built binary path)."""
    command, env, out, binary, _ = start_campaign(plan, timeout, repetitions, toolchain, dirty, extra_env, build, skip_build)
    return subprocess.run(command, env=env, text=True, capture_output=True, timeout=300), out, binary


def run_campaign_killed_after(plan, invocations):
    """Kill the runner once the stub has been invoked `invocations` times; return the output path."""
    command, env, out, _, log = start_campaign(plan, timeout=60)
    with subprocess.Popen(command, env=env, text=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) as process:
        deadline = time.time() + 60
        while time.time() < deadline and not (log.exists() and len(log.read_text().splitlines()) >= invocations):
            time.sleep(0.05)
        time.sleep(0.5)
        process.kill()
    return out


def summaries_by_name(document):
    return {s["workload"]["name"]: s for s in document["summaries"]}


ARM_CPUINFO = """processor\t: 0
BogoMIPS\t: 2100.00
Features\t: fp asimd
CPU implementer\t: 0x41
CPU architecture: 8
CPU variant\t: 0x1
CPU part\t: 0xd40
CPU revision\t: 1

processor\t: 1
BogoMIPS\t: 2100.00
CPU implementer\t: 0x41
CPU part\t: 0xd40
"""
X86_CPUINFO = """processor\t: 0
vendor_id\t: GenuineIntel
model name\t: Intel(R) Xeon(R) Platinum 8488C
stepping\t: 8

processor\t: 1
model name\t: Intel(R) Xeon(R) Platinum 8488C
"""


class WorktreeStatusTests(unittest.TestCase):
    def test_untracked_build_inputs_make_the_worktree_dirty(self):
        sys.path.insert(0, str(RUNNER.parent))
        import run_campaign
        repo = pathlib.Path(tempfile.mkdtemp(prefix="topic028-repo-"))
        subprocess.run(["git", "init", "-q", repo], check=True)
        (repo / ".gitignore").write_text("target/\n")
        (repo / "src.rs").write_text("fn main() {}\n")
        subprocess.run(["git", "-C", str(repo), "add", "."], check=True)
        subprocess.run(["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "init"], check=True)
        (repo / "target").mkdir()
        (repo / "target" / "artifact").write_text("ignored")
        self.assertEqual(run_campaign.worktree_status(repo, repo / "campaign.json"), [])
        (repo / "campaign.json").write_text("{}")
        self.assertEqual(run_campaign.worktree_status(repo, repo / "campaign.json"), [])
        (repo / "build.rs").write_text("fn main() {}\n")
        self.assertEqual(run_campaign.worktree_status(repo, repo / "campaign.json"), ["?? build.rs"])


class CpuModelTests(unittest.TestCase):
    def read(self, text):
        sys.path.insert(0, str(RUNNER.parent))
        import run_campaign
        path = pathlib.Path(tempfile.mkdtemp(prefix="topic028-cpuinfo-")) / "cpuinfo"
        path.write_text(text)
        return run_campaign.cpu_model(path)

    def test_arm_identity_keeps_implementer_part_variant_and_revision(self):
        self.assertEqual(self.read(ARM_CPUINFO), "CPU implementer 0x41, CPU variant 0x1, CPU part 0xd40, CPU revision 1")

    def test_x86_identity_is_the_model_name(self):
        self.assertEqual(self.read(X86_CPUINFO), "Intel(R) Xeon(R) Platinum 8488C")


class RunCampaignTests(unittest.TestCase):
    def test_summary_applies_the_declared_paired_three_percent_rule(self):
        process, out, _ = run_campaign({"locked_steal_balanced#1": {"elapsed_ns": 980}})
        self.assertEqual(process.returncode, 0, process.stderr)
        summaries = summaries_by_name(json.loads(out.read_text()))
        mixed = summaries["tiny_balanced"]
        self.assertEqual(mixed["lowest_median"], "locked_steal_balanced")
        self.assertEqual(sorted(mixed["paired_ratios"]), [0.9] * 7 + [0.98])
        self.assertEqual(mixed["selected"], "unresolved")
        decided = summaries["tiny_skewed"]
        self.assertEqual(decided["paired_ratios"], [1.1] * 8)
        self.assertEqual(decided["selected"], "global_fifo")
        self.assertEqual(summaries["medium_balanced"]["selected"], "locked_steal_balanced")

    def test_failed_and_timed_out_processes_are_retained_and_the_campaign_continues(self):
        process, out, _ = run_campaign({"global_fifo#2": {"fail": True}, "locked_steal_balanced#3": {"hang": True}}, timeout=1)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(sorted(r["status"] for r in document["rows"] if r["status"] != "ok"), ["failed", "timeout"])
        failed = next(r for r in document["rows"] if r["status"] == "failed")
        self.assertEqual(failed["returncode"], 3)
        self.assertIn("stub failure", failed["stderr"])
        self.assertNotIn("elapsed_ns", failed)
        timed_out = next(r for r in document["rows"] if r["status"] == "timeout")
        self.assertEqual(timed_out["stdout"], "partial output before hang")
        self.assertEqual(timed_out["stderr"], "partial stderr")
        self.assertEqual(len([r for r in document["rows"] if r["workload"]["name"] == "coarse_skewed"]), 32)
        affected = summaries_by_name(document)["tiny_balanced"]
        self.assertEqual(affected["selected"], "unresolved")
        self.assertEqual(affected["incomplete_repetitions"], 2)
        self.assertEqual(affected["aa_incomplete_repetitions"], 0)
        self.assertEqual(affected["failures"], 2)
        self.assertEqual(document["status"], "complete")

    def test_zero_or_mismatched_output_is_a_failed_row_and_never_divides(self):
        process, out, _ = run_campaign({"global_fifo#2": {"elapsed_ns": 0}, "locked_steal_balanced#2": {"echo_candidate": "global_fifo"}})
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        failed = {r["reason"][:20]: r for r in document["rows"] if r["status"] == "failed"}
        self.assertEqual(sorted(failed), ["echoed arguments {'c", "elapsed_ns '0' is no"])
        self.assertTrue(all("elapsed_ns" not in r for r in failed.values()))
        affected = summaries_by_name(document)["tiny_balanced"]
        self.assertEqual(affected["incomplete_repetitions"], 2)
        self.assertEqual(affected["selected"], "unresolved")

    def test_a_one_sided_aa_control_withholds_selection(self):
        plan = {f"global_fifo#{occurrence}": {"elapsed_ns": 900} for occurrence in [1, 6, 8, 11, 13, 18, 20, 23]}
        process, out, _ = run_campaign(plan)
        self.assertEqual(process.returncode, 0, process.stderr)
        summaries = summaries_by_name(json.loads(out.read_text()))
        biased = summaries["tiny_balanced"]
        self.assertEqual(biased["aa_ratios"], [0.9] * 8)
        self.assertEqual(biased["aa_null_check"], "failed")
        self.assertEqual(biased["selected"], "unresolved")
        clean = summaries["tiny_skewed"]
        self.assertEqual(clean["aa_null_check"], "passed")
        self.assertEqual(clean["selected"], "global_fifo")

    def test_toolchain_flag_routes_the_compiler_probes(self):
        process, out, _ = run_campaign({}, repetitions=4, toolchain="topic028-no-such-toolchain", build="rustup run topic028-no-such-toolchain true")
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        environment = document["environment"]
        self.assertEqual(environment["toolchain"], "topic028-no-such-toolchain")
        self.assertIsNone(environment["rustc"])
        self.assertIsNone(environment["cargo"])
        self.assertEqual(document["build"]["status"], "failed")
        self.assertEqual(document["status"], "invalid: build failed")
        self.assertTrue(all(s["selected"] == "unresolved" for s in document["summaries"]))
        command, env, out, _, _ = start_campaign({}, repetitions=4)
        subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        document = json.loads(out.read_text())
        self.assertIsNone(document["environment"]["toolchain"])
        self.assertEqual([c["path"] for c in document["build"]["compiler_chain"] if c["role"] == "compiler"], [env["FAKE_RUSTC"]])
        self.assertIn("0.0.0-stub", document["environment"]["rustc"])

    def test_compiler_probes_honour_the_source_checkouts_toolchain_file(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4)
        repo = binary.parents[3]
        (repo / "rust-toolchain.toml").write_text('[toolchain]' + chr(10) + 'channel = "topic028-no-such-toolchain"' + chr(10))
        subprocess.run(["git", "-C", str(repo), "add", "rust-toolchain.toml"], check=True)
        subprocess.run(["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "toolchain"], check=True)
        command = [("rustc -Vv >/dev/null && " + c) if c.startswith("mkdir -p") else c for c in command]
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertIsNone(document["environment"]["rustc"])
        self.assertEqual(document["build"]["status"], "failed")
        self.assertEqual(document["status"], "invalid: build failed")

    def test_only_the_eight_repetition_protocol_can_select(self):
        process, out, _ = run_campaign({}, repetitions=4)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertFalse(document["protocol_repetitions"])
        for summary in document["summaries"]:
            self.assertEqual(summary["selected"], "unresolved", summary["workload"]["name"])
            self.assertEqual(len(summary["paired_ratios"]), 4)
        self.assertEqual(summaries_by_name(document)["medium_balanced"]["lowest_median"], "locked_steal_balanced")

    def test_an_incomplete_control_pair_invalidates_the_cell(self):
        process, out, _ = run_campaign({"global_fifo#1": {"fail": True}})
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        failed = [r for r in document["rows"] if r["status"] == "failed"]
        self.assertEqual([(r["arm"], r["label"]) for r in failed], [("control", "A")])
        affected = summaries_by_name(document)["tiny_balanced"]
        self.assertEqual(affected["incomplete_repetitions"], 0)
        self.assertEqual(affected["aa_incomplete_repetitions"], 1)
        self.assertEqual(affected["selected"], "unresolved")
        self.assertEqual(summaries_by_name(document)["medium_balanced"]["selected"], "locked_steal_balanced")

    def test_a_failed_warmup_invalidates_the_cell(self):
        process, out, _ = run_campaign({"global_fifo#0": {"fail": True}})
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual([w["status"] for w in document["warmups"]][:2], ["failed", "ok"])
        affected = summaries_by_name(document)["tiny_balanced"]
        self.assertEqual(affected["warmup_failures"], 1)
        self.assertEqual(affected["selected"], "unresolved")
        self.assertEqual(summaries_by_name(document)["tiny_skewed"]["warmup_failures"], 0)
        self.assertEqual(summaries_by_name(document)["tiny_skewed"]["selected"], "global_fifo")

    def test_observations_are_checkpointed_before_the_campaign_completes(self):
        out = run_campaign_killed_after({"locked_steal_balanced#2": {"hang": True}}, invocations=7)
        partial = json.loads(out.read_text())
        self.assertEqual(partial["status"], "running")
        self.assertEqual(len(partial["warmups"]), 2)
        self.assertEqual([r["status"] for r in partial["rows"]], ["ok"] * 4 + ["running"])
        started = partial["rows"][-1]
        self.assertEqual((started["candidate"], started["arm"], started["repetition"]), ("locked_steal_balanced", "treatment", 1))
        self.assertIn("command", started)

    def test_invalid_utf8_output_is_a_failed_row_and_the_campaign_continues(self):
        process, out, _ = run_campaign({"global_fifo#1": {"garbage": True}}, repetitions=4)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        failed = [r for r in document["rows"] if r["status"] == "failed"]
        self.assertEqual(len(failed), 1)
        self.assertEqual(failed[0]["returncode"], 2)
        self.assertIn("garbage", failed[0]["stdout"])
        self.assertEqual(len(document["rows"]), 5 * 4 * 4)
        self.assertEqual(document["status"], "complete")

    def test_the_campaign_runs_from_a_pinned_copy_of_the_binary(self):
        process, out, binary = run_campaign({"global_fifo#3": {"replace_binary": True}}, repetitions=4)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual({r["elapsed_ns"] for r in document["rows"]}, {900, 1000, 1100})
        self.assertNotEqual(hashlib.sha256(binary.read_bytes()).hexdigest(), document["environment"]["binary_sha256"])
        self.assertEqual(hashlib.sha256(STUB.encode()).hexdigest(), document["environment"]["binary_sha256"])
        self.assertEqual(document["environment"]["binary_sha256_final"], document["environment"]["binary_sha256"])
        self.assertEqual(document["status"], "complete")

    def test_a_binary_that_cannot_start_is_a_failed_row(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4, skip_build=True)
        binary.chmod(0o644)
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual({r["status"] for r in document["warmups"] + document["rows"]}, {"failed"})
        self.assertIn("launch failed", document["rows"][0]["reason"])
        self.assertEqual(len(document["rows"]), 5 * 4 * 4)
        self.assertTrue(all(s["selected"] == "unresolved" for s in document["summaries"]))
        self.assertEqual(document["status"], "invalid: binary provenance unverified")

    def test_a_missing_binary_cannot_be_pinned(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4, skip_build=True)
        binary.unlink()
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertIsNone(document["environment"]["binary_sha256"])
        self.assertIsNone(document["environment"]["binary_copy"])
        self.assertEqual({r["status"] for r in document["rows"]}, {"failed"})
        self.assertEqual(document["status"], "invalid: binary could not be pinned")

    def test_control_and_treatment_pairs_alternate_schedule_slots(self):
        process, out, _ = run_campaign({})
        self.assertEqual(process.returncode, 0, process.stderr)
        rows = [r for r in json.loads(out.read_text())["rows"] if r["workload"]["name"] == "tiny_balanced"]
        by_repetition = {}
        for row in rows:
            by_repetition.setdefault(row["repetition"], []).append((row["arm"], row["label"]))
        self.assertEqual(by_repetition[0], [("control", "A"), ("treatment", "global_fifo"), ("control", "B"), ("treatment", "locked_steal_balanced")])
        self.assertEqual(by_repetition[1], [("treatment", "locked_steal_balanced"), ("control", "B"), ("treatment", "global_fifo"), ("control", "A")])
        self.assertEqual(by_repetition[2], [("control", "B"), ("treatment", "locked_steal_balanced"), ("control", "A"), ("treatment", "global_fifo")])
        self.assertEqual(by_repetition[3], [("treatment", "global_fifo"), ("control", "A"), ("treatment", "locked_steal_balanced"), ("control", "B")])
        for repetition in range(4, 8):
            self.assertEqual(by_repetition[repetition], by_repetition[repetition - 4])
        for label in ["global_fifo", "locked_steal_balanced", "A", "B"]:
            slots = sorted(order.index(next(entry for entry in order if entry[1] == label)) for order in by_repetition.values())
            self.assertEqual(slots, [0, 0, 1, 1, 2, 2, 3, 3], label)

    def test_identical_artifact_aa_arm_flows_through_the_same_paths(self):
        process, out, _ = run_campaign({})
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        controls = [r for r in document["rows"] if r["arm"] == "control"]
        self.assertEqual(len(controls), 5 * 8 * 2)
        self.assertEqual(sorted({r["label"] for r in controls}), ["A", "B"])
        self.assertEqual({r["candidate"] for r in controls}, {"global_fifo"})
        orders = {}
        for row in controls:
            orders.setdefault((row["workload"]["name"], row["repetition"]), []).append(row["label"])
        self.assertEqual(sorted({tuple(v) for v in orders.values()}), [("A", "B"), ("B", "A")])
        summary = summaries_by_name(document)["tiny_balanced"]
        self.assertEqual(summary["aa_ratios"], [1.0] * 8)

    def test_environment_records_artifact_and_toolchain_identity(self):
        process, out, binary = run_campaign({})
        self.assertEqual(process.returncode, 0, process.stderr)
        environment = json.loads(out.read_text())["environment"]
        self.assertEqual(environment["binary_sha256"], hashlib.sha256(binary.read_bytes()).hexdigest())
        self.assertEqual(environment["binary"], str(binary))
        for key in ["source_commit", "worktree_dirty", "source_sha256", "rustc", "cargo", "rustflags", "cpu_model", "available_cpus", "affinity", "uname", "hostname"]:
            self.assertIn(key, environment)
        self.assertIn("topics/028-work-stealing-deques/src/lib.rs", environment["source_sha256"])
        self.assertEqual(len(environment["affinity"]), environment["available_cpus"])
        self.assertEqual(environment["affinity"], sorted(environment["affinity"]))
        self.assertFalse(environment["worktree_dirty"])
        self.assertTrue(all(environment["source_sha256"].values()))

    def test_every_cargo_rustflags_source_is_recorded(self):
        flags = {"CARGO_ENCODED_RUSTFLAGS": "-C\x1ftarget-cpu=native", "CARGO_BUILD_RUSTFLAGS": "-C opt-level=2"}
        process, out, _ = run_campaign({}, repetitions=4, extra_env=dict(flags, RUSTFLAGS=""))
        self.assertEqual(process.returncode, 0, process.stderr)
        recorded = json.loads(out.read_text())["environment"]["rustflags"]
        self.assertEqual(recorded["RUSTFLAGS"], "")
        self.assertEqual(recorded["CARGO_ENCODED_RUSTFLAGS"], "-C\x1ftarget-cpu=native")
        self.assertEqual(recorded["CARGO_BUILD_RUSTFLAGS"], "-C opt-level=2")
        self.assertIn("cargo_config_sha256", recorded)

    def test_target_specific_cargo_rustflags_are_recorded(self):
        extra = {"CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS": "-C target-cpu=neoverse-v1", "CARGO_BUILD_TARGET": "aarch64-unknown-linux-gnu"}
        process, out, _ = run_campaign({}, repetitions=4, extra_env=extra)
        self.assertEqual(process.returncode, 0, process.stderr)
        recorded = json.loads(out.read_text())["environment"]["rustflags"]
        self.assertEqual(recorded["target_specific"], {"CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS": "-C target-cpu=neoverse-v1"})
        self.assertEqual(recorded["CARGO_BUILD_TARGET"], "aarch64-unknown-linux-gnu")

    def test_host_identity_is_recorded_in_receipt_comparable_form(self):
        process, out, _ = run_campaign({}, repetitions=4)
        self.assertEqual(process.returncode, 0, process.stderr)
        environment = json.loads(out.read_text())["environment"]
        self.assertEqual(environment["uname_a"], subprocess.run(["uname", "-a"], text=True, capture_output=True, check=True).stdout.strip())
        self.assertIn("cpu_identification", environment)
        self.assertEqual(environment["runner_sha256"], hashlib.sha256(RUNNER.read_bytes()).hexdigest())
        self.assertTrue(environment["runner_matches_source"])

    def test_the_runner_builds_the_binary_and_records_the_build(self):
        process, out, binary = run_campaign({}, repetitions=4)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["build"]["status"], "ok")
        self.assertIn("cp ", document["build"]["command"])
        self.assertEqual(document["build"]["cwd"], str(binary.parents[3]))
        self.assertEqual(document["environment"]["binary"], str(binary))
        self.assertEqual(document["environment"]["binary_sha256"], hashlib.sha256(STUB.encode()).hexdigest())
        self.assertEqual(document["status"], "complete")

    def test_a_failed_build_is_an_invalid_campaign_with_no_rows(self):
        process, out, _ = run_campaign({}, repetitions=4, build="echo compiler exploded >&2; exit 7")
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["build"]["status"], "failed")
        self.assertEqual(document["build"]["returncode"], 7)
        self.assertIn("compiler exploded", document["build"]["stderr"])
        self.assertEqual(document["rows"], [])
        self.assertEqual(document["status"], "invalid: build failed")

    def test_the_built_binary_is_resolved_from_cargos_target_directory(self):
        root = pathlib.Path(tempfile.mkdtemp(prefix="topic028-target-"))
        alt = root / "alt-target"
        produced = alt / "release" / "examples" / "compare"
        build = stub_build("\"$STUB_SOURCE\"", str(produced))
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build, extra_env={"CARGO_TARGET_DIR": str(alt)})
        command = [c for c in command if c != "--binary" and c != "target/release/examples/compare"]
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["build"]["target_directory"], str(alt))
        self.assertEqual(document["environment"]["binary"], str(alt / "release/examples/compare"))
        self.assertEqual(document["build"]["status"], "ok")
        self.assertEqual(document["status"], "complete")

    def test_the_built_binary_follows_cargos_compiler_artifact_message(self):
        root = pathlib.Path(tempfile.mkdtemp(prefix="topic028-triple-"))
        produced = root / "alt" / "aarch64-unknown-linux-gnu" / "release" / "examples" / "compare"
        build = stub_build("\"$STUB_SOURCE\"", str(produced), running_args="--target aarch64-unknown-linux-gnu")
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build, extra_env={"CARGO_BUILD_TARGET": "aarch64-unknown-linux-gnu"})
        command = [c for c in command if c != "--binary" and c != "target/release/examples/compare"]
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["build"]["executable"], str(produced))
        self.assertEqual(document["environment"]["binary"], str(produced))
        self.assertEqual(document["status"], "complete")

    def test_a_build_that_reports_no_artifact_is_invalid_without_an_explicit_binary(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build="mkdir -p target/release/examples && cp \"$STUB_SOURCE\" target/release/examples/compare")
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        command = [c for c in command if c != "--binary" and c != "target/release/examples/compare"]
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["build"]["status"], "no artifact")
        self.assertEqual(document["build"]["returncode"], 0)
        self.assertIsNone(document["build"]["executable"])
        self.assertEqual(document["rows"], [])
        self.assertEqual(document["status"], "invalid: build reported no artifact")

    def test_compiler_overrides_are_recorded_and_probed(self):
        root = pathlib.Path(tempfile.mkdtemp(prefix="topic028-rustc-"))
        fake = root / "rustc-stub"
        fake.write_text("#!/bin/sh" + chr(10) + "echo 'rustc 9.9.9-stub'" + chr(10) + "echo 'release: 9.9.9-stub'" + chr(10))
        fake.chmod(0o755)
        wrapper = root / "wrapper"
        wrapper.write_text("#!/bin/sh" + chr(10) + 'exec "$@"' + chr(10))
        wrapper.chmod(0o755)
        process, out, _ = run_campaign({}, repetitions=4, extra_env={"RUSTC": str(fake), "RUSTC_WRAPPER": str(wrapper)})
        self.assertEqual(process.returncode, 0, process.stderr)
        environment = json.loads(out.read_text())["environment"]
        self.assertIn("9.9.9-stub", environment["rustc"])
        self.assertEqual(environment["compiler_overrides"]["RUSTC"], {"value": str(fake), "resolved": str(fake), "sha256": hashlib.sha256(fake.read_bytes()).hexdigest()})
        self.assertEqual(environment["compiler_overrides"]["RUSTC_WRAPPER"]["sha256"], hashlib.sha256(wrapper.read_bytes()).hexdigest())
        self.assertIsNone(environment["compiler_overrides"]["RUSTC_WORKSPACE_WRAPPER"])

    def test_relative_compiler_overrides_resolve_against_the_source_tree(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4)
        repo = binary.parents[3]
        wrapper = repo / "wrapper.sh"
        wrapper.write_text("#!/bin/sh" + chr(10) + 'exec "$@"' + chr(10))
        wrapper.chmod(0o755)
        subprocess.run(["git", "-C", str(repo), "add", "wrapper.sh"], check=True)
        subprocess.run(["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "wrapper"], check=True)
        env["RUSTC_WRAPPER"] = "./wrapper.sh"
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300, cwd=tempfile.mkdtemp(prefix="topic028-cwd-"))
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["environment"]["compiler_overrides"]["RUSTC_WRAPPER"], {"value": "./wrapper.sh", "resolved": str(wrapper), "sha256": hashlib.sha256(wrapper.read_bytes()).hexdigest()})
        self.assertEqual(document["status"], "complete")
        env["RUSTC_WRAPPER"] = "./no-such-wrapper"
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        document = json.loads(out.read_text())
        self.assertIsNone(document["environment"]["compiler_overrides"]["RUSTC_WRAPPER"]["sha256"])
        self.assertEqual(document["status"], "invalid: compiler identity unavailable")

    def test_configuration_form_compiler_overrides_are_recorded_and_validated(self):
        root = pathlib.Path(tempfile.mkdtemp(prefix="topic028-rustc-"))
        fake = root / "rustc-stub"
        fake.write_text("#!/bin/sh" + chr(10) + "echo 'rustc 8.8.8-stub'" + chr(10) + "echo 'release: 8.8.8-stub'" + chr(10))
        fake.chmod(0o755)
        process, out, _ = run_campaign({}, repetitions=4, extra_env={"CARGO_BUILD_RUSTC": str(fake)})
        self.assertEqual(process.returncode, 0, process.stderr)
        environment = json.loads(out.read_text())["environment"]
        self.assertIn("8.8.8-stub", environment["rustc"])
        self.assertEqual(environment["compiler_overrides"]["CARGO_BUILD_RUSTC"]["sha256"], hashlib.sha256(fake.read_bytes()).hexdigest())
        process, out, _ = run_campaign({}, repetitions=4, extra_env={"CARGO_BUILD_RUSTC_WRAPPER": "/no/such/wrapper"})
        document = json.loads(out.read_text())
        self.assertEqual(document["status"], "invalid: compiler identity unavailable")

    def test_a_custom_target_specification_is_hashed(self):
        spec = pathlib.Path(tempfile.mkdtemp(prefix="topic028-target-")) / "custom.json"
        spec.write_text('{"llvm-target": "aarch64-unknown-linux-gnu"}' + chr(10))
        process, out, _ = run_campaign({}, repetitions=4, extra_env={"CARGO_BUILD_TARGET": str(spec)})
        self.assertEqual(process.returncode, 0, process.stderr)
        recorded = json.loads(out.read_text())["environment"]["rustflags"]
        self.assertEqual(recorded["CARGO_BUILD_TARGET"], str(spec))
        self.assertEqual(recorded["target_spec_sha256"], hashlib.sha256(spec.read_bytes()).hexdigest())
        process, out, _ = run_campaign({}, repetitions=4, extra_env={"CARGO_BUILD_TARGET": "/no/such/target.json"})
        self.assertEqual(json.loads(out.read_text())["status"], "invalid: source identity not reproducible")

    def test_an_ignored_cargo_input_makes_the_source_dirty(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4)
        repo = binary.parents[3]
        (repo / ".gitignore").write_text("/target/" + chr(10) + "build.rs" + chr(10) + "notes.txt" + chr(10))
        subprocess.run(["git", "-C", str(repo), "add", ".gitignore"], check=True)
        subprocess.run(["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "ignore"], check=True)
        (repo / "topics/028-work-stealing-deques/notes.txt").write_text("ignored and harmless" + chr(10))
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(json.loads(out.read_text())["status"], "complete")
        (repo / "topics/028-work-stealing-deques/build.rs").write_text("fn main() {}" + chr(10))
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        document = json.loads(out.read_text())
        self.assertEqual(document["environment"]["worktree_status"], ["!! topics/028-work-stealing-deques/build.rs"])
        self.assertEqual(document["status"], "invalid: source identity not reproducible")

    def test_the_effective_compiler_chain_comes_from_cargos_verbose_invocation(self):
        root = pathlib.Path(tempfile.mkdtemp(prefix="topic028-chain-"))
        wrapper = root / "wrapper"
        wrapper.write_text("#!/bin/sh" + chr(10) + 'exec "$@"' + chr(10))
        wrapper.chmod(0o755)
        fake = root / "fake-rustc"
        fake.write_text("#!/bin/sh" + chr(10) + "echo 'rustc 7.7.7-chain'" + chr(10) + "echo 'release: 7.7.7-chain'" + chr(10))
        fake.chmod(0o755)
        spec = root / "custom.json"
        spec.write_text('{"llvm-target": "aarch64-unknown-linux-gnu"}' + chr(10))
        build = stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare", running_args=f"--target {spec}", compiler=f"CARGO_PKG_NAME=compare {wrapper} {fake}")
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build)
        command = [c for c in command if c != "--binary" and c != "target/release/examples/compare"]
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        chain = [c for c in document["build"]["compiler_chain"] if c["role"] in ("wrapper", "compiler")]
        self.assertEqual([c["path"] for c in chain], [str(wrapper), str(fake)])
        self.assertEqual(chain[1]["sha256"], hashlib.sha256(fake.read_bytes()).hexdigest())
        self.assertIn("7.7.7-chain", document["environment"]["rustc"])
        self.assertEqual(document["build"]["target"], str(spec))
        self.assertEqual(document["build"]["target_spec_sha256"], hashlib.sha256(spec.read_bytes()).hexdigest())
        self.assertEqual(document["status"], "complete")

    def test_a_named_custom_target_is_found_through_rust_target_path(self):
        root = pathlib.Path(tempfile.mkdtemp(prefix="topic028-rtp-"))
        (root / "specs").mkdir()
        spec = root / "specs" / "mycpu-unknown-none.json"
        spec.write_text('{"llvm-target": "aarch64-unknown-none"}' + chr(10))
        build = stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare", running_args="--target mycpu-unknown-none")
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build, extra_env={"RUST_TARGET_PATH": str(root / "specs")})
        command = [c for c in command if c != "--binary" and c != "target/release/examples/compare"]
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["build"]["target"], "mycpu-unknown-none")
        self.assertEqual(document["build"]["target_spec_sha256"], hashlib.sha256(spec.read_bytes()).hexdigest())

    def test_ignored_directories_are_inspected_for_cargo_inputs(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4)
        repo = binary.parents[3]
        (repo / ".gitignore").write_text("/target/" + chr(10) + "vendor/" + chr(10) + "scratch/" + chr(10))
        subprocess.run(["git", "-C", str(repo), "add", ".gitignore"], check=True)
        subprocess.run(["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "ignore dirs"], check=True)
        (repo / "scratch").mkdir()
        (repo / "scratch" / "notes.txt").write_text("harmless" + chr(10))
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(json.loads(out.read_text())["status"], "complete")
        (repo / "vendor" / "dep" / "src").mkdir(parents=True)
        (repo / "vendor" / "dep" / "Cargo.toml").write_text("[package]" + chr(10))
        (repo / "vendor" / "dep" / "src" / "lib.rs").write_text("" + chr(10))
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        document = json.loads(out.read_text())
        self.assertEqual(sorted(document["environment"]["worktree_status"]), ["!! vendor/dep/Cargo.toml", "!! vendor/dep/src/lib.rs"])
        self.assertEqual(document["status"], "invalid: source identity not reproducible")

    def test_a_custom_build_without_a_compiler_chain_is_invalid(self):
        message = json.dumps({"reason": "compiler-artifact", "target": {"name": "compare", "kind": ["example"]}, "executable": "target/release/examples/compare"})
        build = f"mkdir -p target/release/examples && cp \"$STUB_SOURCE\" target/release/examples/compare && echo '{message}'"
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build)
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertIsNone(document["build"]["compiler_chain"])
        self.assertEqual(document["status"], "invalid: compiler identity unavailable")
        self.assertTrue(all(s["selected"] == "unresolved" for s in document["summaries"]))

    def test_relative_rust_target_path_resolves_against_the_source_tree(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare", running_args="--target mycpu-unknown-none"))
        repo = binary.parents[3]
        (repo / "specs").mkdir()
        spec = repo / "specs" / "mycpu-unknown-none.json"
        spec.write_text('{"llvm-target": "aarch64-unknown-none"}' + chr(10))
        subprocess.run(["git", "-C", str(repo), "add", "specs"], check=True)
        subprocess.run(["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "specs"], check=True)
        env["STUB_SOURCE"] = str(repo.parent / "stub.py")
        env["RUST_TARGET_PATH"] = "specs"
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300, cwd=tempfile.mkdtemp(prefix="topic028-cwd-"))
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["build"]["target_spec_sha256"], hashlib.sha256(spec.read_bytes()).hexdigest())
        self.assertEqual(document["status"], "complete")

    def test_an_explicit_binary_must_match_the_reported_artifact(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare"))
        repo = binary.parents[3]
        stale = repo / "target" / "stale-compare"
        stale.parent.mkdir(parents=True, exist_ok=True)
        stale.write_text(STUB)
        stale.chmod(0o755)
        command = [("target/stale-compare" if c == "target/release/examples/compare" else c) for c in command]
        env["STUB_SOURCE"] = str(repo.parent / "stub.py")
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["build"]["executable"], "target/release/examples/compare")
        self.assertEqual(document["rows"], [])
        self.assertEqual(document["status"], "invalid: build artifact mismatch")

    def test_an_explicit_binary_must_be_produced_by_the_build(self):
        message_free = "mkdir -p target/release/examples && echo \"     Running \\`" + sys.executable + " --crate-name compare -C opt-level=3\\`\" >&2"
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=message_free)
        binary.parent.mkdir(parents=True, exist_ok=True)
        binary.write_text(STUB)
        binary.chmod(0o755)
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["rows"], [])
        self.assertEqual(document["status"], "invalid: build artifact mismatch")
        self.assertIn("unchanged", document["build"]["reason"])
        rewriting = message_free + " && printf '\\n' >> target/release/examples/compare"
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=rewriting)
        binary.parent.mkdir(parents=True, exist_ok=True)
        binary.write_text(STUB)
        binary.chmod(0o755)
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(json.loads(out.read_text())["status"], "complete")

    def test_the_invoked_linker_joins_the_compiler_chain(self):
        root = pathlib.Path(tempfile.mkdtemp(prefix="topic028-linker-"))
        linker = root / "my-ld"
        linker.write_text("#!/bin/sh" + chr(10) + 'exec "$@"' + chr(10))
        linker.chmod(0o755)
        build = stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare", running_args=f"-C linker={linker}")
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build)
        command = [c for c in command if c != "--binary" and c != "target/release/examples/compare"]
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        roles = [(c["role"], c["path"]) for c in document["build"]["compiler_chain"]]
        self.assertEqual(roles[-1], ("linker", str(linker)))
        self.assertEqual(document["build"]["compiler_chain"][-1]["sha256"], hashlib.sha256(linker.read_bytes()).hexdigest())
        self.assertEqual(document["status"], "complete")

    def test_a_timed_out_build_that_exits_during_cleanup_is_still_recorded(self):
        from unittest import mock
        sys.path.insert(0, str(RUNNER.parent))
        import run_campaign
        with mock.patch.object(run_campaign.os, "killpg", side_effect=ProcessLookupError):
            record = run_campaign.build(tempfile.mkdtemp(prefix="topic028-build-"), "sleep 1", 0.2)
        self.assertEqual(record["status"], "timeout")

    def test_bare_tool_names_resolve_through_a_source_relative_path(self):
        build = stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare", running_args="-C linker=my-ld")
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build)
        repo = binary.parents[3]
        launch = pathlib.Path(tempfile.mkdtemp(prefix="topic028-launch-"))
        for base, body in ((repo, "exec source-ld"), (launch, "exec launch-ld")):
            (base / "tools").mkdir()
            tool = base / "tools" / "my-ld"
            tool.write_text("#!/bin/sh" + chr(10) + body + chr(10))
            tool.chmod(0o755)
        (repo / ".gitignore").write_text("/target/" + chr(10) + "/tools/" + chr(10))
        subprocess.run(["git", "-C", str(repo), "add", ".gitignore"], check=True)
        subprocess.run(["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "tools"], check=True)
        env["STUB_SOURCE"] = str(repo.parent / "stub.py")
        env["PATH"] = "tools:" + env["PATH"]
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300, cwd=launch)
        self.assertEqual(process.returncode, 0, process.stderr)
        linker = json.loads(out.read_text())["build"]["compiler_chain"][-1]
        self.assertEqual(linker["resolved"], str(repo / "tools" / "my-ld"))
        self.assertEqual(linker["sha256"], hashlib.sha256((repo / "tools" / "my-ld").read_bytes()).hexdigest())

    def test_target_search_paths_from_the_invocation_environment_are_honoured(self):
        root = pathlib.Path(tempfile.mkdtemp(prefix="topic028-envspec-"))
        (root / "specs").mkdir()
        spec = root / "specs" / "mycpu-unknown-none.json"
        spec.write_text('{"llvm-target": "aarch64-unknown-none"}' + chr(10))
        build = stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare", running_args="--target mycpu-unknown-none", env_prefix=f"CARGO_PKG_NAME=compare RUST_TARGET_PATH={root / 'specs'}")
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build)
        command = [c for c in command if c != "--binary" and c != "target/release/examples/compare"]
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        env.pop("RUST_TARGET_PATH", None)
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["build"]["invocation_env"]["RUST_TARGET_PATH"], str(root / "specs"))
        self.assertEqual(document["build"]["target_spec_sha256"], hashlib.sha256(spec.read_bytes()).hexdigest())
        self.assertEqual(document["status"], "complete")

    def test_a_named_target_that_is_neither_builtin_nor_found_is_invalid(self):
        build = stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare", running_args="--target mycpu-unknown-none")
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build)
        command = [c for c in command if c != "--binary" and c != "target/release/examples/compare"]
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        env.pop("RUST_TARGET_PATH", None)
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertFalse(document["build"]["target_is_builtin"])
        self.assertIsNone(document["build"]["target_spec_sha256"])
        self.assertEqual(document["status"], "invalid: source identity not reproducible")

    def test_a_build_that_compiles_the_example_twice_is_ambiguous(self):
        once = stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare")
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=f"{once} && {once}")
        command = [c for c in command if c != "--binary" and c != "target/release/examples/compare"]
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["build"]["status"], "ambiguous")
        self.assertEqual(document["rows"], [])
        self.assertEqual(document["status"], "invalid: build artifact ambiguous")

    def test_tools_resolve_through_the_invocation_path(self):
        root = pathlib.Path(tempfile.mkdtemp(prefix="topic028-cfgpath-"))
        (root / "bin").mkdir()
        tool = root / "bin" / "my-ld"
        tool.write_text("#!/bin/sh" + chr(10) + "exec configured-ld" + chr(10))
        tool.chmod(0o755)
        build = stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare", running_args="-C linker=my-ld", env_prefix=f"PATH={root / 'bin'}:/usr/bin")
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build)
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        linker = json.loads(out.read_text())["build"]["compiler_chain"][-1]
        self.assertEqual(linker["resolved"], str(tool))
        self.assertEqual(linker["sha256"], hashlib.sha256(tool.read_bytes()).hexdigest())

    def test_an_output_path_that_is_a_tracked_file_is_refused(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4)
        repo = binary.parents[3]
        command = [(str(repo / "Cargo.toml") if c == str(out) else c) for c in command]
        before = (repo / "Cargo.toml").read_text()
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertNotEqual(process.returncode, 0)
        self.assertIn("tracked", process.stderr)
        self.assertEqual((repo / "Cargo.toml").read_text(), before)

    def test_invocation_environment_values_are_redacted(self):
        build = stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare", env_prefix="CARGO_PKG_NAME=compare API_TOKEN=$STUB_SECRET RUST_TARGET_PATH=/specs")
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build)
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        env["STUB_SECRET"] = "hunter2secret"
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        text = out.read_text()
        self.assertNotIn("hunter2secret", text)
        document = json.loads(text)
        recorded = document["build"]["invocation_env"]
        self.assertEqual(recorded["CARGO_PKG_NAME"], "compare")
        self.assertEqual(recorded["RUST_TARGET_PATH"], "/specs")
        self.assertEqual(recorded["API_TOKEN"], {"sha256": hashlib.sha256(b"hunter2secret").hexdigest()})
        self.assertIn("API_TOKEN=<redacted>", document["build"]["stderr"])

    def test_the_cargo_that_performed_the_build_is_probed(self):
        root = pathlib.Path(tempfile.mkdtemp(prefix="topic028-cargo-"))
        fake_cargo = root / "cargo"
        fake_cargo.write_text("#!/bin/sh" + chr(10) + "echo 'cargo 9.9.9-chain (deadbeef 2026-01-01)'" + chr(10))
        fake_cargo.chmod(0o755)
        build = stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare", env_prefix=f"CARGO={fake_cargo}")
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build)
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertIn("9.9.9-chain", document["environment"]["cargo"])
        cargo = [c for c in document["build"]["compiler_chain"] if c["role"] == "cargo"]
        self.assertEqual(cargo[0]["sha256"], hashlib.sha256(fake_cargo.read_bytes()).hexdigest())

    def test_a_blank_identity_probe_withholds_selection(self):
        root = pathlib.Path(tempfile.mkdtemp(prefix="topic028-silent-"))
        silent = root / "silent-rustc"
        silent.write_text("#!/bin/sh" + chr(10) + "exit 0" + chr(10))
        silent.chmod(0o755)
        build = stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare", compiler=str(silent))
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build)
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["environment"]["rustc"], "")
        self.assertEqual(document["status"], "invalid: compiler identity unavailable")

    def test_a_target_directory_overlapping_tracked_inputs_is_refused(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4)
        repo = binary.parents[3]
        env["CARGO_TARGET_DIR"] = str(repo / "topics")
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertNotEqual(process.returncode, 0)
        self.assertIn("target directory", process.stderr)
        self.assertFalse(out.exists())

    def test_macos_runtime_overrides_are_recorded(self):
        process, out, _ = run_campaign({}, repetitions=4, extra_env={"DYLD_INSERT_LIBRARIES": "/lib/x.dylib", "MallocNanoZone": "0", "MallocStackLogging": "1"})
        self.assertEqual(process.returncode, 0, process.stderr)
        runtime = json.loads(out.read_text())["environment"]["runtime_env"]
        self.assertEqual(runtime["DYLD_INSERT_LIBRARIES"], "/lib/x.dylib")
        self.assertEqual(runtime["MallocNanoZone"], "0")
        self.assertEqual(runtime["MallocStackLogging"], "1")

    def test_bare_overrides_resolve_through_the_source_relative_path(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4)
        repo = binary.parents[3]
        (repo / "tools").mkdir()
        wrapper = repo / "tools" / "my-wrapper"
        wrapper.write_text("#!/bin/sh" + chr(10) + 'exec "$@"' + chr(10))
        wrapper.chmod(0o755)
        (repo / ".gitignore").write_text("/target/" + chr(10) + "/tools/" + chr(10))
        subprocess.run(["git", "-C", str(repo), "add", ".gitignore"], check=True)
        subprocess.run(["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "tools"], check=True)
        env["RUSTC_WRAPPER"] = "my-wrapper"
        env["PATH"] = "tools:" + env["PATH"]
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300, cwd=tempfile.mkdtemp(prefix="topic028-cwd-"))
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["environment"]["compiler_overrides"]["RUSTC_WRAPPER"]["resolved"], str(wrapper))
        self.assertEqual(document["status"], "complete")

    def test_a_linker_named_by_the_target_specification_is_hashed(self):
        root = pathlib.Path(tempfile.mkdtemp(prefix="topic028-speclinker-"))
        linker = root / "spec-ld"
        linker.write_text("#!/bin/sh" + chr(10) + 'exec "$@"' + chr(10))
        linker.chmod(0o755)
        spec = root / "custom.json"
        spec.write_text(json.dumps({"llvm-target": "aarch64-unknown-linux-gnu", "linker": str(linker)}) + chr(10))
        build = stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare", running_args=f"--target {spec}")
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build)
        command = [c for c in command if c != "--binary" and c != "target/release/examples/compare"]
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        chain = json.loads(out.read_text())["build"]["compiler_chain"]
        entry = [c for c in chain if c["role"] == "target-linker"][0]
        self.assertEqual(entry["resolved"], str(linker))
        self.assertEqual(entry["sha256"], hashlib.sha256(linker.read_bytes()).hexdigest())

    def test_an_output_path_that_is_a_tracked_symlink_is_refused(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4)
        repo = binary.parents[3]
        elsewhere = pathlib.Path(tempfile.mkdtemp(prefix="topic028-else-")) / "campaign.json"
        link = repo / "results.json"
        link.symlink_to(elsewhere)
        subprocess.run(["git", "-C", str(repo), "add", "results.json"], check=True)
        subprocess.run(["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "link"], check=True)
        command = [(str(link) if c == str(out) else c) for c in command]
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertNotEqual(process.returncode, 0)
        self.assertIn("tracked", process.stderr)
        self.assertTrue(link.is_symlink())
        self.assertFalse(elsewhere.exists())

    def test_an_unreadable_identity_file_hashes_to_none(self):
        sys.path.insert(0, str(RUNNER.parent))
        import run_campaign
        path = pathlib.Path(tempfile.mkdtemp(prefix="topic028-unreadable-")) / "tool"
        path.write_text("#!/bin/sh" + chr(10))
        path.chmod(0o111)
        if os.access(path, os.R_OK):
            self.skipTest("this account can read execute-only files")
        self.assertIsNone(run_campaign.sha256_of(path))

    def test_compiler_probes_run_with_the_invocation_environment(self):
        root = pathlib.Path(tempfile.mkdtemp(prefix="topic028-dispatch-"))
        dispatcher = root / "dispatch-rustc"
        dispatcher.write_text("#!/bin/sh" + chr(10) + 'echo "rustc 1.0.0-${STUB_FLAVOR:-plain}"' + chr(10) + 'echo "release: 1.0.0-${STUB_FLAVOR:-plain}"' + chr(10))
        dispatcher.chmod(0o755)
        build = stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare", compiler=f"STUB_FLAVOR=dispatch {dispatcher}")
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build)
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        env.pop("STUB_FLAVOR", None)
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertIn("1.0.0-dispatch", json.loads(out.read_text())["environment"]["rustc"])

    def test_prefix_lookalike_secrets_are_hashed(self):
        build = stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare", env_prefix="RUST_API_TOKEN=$STUB_SECRET CARGO_BUILD_TOKEN=$STUB_SECRET CARGO_TARGET_SECRET=$STUB_SECRET RUST_BACKTRACE=1 CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=cc")
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build, extra_env={"RUST_SECRET_KEY": "hunter2runtime"})
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        env["STUB_SECRET"] = "hunter2secret"
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        text = out.read_text()
        self.assertNotIn("hunter2secret", text)
        self.assertNotIn("hunter2runtime", text)
        document = json.loads(text)
        recorded = document["build"]["invocation_env"]
        for key in ["RUST_API_TOKEN", "CARGO_BUILD_TOKEN", "CARGO_TARGET_SECRET"]:
            self.assertEqual(recorded[key], {"sha256": hashlib.sha256(b"hunter2secret").hexdigest()}, key)
        self.assertEqual(recorded["RUST_BACKTRACE"], "1")
        self.assertEqual(recorded["CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER"], "cc")
        self.assertEqual(document["environment"]["runtime_env"]["RUST_SECRET_KEY"], {"sha256": hashlib.sha256(b"hunter2runtime").hexdigest()})

    def test_the_inferred_linker_is_recorded_when_none_is_named(self):
        process, out, _ = run_campaign({}, repetitions=4)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        linker = [c for c in document["build"]["compiler_chain"] if c["role"] == "linker"]
        self.assertEqual(len(linker), 1)
        self.assertTrue(linker[0]["inferred"])
        self.assertEqual(linker[0]["path"], "cc")
        self.assertTrue(linker[0]["resolved"].endswith("/cc"))
        self.assertIsNotNone(linker[0]["sha256"])
        self.assertTrue(document["environment"]["linker"])

    def test_an_output_path_matching_a_cargo_input_is_refused(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4)
        repo = binary.parents[3]
        command = [(str(repo / "topics/028-work-stealing-deques/build.rs") if c == str(out) else c) for c in command]
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertNotEqual(process.returncode, 0)
        self.assertIn("Cargo input", process.stderr)
        self.assertFalse((repo / "topics/028-work-stealing-deques/build.rs").exists())

    def test_external_compiler_inputs_named_in_rustflags_are_hashed(self):
        profile = pathlib.Path(tempfile.mkdtemp(prefix="topic028-pgo-")) / "run.profdata"
        profile.write_bytes(b"profile bytes")
        process, out, _ = run_campaign({}, repetitions=4, extra_env={"RUSTFLAGS": f"-Cprofile-use={profile} -C target-cpu=native"})
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["environment"]["rustflags"]["external_inputs"], {str(profile): hashlib.sha256(b"profile bytes").hexdigest()})
        self.assertEqual(document["status"], "complete")
        process, out, _ = run_campaign({}, repetitions=4, extra_env={"CARGO_ENCODED_RUSTFLAGS": "-C\x1fprofile-use=/no/such/run.profdata"})
        self.assertEqual(json.loads(out.read_text())["status"], "invalid: source identity not reproducible")

    def test_an_empty_wrapper_override_is_a_disabled_wrapper(self):
        process, out, _ = run_campaign({}, repetitions=4, extra_env={"RUSTC_WRAPPER": "", "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER": ""})
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["environment"]["compiler_overrides"]["RUSTC_WRAPPER"], {"value": "", "disabled": True})
        self.assertEqual(document["status"], "complete")

    def test_inputs_named_in_the_invocation_flags_are_hashed(self):
        profile = pathlib.Path(tempfile.mkdtemp(prefix="topic028-cfgpgo-")) / "cfg.profdata"
        profile.write_bytes(b"from cargo config")
        build = stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare", running_args=f"-C profile-use={profile}")
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build)
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        env.pop("RUSTFLAGS", None)
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["build"]["external_inputs"], {str(profile): hashlib.sha256(b"from cargo config").hexdigest()})
        profile.unlink()
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(json.loads(out.read_text())["status"], "invalid: source identity not reproducible")

    def test_the_builtin_target_probe_uses_the_invocation_environment(self):
        root = pathlib.Path(tempfile.mkdtemp(prefix="topic028-targets-"))
        dispatcher = root / "dispatch-rustc"
        dispatcher.write_text("#!/bin/sh" + chr(10) + 'case "$1" in' + chr(10) + "  -Vv) echo 'rustc 1.0.0-stub'; echo 'release: 1.0.0-stub' ;;" + chr(10)
                              + '  --print) if [ "$STUB_BACKEND" = "wide" ]; then echo mycpu-unknown-none; fi; echo aarch64-unknown-linux-gnu ;;' + chr(10) + "esac" + chr(10))
        dispatcher.chmod(0o755)
        build = stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare", running_args="--target mycpu-unknown-none", compiler=f"STUB_BACKEND=wide {dispatcher}")
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build)
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        env.pop("RUST_TARGET_PATH", None)
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertTrue(document["build"]["target_is_builtin"])
        self.assertEqual(document["status"], "complete")

    def test_coverage_modes_are_not_treated_as_files(self):
        process, out, _ = run_campaign({}, repetitions=4, extra_env={"RUSTFLAGS": "-C instrument-coverage=all"})
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["environment"]["rustflags"]["external_inputs"], {})
        self.assertEqual(document["status"], "complete")

    def test_a_tracked_scratch_path_is_left_alone(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4)
        repo = binary.parents[3]
        (repo / "results").mkdir()
        partial = repo / "results" / "campaign.json.partial"
        partial.write_text("tracked scratch" + chr(10))
        (repo / ".gitignore").write_text("/target/" + chr(10) + "/results/campaign.json" + chr(10))
        subprocess.run(["git", "-C", str(repo), "add", ".gitignore", "results/campaign.json.partial"], check=True)
        subprocess.run(["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "scratch"], check=True)
        command = [(str(repo / "results" / "campaign.json") if c == str(out) else c) for c in command]
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(partial.read_text(), "tracked scratch" + chr(10))
        self.assertEqual(json.loads((repo / "results" / "campaign.json").read_text())["status"], "complete")

    def test_untracked_cargo_inputs_beneath_the_target_directory_are_dirty(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4)
        repo = binary.parents[3]
        target = repo / "build-out"
        (repo / ".gitignore").write_text("/target/" + chr(10) + "/build-out/" + chr(10))
        subprocess.run(["git", "-C", str(repo), "add", ".gitignore"], check=True)
        subprocess.run(["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "target"], check=True)
        (target / "release" / "build" / "pkg-abc" / "out").mkdir(parents=True)
        (target / "release" / "build" / "pkg-abc" / "out" / "generated.rs").write_text("" + chr(10))
        env["CARGO_TARGET_DIR"] = str(target)
        env["STUB_SOURCE"] = str(repo.parent / "stub.py")
        command = [c for c in command if c != "--binary" and c != "target/release/examples/compare"]
        command[command.index("--build-command") + 1] = stub_build("\"$STUB_SOURCE\"", str(target / "release/examples/compare"))
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(json.loads(out.read_text())["status"], "complete")
        (target / "dep" / "src").mkdir(parents=True)
        (target / "dep" / "src" / "lib.rs").write_text("" + chr(10))
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        document = json.loads(out.read_text())
        self.assertIn("!! build-out/dep/src/lib.rs", document["environment"]["worktree_status"])
        self.assertEqual(document["status"], "invalid: source identity not reproducible")

    def test_unknown_profile_variables_are_hashed(self):
        process, out, _ = run_campaign({}, repetitions=4, extra_env={"CARGO_PROFILE_API_TOKEN": "hunter2profile", "CARGO_PROFILE_RELEASE_LTO": "fat"})
        self.assertEqual(process.returncode, 0, process.stderr)
        text = out.read_text()
        self.assertNotIn("hunter2profile", text)
        recorded = json.loads(text)["environment"]["cargo_profile"]
        self.assertEqual(recorded["CARGO_PROFILE_API_TOKEN"], {"sha256": hashlib.sha256(b"hunter2profile").hexdigest()})
        self.assertEqual(recorded["CARGO_PROFILE_RELEASE_LTO"], "fat")

    def test_injected_runtime_libraries_are_hashed(self):
        lib = pathlib.Path(tempfile.mkdtemp(prefix="topic028-dylib-")) / "liballoc.dylib"
        lib.write_bytes(b"not really a library")
        process, out, _ = run_campaign({}, repetitions=4, extra_env={"DYLD_INSERT_LIBRARIES": str(lib)})
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["environment"]["runtime_libraries"], {str(lib): hashlib.sha256(b"not really a library").hexdigest()})
        self.assertEqual(document["status"], "complete")
        process, out, _ = run_campaign({}, repetitions=4, extra_env={"LD_PRELOAD": "/no/such/liballoc.so"})
        document = json.loads(out.read_text())
        self.assertIsNone(document["environment"]["runtime_libraries"]["/no/such/liballoc.so"])
        self.assertEqual(document["status"], "invalid: source identity not reproducible")

    def test_a_builtin_target_wins_over_a_colliding_specification(self):
        root = pathlib.Path(tempfile.mkdtemp(prefix="topic028-collide-"))
        (root / "specs").mkdir()
        (root / "specs" / "aarch64-unknown-linux-gnu.json").write_text(json.dumps({"linker": "/no/such/linker"}) + chr(10))
        build = stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare", running_args="--target aarch64-unknown-linux-gnu")
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build, extra_env={"RUST_TARGET_PATH": str(root / "specs")})
        command = [c for c in command if c != "--binary" and c != "target/release/examples/compare"]
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertTrue(document["build"]["target_is_builtin"])
        self.assertIsNone(document["build"]["target_spec_sha256"])
        self.assertEqual([c for c in document["build"]["compiler_chain"] if c["role"] == "target-linker"], [])
        self.assertEqual(document["status"], "complete")

    def test_measured_processes_run_in_the_source_tree_and_preloads_resolve_there(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4)
        repo = binary.parents[3]
        lib = repo / "liballoc.dylib"
        lib.write_bytes(b"source library")
        (repo / ".gitignore").write_text("/target/" + chr(10) + "/liballoc.dylib" + chr(10))
        subprocess.run(["git", "-C", str(repo), "add", ".gitignore"], check=True)
        subprocess.run(["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "lib"], check=True)
        launch = pathlib.Path(tempfile.mkdtemp(prefix="topic028-launch-"))
        (launch / "liballoc.dylib").write_bytes(b"launch library")
        env["DYLD_INSERT_LIBRARIES"] = "./liballoc.dylib"
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300, cwd=launch)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["environment"]["runtime_libraries"], {"./liballoc.dylib": hashlib.sha256(b"source library").hexdigest()})
        self.assertEqual({r["cwd"] for r in document["rows"]}, {str(repo)})

    def test_direct_linker_input_files_are_hashed(self):
        obj = pathlib.Path(tempfile.mkdtemp(prefix="topic028-obj-")) / "hook.o"
        obj.write_bytes(b"object bytes")
        process, out, _ = run_campaign({}, repetitions=4, extra_env={"RUSTFLAGS": f"-Clink-arg={obj} -Clink-arg=-Wl,--gc-sections"})
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["environment"]["rustflags"]["external_inputs"], {str(obj): hashlib.sha256(b"object bytes").hexdigest()})
        self.assertEqual(document["status"], "complete")
        process, out, _ = run_campaign({}, repetitions=4, extra_env={"RUSTFLAGS": "-Clink-arg=/no/such/hook.o"})
        self.assertEqual(json.loads(out.read_text())["status"], "invalid: source identity not reproducible")

    def test_config_files_named_in_the_build_command_are_hashed(self):
        config = pathlib.Path(tempfile.mkdtemp(prefix="topic028-cfg-")) / "build.toml"
        config.write_text("[build]" + chr(10))
        build = f"true --config {config} --config build.jobs=2 && " + stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare")
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build)
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        hashes = json.loads(out.read_text())["build"]["config_sha256"]
        self.assertEqual(hashes, {str(config): hashlib.sha256(("[build]" + chr(10)).encode()).hexdigest()})

    def test_the_compiler_probe_runs_through_the_wrapper_chain(self):
        root = pathlib.Path(tempfile.mkdtemp(prefix="topic028-wrapchain-"))
        backend = root / "backend-rustc"
        backend.write_text("#!/bin/sh" + chr(10) + "echo 'rustc 5.5.5-backend'" + chr(10) + "echo 'release: 5.5.5-backend'" + chr(10))
        backend.chmod(0o755)
        wrapper = root / "wrapper"
        wrapper.write_text("#!/bin/sh" + chr(10) + "shift" + chr(10) + f'exec {backend} "$@"' + chr(10))
        wrapper.chmod(0o755)
        build = stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare", compiler=f"{wrapper} $FAKE_RUSTC")
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build)
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertIn("5.5.5-backend", json.loads(out.read_text())["environment"]["rustc"])

    def test_coloured_running_lines_are_parsed_and_redacted(self):
        build = stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare", env_prefix="API_TOKEN=$STUB_SECRET", ansi=True)
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build)
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        env["STUB_SECRET"] = "hunter2ansi"
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        text = out.read_text()
        self.assertNotIn("hunter2ansi", text)
        document = json.loads(text)
        self.assertEqual(document["build"]["compiler_invocations"], 1)
        self.assertEqual(document["status"], "complete")

    def test_an_output_reaching_a_tracked_file_through_a_symlinked_directory_is_refused(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4)
        repo = binary.parents[3]
        alias = repo.parent / "alias"
        alias.symlink_to(repo)
        tracked = repo / "Cargo.toml"
        before = tracked.read_bytes()
        command[command.index("--out") + 1] = str(alias / "Cargo.toml")
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertNotEqual(process.returncode, 0)
        self.assertIn("tracked", process.stderr)
        self.assertEqual(tracked.read_bytes(), before)

    def test_bare_response_files_in_rustflags_are_hashed(self):
        response = pathlib.Path(tempfile.mkdtemp(prefix="topic028-rsp-")) / "args.rsp"
        response.write_text("-C opt-level=3" + chr(10))
        process, out, _ = run_campaign({}, repetitions=4, extra_env={"RUSTFLAGS": f"@{response}"})
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["environment"]["rustflags"]["external_inputs"], {str(response): hashlib.sha256(("-C opt-level=3" + chr(10)).encode()).hexdigest()})
        process, out, _ = run_campaign({}, repetitions=4, extra_env={"RUSTFLAGS": "@/no/such/args.rsp"})
        self.assertEqual(json.loads(out.read_text())["status"], "invalid: source identity not reproducible")

    def test_the_target_list_probe_runs_through_the_wrapper_chain(self):
        root = pathlib.Path(tempfile.mkdtemp(prefix="topic028-wraptargets-"))
        backend = root / "backend-rustc"
        backend.write_text("#!/bin/sh" + chr(10) + 'case "$*" in *target-list*) echo other-unknown-none;; *) echo "rustc 5.5.5-backend"; echo "release: 5.5.5-backend";; esac' + chr(10))
        backend.chmod(0o755)
        wrapper = root / "wrapper"
        wrapper.write_text("#!/bin/sh" + chr(10) + "shift" + chr(10) + f'exec {backend} "$@"' + chr(10))
        wrapper.chmod(0o755)
        spec = root / "aarch64-unknown-linux-gnu.json"
        spec.write_text(json.dumps({"llvm-target": "aarch64-unknown-linux-gnu", "linker": "cc"}))
        build = stub_build("\"$STUB_SOURCE\"", "target/release/examples/compare", compiler=f"{wrapper} $FAKE_RUSTC", running_args="--target aarch64-unknown-linux-gnu")
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build=build, extra_env={"RUST_TARGET_PATH": str(root)})
        env["STUB_SOURCE"] = str(binary.parents[3].parent / "stub.py")
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        record = json.loads(out.read_text())["build"]
        self.assertIs(record["target_is_builtin"], False)
        self.assertEqual(record["target_spec_sha256"], hashlib.sha256(spec.read_bytes()).hexdigest())

    def test_environment_mutating_build_commands_are_refused(self):
        for build in ("env -u BACKEND cargo build --release", "BACKEND=A cargo build --release", "unset BACKEND; cargo build --release", "export BACKEND=A && cargo build"):
            with self.subTest(build=build):
                process, out, _ = run_campaign({}, repetitions=4, build=build)
                self.assertNotEqual(process.returncode, 0)
                self.assertIn("environment", process.stderr)
                self.assertFalse(out.exists())

    def test_loaded_shared_objects_are_hashed_where_the_loader_finds_them(self):
        module = pathlib.Path(tempfile.mkdtemp(prefix="topic028-loaded-"))
        found = shutil.which("sh")
        assert found is not None
        shell = pathlib.Path(found).resolve()
        objects = run_campaign_module().loaded_objects(shell, cwd=module, env=dict(os.environ))
        self.assertTrue(objects)
        libc = next(path for path in objects if "/libc." in path)
        self.assertEqual(objects[libc], hashlib.sha256(pathlib.Path(libc).read_bytes()).hexdigest())
        shutil.copy2(libc, module / pathlib.Path(libc).name)
        redirected = run_campaign_module().loaded_objects(shell, cwd=module, env=dict(os.environ, LD_LIBRARY_PATH=str(module)))
        self.assertIn(str(module / pathlib.Path(libc).name), redirected)

    def test_the_campaign_records_loaded_objects_of_the_measured_binary(self):
        process, out, _ = run_campaign({}, repetitions=4)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["environment"]["loaded_objects"], {})
        self.assertEqual(document["status"], "complete")

    def test_malformed_rustflags_are_recorded_instead_of_aborting(self):
        process, out, _ = run_campaign({}, repetitions=4, extra_env={"RUSTFLAGS": "-C opt-level=3 'unterminated"})
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["environment"]["rustflags"]["external_inputs"], {})
        self.assertEqual(document["status"], "complete")

    def test_inputs_named_inside_response_files_are_hashed(self):
        root = pathlib.Path(tempfile.mkdtemp(prefix="topic028-rspin-"))
        hook = root / "hook.o"
        hook.write_bytes(b"hook object")
        linker_response = root / "link.rsp"
        linker_response.write_text(f"-Wl,--gc-sections {hook}" + chr(10))
        rustc_response = root / "rustc.rsp"
        rustc_response.write_text("-C" + chr(10) + f"link-arg=@{linker_response}" + chr(10))
        process, out, _ = run_campaign({}, repetitions=4, extra_env={"RUSTFLAGS": f"@{rustc_response}"})
        self.assertEqual(process.returncode, 0, process.stderr)
        inputs = json.loads(out.read_text())["environment"]["rustflags"]["external_inputs"]
        self.assertEqual(set(inputs), {str(rustc_response), str(linker_response), str(hook)})
        self.assertEqual(inputs[str(hook)], hashlib.sha256(b"hook object").hexdigest())
        hook.unlink()
        process, out, _ = run_campaign({}, repetitions=4, extra_env={"RUSTFLAGS": f"-Clink-arg=@{linker_response}"})
        self.assertEqual(json.loads(out.read_text())["status"], "invalid: source identity not reproducible")

    def test_path_qualified_and_wrapped_environment_mutators_are_refused(self):
        for build in ("/usr/bin/env -u BACKEND cargo build", "command env -u BACKEND cargo build", "nohup /usr/bin/env -u BACKEND cargo build", "sh -c 'unset BACKEND; cargo build'", "sudo cargo build"):
            with self.subTest(build=build):
                process, out, _ = run_campaign({}, repetitions=4, build=build)
                self.assertNotEqual(process.returncode, 0)
                self.assertIn("environment", process.stderr)
                self.assertFalse(out.exists())

    def test_a_runtime_library_rewritten_during_the_campaign_invalidates_it(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4)
        repo = binary.parents[3]
        lib = repo / "liballoc.dylib"
        lib.write_bytes(b"first")
        (repo / ".gitignore").write_text("/target/" + chr(10) + "/liballoc.dylib" + chr(10))
        subprocess.run(["git", "-C", str(repo), "add", ".gitignore"], check=True)
        subprocess.run(["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "lib"], check=True)
        pathlib.Path(env["STUB_PLAN"]).write_text(json.dumps({"global_fifo#2": {"rewrite_file": [str(lib), "second"]}}))
        env["DYLD_INSERT_LIBRARIES"] = "./liballoc.dylib"
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["status"], "invalid: loaded objects changed during the campaign")
        self.assertEqual(document["environment"]["runtime_libraries_final"]["./liballoc.dylib"], hashlib.sha256(b"second").hexdigest())
        self.assertTrue(all(row["selected"] == "unresolved" for row in document["summaries"]))

    def test_a_path_dependency_under_a_deps_directory_is_dirty(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4)
        repo = binary.parents[3]
        dep = repo / "target/release/deps/helper/src/lib.rs"
        dep.parent.mkdir(parents=True)
        dep.write_text("pub fn helper() {}" + chr(10))
        artifact = repo / "target/release/deps/libhelper-abc.rlib"
        artifact.write_bytes(b"rlib")
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertIs(document["environment"]["worktree_dirty"], True)
        self.assertIn("!! target/release/deps/helper/src/lib.rs", document["environment"]["worktree_status"])
        self.assertEqual(document["status"], "invalid: source identity not reproducible")

    def test_cargo_profile_overrides_are_recorded(self):
        process, out, _ = run_campaign({}, repetitions=4, extra_env={"CARGO_PROFILE_RELEASE_OPT_LEVEL": "0", "CARGO_PROFILE_RELEASE_LTO": "true"})
        self.assertEqual(process.returncode, 0, process.stderr)
        recorded = json.loads(out.read_text())["environment"]["cargo_profile"]
        self.assertEqual(recorded, {"CARGO_PROFILE_RELEASE_LTO": "true", "CARGO_PROFILE_RELEASE_OPT_LEVEL": "0"})

    def test_runtime_overrides_inherited_by_children_are_recorded(self):
        process, out, _ = run_campaign({}, repetitions=4, extra_env={"GLIBC_TUNABLES": "glibc.malloc.arena_max=1", "RUST_MIN_STACK": "1048576", "UNRELATED_VAR": "x"})
        self.assertEqual(process.returncode, 0, process.stderr)
        runtime = json.loads(out.read_text())["environment"]["runtime_env"]
        self.assertEqual(runtime["GLIBC_TUNABLES"], "glibc.malloc.arena_max=1")
        self.assertEqual(runtime["RUST_MIN_STACK"], "1048576")
        self.assertNotIn("UNRELATED_VAR", runtime)
        self.assertNotIn("LD_PRELOAD", runtime)

    def test_a_timed_out_build_takes_its_descendants_with_it(self):
        marker = "topic028-build-marker-" + os.urandom(6).hex()
        sleeper = pathlib.Path(tempfile.mkdtemp(prefix="topic028-sleeper-")) / marker
        shutil.copy(shutil.which("sleep") or "/bin/sleep", sleeper)
        command, env, out, _, _ = start_campaign({}, repetitions=4, build=f"{sleeper} 300 & {sleeper} 300")
        command += ["--build-timeout", "1"]
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["build"]["status"], "timeout")
        self.assertEqual(document["status"], "invalid: build failed")
        time.sleep(0.5)
        survivors = subprocess.run(["pgrep", "-f", marker], text=True, capture_output=True).stdout.split()
        self.assertEqual(survivors, [])

    def test_a_prebuilt_binary_has_unverified_provenance(self):
        process, out, _ = run_campaign({}, repetitions=4, skip_build=True)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual(document["build"]["status"], "skipped")
        self.assertEqual(len(document["rows"]), 5 * 4 * 4)
        self.assertEqual(document["status"], "invalid: binary provenance unverified")
        self.assertTrue(all(s["selected"] == "unresolved" for s in document["summaries"]))

    def test_cargo_config_hashes_walk_up_from_the_source_tree(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4)
        repo = binary.parents[3]
        (repo / ".cargo").mkdir()
        (repo / ".cargo" / "config.toml").write_text("[build]" + chr(10) + 'rustflags = ["-C", "target-cpu=native"]' + chr(10))
        subprocess.run(["git", "-C", str(repo), "add", ".cargo"], check=True)
        subprocess.run(["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "config"], check=True)
        (repo.parent / ".cargo").mkdir()
        (repo.parent / ".cargo" / "config").write_text("[build]" + chr(10))
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        hashes = json.loads(out.read_text())["environment"]["rustflags"]["cargo_config_sha256"]
        self.assertIn(str(repo / ".cargo" / "config.toml"), hashes)
        self.assertIn(str(repo.parent / ".cargo" / "config"), hashes)

    def test_relative_cargo_home_resolves_against_the_source_tree(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4)
        repo = binary.parents[3]
        (repo / "chome").mkdir()
        config = repo / "chome" / "config.toml"
        config.write_text("[build]" + chr(10))
        (repo / ".gitignore").write_text("/target/" + chr(10) + "/chome/" + chr(10))
        subprocess.run(["git", "-C", str(repo), "add", ".gitignore"], check=True)
        subprocess.run(["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "home"], check=True)
        env["CARGO_HOME"] = "chome"
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300, cwd=tempfile.mkdtemp(prefix="topic028-cwd-"))
        self.assertEqual(process.returncode, 0, process.stderr)
        hashes = json.loads(out.read_text())["environment"]["rustflags"]["cargo_config_sha256"]
        self.assertIn(str(config), hashes)

    def test_a_dirty_source_tree_withholds_selection(self):
        process, out, _ = run_campaign({}, dirty=True)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertTrue(document["environment"]["worktree_dirty"])
        self.assertEqual(document["environment"]["worktree_status"], ["?? topics/028-work-stealing-deques/build.rs"])
        self.assertEqual(document["status"], "invalid: source identity not reproducible")
        self.assertTrue(all(s["selected"] == "unresolved" for s in document["summaries"]))
        self.assertEqual(summaries_by_name(document)["medium_balanced"]["lowest_median"], "locked_steal_balanced")


if __name__ == "__main__":
    unittest.main()
