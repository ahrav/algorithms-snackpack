#!/usr/bin/env python3
"""Reducer checks for analyze.py against synthetic two-host campaigns.

Each synthetic host replays run.py's schedule: ten workloads, one warmup process
per candidate, six measured blocks with the rotation and reversal run.py uses.
Run from anywhere: python3 topics/033-tarjan-strongly-connected-components/measurements/test_analyze.py
"""
import json
import pathlib
import subprocess
import sys
import tempfile
import unittest

ANALYZE = pathlib.Path(__file__).resolve().with_name("analyze.py")
WORKLOADS = ['tiny-first', 'tiny', 'isolated', 'chain', 'cycle', 'groups', 'permuted', 'duplicates', 'dense', 'random']
CANDIDATES = ['tarjan', 'gabow', 'kosaraju', 'prepared', 'tarjan-aa']
HASHES = {'src/lib.rs': 'a' * 64, 'benches/scc.rs': 'b' * 64, 'measurements/run.py': 'c' * 64}


def records():
    for w in WORKLOADS:
        schedule = [(c, 'warmup', -1, i) for i, c in enumerate(CANDIDATES)]
        for b in range(6):
            shift = b // 2
            order = CANDIDATES[shift:] + CANDIDATES[:shift]
            if b % 2:
                order.reverse()
            schedule += [(c, 'measure', b, i) for i, c in enumerate(order)]
        for c, phase, block, position in schedule:
            ns = 1000.0 * (1 + CANDIDATES.index(c)) + block
            yield {'workload': w, 'candidate': c, 'phase': phase, 'block': block, 'position': position,
                   'exit': 0, 'certified': True, 'vertices': 7, 'edges': 8, 'components': 4,
                   'reps': 1, 'ns_total': ns, 'ns_per_op': ns}


def write_host(base, host, hashes, recs=None):
    run = pathlib.Path(base, host, 'results')
    run.mkdir(parents=True)
    (run / 'environment.json').write_text(json.dumps({'hostname': host, 'architecture': host, 'source_hashes': hashes}))
    (run / 'records.jsonl').write_text(''.join(json.dumps(r) + '\n' for r in (recs or records())))


def reduce(base):
    summary = pathlib.Path(base, 'summary.json')
    process = subprocess.run([sys.executable, str(ANALYZE), base, str(summary)], text=True, capture_output=True)
    return process, summary


class AnalyzeTests(unittest.TestCase):
    def test_identical_source_campaign_reduces(self):
        with tempfile.TemporaryDirectory(prefix='topic033-') as base:
            for host in ['arm', 'x86']:
                write_host(base, host, HASHES)
            process, summary = reduce(base)
            self.assertEqual(process.returncode, 0, process.stderr)
            out = json.loads(summary.read_text())
            self.assertEqual(set(out), {'arm', 'x86'})
            self.assertTrue(all(c['outgoing_only_selection'] == 'tarjan' for h in out.values() for c in h['cells'].values()))
            self.assertEqual(out['arm']['source_hashes'], HASHES)

    def test_inconsistent_graph_metadata_is_rejected(self):
        with tempfile.TemporaryDirectory(prefix='topic033-') as base:
            write_host(base, 'arm', HASHES)
            recs = list(records())
            recs[-1]['components'] += 1
            write_host(base, 'x86', HASHES, recs)
            process, summary = reduce(base)
            self.assertNotEqual(process.returncode, 0, 'analyze.py summarized a workload whose processes disagree on component count')
            self.assertIn('graph metadata', process.stderr)
            self.assertFalse(summary.exists())

    def test_mixed_source_campaign_is_rejected(self):
        with tempfile.TemporaryDirectory(prefix='topic033-') as base:
            write_host(base, 'arm', HASHES)
            write_host(base, 'x86', dict(HASHES, **{'src/lib.rs': 'd' * 64}))
            process, summary = reduce(base)
            self.assertNotEqual(process.returncode, 0, 'analyze.py summarized hosts that ran different src/lib.rs')
            self.assertIn('source_hashes', process.stderr)
            self.assertFalse(summary.exists(), 'analyze.py wrote a summary for a mixed-source campaign')


if __name__ == '__main__':
    unittest.main()
