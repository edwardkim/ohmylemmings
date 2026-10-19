#!/usr/bin/env python3
"""Verify original Fun01 rescue replays against executed functions and SDL pixels."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import time

from check_exit import collect, reference_run, replay_text, run_native
from check_desktop import verify_pixels
from check_skill_interactions import compare_states
from state_replay import ROOT, digest, verified_assets
from verify_native_view import png_from_ppm
from verify_nuke import latest_complete_state


def compare(directory, executable, inputs, steps):
    config = json.loads((directory / 'config.json').read_text()) | dict(steps=steps)
    states = collect(directory)
    expected = reference_run(executable, config, directory / 'terrain-initial.bin', directory / 'objects.bin',
                             directory / 'bash-masks.bin', directory / 'mine-masks.bin',
                             directory / 'object-map-initial.bin', directory / 'explosion-mask.bin', inputs,
                             tuple(config[k] for k in ['diggers', 'bashers', 'miners', 'builders', 'blockers', 'bombers', 'climbers', 'floaters']))
    (directory / 'reference.json').write_text(json.dumps(expected))
    compare_states(states, expected, 'Fun01 exit')
    if states[-1]['exit']['saved'] != 10 or states[-1]['alive'] != 0 or states[-1]['removed'] != 10:
        raise AssertionError('Fun01 did not rescue all ten actors')
    if states[-1]['digger']['remaining'] != config['diggers'] - 1:
        raise AssertionError('Digger input did not execute exactly once')
    if states[-1]['digger']['terrain_fnv1a64'] == states[0]['digger']['terrain_fnv1a64']:
        raise AssertionError('no excavation exercised')
    entries, rescues = [], []
    for actor in states[-1]['actors']:
        entry = next(s['control_step'] for s in states if actor['id'] in s['exit']['exiting'])
        rescue = next(s['control_step'] for s in states if len(s['actors']) > actor['id']
                      and s['actors'][actor['id']]['removed'] == 'Rescued')
        if rescue - entry != 8:
            raise AssertionError('rescue was not eight logical ticks after entry')
        entries.append([actor['id'], entry])
        rescues.append([actor['id'], rescue])
    # Keep a small, reviewable selection; all raw SDL readbacks remain retained.
    first = min(step for _, step in entries)
    for step in [0, 61, first, first + 7, first + 8, steps]:
        ppm = directory / f'frame-{step:06}.ppm'
        if ppm.exists():
            png_from_ppm(ppm)
    return dict(snapshots=len(states), sha256=digest(states), pixel_exact_captures=verify_pixels(directory),
                inputs=inputs, entries=entries, rescues=rescues, saved=10)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('output', type=Path)
    p.add_argument('--binary', type=Path, default=ROOT / 'target/release/lemmings-desktop')
    p.add_argument('--reference', type=Path, required=True)
    p.add_argument('--assets', type=Path, default=ROOT / 'assets/local/dos-original')
    p.add_argument('--xdotool', type=Path)
    args = p.parse_args()
    args.output = args.output.resolve()
    if not args.output.is_relative_to(ROOT / 'assets/local'):
        raise ValueError('original evidence must stay in assets/local')
    verified_assets(args.assets)
    initial_binary = hashlib.sha256(args.binary.read_bytes()).hexdigest()
    args.output.mkdir()
    steps = 700
    inputs = [(60, 0, 'AssignDigger', 0)]
    replay = args.output / 'F01.inputs.txt'
    replay.write_text(replay_text(inputs))
    rows = []
    for name, render, fast in [('F01-single', 1, True), ('F01-sparse', 7, True), ('F01-realtime', 1, False)]:
        start = time.monotonic()
        directory = args.output / name
        run_native(args.binary, directory, replay, render, steps=steps, assets=args.assets, fast=fast)
        row = dict(name=name, elapsed_seconds=round(time.monotonic() - start, 3),
                   **compare(directory, args.reference, inputs, steps))
        rows.append(row)
        print(json.dumps(row), flush=True)
    if len({row['sha256'] for row in rows}) != 1:
        raise AssertionError('render/pacing changed original rescue states')
    if args.xdotool:
        directory = args.output / 'F01-mouse'
        process = subprocess.Popen([str(args.binary.resolve()), '--assets', str(args.assets), '--steps', str(steps),
                                    '--capture', str(directory)], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        def xdo(*parts):
            return subprocess.check_output([str(args.xdotool), *parts], text=True).strip()
        try:
            end = time.monotonic() + 15
            clicked = False
            while time.monotonic() < end and process.poll() is None:
                path = directory / 'trace.jsonl'
                state = latest_complete_state(path) if path.exists() else None
                if state and state['control_step'] >= 60 and state['actors'][0]['action'] == 'Walking':
                    windows = xdo('search', '--onlyvisible', '--pid', str(process.pid), '--name', 'Oh My Lemmings').splitlines()
                    if len(windows) != 1:
                        raise ValueError('one own SDL window required')
                    xdo('windowfocus', '--sync', windows[0])
                    xdo('key', 'd')
                    actor = state['actors'][0]
                    xdo('mousemove', '--window', windows[0], str((actor['x'] - 624) * 3), str((actor['y'] - 5) * 3), 'click', '1')
                    clicked = True
                    break
                time.sleep(0.002)
            stdout, stderr = process.communicate(timeout=50)
            if process.returncode or not clicked:
                raise ValueError(f'actual Digger click failed: {stdout} {stderr}')
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()
        actual = []
        for line in (directory / 'inputs.txt').read_text().splitlines():
            step, sequence, command, value = line.split()
            actual.append((int(step), int(sequence), command, int(value)))
        if len(actual) != 1 or actual[0][2:] != ('AssignDigger', 0):
            raise AssertionError('one actual Digger assignment required')
        rows.append(dict(name='F01-mouse', **compare(directory, args.reference, actual, steps)))
        print(json.dumps(rows[-1]), flush=True)
    verified_assets(args.assets)
    if hashlib.sha256(args.binary.read_bytes()).hexdigest() != initial_binary:
        raise ValueError('binary changed during validation')
    (args.output / 'summary.json').write_text(json.dumps(dict(status='PASS', runs=rows, binary_sha256=initial_binary,
        reference_sha256=hashlib.sha256(args.reference.read_bytes()).hexdigest(),
        scope='Fun01 excavation/exit/all ten rescues; no result screen or whole DOS state'), indent=2) + '\n')


if __name__ == '__main__':
    main()
