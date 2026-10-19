#!/usr/bin/env python3
"""Compare original Fun06 Bomber replays to executed functions and actual UI."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import time

from check_bomber import collect, reference_run, replay_text, run_native
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
                             directory/'bash-masks.bin', directory/'mine-masks.bin', directory/'object-map-initial.bin', directory/'explosion-mask.bin', inputs, (config['diggers'], config['bashers'], config['miners'], config['builders'], config['blockers'], config['bombers']))
    if states != expected:
        (directory/'reference.json').write_text(json.dumps(expected))
        for step, (a, b) in enumerate(zip(states, expected)):
            if a != b:
                raise AssertionError(f'Fun06 first reference difference {step}: {a} != {b}')
        raise AssertionError('missing boundaries')
    if not any(s['bomber']['remaining']==config['bombers']-1 and a['removed']=='Exploded' for s in states for a in s['actors']):
        raise AssertionError('no original Bomber assignment/explosion')
    if states[-1]['digger']['terrain_fnv1a64']==states[0]['digger']['terrain_fnv1a64']:
        raise AssertionError('no explosion terrain change')
    if states[-1]['blocker']['fields']:raise AssertionError('Blocker field not released')
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
    parser.add_argument('--ui-only',action='store_true',help='only the real key/click replay')
    args = parser.parse_args()
    if args.ui_only and not args.xdotool:raise ValueError('ui-only requires xdotool')
    args.output = args.output.resolve()
    if not args.output.is_relative_to(ROOT/'assets/local'):
        raise ValueError('original evidence must stay in assets/local')
    verified_assets(args.assets)
    binary_sha=hashlib.sha256(args.binary.read_bytes()).hexdigest()
    reference_sha=hashlib.sha256(args.reference.read_bytes()).hexdigest()
    args.output.mkdir()
    inputs = [(80, 0, 'AssignBlocker', 0), (100, 0, 'AssignBomber', 0)]
    replay = args.output/'F06.inputs.txt'
    replay.write_text(replay_text(inputs))
    rows = []
    for name, render, fast in ([] if args.ui_only else [('F06-single', 1, True), ('F06-sparse', 7, True), ('F06-realtime', 1, False)]):
        directory = args.output/name
        run_native(args.binary, directory, replay, render, steps=340, assets=args.assets, level=6, fast=fast, pacing='dos')
        row = dict(name=name, **compare(directory, args.reference, inputs, 340))
        rows.append(row)
        print(json.dumps(row), flush=True)
    if args.xdotool:
        directory = args.output/'F06-mouse'
        process = subprocess.Popen([str(args.binary.resolve()), '--assets', str(args.assets), '--level', '6',
                                    '--steps', '340', '--capture', str(directory)],
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        def xdo(*parts):
            result=subprocess.run([str(args.xdotool), *parts],text=True,capture_output=True)
            if result.returncode==1 and parts[0]=='search':return ''
            if result.returncode:raise ValueError('xdotool: '+result.stderr.strip())
            return result.stdout.strip()
        try:
            time.sleep(.5) # allow compositor placement/resize to settle
            end = time.monotonic()+25
            clicked = False
            prepared = False
            while time.monotonic() < end and process.poll() is None:
                if not prepared:
                    windows = xdo('search', '--onlyvisible', '--pid', str(process.pid), '--name', 'Oh My Lemmings').splitlines()
                    if windows:
                        if len(windows)!=1:raise ValueError('one dedicated SDL window required')
                        geometry=dict(line.split('=',1) for line in xdo('getwindowgeometry','--shell',windows[0]).splitlines())
                        width,height=int(geometry['WIDTH']),int(geometry['HEIGHT'])
                        scale=min(width//320,height//200)
                        if scale<1:raise ValueError('window smaller than logical frame')
                        # A HUD click grants compositor keyboard focus without
                        # assigning any skill; X11 focus alone is insufficient
                        # when the visible window runs through Xwayland.
                        xdo('mousemove','--window',windows[0],str(width//2),str((height-200*scale)//2+188*scale))
                        xdo('click','1')
                        xdo('key','--clearmodifiers','e')
                        time.sleep(.1)
                        prepared=True
                path = directory/'trace.jsonl'
                if path.exists() and prepared:
                    s = latest_complete_state(path)
                    if s and s['control_step'] >= 80 and s['actors'][0]['action'] == 'Walking':
                        windows = xdo('search', '--onlyvisible', '--pid', str(process.pid), '--name', 'Oh My Lemmings').splitlines()
                        if len(windows) != 1:
                            raise ValueError('one dedicated SDL window required')
                        xdo('windowfocus', '--sync', windows[0])
                        geometry=dict(line.split('=',1) for line in xdo('getwindowgeometry','--shell',windows[0]).splitlines())
                        width,height=int(geometry['WIDTH']),int(geometry['HEIGHT'])
                        scale=min(width//320,height//200)
                        if scale<1:raise ValueError('window smaller than logical frame')
                        x=(s['actors'][0]['x']-576)*scale+(width-320*scale)//2
                        y=(s['actors'][0]['y']-5)*scale+(height-200*scale)//2
                        xdo('mousemove', '--window', windows[0], str(x), str(y))
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
        if len(actual) != 1 or actual[0][2:] != ('AssignBomber', 0):
            raise AssertionError('actual UI did not record stable actor ID')
        rows.append(dict(name='F06-mouse', **compare(directory, args.reference, actual, 340)))
        print(json.dumps(rows[-1]), flush=True)
    verified_assets(args.assets)
    if hashlib.sha256(args.binary.read_bytes()).hexdigest()!=binary_sha:raise ValueError('binary changed during verification')
    (args.output/'summary.json').write_text(json.dumps(dict(status='PASS', binary_sha256=binary_sha, reference_sha256=reference_sha, runs=rows), indent=2)+'\n')


if __name__ == '__main__':
    main()
