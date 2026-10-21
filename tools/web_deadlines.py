#!/usr/bin/env python3
"""Summarize raw Web observations without equating rAF timing with scanout."""
import argparse
import json
import math
from pathlib import Path
import sys


def distribution(values):
    if not values or any(type(v) not in (int, float) or not math.isfinite(v) for v in values):
        raise ValueError('finite nonempty observations required')
    ordered = sorted(values)
    return dict(count=len(values), median=(ordered[(len(ordered)-1)//2]+ordered[len(ordered)//2])/2,
                p95=ordered[math.ceil(len(ordered)*0.95)-1], max=ordered[-1])


def frame_summary(rows):
    if not rows:
        raise ValueError('no frame observations')
    for i, row in enumerate(rows):
        if len(row) < 3 or any(type(v) not in (int, float) or not math.isfinite(v) for v in row):
            raise ValueError('invalid frame observation')
        if row[2] < row[1] or (i and row[0] < rows[i-1][0]):
            raise ValueError('backwards frame observation')
    budget = 1000/60
    costs = [r[2]-r[1] for r in rows]
    delays = [r[1]-r[0] for r in rows]
    late = [max(0, r[2]-r[0]-budget) for r in rows]
    gaps = [b[0]-a[0] for a,b in zip(rows,rows[1:])]
    return dict(frames=len(rows), callback_ms=distribution(costs),
                callback_over_budget=sum(v > budget for v in costs),
                start_delay_ms=distribution(delays), completion_late_ms=distribution(late),
                completion_misses=sum(v > 0 for v in late),
                timestamp_gaps_over_two_periods=sum(v > 2*budget for v in gaps),
                scope='60Hz contract: end <= rAF timestamp + 1000/60ms; not physical scanout')


def memory_summary(samples, wasm_start, wasm_end, recorder_bytes):
    if not samples or type(wasm_start) is not int or type(wasm_end) is not int or not 0 < wasm_start <= wasm_end:
        raise ValueError('memory observations and monotonic WASM sizes required')
    if type(recorder_bytes) is not int or recorder_bytes < 0:
        raise ValueError('invalid recorder capacity')
    for i, row in enumerate(samples):
        for key in ['at_ms','usedSize','totalSize']:
            if type(row[key]) not in (int, float) or not math.isfinite(row[key]) or row[key] < 0:
                raise ValueError('invalid memory sample')
        if row['usedSize'] > row['totalSize'] or (i and row['at_ms'] < samples[i-1]['at_ms']):
            raise ValueError('invalid memory size/order')
    extra = {}
    for field in ['backingStorageSize', 'embedderHeapUsedSize']:
        if any(field in row for row in samples):
            if not all(type(row.get(field)) in (int, float) and math.isfinite(row[field]) and row[field] >= 0 for row in samples):
                raise ValueError('incomplete/invalid optional memory counter')
            extra[field+'_sampled_peak_bytes'] = max(row[field] for row in samples)
    return dict(**extra, samples=len(samples), js_used_sampled_peak_bytes=max(s['usedSize'] for s in samples),
                js_used_last_sample_bytes=samples[-1]['usedSize'], js_capacity_sampled_peak_bytes=max(s['totalSize'] for s in samples),
                first_sample_at_ms=samples[0]['at_ms'], last_sample_at_ms=samples[-1]['at_ms'],
                max_sample_gap_ms=max((b['at_ms']-a['at_ms'] for a,b in zip(samples,samples[1:])),default=0),
                continuous_js_peak_observed=False, wasm_committed_start_bytes=wasm_start,
                wasm_committed_peak_bytes=wasm_end, recorder_reserved_bytes=recorder_bytes,
                scope='CDP isolate heap sampled maximum; misses between-sample peaks. WASM committed memory never shrinks. Do not sum overlapping backing storage/heap/WASM counters.')


ROOT = Path(__file__).resolve().parents[1]
if __package__ in (None, ''):
    sys.path.insert(0, str(ROOT))
from tools.compatibility_audit import check_pin, require


def validate_ready(rows, verdict):
    require(len(rows) == 3, 'three timing repeats required')
    ready = all(r['completion_misses'] == r['callback_over_budget'] == 0 for r in rows)
    require(type(verdict['web_deadline_ready']) is bool and verdict['web_deadline_ready'] == ready,
            'stored readiness disagrees with recorded misses')
    if not ready:
        require(all(verdict.get(field, False) is False for field in ['g11_ready', 'g12_entry_ready']),
                'unresolved Web gate cannot declare G11/G12 ready')
        require(str(verdict.get('followup', '')).startswith('https://github.com/edwardkim/ohmylemmings/issues/'),
                'misses require an open follow-up')
    return ready


def outside_presents(raw):
    callbacks = raw['callbackDetails']
    count = 0
    index = 0
    for frame in raw['frames']:
        while index < len(callbacks) and callbacks[index][2] < frame[0]:
            index += 1
        if index == len(callbacks) or not callbacks[index][1] <= frame[0] <= callbacks[index][2]:
            count += 1
    return count


def audit(path, *, local=False, root=ROOT):
    data = json.loads(path.read_text())
    require(data['schema_version'] == 1 and data['status'] == 'MEASURED-WITH-DISPATCH-FOLLOWUP', 'evidence format')
    for pin in data['source']:
        check_pin(root, pin)
    reference = json.loads(check_pin(root, data['reference']))['web'][0]['output_sha256']
    vectors = [json.loads(line) for line in check_pin(root, data['observations']).decode().splitlines()]
    require(len({r['id'] for r in vectors}) == len(vectors) == len(data['runs']) == 7, 'missing/duplicate run')
    by_id = {r['id']:r for r in vectors}
    require({r['id'] for r in data['runs']} == set(by_id) and len({r['id'] for r in data['runs']}) == 7,
            'run identities differ from observations')
    summaries = []
    profiles = {}
    for run in data['runs']:
        vector = by_id[run['id']]
        rows = [[v/1_000_000 for v in row] for row in vector['callback_ns']]
        summary = frame_summary(rows)
        require(summary == run['frame_summary'], 'frame summary differs')
        profiles.setdefault(run['profile'], []).append(run['repeat'])
        require(60 <= run['seconds'] < 62 and run['ticks'] == 841 and run['output_sha256'] == reference
                and run['underruns'] == 0 and run['visibility_events'] == 0, 'invalid/interrupted workload')
        require(run['outside_raf_presents'] >= 0, 'invalid outside-RAF count')
        if run['profile'] == 'timing':
            require(not vector['heap_samples'] and run['heap_interval_ms'] == 0, 'timing profile sampled memory')
            summaries.append(summary)
        if run['profile'] == 'memory':
            require(run['heap_interval_ms'] == 500 and len(vector['heap_samples']) >= 100, 'insufficient heap samples')
            require(0 <= vector['heap_samples'][0]['at_ms'] <= vector['heap_samples'][-1]['at_ms'] < run['seconds']*1000, 'heap samples outside interval')
            require(memory_summary(vector['heap_samples'], run['wasm_start'], run['wasm_end'], run['recorder_bytes']) == run['memory'],
                    'memory summary differs')
        if local:
            raw = json.loads(check_pin(root, run['raw']))
            require(vector['callback_ns'] == [[round(v*1_000_000) for v in row[:3]] for row in raw['callbackDetails']]
                    and vector['heap_samples'] == raw['heapSamples'], 'public observations differ from raw')
            require(run['outside_raf_presents'] == outside_presents(raw), 'outside-RAF count differs')
            require(run['output_sha256'] == raw['outputHashes'] and run['ticks'] == len(raw['ticks'])
                    and run['underruns'] == raw['audio']['underruns']-raw['initialAudioUnderruns'], 'raw result differs')
            for i, name in enumerate(raw['callbackColumns'][3:9], 3):
                require(run['cost_ms'][name] == distribution([r[i] for r in raw['callbackDetails']]), 'raw cost summary differs')
    require(profiles == dict(memory=[1,2,3],control=[1],timing=[1,2,3]), 'profile/repeat coverage')
    ready = validate_ready(summaries, data['verdict'])
    require(not ready, 'this record retains unresolved dispatch misses')
    if local:
        for pin in data['recorded_producer']:
            check_pin(root, pin)
        baseline = json.loads(check_pin(root, data['reference']))
        for pin in baseline['original_files']:
            check_pin(root, pin)
    return dict(evidence_valid=True, web_deadline_ready=ready, g11_ready=False, g12_entry_ready=False,
                timing_misses=[r['completion_misses'] for r in summaries],
                timing_callback_over_budget=[r['callback_over_budget'] for r in summaries],
                followup=data['verdict']['followup'], local_evidence_checked=local,
                scope='All observed misses retained; timing/memory profiles and sampled/continuous peaks distinct')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest', type=Path, default=ROOT/'assets/manifests/web_deadlines.json')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--require-ready', action='store_true')
    args = parser.parse_args()
    result = audit(args.manifest, local=args.local)
    print(json.dumps(result))
    if args.require_ready and not result['web_deadline_ready']:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
