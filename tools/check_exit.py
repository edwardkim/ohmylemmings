#!/usr/bin/env python3
"""Check exit/action priority, eight-frame rescues and populations against Pascal."""
import argparse
from contextlib import nullcontext
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

from check_nuke import ADAPTERS as NUKE_ADAPTERS, collect as collect_jobs, reference_run, replay_text
from check_basher import synthetic_masks as bashing_masks
from check_miner import synthetic_masks as mining_masks
from check_desktop import verify_pixels
from check_skill_interactions import compare_states
from state_replay import ROOT, digest

GOLDEN = ROOT / 'tools/fixtures/exit_golden.json'
ADAPTERS = ['build_exit_reference.py'] + NUKE_ADAPTERS
SKILLS = ['Climber', 'Floater', 'Bomber', 'Blocker', 'Builder', 'Basher', 'Miner', 'Digger']
CASES = {
    'walking': (False, []),
    'paused_exit': (False, [(103, 0, 'Pause', 0), (125, 0, 'Resume', 0)]),
    'armed_athlete': (False, [(80, 0, 'AssignClimber', 0), (80, 1, 'AssignFloater', 0), (80, 2, 'AssignBomber', 0)]),
    'nuke_during_exit': (False, [(103, 0, 'Nuke', 0)]),
    'exiting_assignments': (False, [(103, i, 'Assign' + skill, 0) for i, skill in enumerate(SKILLS)]),
    'removed_assignments': (False, [(120, i, 'Assign' + skill, 0) for i, skill in enumerate(SKILLS)]),
    'splat_exit': (True, []),
    'floating_exit': (True, [(54, 0, 'AssignFloater', 0)]),
    'splat_cancels_bomber': (True, [(54, 0, 'AssignBomber', 0)]),
}


def validate_exit_history(states, rows):
    if len(states) != len(rows):
        raise ValueError('missing exit boundary')
    previous = 0
    for state, row in zip(states, rows):
        if set(row) != {'saved', 'exiting'} or type(row['saved']) is not int or row['saved'] < previous:
            raise ValueError('invalid rescue count/schema')
        if type(row['exiting']) is not list or any(type(v) is not int for v in row['exiting']):
            raise ValueError('invalid exiting IDs')
        rescued = [a for a in state['actors'] if a['removed'] == 'Rescued']
        exiting = [a['id'] for a in state['actors'] if a['action'] == 'Exiting' and a['removed'] is None]
        if any(a['action'] != 'Exiting' or a['frame'] != 7 or a['end_of_animation'] is not True for a in rescued):
            raise ValueError('rescue before exit completion')
        removed = sum(a['removed'] is not None for a in state['actors'])
        if (row['saved'] != len(rescued) or row['exiting'] != exiting
                or state['removed'] != removed or state['released'] != len(state['actors'])
                or state['alive'] + removed != state['released']):
            raise ValueError('inconsistent exit population/IDs')
        previous = row['saved']
        state['exit'] = row


def collect(directory):
    states = collect_jobs(directory)
    rows = [json.loads(line) for line in (directory / 'exit.jsonl').read_text().splitlines()]
    validate_exit_history(states, rows)
    return states


def run_native(binary, directory, replay, render, *, drop=False, steps=420, assets=None, level=1, fast=True):
    command = [str(binary.resolve()), '--steps', str(steps), '--render-every', str(render),
               '--capture', str(directory), '--replay', str(replay)]
    command += ['--assets', str(assets), '--level', str(level)] if assets else ['--synthetic-exit-drop' if drop else '--synthetic-exit']
    if fast:
        command += ['--fast']
    subprocess.run(command, env=dict(os.environ, SDL_VIDEODRIVER=os.environ.get('SDL_VIDEODRIVER', 'dummy')),
                   check=True, capture_output=True, text=True)
    return collect(directory), verify_pixels(directory)


def assert_exercised(name, states):
    if not states[-1]['exit']['saved']:
        raise AssertionError('no rescue exercised: ' + name)
    if name != 'nuke_during_exit' and states[-1]['exit']['saved'] != 10:
        raise AssertionError('incomplete synthetic rescue cohort: ' + name)
    if name == 'exiting_assignments':
        events = {e for s in states for e in s['events']}
        for i in range(8):
            expected = f'Input:103:{i}:' + ('true' if i < 3 else 'false')
            if expected not in events:
                raise AssertionError('exit assignment rule not exercised: ' + expected)
    if name == 'removed_assignments':
        events = {e for s in states for e in s['events']}
        if not all(f'Input:120:{i}:false' in events for i in range(8)):
            raise AssertionError('removed actor accepted a skill')
    if name.startswith('splat_'):
        if not any(a['action'] == 'Exiting' and a['direction'] == 0 for s in states for a in s['actors']):
            raise AssertionError('splat/exit priority not exercised')
    if name == 'splat_cancels_bomber':
        if not any(s['bomber']['actors'] and s['bomber']['actors'][0][1] for s in states):
            raise AssertionError('Bomber not armed')
        if any(s['bomber']['actors'][0][1] for s in states if s['actors'] and s['actors'][0]['action'] == 'Exiting'):
            raise AssertionError('splat restored cancelled timer')
    if name == 'floating_exit':
        entries = [s for s in states if s['actors'] and s['actors'][0]['action'] == 'Exiting']
        if not entries or entries[0]['actors'][0]['y'] >= 140:
            raise AssertionError('Floating did not enter an airborne exit')
    if name == 'paused_exit':
        if states[104]['actors'] != states[125]['actors'] or states[104]['exit'] != states[125]['exit']:
            raise AssertionError('paused exit progressed')


def check(args, root):
    frozen = None if args.record else json.loads(GOLDEN.read_text())
    hashes = {name: hashlib.sha256((ROOT / 'tools/lemmix' / name).read_bytes()).hexdigest() for name in ADAPTERS}
    if frozen and (hashes != frozen['adapter_sha256'] or set(CASES) != set(frozen['cases'])):
        raise ValueError('reference adapter/cases changed; execute reference again')
    rows = {}
    snapshots = captures = 0
    for name, (drop, inputs) in CASES.items():
        if frozen and (frozen['cases'][name]['drop'] != drop or frozen['cases'][name]['inputs'] != [list(i) for i in inputs]):
            raise ValueError('scenario changed: ' + name)
        config = dict(width=640, height=160, minutes=5, total=10, rate=50,
                      entrances=[[40, 0 if drop else 40]], pause_glitch=True, faller_three=True, steps=420)
        terrain = bytes(int(y >= (140 if drop else 100) and x < 500) for y in range(160) for x in range(640))
        raw = bytearray([128]) * (415 * 47)
        for y in range(60 if drop else 96, 144 if drop else 104):
            for x in range(64 if drop else 96, 72 if drop else 104):
                raw[(y // 4 + 4) * 415 + x // 4 + 4] = 129
        paths = []
        for key, data in [('mask', terrain), ('objects', bytes(len(terrain))), ('bash-masks', bashing_masks()),
                          ('mine-masks', mining_masks()), ('raw-map', raw), ('explosion-mask', bytes([255]) * 44)]:
            path = root / (name + '-' + key)
            path.write_bytes(data)
            paths.append(path)
        replay = root / (name + '.txt')
        replay.write_text(replay_text(inputs))
        expected = reference_run(args.reference, config, *paths, inputs) if args.reference else None
        if expected is not None:
            (root / (name + '-reference.json')).write_text(json.dumps(expected))
        for render in [1, 7]:
            context = nullcontext(root) if args.evidence else tempfile.TemporaryDirectory(dir=root)
            with context as directory:
                states, count = run_native(args.binary, Path(directory) / (name + '-' + str(render)), replay, render, drop=drop)
            if expected is not None:
                compare_states(states, expected, name)
            assert_exercised(name, states)
            signature = digest(states)
            if frozen and signature != frozen['cases'][name]['snapshots_sha256']:
                raise AssertionError('recorded reference differs: ' + name)
            if name in rows and rows[name]['snapshots_sha256'] != signature:
                raise AssertionError('render-dependent rescue state: ' + name)
            rows[name] = dict(drop=drop, inputs=inputs, snapshots_sha256=signature, steps=len(states), saved=states[-1]['exit']['saved'])
            snapshots += len(states)
            captures += count
    if args.record:
        provenance = json.loads(args.reference.with_name('reference.provenance.json').read_text())
        GOLDEN.write_text(json.dumps(dict(format='g08-exit-v1', reference=provenance, adapter_sha256=hashes, cases=rows), indent=2) + '\n')
    summary = dict(status='PASS', cases=len(rows), replays=len(rows) * 2, snapshots=snapshots,
                   pixel_exact_captures=captures, reference_executed=args.reference is not None,
                   scope='exit/action ordering, Splat/Floating/Exiting, saved/removal counts, jobs/timers/Pause/Nuke; not whole DOS')
    (root / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(summary))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/release/lemmings-desktop')
    parser.add_argument('--reference', type=Path)
    parser.add_argument('--record', action='store_true')
    parser.add_argument('--evidence', type=Path)
    args = parser.parse_args()
    if args.record and not args.reference:
        raise ValueError('record requires actual reference execution')
    if args.evidence:
        root = args.evidence.resolve()
        if not root.is_relative_to(ROOT / 'assets/local'):
            raise ValueError('local evidence required')
        root.mkdir()
        check(args, root)
    else:
        with tempfile.TemporaryDirectory(prefix='lemmings-exit-') as directory:
            check(args, Path(directory))


if __name__ == '__main__':
    main()
