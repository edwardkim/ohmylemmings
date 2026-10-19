#!/usr/bin/env python3
"""Replay Digger through SDL; compare actual Pascal functions or recorded results."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

from check_desktop import verify_pixels
from state_replay import ROOT, digest

GOLDEN = ROOT/'tools/fixtures/digger_golden.json'
CASES = {
    'first': [(80, 0, 'AssignDigger', 0)],
    'duplicate': [(80, 0, 'AssignDigger', 0), (80, 1, 'AssignDigger', 0)],
    'falling_and_absent': [(54, 0, 'AssignDigger', 0), (60, 0, 'AssignDigger', 999)],
    'pause_assign_resume': [(80, 0, 'Pause', 0), (80, 1, 'AssignDigger', 0), (90, 0, 'Resume', 0)],
    'two_diggers': [(80, 0, 'AssignDigger', 0), (110, 0, 'AssignDigger', 1)],
    'rate_and_dig': [(54, 0, 'SetReleaseRate', 99), (80, 0, 'AssignDigger', 0), (90, 0, 'AssignDigger', 1)],
}


def collect(directory):
    states = [json.loads(s) for s in (directory/'trace.jsonl').read_text().splitlines()]
    details = [json.loads(s) for s in (directory/'digger.jsonl').read_text().splitlines()]
    if len(states) != len(details):
        raise ValueError('missing Digger state')
    mask = bytearray((directory/'terrain-initial.bin').read_bytes())
    builder_path = directory/'builder.jsonl'
    builders = [json.loads(s) for s in builder_path.read_text().splitlines()] if builder_path.exists() else None
    if builders is not None and len(builders) != len(states):
        raise ValueError('missing Builder state boundary')
    for step, (state, detail) in enumerate(zip(states, details)):
        if state['control_step'] != step:
            raise ValueError('missing control boundary')
        previous = -1
        for index in detail['cleared']:
            if not isinstance(index, int) or not previous < index < len(mask) or mask[index] != 1:
                raise ValueError('invalid or repeated terrain deletion')
            mask[index] = 0
            previous = index
        previous = -1
        for index in builders[step]['filled'] if builders is not None else []:
            if (type(index) is not int or not previous < index < len(mask)
                    or mask[index] != 0 or index in detail['cleared'] or step == 0):
                raise ValueError('invalid or repeated terrain addition')
            mask[index] = 1
            previous = index
        state['digger'] = detail
    if bytes(mask) != (directory/'terrain-final.bin').read_bytes():
        raise ValueError('terrain history differs from final full mask')
    return states


def reference_run(executable, config, mask, steel, inputs, inventory=10):
    header = [config[k] for k in ['width', 'height', 'minutes', 'total', 'rate']]
    header += [len(config['entrances']), int(config['pause_glitch']), int(config['faller_three']), config['steps']]
    script = ' '.join(map(str, header))+'\n'+str(inventory)+'\n'
    script += ''.join(f'{x} {y}\n' for x, y in config['entrances'])+str(len(inputs))+'\n'
    codes = dict(Pause=0, Resume=1, SetReleaseRate=2, AssignDigger=3)
    script += ''.join(f'{step} {sequence} {codes[command]} {value}\n' for step, sequence, command, value in inputs)
    result = subprocess.run([str(executable.resolve()), str(mask), str(steel)], input=script,
                            capture_output=True, text=True, check=True)
    return [json.loads(line) for line in result.stdout.splitlines()]


def replay_text(inputs):
    return ''.join(f'{s} {q} {c}'+(f' {v}' if c in ('SetReleaseRate', 'AssignDigger') else '')+'\n'
                   for s, q, c, v in inputs)


def run_native(binary, directory, replay, render_every, *, assets=None, steps=340, fast=True, pacing=None):
    command = [str(binary.resolve()), '--steps', str(steps), '--render-every', str(render_every),
               '--capture', str(directory), '--replay', str(replay)]
    command += ['--assets', str(assets)] if assets else ['--synthetic']
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
    config = dict(width=640, height=160, minutes=5, total=10, rate=50,
                  entrances=[[40, 40]], pause_glitch=True, faller_three=True, steps=340)
    mask = bytes(int(y >= 100 and x < 260) for y in range(160) for x in range(640))
    rows = {}
    captures = snapshots = 0
    frozen = json.loads(GOLDEN.read_text()) if not args.record else None
    for name in ['build_digger_reference.py', 'harness.pas']:
        path = ROOT/'tools/lemmix'/name
        if frozen and frozen['adapter_sha256'][name] != hashlib.sha256(path.read_bytes()).hexdigest():
            raise ValueError('reference adapter changed; execute reference again')
    with tempfile.TemporaryDirectory(prefix='lemmings-digger-') as temporary:
        root = Path(temporary)
        (root/'mask').write_bytes(mask)
        (root/'steel').write_bytes(bytes(len(mask)))
        for name, inputs in CASES.items():
            replay = root/(name+'.txt')
            replay.write_text(replay_text(inputs))
            expected = reference_run(args.reference, config, root/'mask', root/'steel', inputs) if args.reference else None
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
                    raise AssertionError('render-dependent Digger state')
                rows[name] = dict(snapshots_sha256=signature, steps=len(states), inputs=inputs)
                captures += count
                snapshots += len(states)
    if args.record:
        provenance = json.loads(args.reference.with_name('reference.provenance.json').read_text())
        GOLDEN.write_text(json.dumps(dict(format='g07-digger-v1', reference=provenance,
            adapter_sha256={n: hashlib.sha256((ROOT/'tools/lemmix'/n).read_bytes()).hexdigest()
                           for n in ['build_digger_reference.py', 'harness.pas']}, cases=rows), indent=2)+'\n')
    print(json.dumps(dict(status='PASS', cases=len(rows), replays=12, snapshots=snapshots,
                         pixel_exact_captures=captures, reference_executed=args.reference is not None,
                         scope='Digger and G05 fields, complete ordered terrain deletions; not whole DOS')))


if __name__ == '__main__':
    main()
