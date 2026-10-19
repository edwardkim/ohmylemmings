#!/usr/bin/env python3
"""Compare Builder SDL states and complete terrain additions/deletions to Pascal."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

from check_basher import synthetic_masks as bashing_masks
from check_desktop import verify_pixels
from check_miner import collect as collect_miner, replay_text, synthetic_masks as mining_masks
from state_replay import ROOT, digest

GOLDEN = ROOT/'tools/fixtures/builder_golden.json'
ADAPTERS = ['build_builder_reference.py', 'build_miner_reference.py', 'build_basher_reference.py',
            'build_digger_reference.py', 'build_reference.py', 'harness.pas']
CASES = {
    'right_twelve_bricks': [(80, 0, 'AssignBuilder', 0)],
    'left_twelve_bricks': [(200, 0, 'AssignBuilder', 0)],
    'falling_duplicate_absent': [(54, 0, 'AssignBuilder', 0), (80, 0, 'AssignBuilder', 0),
                                 (80, 1, 'AssignBuilder', 0), (80, 2, 'AssignBuilder', 999)],
    'pause_assign_resume': [(80, 0, 'Pause', 0), (80, 1, 'AssignBuilder', 0), (90, 0, 'Resume', 0)],
    'replace_all_jobs': [(80, 0, 'AssignBuilder', 0), (100, 0, 'AssignDigger', 0),
                         (101, 0, 'AssignBuilder', 0), (120, 0, 'AssignBasher', 0),
                         (121, 0, 'AssignBuilder', 0), (140, 0, 'AssignMiner', 0),
                         (141, 0, 'AssignBuilder', 0)],
    'two_builders': [(80, 0, 'AssignBuilder', 0), (110, 0, 'AssignBuilder', 1)],
    'restart_shrugging': [(80, 0, 'AssignBuilder', 0), (272, 0, 'AssignBuilder', 0)],
    'head_collision': [(150, 0, 'AssignBuilder', 0)],
}


def collect(directory):
    states = collect_miner(directory)
    details = [json.loads(s) for s in (directory/'builder.jsonl').read_text().splitlines()]
    if len(states) != len(details):
        raise ValueError('missing Builder inventory boundary')
    for state, detail in zip(states, details):
        state['builder'] = detail
    return states


def reference_run(executable, config, mask, objects, bash_masks, mine_masks, inputs,
                  inventory=(10, 10, 10, 10)):
    header = [config[k] for k in ['width', 'height', 'minutes', 'total', 'rate']]
    header += [len(config['entrances']), int(config['pause_glitch']), int(config['faller_three']), config['steps']]
    script = ' '.join(map(str, header))+'\n'+' '.join(map(str, inventory))+' 1\n'
    script += ''.join(f'{x} {y}\n' for x, y in config['entrances'])+str(len(inputs))+'\n'
    codes = dict(Pause=0, Resume=1, SetReleaseRate=2, AssignDigger=3, AssignBasher=4, AssignMiner=5, AssignBuilder=6)
    script += ''.join(f'{s} {q} {codes[c]} {v}\n' for s, q, c, v in inputs)
    result = subprocess.run([str(executable.resolve()), str(mask), str(objects), str(bash_masks), str(mine_masks)],
                            input=script, capture_output=True, text=True, check=True)
    return [json.loads(line) for line in result.stdout.splitlines()]


def run_native(binary, directory, replay, render, *, steps=420, assets=None, level=7,
               fast=True, pacing=None):
    command = [str(binary.resolve()), '--steps', str(steps), '--render-every', str(render),
               '--capture', str(directory), '--replay', str(replay)]
    command += ['--assets', str(assets), '--level', str(level)] if assets else ['--synthetic-builder']
    if fast:
        command += ['--fast']
    if pacing:
        command += ['--pacing', pacing]
    subprocess.run(command, env=dict(os.environ, SDL_VIDEODRIVER=os.environ.get('SDL_VIDEODRIVER', 'dummy')),
                   check=True, capture_output=True, text=True)
    return collect(directory), verify_pixels(directory)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT/'target/debug/lemmings-desktop')
    parser.add_argument('--reference', type=Path)
    parser.add_argument('--record', action='store_true')
    args = parser.parse_args()
    if args.record and not args.reference:
        raise ValueError('record requires actual reference execution')
    frozen = json.loads(GOLDEN.read_text()) if not args.record else None
    hashes = {name: hashlib.sha256((ROOT/'tools/lemmix'/name).read_bytes()).hexdigest() for name in ADAPTERS}
    if frozen and hashes != frozen['adapter_sha256']:
        raise ValueError('reference adapter changed; execute reference again')
    config = dict(width=640, height=160, minutes=5, total=10, rate=50, entrances=[[40, 40]],
                  pause_glitch=True, faller_three=True, steps=420)
    mask = bytes(int((y >= 100 and x < 500) or (60 <= y < 100 and 160 <= x < 170))
                 for y in range(160) for x in range(640))
    mask_hash = hashlib.sha256(mask).hexdigest()
    if frozen and frozen['synthetic_terrain_sha256'] != mask_hash:
        raise ValueError('synthetic terrain changed; execute reference again')
    rows = {}
    captures = snapshots = 0
    with tempfile.TemporaryDirectory(prefix='lemmings-builder-') as temporary:
        root = Path(temporary)
        (root/'mask').write_bytes(mask)
        (root/'objects').write_bytes(bytes(len(mask)))
        (root/'mine-masks').write_bytes(mining_masks())
        (root/'bash-masks').write_bytes(bashing_masks())
        for name, inputs in CASES.items():
            replay = root/(name+'.txt')
            replay.write_text(replay_text(inputs))
            expected = reference_run(args.reference, config, root/'mask', root/'objects', root/'bash-masks',
                                     root/'mine-masks', inputs) if args.reference else None
            for render in [1, 7]:
                with tempfile.TemporaryDirectory(prefix=name+'-', dir=root) as case:
                    states, count = run_native(args.binary, Path(case)/'capture', replay, render)
                if expected is not None and states != expected:
                    for step, (actual, wanted) in enumerate(zip(states, expected)):
                        if actual != wanted:
                            raise AssertionError(f'{name} first reference difference {step}: {actual} != {wanted}')
                    raise AssertionError('reference length differs')
                signature = digest(states)
                if frozen and signature != frozen['cases'][name]['snapshots_sha256']:
                    raise AssertionError('recorded reference differs: '+name)
                if name in rows and signature != rows[name]['snapshots_sha256']:
                    raise AssertionError('render-dependent Builder state')
                if not any(a['action'] == 'Building' for s in states for a in s['actors']):
                    raise AssertionError('case never exercised Building')
                if name in ('right_twelve_bricks', 'left_twelve_bricks'):
                    if not any(a['action'] == 'Shrugging' for s in states for a in s['actors']):
                        raise AssertionError('case never exhausted all twelve bricks')
                rows[name] = dict(snapshots_sha256=signature, steps=len(states), inputs=inputs)
                captures += count
                snapshots += len(states)
    if args.record:
        provenance = json.loads(args.reference.with_name('reference.provenance.json').read_text())
        GOLDEN.write_text(json.dumps(dict(format='g07-builder-v1', reference=provenance,
            adapter_sha256=hashes, synthetic_terrain_sha256=mask_hash, cases=rows), indent=2)+'\n')
    print(json.dumps(dict(status='PASS', cases=len(rows), replays=len(rows)*2, snapshots=snapshots,
                         pixel_exact_captures=captures, reference_executed=args.reference is not None,
                         scope='Builder/four jobs/G05 fields and every boundary terrain addition/deletion; not whole DOS')))


if __name__ == '__main__':
    main()
