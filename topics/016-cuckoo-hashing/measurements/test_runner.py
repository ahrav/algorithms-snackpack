"""Smoke check: invoke() retains every attempt, including failed and timed-out ones, in attempts.jsonl before aborting."""

import json
import pathlib
import re
import subprocess
import tempfile

source = pathlib.Path(__file__).with_name("runner.py").read_text()
match = re.search(r"^def invoke\(.*\):\n(?: .*\n)+", source, re.M)
assert match is not None, "invoke() definition not found in runner.py"
helper = re.search(r"^def text_of\(.*\n", source, re.M)
raw = pathlib.Path(tempfile.mkdtemp())
namespace = {"ATTEMPTS": raw / "attempts.jsonl", "subprocess": subprocess, "json": json, "TIMEOUT": 0.2}
exec((helper.group(0) if helper else "") + match.group(0), namespace)  # noqa: S102 - runner.py runs a campaign on import
try:
    namespace["invoke"](["false"])
except AssertionError:
    pass
else:
    raise SystemExit("expected AssertionError on nonzero exit")
assert (raw / "attempts.jsonl").exists(), "failed attempt not retained"
hang = ["bash", "-c", "printf before-timeout; sleep 2"]
try:
    namespace["invoke"](hang)
except AssertionError:
    pass
else:
    raise SystemExit("expected AssertionError on timeout")
records = [json.loads(line) for line in (raw / "attempts.jsonl").read_text().splitlines()]
assert records[0]["argv"] == ["false"] and records[0]["returncode"] != 0, records
assert records[1]["argv"] == hang and records[1]["returncode"] == "timeout", records
assert records[1]["stdout"] == "before-timeout", records[1]
print("ok: failed and timed-out attempts retained")
