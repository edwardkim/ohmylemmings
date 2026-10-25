#!/usr/bin/env python3
"""Exercise a real SDL2 event loop/renderer with synthetic assets, offline."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile

from state_replay import ROOT, digest, run_engine


def verify_scaled_frame(width, height, pixels, logical):
    if len(logical)!=320*200*3 or len(pixels)!=width*height*3:
        raise AssertionError('truncated framebuffer')
    scale=min(width//320,height//200)
    if scale<1:raise AssertionError('capture is smaller than the logical frame')
    left=(width-320*scale)//2;top=(height-200*scale)//2
    for y in range(height):
        if top<=y<top+200*scale:
            row=logical[((y-top)//scale)*960:((y-top)//scale+1)*960]
            expected=bytes(left*3)+b''.join(row[x:x+3]*scale for x in range(0,960,3))+bytes((width-left-320*scale)*3)
        else:expected=bytes(width*3)
        if pixels[y*width*3:(y+1)*width*3]!=expected:
            raise AssertionError(f'SDL integer scaling/readback differs at output row {y}')


def verify_pixels(directory):
    count=0
    for ppm in sorted(directory.glob('frame-*.ppm')):
        magic,dimensions,maximum,pixels=ppm.read_bytes().split(b'\n',3)
        width,height=map(int,dimensions.split())
        if magic!=b'P6' or maximum!=b'255':raise AssertionError('unexpected SDL readback format')
        verify_scaled_frame(width,height,pixels,ppm.with_suffix('.rgb').read_bytes())
        count+=1
    if count==0:raise AssertionError('no SDL captures')
    return count


def run_case(binary, directory, render_every, demo=False, replay=None):
    env = dict(os.environ, SDL_VIDEODRIVER='dummy')
    command = [str(binary), '--synthetic', '--fast', '--steps', '120' if demo else '340',
               '--render-every', str(render_every), '--capture', str(directory)]
    if demo:
        command.append('--demo-input')
    if replay:
        command += ['--replay', str(replay)]
    result = subprocess.run(command, check=True, capture_output=True, text=True, cwd=ROOT, env=env)
    if 'LEMMINGS_RENDERER mode=software ' not in result.stderr:
        raise AssertionError('missing startup software fallback diagnostic')
    summary = json.loads(result.stdout)
    if summary['video_driver'] != 'dummy' or summary['renderer'] != 'software':
        raise AssertionError('expected actual SDL2 dummy video/software renderer')
    snapshots = [json.loads(line) for line in (directory/'trace.jsonl').read_text().splitlines()]
    if summary['control_step'] != len(snapshots)-1:
        raise AssertionError('missing native snapshots')
    return snapshots, summary, verify_pixels(directory)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT/'target/debug/lemmings-desktop')
    args = parser.parse_args()
    config = dict(width=640, height=160, minutes=5, total=10, rate=50, entrances=[[40, 40]],
                  pause_glitch=True, faller_three=True, steps=340, inputs=[], level='desktop synthetic')
    mask = bytes(int(y >= 100 and x < 260) for y in range(160) for x in range(640))
    with tempfile.TemporaryDirectory(prefix='lemmings-desktop-') as temporary:
        root = Path(temporary)
        mask_path = root/'mask.bin'
        mask_path.write_bytes(mask)
        expected = run_engine(config, mask_path, 'rust')['snapshots']
        single, summary, captures = run_case(args.binary, root/'single', 1)
        sparse, sparse_summary, sparse_captures = run_case(args.binary, root/'sparse', 7)
        if single != expected or sparse != expected or sparse_summary['frames'] >= summary['frames']:
            raise AssertionError('native state depends on rendering or differs from headless G05 runner')
        demo, demo_summary, demo_captures = run_case(args.binary, root/'demo', 1, demo=True)
        if (demo_summary['control_step'], demo_summary['iteration'], demo_summary['input_count'], demo_summary['camera']) != (100, 82, 3, 0):
            raise AssertionError('SDL Pause/Resume/RR/camera/Escape events differ')
        inputs = [dict(control_step=16, sequence=0, command='Pause'),
                  dict(control_step=34, sequence=0, command='Resume'),
                  dict(control_step=54, sequence=0, command='SetReleaseRate', rate=51)]
        expected_demo = run_engine(config | dict(steps=100, inputs=inputs), mask_path, 'rust')['snapshots']
        if demo != expected_demo:
            raise AssertionError('SDL input trace differs from deterministic input replay')
        replay, replay_summary, replay_captures = run_case(args.binary, root/'replay', 7, replay=root/'demo/inputs.txt')
        if replay[:101] != demo or replay_summary['input_count'] != 3:
            raise AssertionError('saved native input replay differs')
        # Refuse to replace a capture; the first run remains reviewable.
        again = subprocess.run([str(args.binary), '--synthetic', '--fast', '--steps', '1', '--capture', str(root/'single')],
                               capture_output=True, text=True, env=dict(os.environ, SDL_VIDEODRIVER='dummy'))
        if again.returncode == 0:
            raise AssertionError('capture overwrite accepted')
        print(json.dumps(dict(status='PASS', driver='SDL2 dummy/software', replay_cases=4,
                              snapshots=len(single)+len(sparse)+len(demo)+len(replay),
                              pixel_exact_captures=captures+sparse_captures+demo_captures+replay_captures,
                              synthetic_state_sha256=digest(single), input_demo_signature=digest(demo))))


if __name__ == '__main__':
    main()
