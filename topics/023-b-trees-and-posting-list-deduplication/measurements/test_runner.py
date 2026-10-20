"""Smoke check: a rerun into an existing output directory leaves attempts.jsonl describing only the current campaign."""

import json
import pathlib
import subprocess
import sys
import tempfile

run = pathlib.Path(__file__).resolve().parent.parent / "scripts" / "run.py"
out = pathlib.Path(tempfile.mkdtemp())
for _ in range(2):
    subprocess.run([sys.executable, str(run), str(out)], check=True, stdout=subprocess.DEVNULL)
schedule = json.loads((out / "schedule.json").read_text())
attempts = [json.loads(line) for line in (out / "attempts.jsonl").read_text().splitlines()]
raw = (out / "raw.jsonl").read_text().splitlines()
assert len(raw) == len(schedule), (len(raw), len(schedule))
assert len(attempts) == len(schedule), f"attempts.jsonl holds {len(attempts)} records for a {len(schedule)}-process campaign"
assert [{k: a[k] for k in ("block", "cell", "label", "candidate")} for a in attempts] == schedule
print(f"ok: {len(attempts)} attempts match the current schedule after rerun")
