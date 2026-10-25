#!/usr/bin/env python3
"""Check real SDL startup selection and identical simulation/pixels across backends."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile

from check_desktop import verify_pixels
from state_replay import ROOT, digest


def run(binary, directory, driver, software, accelerated=False):
    env = dict(os.environ, SDL_VIDEODRIVER=driver, SDL_AUDIODRIVER='dummy')
    env.pop('SDL_RENDER_DRIVER', None)
    if software:
        env['SDL_RENDER_DRIVER'] = 'software'
    result = subprocess.run([str(binary.resolve()), '--synthetic', '--fast', '--steps', '80',
                             '--render-every', '79', '--capture', str(directory)],
                            cwd=ROOT, env=env, check=True, capture_output=True, text=True, timeout=30)
    mode = 'accelerated' if accelerated else 'software'
    if f'LEMMINGS_RENDERER mode={mode} ' not in result.stderr:
        raise AssertionError('startup selection differs: ' + result.stderr)
    if software and 'software requested' not in result.stderr:
        raise AssertionError('explicit software choice not reported')
    summary = json.loads(result.stdout)
    if (summary['renderer'] != 'software') != accelerated:
        raise AssertionError('actual SDL renderer differs from startup selection')
    states = [json.loads(row) for row in (directory/'trace.jsonl').read_text().splitlines()]
    captures = verify_pixels(directory)
    (directory/'startup.log').write_text(result.stderr)
    return states, dict(driver=driver, renderer=summary['renderer'], mode=mode,
                        states=len(states), pixel_exact_captures=captures)


def check(binary, output, gpu_driver=None):
    output.mkdir(parents=True, exist_ok=False)
    cases = [('unavailable', 'dummy', False, False), ('requested', 'dummy', True, False)]
    if gpu_driver:
        cases += [('accelerated', gpu_driver, False, True), ('forced-software', gpu_driver, True, False)]
    baseline = None
    rows = []
    for name, driver, software, accelerated in cases:
        states, row = run(binary, output/name, driver, software, accelerated)
        if baseline is None:
            baseline = states
        elif states != baseline:
            raise AssertionError('renderer changed simulation: ' + name)
        rows.append(row)
    report = dict(status='PASS', cases=rows, state_sha256=digest(baseline),
                  scope='SDL renderer selection, synthetic pixels and simulation; no FPS qualification')
    (output/'summary.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT/'target/release/lemmings-desktop')
    parser.add_argument('--gpu-video-driver', help='Require acceleration on this available SDL video driver')
    parser.add_argument('--evidence', type=Path)
    args = parser.parse_args()
    if args.evidence:
        check(args.binary, args.evidence, args.gpu_video_driver)
    else:
        with tempfile.TemporaryDirectory(prefix='lemmings-renderer-') as temporary:
            check(args.binary, Path(temporary)/'evidence', args.gpu_video_driver)


if __name__ == '__main__':
    main()
