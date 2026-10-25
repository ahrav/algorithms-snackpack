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
    binary = repo / "target/release/examples/compare"
    if build is None:
        build = f"mkdir -p target/release/examples && cp {stub} {binary}"
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
    env = dict(os.environ, STUB_PLAN=str(root / "plan.json"), STUB_LOG=str(log), STUB_BINARY=str(binary), **(extra_env or {}))
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

    def test_compiler_probes_honour_the_source_checkouts_toolchain_file(self):
        command, env, out, binary, _ = start_campaign({}, repetitions=4)
        repo = binary.parents[3]
        (repo / "rust-toolchain.toml").write_text('[toolchain]' + chr(10) + 'channel = "topic028-no-such-toolchain"' + chr(10))
        subprocess.run(["git", "-C", str(repo), "add", "rust-toolchain.toml"], check=True)
        subprocess.run(["git", "-C", str(repo), "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "toolchain"], check=True)
        process = subprocess.run(command, env=env, text=True, capture_output=True, timeout=300)
        self.assertEqual(process.returncode, 0, process.stderr)
        document = json.loads(out.read_text())
        self.assertIsNone(document["environment"]["rustc"])
        self.assertEqual(document["status"], "invalid: compiler identity unavailable")

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
        command, env, out, binary, _ = start_campaign({}, repetitions=4, build="true")
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
        message = json.dumps({"reason": "compiler-artifact", "target": {"name": "compare", "kind": ["example"]}, "executable": str(produced)})
        build = f"mkdir -p {produced.parent} && cp \"$STUB_SOURCE\" {produced} && echo '{message}'"
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
        message = json.dumps({"reason": "compiler-artifact", "target": {"name": "compare", "kind": ["example"]}, "executable": str(produced)})
        build = f"mkdir -p {produced.parent} && cp \"$STUB_SOURCE\" {produced} && echo '{message}'"
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
        command, env, out, binary, _ = start_campaign({}, repetitions=4)
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
