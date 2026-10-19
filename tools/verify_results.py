#!/usr/bin/env python3
"""Verify live result R/left/right mouse on an isolated X11 display (synthetic assets)."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time

from check_completion import collect
from check_desktop import verify_pixels
from state_replay import ROOT
from verify_native_view import png_from_ppm


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT/'target/release/lemmings-desktop')
    parser.add_argument('--xdotool', type=Path, required=True)
    parser.add_argument('--evidence', type=Path, required=True)
    args = parser.parse_args()
    root = args.evidence.resolve()
    if not root.is_relative_to(ROOT/'assets/local'):
        raise ValueError('local evidence required')
    root.mkdir()
    output = root/'capture'
    process = subprocess.Popen([str(args.binary.resolve()), '--synthetic-outcome', 'win', '--capture', str(output)],
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    def command(*words):
        return subprocess.check_output([str(args.xdotool.resolve()), *words], text=True, timeout=10).strip()
    def wait_result(folder):
        # A result capture has a distinct complete-scene background, regardless of the control step.
        deadline = time.monotonic()+35
        while time.monotonic() < deadline and process.poll() is None:
            for frame in sorted(folder.glob('frame-*.rgb'), reverse=True):
                data = frame.read_bytes()
                if len(data) == 320*200*3 and data[:3] == bytes([12, 18, 32]):
                    ppm = frame.with_suffix('.ppm')
                    if ppm.exists() and len(ppm.read_bytes()) > 960*600*3:
                        return ppm
            time.sleep(.05)
        raise AssertionError('live result not observed')
    try:
        first = wait_result(output)
        windows = command('search', '--onlyvisible', '--name', 'Oh My Lemmings').splitlines()
        if len(windows) != 1:
            raise AssertionError('use an isolated X11 display with one application window')
        command('windowfocus', '--sync', windows[0])
        command('key', 'r')
        second = wait_result(output/'attempt-0001')
        command('mousemove', '--window', windows[0], '450', '400', 'click', '1')
        third = wait_result(output/'attempt-0002')
        command('click', '3')
        stdout, stderr = process.communicate(timeout=10)
        if process.returncode:
            raise AssertionError(stderr)
    finally:
        if process.poll() is None:
            process.terminate(); process.communicate(timeout=10)
    folders = [output, output/'attempt-0001', output/'attempt-0002']
    states = [collect(folder) for folder in folders]
    if states[0] != states[1]:
        raise AssertionError('live R did not restore initial state')
    if [rows[-1]['completion']['phase'] for rows in states] != ['Won', 'Won', 'Lost']:
        raise AssertionError('live left click did not load next level')
    captures = sum(verify_pixels(folder) for folder in folders)
    for frame in [first, second, third]:
        png_from_ppm(frame)
    summary = json.loads(stdout)
    if summary['video_driver'] != 'x11' or summary['input_count'] != 0:
        raise AssertionError('scene commands entered the gameplay replay')
    report = dict(status='PASS', driver='SDL2 x11/software', actions=['R', 'left click', 'right click'],
                  attempts=3, snapshots=sum(map(len, states)), pixel_exact_captures=captures, final=summary)
    (root/'summary.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
