#!/usr/bin/env python3
"""Record three isolated DOS Fun01 runs. Requires DOSBox 0.74-3, xdotool, PyAV 14.2.0.

Use a dedicated X11 display. Original files are verified, copied and never patched.
All runtime configuration, recordings and captures stay under ignored assets/local/.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import time

from dos_observation import game_frame, match_anchors, repeat_verdict
from reference_assets import ROOT, verify_assets

MAPPER = '''key_enter "key 13"
key_1 "key 49"
key_2 "key 50"
key_f1 "key 282"
key_f4 "key 285"
key_3 "key 51"
key_space "key 32"
key_a "key 97"
key_p "key 112"
hand_scrshot "key 115"
hand_video "key 118"
hand_shutdown "key 120"
'''
SETTINGS = '''[sdl]
fullscreen=false
output=surface
windowresolution=original
waitonerror=false
priority=normal,normal
usescancodes=false
mapperfile={directory}/mapper.map
[dosbox]
machine=vgaonly
memsize=16
captures={directory}/capture
[render]
frameskip=0
aspect=false
scaler=none
[cpu]
core=normal
cputype=386
cycles=fixed 10000
[mixer]
nosound=false
rate=44100
blocksize=1024
prebuffer=25
[midi]
mpu401=none
mididevice=none
[sblaster]
sbtype=sbpro2
sbbase=220
irq=7
dma=1
oplmode=opl2
oplemu=default
oplrate=44100
[gus]
gus=false
[speaker]
pcspeaker=false
tandy=off
disney=false
[joystick]
joysticktype=none
[dos]
xms=true
ems=true
umb=true
keyboardlayout=none
[autoexec]
mount c "{directory}/drive"
c:
VGALEMMI /v
'''
PROCEDURE = [('2', 500), ('Return', 7000), ('s', 200), ('F4', 5000),
             ('s', 200), ('3', 5000), ('s', 200), ('F1', 5000),
             ('s', 200), ('v', 300), ('space', 2000),
             ('cursor_down_right', 2000), ('wait', 22000),
             ('s', 200), ('v', 200), ('x', 0)]
ANCHORS = [0, 35, 54, 60, 100, 170, 340]


def sha(data):
    return hashlib.sha256(data).hexdigest()


def identity(path):
    return dict(name=path.name, size=path.stat().st_size, sha256=sha(path.read_bytes()))


def save_json(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n')


def rgb_bytes(frame):
    rgb = frame.reformat(format='rgb24')
    plane = rgb.planes[0]
    raw = bytes(plane)
    return b''.join(raw[y*plane.line_size:y*plane.line_size+rgb.width*3] for y in range(rgb.height))


def save_ppm(path, width, height, rgb):
    with path.open('xb') as f:
        f.write(f'P6\n{width} {height}\n255\n'.encode()+rgb)


def record(args, directory, manifest, runtime=None, *, settings=SETTINGS,
           procedure=PROCEDURE, observer=None, direct_window_keys=False,
           terminate_after_procedure=False):
    directory.mkdir()
    (directory/'drive').mkdir()
    (directory/'capture').mkdir()
    for row in manifest['files']:
        shutil.copyfile(ROOT/manifest['local_directory']/row['name'], directory/'drive'/row['name'])
    if runtime is not None:
        (directory/'drive/RUSSELL.DAT').write_bytes(runtime)
    (directory/'mapper.map').write_text(MAPPER)
    (directory/'dosbox.conf').write_text(settings.format(directory=directory))
    env = dict(os.environ, SDL_VIDEODRIVER='x11', SDL_AUDIODRIVER='dummy')
    inputs, start = [], time.monotonic()

    def command(*parts):
        return subprocess.check_output([str(args.xdotool), *parts], env=env, text=True,
                                       stderr=subprocess.DEVNULL, timeout=10).strip()

    def key(name):
        selector = ['--pid', str(process.pid)] if direct_window_keys else []
        windows = command('search', '--onlyvisible', *selector, '--name', 'DOSBox').splitlines()
        if len(windows) != 1:
            raise ValueError('a dedicated X11 display with one DOSBox window is required')
        command('windowfocus', '--sync', windows[0])
        inputs.append(dict(host_elapsed_ms=round((time.monotonic()-start)*1000), key=name))
        target = ['--window', windows[0]] if direct_window_keys else []
        try:
            command('key', *target, '--clearmodifiers', '--delay', '150', name)
        except subprocess.CalledProcessError as error:
            # DOSBox's mapped shutdown can destroy the target on keydown,
            # before xdotool sends keyup. Accept only that child's clean exit.
            if not direct_window_keys or name != 'x' or error.returncode != 1:
                raise
            try:
                returncode = process.wait(timeout=0.5)
            except subprocess.TimeoutExpired:
                raise error
            if returncode != 0:
                raise error

    with (directory/'dosbox.log').open('w') as log:
        process = subprocess.Popen([str(args.dosbox), '-conf', str(directory/'dosbox.conf')],
                                   env=env, stdout=log, stderr=log)
        observation = None
        try:
            if observer is not None:
                observation = observer(process)
            time.sleep(2)
            if runtime is None:
                key('s')  # machine selection screen
                key('Return')
                time.sleep(7)
                key('s')  # key disk error
                key('x')
            else:
                for name, wait_ms in procedure:
                    if name == 'cursor_down_right':
                        inputs.append(dict(host_elapsed_ms=round((time.monotonic()-start)*1000),
                                           key='keydown a p'))
                        command('keydown', 'a', 'keydown', 'p')
                        time.sleep(wait_ms/1000)
                        command('keyup', 'p', 'keyup', 'a')
                        inputs.append(dict(host_elapsed_ms=round((time.monotonic()-start)*1000),
                                           key='keyup p a'))
                        continue
                    if name != 'wait':
                        key(name)
                    time.sleep(wait_ms/1000)
            if terminate_after_procedure and process.poll() is None:
                process.terminate()
            process.wait(timeout=10)
            if process.returncode and not (terminate_after_procedure and process.returncode == -signal.SIGTERM):
                raise ValueError(f'DOSBox failed: {process.returncode}')
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=10)
            if observation is not None:
                observation.close()
            save_json(directory/'host-inputs.json', inputs)
    verify_assets(manifest, directory/'drive')
    extra = sorted(p.name for p in (directory/'drive').iterdir())
    if set(extra) != {r['name'] for r in manifest['files']} | {'RUSSELL.DAT'}:
        raise ValueError('unexpected DOS runtime files')
    return (directory/'drive/RUSSELL.DAT').read_bytes()


def analyze(directory, native):
    import av  # optional external decoder; offline unit tests never import it
    expected = [(step, sha(game_frame(320, 200, (native/f'frame-{step:06}.rgb').read_bytes())))
                for step in ANCHORS]
    signatures = {signature: step for step, signature in expected}
    observed, videos, samples, index = [], [], set(), 0
    for path in sorted((directory/'capture').glob('*.avi')):
        with av.open(str(path)) as container:
            stream = container.streams.video[0]
            video = identity(path) | dict(width=stream.width, height=stream.height,
                                          rate=str(stream.average_rate), decoded_frames=0,
                                          format=stream.codec_context.format.name)
            for local_index, frame in enumerate(container.decode(video=0)):
                video['decoded_frames'] += 1
                if (frame.width, frame.height) != (640, 400):
                    continue  # menu has 640x350; retained in the original video
                rgb = rgb_bytes(frame)
                signature = sha(game_frame(frame.width, frame.height, rgb))
                observed.append((index, signature))
                if signature in signatures and signature not in samples:
                    step = signatures[signature]
                    save_ppm(directory/f'dos-anchor-{step:06}.ppm', frame.width, frame.height, rgb)
                    samples.add(signature)
                # Preserve periodic actual frames even when no anchor matches.
                if index % 140 == 0:
                    save_ppm(directory/f'dos-video-{index:06}.ppm', frame.width, frame.height, rgb)
                index += 1
            videos.append(video)
    save_json(directory/'video-frames.json', observed)
    result = match_anchors(expected, observed)
    return result | dict(videos=videos, game_video_frames=index,
                         scene_sha256=[s for _, s in expected],
                         host_inputs=identity(directory/'host-inputs.json'),
                         runtime_after=identity(directory/'drive/RUSSELL.DAT'))


def main():
    import av
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--dosbox', type=Path, required=True)
    parser.add_argument('--xdotool', type=Path, required=True)
    parser.add_argument('--native', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.dosbox, args.xdotool = args.dosbox.resolve(), args.xdotool.resolve()
    args.native, args.output = args.native.resolve(), args.output.resolve()
    if (ROOT/'assets/local').resolve() not in args.output.parents:
        raise ValueError('original recordings must stay in ignored assets/local/')
    if av.__version__ != '14.2.0':
        raise ValueError('pin PyAV 14.2.0 for capture decoding')
    manifest = json.loads((ROOT/'assets/manifests/dos_original.json').read_text())
    verify_assets(manifest, ROOT/manifest['local_directory'])
    args.output.mkdir()  # never overwrite an existing observation
    settings = dict(config_template=SETTINGS, mapper=MAPPER, procedure=PROCEDURE,
                    dosbox=identity(args.dosbox), decoder_version=av.__version__,
                    decoder_libraries={k: list(v) for k, v in av.library_versions.items()},
                    observation_kind='video', original_profile=manifest['profile'],
                    original_files=manifest['files'], pc_mode='2 high performance',
                    host_audio='SDL dummy; AdLib emulation enabled; audio not compared',
                    gameplay_inputs=[], coordinate_crop=[0, 0, 320, 160],
                    normalization='exact 2x duplicates verified; R5/G6/B5 codes for both sources')
    settings_hash = sha(json.dumps(settings, sort_keys=True, separators=(',', ':')).encode())
    save_json(args.output/'settings.json', settings)
    original_runtime = record(args, args.output/'bootstrap', manifest)
    if len(original_runtime) != 256 or original_runtime[0] != 9:
        raise ValueError('unexpected generated runtime state; inspect bootstrap evidence')
    runtime = bytes([0])+original_runtime[1:]
    save_json(args.output/'runtime-seed.json', dict(generated_sha256=sha(original_runtime),
              seeded_sha256=sha(runtime), length=256, changed_offsets=[0], before=[9], after=[0],
              exe_or_original_dat_modified=False,
              limitation='seeded runtime disk-check counter; original key disk not acquired/emulated'))
    rows = []
    for number in range(1, 4):
        directory = args.output/f'run-{number}'
        record(args, directory, manifest, runtime)
        row = analyze(directory, args.native) | dict(run=number, settings_sha256=settings_hash)
        save_json(directory/'comparison.json', row)
        rows.append(row)
        print(json.dumps(dict(run=number, status=row['status'], frames=row['game_video_frames'])), flush=True)
    summary = dict(status=repeat_verdict(rows), settings_sha256=settings_hash,
                   runtime_seed=identity(args.output/'runtime-seed.json'), runs=rows)
    save_json(args.output/'summary.json', summary)
    print(json.dumps(dict(status=summary['status'], runs=3)), flush=True)


if __name__ == '__main__':
    main()
