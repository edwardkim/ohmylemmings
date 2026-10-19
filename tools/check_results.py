#!/usr/bin/env python3
"""Exercise result-scene SDL commands and independent attempts without original assets."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile

from check_completion import collect
from check_desktop import verify_pixels
from state_replay import ROOT, digest


def read_jsonl(path):
    return [json.loads(line) for line in path.read_text().splitlines()]


def execute(binary, directory, kind, actions, *, render=7, steps=1200, replay=None, assets=None, level=1, capture=True):
    command = [str(binary.resolve()), '--fast', '--steps', str(steps), '--render-every', str(render)]
    command += ['--assets', str(assets), '--level', str(level)] if assets else ['--synthetic-outcome', kind]
    if capture:
        command += ['--capture', str(directory)]
    if actions:
        command += ['--result-actions', ','.join(actions)]
    if replay:
        command += ['--replay', str(replay)]
    result = subprocess.run(command, check=True, text=True, capture_output=True,
                            env=dict(os.environ, SDL_VIDEODRIVER='dummy'), timeout=240 if assets else 20)
    return json.loads(result.stdout)


def attempts(root):
    return [root, *sorted(root.glob('attempt-*'))]


def check(binary, root, assets=None):
    results = []
    snapshots = captures = 0
    replay = root/'inputs.txt'
    # The second attempt must accept fresh live input, not replay Pause/Resume or the unused Nuke.
    replay.write_text('16 0 Pause\n34 0 Resume\n900 0 Nuke\n')
    scenarios = [('win', ['restart', 'continue'], 1200, replay),
                 ('loss', ['continue'], 1200, None),
                 ('zero', ['restart'], 1, None)]
    for kind, actions, steps, inputs in scenarios:
        prior = None
        for render in [1, 7]:
            directory = root/f'{kind}-{render}'
            summary = execute(binary, directory, kind, actions, steps=steps, render=render, replay=inputs)
            folders = attempts(directory)
            if len(folders) != len(actions)+1:
                raise AssertionError('lost or duplicated result transition')
            transitions = read_jsonl(directory/'transitions.jsonl')
            if [t['action'].lower() for t in transitions] != actions:
                raise AssertionError('wrong scene command order')
            states = [collect(folder) for folder in folders]
            if any(rows[0]['control_step'] != 0 or rows[0]['actors'] for rows in states):
                raise AssertionError('attempt did not start from boundary zero')
            if kind == 'win':
                if [rows[-1]['completion']['phase'] for rows in states] != ['Won', 'Won', 'Lost']:
                    raise AssertionError('next fixture was not loaded or outcome leaked')
                if [t['target'] for t in transitions] != [1, 2]:
                    raise AssertionError('restart/next level IDs differ')
                if (folders[0]/'inputs.txt').read_text() != '16 0 Pause\n34 0 Resume\n':
                    raise AssertionError('first replay consumed commands after result')
                if (folders[1]/'inputs.txt').read_text() or (folders[2]/'inputs.txt').read_text():
                    raise AssertionError('old replay leaked across result')
            else:
                if states[0] != states[1]:
                    raise AssertionError('retry differs from fresh initial session')
                for filename in ['terrain-initial.bin', 'object-map-initial.bin', 'objects.bin', 'traps.bin', 'config.json']:
                    if (folders[0]/filename).read_bytes() != (folders[1]/filename).read_bytes():
                        raise AssertionError('attempt asset/config changed on retry')
            if prior is not None and states != prior:
                raise AssertionError('scene transitions depend on rendering')
            prior = states
            for folder, rows in zip(folders, states):
                snapshots += len(rows)
                captures += verify_pixels(folder)
                run = json.loads((folder/'run.json').read_text())
                if run['input_count'] != len((folder/'inputs.txt').read_text().splitlines()):
                    raise AssertionError('attempt summary/input log mismatch')
            if summary['renderer'] != 'software' or summary['video_driver'] != 'dummy':
                raise AssertionError('SDL renderer not exercised')
            results.append(dict(case=kind, render=render, attempts=len(states), sha256=digest(states)))
    # Catch-up path without capture must perform the same transitions.
    summary = execute(binary, root/'unused', 'win', ['restart', 'continue'], capture=False)
    win_final = collect(root/'win-7/attempt-0002')[-1]['control_step']
    if summary['control_step'] != win_final:
        raise AssertionError('uncaptured transition boundary differs')
    # A non-completing bounded game must not silently claim its scheduled actions were applied.
    bad = subprocess.run([str(binary.resolve()), '--synthetic', '--fast', '--steps', '1',
                          '--result-actions', 'restart'], text=True, capture_output=True,
                         env=dict(os.environ, SDL_VIDEODRIVER='dummy'), timeout=20)
    if bad.returncode == 0 or 'before all result actions' not in bad.stderr:
        raise AssertionError('missing result action was silently accepted')
    original = []
    if assets:
        replay.write_text('60 0 AssignDigger 0\n900 0 Nuke\n')
        for action, target in [('restart', 1), ('continue', 2)]:
            directory = root/f'original-{action}'
            execute(binary, directory, '', [action], steps=1000, replay=replay, assets=assets)
            target_dir = directory/'attempt-0001'
            baseline = root/f'original-fresh-{target}'
            execute(binary, baseline, '', [], steps=1000, assets=assets, level=target)
            if collect(target_dir) != collect(baseline):
                raise AssertionError('original restart/next differs from independently loaded level')
            if (target_dir/'terrain-initial.bin').read_bytes() != (baseline/'terrain-initial.bin').read_bytes():
                raise AssertionError('original terrain was not restored')
            if (target_dir/'inputs.txt').read_text():
                raise AssertionError('original old replay leaked')
            for folder in [directory, target_dir, baseline]:
                snapshots += len(collect(folder)); captures += verify_pixels(folder)
            original.append(dict(action=action, target=target, status='PASS'))
    report = dict(status='PASS', driver='SDL2 dummy/software', cases=results,
                  snapshots=snapshots, pixel_exact_captures=captures, original=original,
                  scope='result scene, restart/next, fresh session and capture/replay boundaries; not DOS artwork')
    (root/'summary.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT/'target/release/lemmings-desktop')
    parser.add_argument('--evidence', type=Path)
    parser.add_argument('--assets', type=Path)
    args = parser.parse_args()
    if args.evidence:
        root = args.evidence.resolve()
        if not root.is_relative_to(ROOT/'assets/local'):
            raise ValueError('evidence must stay under assets/local')
        root.mkdir()
        check(args.binary, root, args.assets)
    else:
        if args.assets:
            raise ValueError('original run requires local --evidence')
        with tempfile.TemporaryDirectory(prefix='lemmings-results-') as temporary:
            check(args.binary, Path(temporary))


if __name__ == '__main__':
    main()
