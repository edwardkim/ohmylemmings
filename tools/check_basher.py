#!/usr/bin/env python3
"""Compare Basher SDL replays and every terrain deletion to executed reference."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

from check_digger import collect as collect_digger
from check_desktop import verify_pixels
from state_replay import ROOT, digest

GOLDEN = ROOT/'tools/fixtures/basher_golden.json'
ADAPTERS = ['build_basher_reference.py', 'build_digger_reference.py', 'build_reference.py', 'harness.pas']
CASES = {
    'right': [(80, 0, 'AssignBasher', 0)],
    'left': [(110, 0, 'AssignBasher', 0)],
    'duplicate_and_absent': [(54, 0, 'AssignBasher', 0), (80, 0, 'AssignBasher', 0),
                             (80, 1, 'AssignBasher', 0), (80, 2, 'AssignBasher', 999)],
    'pause_assign_resume': [(80, 0, 'Pause', 0), (80, 1, 'AssignBasher', 0), (90, 0, 'Resume', 0)],
    'replace_jobs': [(80, 0, 'AssignBasher', 0), (100, 0, 'AssignDigger', 0), (101, 0, 'AssignBasher', 0)],
    'two_bashers': [(80, 0, 'AssignBasher', 0), (110, 0, 'AssignBasher', 1)],
}


def synthetic_masks():
    masks = bytearray(160)
    for direction in range(2):
        for frame in range(4):
            xs = [8+frame*2, 9+frame*2] if direction == 0 else [6-frame*2, 7-frame*2]
            for y in range(10):
                for x in xs:
                    masks[(direction*4+frame)*20+y*2+x//8] |= 128 >> (x % 8)
    return bytes(masks)


def collect(directory):
    states = collect_digger(directory)
    details = [json.loads(s) for s in (directory/'basher.jsonl').read_text().splitlines()]
    if len(states) != len(details):
        raise ValueError('missing Basher inventory boundary')
    for state, detail in zip(states, details):
        state['basher'] = detail
    return states


def replay_text(inputs):
    return ''.join(f'{s} {q} {c}'+(f' {v}' if c not in ('Pause', 'Resume') else '')+'\n'
                   for s, q, c, v in inputs)


def reference_run(executable, config, mask, objects, masks, inputs, inventory=(10, 10)):
    header = [config[k] for k in ['width', 'height', 'minutes', 'total', 'rate']]
    header += [len(config['entrances']), int(config['pause_glitch']), int(config['faller_three']), config['steps']]
    script = ' '.join(map(str, header))+'\n'+' '.join(map(str, inventory))+'\n'
    script += ''.join(f'{x} {y}\n' for x, y in config['entrances'])+str(len(inputs))+'\n'
    codes = dict(Pause=0, Resume=1, SetReleaseRate=2, AssignDigger=3, AssignBasher=4)
    script += ''.join(f'{s} {q} {codes[c]} {v}\n' for s, q, c, v in inputs)
    result = subprocess.run([str(executable.resolve()), str(mask), str(objects), str(masks)],
                            input=script, capture_output=True, text=True, check=True)
    return [json.loads(line) for line in result.stdout.splitlines()]


def run_native(binary, directory, replay, render, *, steps=220, assets=None, level=1, fast=True, pacing=None):
    command = [str(binary.resolve()), '--steps', str(steps), '--render-every', str(render),
               '--capture', str(directory), '--replay', str(replay)]
    command += ['--assets', str(assets), '--level', str(level)] if assets else ['--synthetic-basher']
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
    if frozen and hashlib.sha256(synthetic_masks()).hexdigest() != frozen['synthetic_masks_sha256']:
        raise ValueError('synthetic masks changed; execute reference again')
    config = dict(width=640, height=160, minutes=5, total=10, rate=50, entrances=[[40, 40]],
                  pause_glitch=True, faller_three=True, steps=220)
    mask = bytes(int((y >= 100 and x < 260) or (70 <= y < 100 and (5 <= x < 50 or 80 <= x < 150)))
                 for y in range(160) for x in range(640))
    rows = {}
    captures = snapshots = 0
    with tempfile.TemporaryDirectory(prefix='lemmings-basher-') as temporary:
        root = Path(temporary)
        (root/'mask').write_bytes(mask)
        (root/'objects').write_bytes(bytes(len(mask)))
        (root/'masks').write_bytes(synthetic_masks())
        for name, inputs in CASES.items():
            replay = root/(name+'.txt')
            replay.write_text(replay_text(inputs))
            expected = reference_run(args.reference, config, root/'mask', root/'objects', root/'masks', inputs) if args.reference else None
            for render in [1, 7]:
                states, count = run_native(args.binary, root/f'{name}-{render}', replay, render)
                if expected is not None and states != expected:
                    for step, (a, b) in enumerate(zip(states, expected)):
                        if a != b:
                            raise AssertionError(f'{name} first reference difference {step}: {a} != {b}')
                    raise AssertionError('reference length differs')
                signature = digest(states)
                if frozen and signature != frozen['cases'][name]['snapshots_sha256']:
                    raise AssertionError('recorded reference differs: '+name)
                if name in rows and signature != rows[name]['snapshots_sha256']:
                    raise AssertionError('render-dependent Basher state')
                if not any(a['action'] == 'Bashing' for s in states for a in s['actors']):
                    raise AssertionError('case never exercised Bashing')
                rows[name] = dict(snapshots_sha256=signature, steps=len(states), inputs=inputs)
                captures += count
                snapshots += len(states)
    if args.record:
        provenance = json.loads(args.reference.with_name('reference.provenance.json').read_text())
        GOLDEN.write_text(json.dumps(dict(format='g07-basher-v1', reference=provenance,
            adapter_sha256=hashes, synthetic_masks_sha256=hashlib.sha256(synthetic_masks()).hexdigest(), cases=rows), indent=2)+'\n')
    print(json.dumps(dict(status='PASS', cases=len(rows), replays=len(rows)*2, snapshots=snapshots,
                         pixel_exact_captures=captures, reference_executed=args.reference is not None,
                         scope='Basher/Digger/G05 fields and every terrain deletion; not whole DOS')))


if __name__ == '__main__':
    main()
