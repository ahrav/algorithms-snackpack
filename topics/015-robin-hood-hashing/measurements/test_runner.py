"""Smoke check: run() retains every attempt, including a timed-out one, in raw/attempts.jsonl before aborting."""

import json
import pathlib
import re
import subprocess
import tempfile

source = pathlib.Path(__file__).with_name("runner.py").read_text()
match = re.search(r"^def run\(args\):\n(?: .*\n)+", source, re.M)
assert match is not None, "run() definition not found in runner.py"
raw = pathlib.Path(tempfile.mkdtemp())
namespace = {"RAW": raw, "subprocess": subprocess, "json": json, "TIMEOUT": 0.2}
exec(match.group(0), namespace)  # noqa: S102 - runner.py executes a campaign on import, so only run() is loaded
try:
    namespace["run"](["false"])
except AssertionError:
    pass
else:
    raise SystemExit("expected AssertionError on nonzero exit")
try:
    namespace["run"](["sleep", "2"])
except AssertionError:
    pass
else:
    raise SystemExit("expected AssertionError on timeout")
records = [json.loads(line) for line in (raw / "attempts.jsonl").read_text().splitlines()]
assert records[0]["argv"] == ["false"] and records[0]["returncode"] != 0, records
assert records[1]["argv"] == ["sleep", "2"] and records[1]["returncode"] == "timeout", records
print("ok: failed and timed-out attempts retained")
