#!/usr/bin/env python3
"""Verify the scoped COMP evidence index; --local also requires private proof files.

This checks evidence integrity and coverage. It does not run a game or turn
recorded observations into new DOS certification. See issue-104's report.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
DETAILS = {
    1: 'sections damaged', 2: 'catalog odd special invalid_load',
    3: 'composition palette masks edits', 4: 'clock startup first_spawn',
    5: 'rates entrances cap', 6: 'walking stepping fall bounds',
    7: 'normal rejection zero_inventory permanent',
    8: 'ordered_assignment shared_terrain blocker explosion',
    9: 'steel one_way miner_bug splat_exit',
    10: 'slots trap_reuse hazards priority',
    11: 'time_zero target_boundary integer_percent frozen_result restart',
    12: 'pause glitch_options',
    13: 'denominator duplicate scan spawn_stop glitch_options',
    14: 'overlap remote trapped raw_input',
    15: 'pacing canvas hidden_resume atomic_load',
    16: 'catalog driver pcm events lifecycle device special_music',
    17: 'load success loss restart', 18: 'f01 repeats provenance',
}
REQUIRED_DETAILS = {f'COMP-{number:02}.{detail}'
                    for number, details in DETAILS.items() for detail in details.split()}
LEVELS = {f'{rank}-{number:02}' for rank in ['fun', 'tricky', 'taxing', 'mayhem']
          for number in range(1, 31)}
VARIANTS = {(1, 30), (7, 60), (13, 144)}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':'),
                                     ensure_ascii=False).encode()).hexdigest()


def check_pin(root, pin):
    path = Path(pin['path'])
    path = path if path.is_absolute() else root / path
    require(path.is_file(), f'missing evidence: {pin["path"]}')
    data = path.read_bytes()
    require(len(data) == pin['bytes'] and hashlib.sha256(data).hexdigest() == pin['sha256'],
            f'evidence changed: {pin["path"]}')
    return data


def validate_requirements(rows, evidence, sources, root):
    require(len(rows) == len(REQUIRED_DETAILS)
            and {r['id'] for r in rows} == REQUIRED_DETAILS, 'missing/duplicate COMP detail')
    for row in rows:
        require(row['verdict'] == 'PASS-SCOPED', f'unresolved: {row["id"]}')
        require(bool(row['evidence']) and set(row['evidence']) <= evidence,
                f'missing/unknown evidence: {row["id"]}')
        require(bool(row['product']) and set(row['product']) <= sources,
                f'unpinned product: {row["id"]}')
        require(bool(row['tests']), f'missing tests: {row["id"]}')
        for test in row['tests']:
            require(test['path'] in sources, f'unpinned test: {row["id"]}')
            text = (root / test['path']).read_text()
            require(re.search(r'\bfn\s+' + re.escape(test['name']) + r'\s*\(', text),
                    f'missing test anchor: {test}')


def read_coverage(coverage, root):
    result = dict(coverage)
    for key in ['solutions', 'initial']:
        if isinstance(result[key], dict):
            result[key] = [json.loads(line) for line in check_pin(root, result[key]).decode().splitlines()]
    return result


def validate_coverage(coverage, root):
    coverage = read_coverage(coverage, root)
    authored = {}
    for pin in coverage['input_manifests']:
        data = json.loads(check_pin(root, pin))
        for row in data['cases']:
            key = (row['id'], row['phase'])
            require(key not in authored, f'duplicate input: {key}')
            require(hashlib.sha256(row['input_text'].encode()).hexdigest() == row['input_sha256'],
                    f'input digest: {key}')
            authored[key] = row
    expected = {(level, phase) for level in LEVELS for phase in ['Won', 'Lost']}
    rows = coverage['solutions']
    require(len(rows) == 240 and {(r['id'], r['phase']) for r in rows} == expected
            and set(authored) == expected, 'incomplete solution coverage')
    catalog = {r['id']: r for r in json.loads((root / 'assets/manifests/dos_compatibility.json').read_text())['levels']}
    for row in rows:
        key = (row['id'], row['phase'])
        require(row['key'] == row['id'] + '-' + row['phase'].lower(), f'case identity: {key}')
        original = authored[key]
        for field in ['steps', 'input_sha256', 'final']:
            expected_value = original['discovery_result'] if field == 'final' else original[field]
            require(row[field] == expected_value, f'input/result changed: {key}/{field}')
        final = row['final']
        header = catalog[row['id']]
        require((final['original_total'], final['required']) == (header['total'], header['required']),
                f'original goal changed: {key}')
        require(final['phase'] == row['phase'] and final['reason'] in ['PopulationExhausted', 'TimeExpired']
                and (final['rescued'] >= final['required']) == (row['phase'] == 'Won'), f'outcome: {key}')
        target = final['required'] * 100 // final['original_total'] if final['original_total'] else 0
        done = final['rescued'] * 100 // final['lemming_count'] if final['lemming_count'] else 0
        require((final['target_percent'], final['rescued_percent']) == (target, done), f'percent: {key}')
        proof = {k: row[k] for k in ['key', 'id', 'phase', 'steps', 'control_step', 'input_sha256',
                                   'state_maps_sha256', 'dense_frames_sha256', 'reference_sha256', 'final']}
        require(digest(proof) == row['proof_sha256'], f'proof digest: {key}')
        native = row['native']
        require(len(native) == 3 and {(r['render_every'], r['elapsed_rate']) for r in native} == VARIANTS,
                f'native variants: {key}')
        require(all(r['probes'] == row['steps'] + 1 and r['frames'] > 0
                    and r['state_map_sha256'] == row['state_maps_sha256'] for r in native), f'native trace: {key}')
        web = row['web']
        require(len(web) == 3 and {(r['render'], r['rate']) for r in web} == VARIANTS, f'web variants: {key}')
        for run in web:
            require(run['final'] == final and len(run['attempts']) == 2, f'web restart: {key}')
            require(run['attempts'][0] == run['attempts'][1], f'web replay changed: {key}')
            for attempt in run['attempts']:
                require(attempt['final'] == final and attempt['hidden_resume']
                        and attempt['probes'] == row['steps'] + 1 and attempt['frames'] > 0,
                        f'web attempt: {key}')
        restart = row['restarts']
        require(len(restart) == 3 and {r['render'] for r in restart} == {37, 101, 257}, f'SDL restarts: {key}')
        rank, number = row['id'].rsplit('-', 1)
        require(all(len(r['transitions']) == 1 and r['transitions'][0]['action'] == 'Restart'
                    and r['transitions'][0]['level'] == r['transitions'][0]['target'] == int(number)
                    and r['transitions'][0]['difficulty'] == r['transitions'][0]['target_difficulty'] == rank
                    and r['summary']['input_count'] == 0 and r['reset_probes'] > 0 for r in restart),
                f'SDL restart reset: {key}')
    totals = dict(native_replays=len(rows) * 3, native_restart_runs=len(rows) * 3,
                  wasm_replays=len(rows) * 3, wasm_restart_replays=len(rows) * 3,
                  native_probes=sum(r['probes'] for c in rows for r in c['native']),
                  wasm_probes=sum(r['probes'] for c in rows for r in c['web']),
                  sdl_stream_readbacks=sum(r['frames'] for c in rows for r in c['native']),
                  sdl_restart_readbacks=sum(r['readbacks'] for c in rows for r in c['restarts']),
                  canvas_readbacks=sum(r['frames'] for c in rows for r in c['web']),
                  reference_states=sum(c['control_step'] + 1 for c in rows))
    require(totals == coverage['totals'], 'aggregate counts differ')
    initial = coverage['initial']
    require(len(initial) == 120 and {r['id'] for r in initial} == LEVELS, 'incomplete initial coverage')
    for row in initial:
        require(len(row['native']) == len(row['web']) == 3
                and {(r['render_every'], r['elapsed_rate']) for r in row['native']} == VARIANTS
                and {(r['render'], r['rate']) for r in row['web']} == VARIANTS, f'initial variants: {row["id"]}')
        require(all(r['probes'] == 341 and r['state_map_sha256'] == row['state_maps_sha256'] for r in row['native'])
                and all(r['probes'] == 341 and r['hidden_resume'] for r in row['web']), f'initial state: {row["id"]}')


def audit(path, *, local=False, root=ROOT):
    data = json.loads(path.read_text())
    require(data['schema_version'] == 1 and data['verdict'] == 'PASS-SCOPED', 'audit format/status')
    sources = data['product_snapshot']
    require(len({p['path'] for p in sources}) == len(sources), 'duplicate product source')
    for pin in sources:
        check_pin(root, pin)
    evidence = data['evidence']
    require(len({e['id'] for e in evidence}) == len(evidence), 'duplicate evidence ID')
    for pin in evidence:
        check_pin(root, pin)
    validate_requirements(data['requirements'], {e['id'] for e in evidence}, {p['path'] for p in sources}, root)
    validate_coverage(data['coverage'], root)
    if local:
        for pin in data['local_evidence']:
            check_pin(root, pin)
    return dict(status='PASS-SCOPED', comp=18, details=len(REQUIRED_DETAILS), levels=120,
                solutions=240, product_files=len(sources), local_evidence_checked=local,
                scope='Index integrity and recorded coverage; no new DOS/performance measurement')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest', type=Path, default=ROOT / 'assets/manifests/compatibility_audit.json')
    parser.add_argument('--local', action='store_true')
    args = parser.parse_args()
    print(json.dumps(audit(args.manifest, local=args.local), ensure_ascii=False))


if __name__ == '__main__':
    main()
