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
if action.get("hang"):
    print("partial output before hang", flush=True)
    print("partial stderr", file=sys.stderr, flush=True)
    time.sleep(30)
if action.get("fail"):
    print("stub failure", file=sys.stderr)
    sys.exit(3)
base = {"global_fifo": 1000, "locked_steal_balanced": 900, "locked_steal_skewed": 1100}[candidate]
print("candidate=%s workers=%s tasks=%s leaf_work=%s elapsed_ns=%s" % (
    action.get("echo_candidate", candidate), args["--workers"], args["--tasks"], args["--leaf-work"], action.get("elapsed_ns", base)))
"""


def start_campaign(plan, timeout=None):
    """Prepare a scratch directory with a planned stub; return (command, env, output path, stub path, stub log)."""
    root = pathlib.Path(tempfile.mkdtemp(prefix="topic028-campaign-"))
    binary = root / "compare"
    binary.write_text(STUB)
    binary.chmod(0o755)
    (root / "plan.json").write_text(json.dumps(plan))
    log = root / "stub.log"
    out = root / "campaign.json"
    command = [sys.executable, str(RUNNER), "--binary", str(binary), "--repetitions", "4", "--out", str(out)]
    if timeout is not None:
        command += ["--timeout", str(timeout)]
    env = dict(os.environ, STUB_PLAN=str(root / "plan.json"), STUB_LOG=str(log))
    return command, env, out, binary, log


def run_campaign(plan, timeout=None):
    """Run the runner to completion; return (completed process, output path, stub path)."""
    command, env, out, binary, _ = start_campaign(plan, timeout)
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
        self.assertEqual(sorted(mixed["paired_ratios"]), [0.9, 0.9, 0.9, 0.98])
        self.assertEqual(mixed["selected"], "unresolved")
        decided = summaries["tiny_skewed"]
        self.assertEqual(decided["paired_ratios"], [1.1, 1.1, 1.1, 1.1])
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
        self.assertEqual(len([r for r in document["rows"] if r["workload"]["name"] == "coarse_skewed"]), 16)
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
        self.assertGreaterEqual(len(partial["rows"]), 4)

    def test_identical_artifact_aa_arm_flows_through_the_same_paths(self):
        process, out, _ = run_campaign({})
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        controls = [r for r in document["rows"] if r["arm"] == "control"]
        self.assertEqual(len(controls), 5 * 4 * 2)
        self.assertEqual(sorted({r["label"] for r in controls}), ["A", "B"])
        self.assertEqual({r["candidate"] for r in controls}, {"global_fifo"})
        orders = {}
        for row in controls:
            orders.setdefault((row["workload"]["name"], row["repetition"]), []).append(row["label"])
        self.assertEqual(sorted({tuple(v) for v in orders.values()}), [("A", "B"), ("B", "A")])
        summary = summaries_by_name(document)["tiny_balanced"]
        self.assertEqual(summary["aa_ratios"], [1.0, 1.0, 1.0, 1.0])

    def test_environment_records_artifact_and_toolchain_identity(self):
        process, out, binary = run_campaign({})
        self.assertEqual(process.returncode, 0, process.stderr)
        environment = json.loads(out.read_text())["environment"]
        self.assertEqual(environment["binary_sha256"], hashlib.sha256(binary.read_bytes()).hexdigest())
        for key in ["source_commit", "worktree_dirty", "source_sha256", "rustc", "cargo", "rustflags", "cpu_model", "available_cpus", "uname", "hostname"]:
            self.assertIn(key, environment)
        self.assertIn("topics/028-work-stealing-deques/src/lib.rs", environment["source_sha256"])


if __name__ == "__main__":
    unittest.main()
