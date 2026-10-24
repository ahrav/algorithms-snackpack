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
        path.write_text(RUNNER.read_text() if name.endswith("run_campaign.py") else name + chr(10))
    subprocess.run(["git", "init", "-q", repo], check=True)
    subprocess.run(["git", "-C", str(repo), "add", "."], check=True)
    subprocess.run(["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "frozen"], check=True)
    return repo


def start_campaign(plan, timeout=None, repetitions=8, toolchain=None, dirty=False, extra_env=None):
    """Prepare a scratch directory with a planned stub; return (command, env, output path, stub path, stub log)."""
    root = pathlib.Path(tempfile.mkdtemp(prefix="topic028-campaign-"))
    repo = clean_repository(root)
    if dirty:
        (repo / "topics/028-work-stealing-deques/build.rs").write_text("fn main() {}" + chr(10))
    runner = repo / "topics/028-work-stealing-deques/scripts/run_campaign.py"
    binary = root / "compare"
    binary.write_text(STUB)
    binary.chmod(0o755)
    (root / "plan.json").write_text(json.dumps(plan))
    log = root / "stub.log"
    out = root / "campaign.json"
    command = [sys.executable, str(runner), "--binary", str(binary), "--repetitions", str(repetitions), "--out", str(out)]
    if timeout is not None:
        command += ["--timeout", str(timeout)]
    if toolchain is not None:
        command += ["--toolchain", toolchain]
    env = dict(os.environ, STUB_PLAN=str(root / "plan.json"), STUB_LOG=str(log), STUB_BINARY=str(binary), **(extra_env or {}))
    return command, env, out, binary, log


def run_campaign(plan, timeout=None, repetitions=8, toolchain=None, dirty=False, extra_env=None):
    """Run the runner to completion; return (completed process, output path, stub path)."""
    command, env, out, binary, _ = start_campaign(plan, timeout, repetitions, toolchain, dirty, extra_env)
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
        process, out, _ = run_campaign({"global_fifo#1": {"fail": True}, "locked_steal_balanced#3": {"hang": True}}, timeout=1)
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
        process, out, _ = run_campaign({"global_fifo#1": {"elapsed_ns": 0}, "locked_steal_balanced#2": {"echo_candidate": "global_fifo"}})
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        failed = {r["reason"][:20]: r for r in document["rows"] if r["status"] == "failed"}
        self.assertEqual(sorted(failed), ["echoed arguments {'c", "elapsed_ns '0' is no"])
        self.assertTrue(all("elapsed_ns" not in r for r in failed.values()))
        affected = summaries_by_name(document)["tiny_balanced"]
        self.assertEqual(affected["incomplete_repetitions"], 2)
        self.assertEqual(affected["selected"], "unresolved")

    def test_a_one_sided_aa_control_withholds_selection(self):
        plan = {f"global_fifo#{occurrence}": {"elapsed_ns": 900} for occurrence in [2, 5, 8, 11, 14, 17, 20, 23]}
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
        process, out, _ = run_campaign({}, repetitions=4, toolchain="topic028-no-such-toolchain")
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        environment = document["environment"]
        self.assertEqual(environment["toolchain"], "topic028-no-such-toolchain")
        self.assertIsNone(environment["rustc"])
        self.assertIsNone(environment["cargo"])
        self.assertEqual(document["status"], "invalid: compiler identity unavailable")
        self.assertTrue(all(s["selected"] == "unresolved" for s in document["summaries"]))
        process, out, _ = run_campaign({}, repetitions=4)
        environment = json.loads(out.read_text())["environment"]
        self.assertIsNone(environment["toolchain"])
        self.assertIn("release:", environment["rustc"] or "release:")

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
        process, out, _ = run_campaign({"global_fifo#2": {"fail": True}})
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
        out = run_campaign_killed_after({"locked_steal_balanced#2": {"hang": True}}, invocations=9)
        partial = json.loads(out.read_text())
        self.assertEqual(partial["status"], "running")
        self.assertEqual(len(partial["warmups"]), 2)
        self.assertEqual([r["status"] for r in partial["rows"]], ["ok"] * 6 + ["running"])
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
        command, env, out, binary, _ = start_campaign({}, repetitions=4)
        binary.unlink()
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertEqual({r["status"] for r in document["warmups"] + document["rows"]}, {"failed"})
        self.assertIn("launch failed", document["rows"][0]["reason"])
        self.assertEqual(len(document["rows"]), 5 * 4 * 4)
        self.assertTrue(all(s["selected"] == "unresolved" for s in document["summaries"]))
        self.assertIsNone(document["environment"]["binary_sha256"])
        self.assertIsNone(document["environment"]["binary_copy"])
        self.assertEqual(document["status"], "invalid: binary could not be pinned")

    def test_control_and_treatment_pairs_alternate_schedule_slots(self):
        process, out, _ = run_campaign({})
        self.assertEqual(process.returncode, 0, process.stderr)
        rows = [r for r in json.loads(out.read_text())["rows"] if r["workload"]["name"] == "tiny_balanced"]
        by_repetition = {}
        for row in rows:
            by_repetition.setdefault(row["repetition"], []).append((row["arm"], row["label"]))
        self.assertEqual(by_repetition[0], [("treatment", "global_fifo"), ("treatment", "locked_steal_balanced"), ("control", "A"), ("control", "B")])
        self.assertEqual(by_repetition[1], [("control", "B"), ("control", "A"), ("treatment", "locked_steal_balanced"), ("treatment", "global_fifo")])
        self.assertEqual(by_repetition[2], by_repetition[0])
        self.assertEqual(by_repetition[3], by_repetition[1])

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
