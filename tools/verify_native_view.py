#!/usr/bin/env python3
"""Record original-level SDL2 X11 evidence in a fresh assets/local directory.

Run on an isolated Xvfb display. Optional xdotool injects keys into that display.
"""
import argparse
import hashlib
import json
import os
import shutil
from pathlib import Path
import struct
import subprocess
import time
import zlib

from check_desktop import verify_pixels
from state_replay import ROOT, canonical, digest, run_engine, verified_assets


def file_hash(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def png_from_ppm(path):
    magic, dimensions, maximum, pixels = path.read_bytes().split(b'\n', 3)
    width, height = map(int, dimensions.split())
    if magic != b'P6' or maximum != b'255' or len(pixels) != width*height*3:
        raise ValueError('invalid captured framebuffer')
    def chunk(kind, data):
        return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind+data))
    rows = b''.join(b'\0'+pixels[y*width*3:(y+1)*width*3] for y in range(height))
    image = b'\x89PNG\r\n\x1a\n'
    image += chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0))
    image += chunk(b'IDAT', zlib.compress(rows)) + chunk(b'IEND', b'')
    with path.with_suffix('.png').open('xb') as output:
        output.write(image)


def collect(directory, elapsed_ms):
    summary = json.loads((directory/'run.json').read_text())
    if summary['video_driver'] != 'x11' or summary['renderer'] != 'software':
        raise AssertionError('X11/software SDL renderer was not executed')
    snapshots = [json.loads(line) for line in (directory/'trace.jsonl').read_text().splitlines()]
    if len(snapshots) != summary['control_step']+1:
        raise AssertionError('missing native boundary states')
    count = verify_pixels(directory)
    for path in sorted(directory.glob('*.ppm')):
        png_from_ppm(path)
    captures = [dict(name=p.name, size=p.stat().st_size, sha256=file_hash(p))
                for p in sorted(directory.glob('frame-*'))]
    row = dict(run=directory.name, runtime=summary, elapsed_ms=elapsed_ms,
               snapshots=len(snapshots), snapshots_sha256=digest(snapshots),
               trace_file_sha256=file_hash(directory/'trace.jsonl'),
               inputs_file_sha256=file_hash(directory/'inputs.txt'),
               pixel_exact_captures=count, captures=captures)
    return row, snapshots


def key_test(binary, directory, xdotool, env, expected_files):
    start = time.monotonic()
    process = subprocess.Popen([str(binary), '--steps', '1000', '--capture', str(directory)],
                               text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env, cwd=ROOT)
    def command(*args):
        return subprocess.check_output([str(xdotool), *args], text=True, env=env, timeout=10).strip()
    try:
        deadline = time.monotonic()+15
        while not (directory/'frame-000060.ppm').exists():
            if process.poll() is not None or time.monotonic()>deadline:
                raise AssertionError('native X11 window did not reach first walking actor')
            time.sleep(0.05)
        windows = command('search', '--onlyvisible', '--name', 'Oh My Lemmings').splitlines()
        if len(windows) != 1:
            raise AssertionError('use an isolated Xvfb display with exactly one test window')
        command('windowfocus', '--sync', windows[0])
        command('key', 'space')
        time.sleep(0.3)
        command('key', 'Up', 'Up', 'Down', 'Right', 'Left')
        time.sleep(0.2)
        command('key', 'space')
        time.sleep(0.3)
        command('key', 'Escape')
        stdout, stderr = process.communicate(timeout=10)
        if process.returncode:
            raise AssertionError(f'native input process failed: {stderr}')
    finally:
        if process.poll() is None:
            process.terminate()
            process.communicate(timeout=10)
    row, snapshots = collect(directory, round((time.monotonic()-start)*1000))
    summary = row['runtime']
    if summary['input_count'] != 5 or summary['control_step'] <= summary['iteration'] or summary['camera'] != 624:
        raise AssertionError('X11 Pause/Resume/RR/camera/Escape events not observed')
    inputs = []
    for line in (directory/'inputs.txt').read_text().splitlines():
        words = line.split()
        item = dict(control_step=int(words[0]), sequence=int(words[1]), command=words[2])
        if len(words) == 4:
            item['rate'] = int(words[3])
        inputs.append(item)
    config = dict(width=1584, height=160, minutes=5, total=10, rate=50, entrances=[[704, 36]],
                  pause_glitch=True, faller_three=True, steps=summary['control_step'], inputs=inputs, level='Fun 01 Just dig!')
    mask_path = directory.parent/'f00.mask'
    expected = run_engine(config, mask_path, 'rust', input_files=expected_files)['snapshots']
    if snapshots != expected or snapshots[-1]['paused'] or snapshots[-1]['release_rate'] != 51:
        raise AssertionError('native X11 event log differs from headless replay')
    row['observed_inputs'] = inputs
    row['comparison'] = 'MATCH_SUPPORTED with exact recorded integer boundaries replayed in G05'
    return row


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--binary', type=Path, default=ROOT/'target/release/lemmings-desktop')
    parser.add_argument('--assets', type=Path, default=ROOT/'assets/local/dos-original')
    parser.add_argument('--xdotool', type=Path)
    args = parser.parse_args()
    if not args.output.resolve().is_relative_to(ROOT/'assets/local'):
        raise ValueError('original-derived evidence must stay under assets/local')
    files = verified_assets(args.assets)
    frozen = json.loads((ROOT/'assets/manifests/state_replay.json').read_text())
    expected = {r['scenario']: r for r in frozen['scenarios']}
    args.output.mkdir(parents=True)
    # Export the same already-verified binary initial mask through the resource example.
    with (args.output/'terrain.jsonl').open('x') as output:
        subprocess.run(['cargo', 'run', '--quiet', '--locked', '--offline', '-p', 'lemmings-resources',
                        '--example', 'inspect_terrain', '--', str(args.assets), str(args.output/'terrain')],
                       cwd=ROOT, check=True, stdout=output)
    mask = bytes(int(p < 128) for p in (args.output/'terrain/level-000.pixels.bin').read_bytes())
    (args.output/'f00.mask').write_bytes(mask)
    if hashlib.sha256(mask).hexdigest() != frozen['mask_sha256']:
        raise AssertionError('initial F00 mask differs')
    env = dict(os.environ, SDL_VIDEODRIVER='x11')
    rows = []
    cases = [('F00-realtime', 'F00', None, False, 1), ('F00-sparse', 'F00', None, True, 7),
             ('F02-98', 'F02-98', '54 0 SetReleaseRate 98\n', True, 7),
             ('F02-99', 'F02-99', '54 0 SetReleaseRate 99\n', True, 7),
             ('F03-glitch1', 'F03-glitch1', '16 0 Pause\n34 0 Resume\n', True, 7)]
    for name, scenario, script, fast, every in cases:
        directory = args.output/name
        command = [str(args.binary.resolve()), '--assets', str(args.assets), '--steps', '340',
                   '--render-every', str(every), '--capture', str(directory)]
        if fast:
            command += ['--fast']
        if script:
            path = args.output/(name+'.inputs.txt')
            path.write_text(script)
            command += ['--replay', str(path)]
        start = time.monotonic()
        subprocess.run(command, check=True, capture_output=True, text=True, env=env, cwd=ROOT, timeout=90)
        row, snapshots = collect(directory, round((time.monotonic()-start)*1000))
        if digest(snapshots) != expected[scenario]['snapshots_sha256']:
            raise AssertionError(f'{name}: native trace differs from recorded Lemmix method execution')
        row['reference_scenario'] = scenario
        row['comparison'] = 'MATCH_SUPPORTED with recorded Lemmix method execution'
        rows.append(row)
    if args.xdotool:
        rows.append(key_test(args.binary.resolve(), args.output/'X11-keyboard', Path(shutil.which(str(args.xdotool)) or args.xdotool).resolve(), env, files))
    record = dict(format='g06-native-view-evidence-v1', tracking_issue=27,
                  tracking_url='https://github.com/edwardkim/ohmylemmings/issues/27', original_files=files,
                  binary_sha256=file_hash(args.binary), cargo_lock_sha256=file_hash(ROOT/'Cargo.lock'),
                  app_sources={str(p.relative_to(ROOT)): file_hash(p) for p in sorted((ROOT/'apps/desktop').rglob('*.rs'))},
                  reference_manifest_sha256=file_hash(ROOT/'assets/manifests/state_replay.json'),
                  runs=rows, unverified=['DOS execution/screens', 'skills', 'object triggers/rescue/result', 'audio', 'Windows'])
    (args.output/'evidence.json').write_text(json.dumps(record, indent=2)+'\n')
    print(json.dumps(dict(status='PASS', native_runs=len(rows), snapshots=sum(r['snapshots'] for r in rows),
                          pixel_exact_captures=sum(r['pixel_exact_captures'] for r in rows),
                          realtime_ms=rows[0]['elapsed_ms'], realtime_frames=rows[0]['runtime']['frames'],
                          keyboard_checked=bool(args.xdotool))))


if __name__ == '__main__':
    main()
