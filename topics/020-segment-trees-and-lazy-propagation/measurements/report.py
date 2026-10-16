"""Combine per-host raw summaries into measurements/summary.json and print the README table.

Usage: python3 report.py arm=/path/to/arm/raw/summary.json x86=/path/to/x86/raw/summary.json

Each host value is the runner's raw/summary.json, copied unchanged under its host key.
Table cells are median_ns / 8000: nanoseconds per eight internal replays become
microseconds per one replay, with the descriptive min/max of the eight process
aggregates divided the same way.
"""

import json
import pathlib
import sys

HERE = pathlib.Path(__file__).resolve().parent
CELLS = ["tiny", "read_only", "mixed", "short", "update_heavy", "hot", "adversarial", "lifecycle"]
REPLAYS_PER_PROCESS = 8


def micros(ns):
    return ns / (1000 * REPLAYS_PER_PROCESS)


def cell_text(summary, cell):
    s = summary[cell]
    label = s["winner"]
    if label in ("unresolved", "invalid"):
        label = min(s["median_ns"], key=s["median_ns"].get)
    lo, hi = s["range_ns"][label]
    return f"{micros(s['median_ns'][label]):.2f} [{micros(lo):.2f},{micros(hi):.2f}]"


def table(hosts):
    names = list(hosts)
    lines = ["| Workload | Winner per host | " + " | ".join(f"{h} us [min,max]" for h in names) + " |"]
    lines.append("|---|---|" + "---:|" * len(names))
    for cell in CELLS:
        winners = {hosts[h][cell]["winner"] for h in names}
        label = winners.pop() if len(winners) == 1 else ", ".join(f"{h}: {hosts[h][cell]['winner']}" for h in names)
        lines.append(f"| {cell} | {label} | " + " | ".join(cell_text(hosts[h], cell) for h in names) + " |")
    return "\n".join(lines)


def main(argv):
    hosts = {}
    for arg in argv:
        host, _, path = arg.partition("=")
        if not path:
            raise SystemExit(__doc__)
        try:
            hosts[host] = json.loads(pathlib.Path(path).read_text())
        except (OSError, json.JSONDecodeError) as e:
            raise SystemExit(f"{host}: cannot read raw summary {path}: {e}") from e
    if not hosts:
        raise SystemExit(__doc__)
    (HERE / "summary.json").write_text(json.dumps(hosts, indent=2) + "\n")
    print(table(hosts))


if __name__ == "__main__":
    main(sys.argv[1:])
