#!/usr/bin/env python3
"""Compare original Fun07 Builder replays to executed functions and actual UI."""
import argparse
import json
from pathlib import Path
import subprocess
import time

from check_builder import collect, reference_run, replay_text, run_native
from check_desktop import verify_pixels
from state_replay import ROOT, digest, verified_assets
from verify_native_view import png_from_ppm


def latest_complete_state(path):
    # A writer can be between write/flush operations when the observer reads.
    # Only newline-terminated records represent a complete control boundary.
    text=path.read_text()
    complete=text[:text.rfind('\n')+1].splitlines()
    return json.loads(complete[-1]) if complete else None


def compare(directory, executable, inputs, steps):
    config = json.loads((directory/'config.json').read_text()) | dict(steps=steps)
    states = collect(directory)
    expected = reference_run(executable, config, directory/'terrain-initial.bin', directory/'objects.bin',
                             directory/'bash-masks.bin', directory/'mine-masks.bin', inputs, (config['diggers'], config['bashers'], config['miners'], config['builders']))
    if states != expected:
        (directory/'reference.json').write_text(json.dumps(expected))
        for step, (a, b) in enumerate(zip(states, expected)):
            if a != b:
                raise AssertionError(f'Fun07 first reference difference {step}: {a} != {b}')
        raise AssertionError('missing boundaries')
    if not any(s['builder']['remaining'] == config['builders']-1 and a['action'] == 'Building' for s in states for a in s['actors']):
        raise AssertionError('no original Builder assignment')
    if states[-1]['digger']['terrain_fnv1a64'] == states[0]['digger']['terrain_fnv1a64']:
        raise AssertionError('no terrain addition')
    for ppm in directory.glob('*.ppm'):
        png_from_ppm(ppm)
    return dict(snapshots=len(states), sha256=digest(states), pixel_exact_captures=verify_pixels(directory), inputs=inputs)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--binary', type=Path, default=ROOT/'target/release/lemmings-desktop')
    parser.add_argument('--reference', type=Path, required=True)
    parser.add_argument('--assets', type=Path, default=ROOT/'assets/local/dos-original')
    parser.add_argument('--xdotool', type=Path)
    args = parser.parse_args()
    args.output = args.output.resolve()
    if not args.output.is_relative_to(ROOT/'assets/local'):
        raise ValueError('original evidence must stay in assets/local')
    verified_assets(args.assets)
    args.output.mkdir()
    inputs = [(80, 0, 'AssignBuilder', 0)]
    replay = args.output/'F07.inputs.txt'
    replay.write_text(replay_text(inputs))
    rows = []
    for name, render, fast in [('F07-single', 1, True), ('F07-sparse', 7, True), ('F07-realtime', 1, False)]:
        directory = args.output/name
        run_native(args.binary, directory, replay, render, steps=340, assets=args.assets, level=7, fast=fast, pacing='dos')
        row = dict(name=name, **compare(directory, args.reference, inputs, 340))
        rows.append(row)
        print(json.dumps(row), flush=True)
    if args.xdotool:
        directory = args.output/'F07-mouse'
        process = subprocess.Popen([str(args.binary.resolve()), '--assets', str(args.assets), '--level', '7',
                                    '--steps', '220', '--capture', str(directory)],
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        def xdo(*parts):
            return subprocess.check_output([str(args.xdotool), *parts], text=True).strip()
        try:
            end = time.monotonic()+25
            clicked = False
            while time.monotonic() < end and process.poll() is None:
                path = directory/'trace.jsonl'
                if path.exists():
                    s = latest_complete_state(path)
                    if s and s['control_step'] >= 80 and s['actors'][0]['action'] == 'Walking':
                        windows = xdo('search', '--onlyvisible', '--name', 'Oh My Lemmings').splitlines()
                        if len(windows) != 1:
                            raise ValueError('one dedicated SDL window required')
                        xdo('windowfocus', '--sync', windows[0])
                        xdo('key', 'u')
                        xdo('mousemove', '--window', windows[0], str((s['actors'][0]['x']-448)*3), str((s['actors'][0]['y']-5)*3))
                        xdo('click', '1')
                        clicked = True
                        break
                time.sleep(0.002)
            stdout, stderr = process.communicate(timeout=25)
            if process.returncode or not clicked:
                raise ValueError(f'actual mouse input failed: {stdout} {stderr}')
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()
        actual = []
        for line in (directory/'inputs.txt').read_text().splitlines():
            step, sequence, command, value = line.split()
            actual.append((int(step), int(sequence), command, int(value)))
        if len(actual) != 1 or actual[0][2:] != ('AssignBuilder', 0):
            raise AssertionError('actual UI did not record stable actor ID')
        rows.append(dict(name='F07-mouse', **compare(directory, args.reference, actual, 220)))
        print(json.dumps(rows[-1]), flush=True)
    verified_assets(args.assets)
    (args.output/'summary.json').write_text(json.dumps(dict(status='PASS', runs=rows), indent=2)+'\n')


if __name__ == '__main__':
    main()
