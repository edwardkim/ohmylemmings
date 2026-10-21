#!/usr/bin/env python3
"""Recheck recorded G11 evidence, without replaying games or claiming readiness.

The default succeeds when this review is internally valid, including an honest
not-ready verdict. --require-ready additionally enforces the G11 entry gate.
"""
import argparse
from collections import Counter
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
if __package__ in (None, ''):
    sys.path.insert(0, str(ROOT))
from tools.compatibility_audit import audit as compatibility_audit, check_pin, require
from tools.performance import summarize


def callback_summary(values):
    require(bool(values) and all(type(v) is int and v > 0 for v in values),
            'callback durations must be positive integer nanoseconds')
    return dict(count=len(values), max_ns=max(values),
                over_budget=sum(v * 60 > 1_000_000_000 for v in values))


def readiness(web):
    """Compute the outstanding Web component of the recorded G11 gate."""
    costs = web['callback_cost_ns']
    require(len(costs) == 3, 'exactly three accepted Web repeats required')
    summaries = [callback_summary(values) for values in costs]
    for field in ['completion_deadline_observed', 'interval_memory_peak_observed']:
        require(type(web[field]) is bool, f'boolean required: {field}')
    unmet = []
    if any(row['over_budget'] for row in summaries):
        unmet.append('web-callback-budget')
    if not web['completion_deadline_observed']:
        unmet.append('web-completion-deadline')
    if not web['interval_memory_peak_observed']:
        unmet.append('web-interval-memory')
    return dict(g11_ready=not unmet, unmet=unmet)


def validate_verdict(verdict, web):
    expected = readiness(web)
    require(type(verdict['g11_ready']) is bool
            and all(verdict[key] == expected[key] for key in expected),
            'stored verdict disagrees with observed evidence')
    if expected['unmet']:
        require(isinstance(verdict.get('followup'), str)
                and verdict['followup'].startswith('https://github.com/edwardkim/ohmylemmings/issues/'),
                'unresolved evidence requires a follow-up issue')


def audit(path, *, local=False, root=ROOT):
    data = json.loads(path.read_text())
    require(data['schema_version'] == 1 and data['status'] == 'REVIEW-VALID', 'review format')
    pins = {pin['id']: pin for pin in data['evidence']}
    evidence = {name: json.loads(check_pin(root, pin)) for name, pin in pins.items()}
    require(len(evidence) == len(data['evidence']), 'duplicate evidence ID')
    comp = compatibility_audit(root / pins['compatibility']['path'], local=local, root=root)
    require(comp['levels'] == 120 and comp['solutions'] == 240 and comp['details'] == 70,
            'functional coverage incomplete')
    perf = evidence['performance']
    kinds = Counter(row['kind'] for row in perf['algorithm'])
    require(kinds == dict(memory_load=120, tick=12, cpu_frame=12, pcm=40,
                          wasm_load=120, wasm_tick=12, wasm_cpu_frame=12,
                          canvas_transfer=12, wasm_pcm=40), 'algorithm workload coverage')
    for row in perf['algorithm']:
        stats = summarize(row['raw_ns'], row['operations_per_sample'])
        require(all(row[key] == value for key, value in stats.items()), 'algorithm statistics changed')
        require(len(row['warmup_ns']) == 3 and all(v > 0 for v in row['warmup_ns'])
                and len(row['initialization_ns']) == 33 and all(v >= 0 for v in row['initialization_ns'])
                and bool(row['output_digest']), 'algorithm warmups/initialization/output missing')
    require(len(perf['workloads']) == 12 and len(perf['sounds']) == 40, 'workload/sound inventory')
    display = evidence['display']
    candidates = [r for r in display['runs'] if r['kind'] == 'candidate']
    require(len(display['runs']) == 6 and len(candidates) == 3
            and {r['pair'] for r in candidates} == {1, 2, 3}, 'native paired runs')
    require(all(r['deadline_misses'] == r['skipped_slots'] == r['underruns'] == 0
                and r['scheduled_frames'] > 0 and r['ticks'] == 841 for r in candidates),
            'native display criteria unmet')
    identity = display['identity']
    require(identity['status'] == 'PASS'
            and sum(r['files'] for r in identity['original_and_synthetic'] if r['same']) == 451
            and len(identity['pcm']) == 40 and all(r['same_as_106'] for r in identity['pcm']),
            'native correction identity incomplete')
    for pin in display['source']:
        check_pin(root, pin)
    web = data['web']
    require(web['completion_deadline_observed'] is False
            and web['interval_memory_peak_observed'] is False,
            'historical recording does not contain deadline/interval peak observations')
    validate_verdict(data['verdict'], web)
    require(data['g12_entry_ready'] is False and data['verdict']['g11_ready'] is False,
            'this frozen review records an unresolved gate')
    require(web['summaries'] == [callback_summary(v) for v in web['callback_cost_ns']],
            'callback summaries differ')
    require(len(perf['web']) == 3, 'accepted Web baseline repeats')
    for values, run in zip(web['callback_cost_ns'], perf['web']):
        require(len(values) == run['callback_ms']['count']
                and max(values) == round(run['callback_ms']['max'] * 1_000_000),
                'callback data differs from pinned baseline')
        if local:
            raw = json.loads(check_pin(root, run['raw']))
            require(values == [round(row[1] * 1_000_000) for row in raw['callbacks']],
                    'public callback vector differs from original recording')
    if local:
        for pin in perf['original_files']:
            check_pin(root, pin)
        for run in display['runs']:
            check_pin(root, run['raw'])
    return dict(review_valid=True, g11_ready=False, g12_entry_ready=False,
                unmet=data['verdict']['unmet'], followup=data['verdict']['followup'],
                comp=comp, algorithm_rows=len(perf['algorithm']),
                web_callbacks=web['summaries'], local_evidence_checked=local,
                scope='Recorded evidence recheck; no new play, deadline or memory measurement')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest', type=Path, default=ROOT / 'assets/manifests/g11_review.json')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--require-ready', action='store_true')
    args = parser.parse_args()
    result = audit(args.manifest, local=args.local)
    print(json.dumps(result, ensure_ascii=False))
    if args.require_ready and not result['g11_ready']:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
