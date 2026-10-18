#!/usr/bin/env python3
"""Rebuild F00 terrain and compare five original-level scenarios with Pascal.

All original-derived masks and full traces stay in a new assets/local directory.
Only hashes and the measurement summary should be committed.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

from state_replay import ROOT, canonical, compare, digest, run_engine, verified_assets


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--reference', required=True)
    parser.add_argument('--assets', type=Path, default=ROOT/'assets/local/dos-original')
    args = parser.parse_args()
    if not args.output.resolve().is_relative_to(ROOT/'assets/local'):
        raise ValueError('original-derived output must stay under assets/local')
    files = verified_assets(args.assets)
    args.output.mkdir(parents=True)  # Refuse overwrites and mixed runs.
    with (args.output/'terrain.jsonl').open('x') as output:
        subprocess.run(['cargo', 'run', '--quiet', '--locked', '--offline', '-p', 'lemmings-resources',
                        '--example', 'inspect_terrain', '--', str(args.assets), str(args.output/'terrain')],
                       cwd=ROOT, stdout=output, check=True)
    pixels = (args.output/'terrain/level-000.pixels.bin').read_bytes()
    mask = bytes(int(v < 128) for v in pixels)
    mask_sha = hashlib.sha256(mask).hexdigest()
    if mask_sha != 'a5311ef0436ca29a5fe4f6feee89baa0b82afce87baaf799f59c813deb5686d7':
        raise ValueError('F00 initial mask differs from the frozen resource result')
    mask_path = args.output/'f00.mask'
    mask_path.write_bytes(mask)
    base = dict(width=1584, height=160, minutes=5, total=10, rate=50, entrances=[[704, 36]],
                pause_glitch=True, faller_three=True, steps=340, inputs=[], level='Fun 01 Just dig!')
    scenarios = [('F00', base)]
    for rate in [98, 99]:
        scenarios.append((f'F02-{rate}', base | dict(inputs=[dict(control_step=54, sequence=0, command='SetReleaseRate', rate=rate)])))
    pause = [dict(control_step=16, sequence=0, command='Pause'), dict(control_step=34, sequence=0, command='Resume')]
    for glitch in [False, True]:
        scenarios.append((f'F03-glitch{int(glitch)}', base | dict(inputs=pause, pause_glitch=glitch)))
    rows = []
    calibration = None
    provenance = None
    for name, config in scenarios:
        (args.output/(name+'.config.json')).write_bytes(canonical(config))
        reference = run_engine(config, mask_path, args.reference, input_files=files)
        provenance = reference['header']['provenance']
        ref_data = canonical(reference)
        (args.output/(name+'.reference.json')).write_bytes(ref_data)
        comparisons = []
        for pattern in ['single', 'burst', 'jitter']:
            rust = run_engine(config, mask_path, 'rust', pattern, input_files=files)
            (args.output/(name+'.rust-'+pattern+'.json')).write_bytes(canonical(rust))
            result = compare(reference, rust)
            (args.output/(name+'.compare-'+pattern+'.json')).write_bytes(canonical(result))
            if result['status'] != 'MATCH_SUPPORTED':
                raise AssertionError(f'{name}/{pattern}: {result}')
            comparisons.append(pattern)
        if name == 'F00':
            calibration = next(s['control_step'] for s in reference['snapshots']
                               if s['actors'] and s['actors'][0]['action'] == 'Walking')
        rows.append(dict(scenario=name, config_sha256=digest(config), input_sha256=digest(config['inputs']),
                         snapshots_sha256=digest(reference['snapshots']), reference_trace_sha256=hashlib.sha256(ref_data).hexdigest(),
                         snapshots=len(reference['snapshots']), render_patterns=comparisons))
    summary = dict(format='g05-original-replay-evidence-v1', tracking_issue=26,
                   observation='extracted Lemmix methods with G05 adapter; full engine/DOS not run',
                   mask_sha256=mask_sha, original_files=files, reference=provenance,
                   scenarios=rows, signature=digest(rows),
                   f01_calibration=dict(control_step=calibration, sequence=0, actor_id=0, skill='digger',
                                        status='measured next boundary only; AssignSkill not implemented/run'),
                   status='MATCH_SUPPORTED', unverified=reference['header']['unverified'])
    expected = json.loads((ROOT/'assets/manifests/state_replay.json').read_text())
    keys = ['scenario', 'config_sha256', 'input_sha256', 'snapshots_sha256', 'snapshots']
    if [{k: row[k] for k in keys} for row in rows] != [{k: row[k] for k in keys} for row in expected['scenarios']]:
        raise AssertionError('original-level state differs from recorded reference execution')
    if calibration != expected['f01_calibration']['control_step']:
        raise AssertionError('F01 calibration boundary changed')
    (args.output/'evidence.json').write_bytes(canonical(summary))
    print(json.dumps(dict(status=summary['status'], scenarios=len(rows), snapshots_per_replay=341,
                          f01_next_boundary=calibration, signature=summary['signature'])))


if __name__ == '__main__':
    main()
