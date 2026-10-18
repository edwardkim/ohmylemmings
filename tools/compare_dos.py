#!/usr/bin/env python3
"""Recheck recorded DOS video hashes; report full and upper Fun01 views separately.

The upper 320x100 view contains Fun01's entrance and walking platform. It excludes
the exit decoration and is never used as a verdict on the full 320x160 playfield.
Requires the same pinned PyAV 14.2.0 decoder as capture_dos.py, without Pillow/numpy.
"""
import argparse
import json
from pathlib import Path

from capture_dos import ANCHORS, identity, rgb_bytes, save_json, save_ppm, sha
from dos_observation import game_frame, match_anchors, pixel_difference, repeat_verdict, scene_signatures
from reference_assets import ROOT, verify_assets
from verify_native_view import png_from_ppm
from check_desktop import verify_pixels
from state_replay import digest


def display_rgb(frame):
    result = bytearray(len(frame))
    for c, maximum in enumerate([31, 63, 31]):
        result[c::3] = frame[c::3].translate(bytes(min(v, maximum)*255//maximum for v in range(256)))
    return bytes(result)


def compare(recording, native, output):
    import av
    if av.__version__ != '14.2.0':
        raise ValueError('pin PyAV 14.2.0')
    settings = json.loads((recording/'settings.json').read_text())
    settings_hash = sha(json.dumps(settings, sort_keys=True, separators=(',', ':')).encode())
    summary = json.loads((recording/'summary.json').read_text())
    if settings_hash != summary['settings_sha256']:
        raise ValueError('recording settings changed')
    manifest = json.loads((ROOT/'assets/manifests/dos_original.json').read_text())
    if settings['original_files'] != manifest['files'] or settings['gameplay_inputs']:
        raise ValueError('original identities or no-input scenario changed')
    verify_assets(manifest, recording/'bootstrap/drive')
    for directory in [recording/'bootstrap', *(recording/f'run-{i}' for i in range(1, 4))]:
        if (directory/'dosbox.conf').read_text() != settings['config_template'].format(directory=directory):
            raise ValueError('actual DOSBox configuration differs from recorded settings')
        if (directory/'mapper.map').read_text() != settings['mapper']:
            raise ValueError('actual DOSBox key mapper differs from recorded settings')
    runtime_seed = json.loads((recording/'runtime-seed.json').read_text())
    runtime_original = (recording/'bootstrap/drive/RUSSELL.DAT').read_bytes()
    runtime_seeded = bytes([0])+runtime_original[1:]
    if (len(runtime_original), runtime_original[0], sha(runtime_original), sha(runtime_seeded)) != (
            256, 9, runtime_seed['generated_sha256'], runtime_seed['seeded_sha256']):
        raise ValueError('runtime seed provenance changed')
    run = json.loads((native/'run.json').read_text())
    if (run['control_step'], run['input_count'], run['camera']) != (340, 0, 624):
        raise ValueError('expected native no-input Fun01 at camera 624 through 340 calls')
    snapshots = [json.loads(line) for line in (native/'trace.jsonl').read_text().splitlines()]
    baseline = json.loads((ROOT/'assets/manifests/native_view.json').read_text())
    reference = next(r for r in baseline['runs'] if r['run'] == 'F00-realtime')
    if len(snapshots) != 341 or digest(snapshots) != reference['snapshots_sha256']:
        raise ValueError('native state differs from recorded F00 baseline')
    native_captures = verify_pixels(native)
    frames = {step: game_frame(320, 200, (native/f'frame-{step:06}.rgb').read_bytes()) for step in ANCHORS}
    expected = {scope: [(step, scene_signatures(frame)[scope]) for step, frame in frames.items()]
                for scope in ['full_320x160', 'upper_320x100']}
    output.mkdir()
    rows = []
    for number in range(1, 4):
        source, dest = recording/f'run-{number}', output/f'run-{number}'
        dest.mkdir()
        recorded = json.loads((source/'comparison.json').read_text())
        if recorded['settings_sha256'] != settings_hash:
            raise ValueError('settings differ between runs')
        verify_assets(manifest, source/'drive')
        if identity(source/'host-inputs.json') != recorded['host_inputs']:
            raise ValueError('host input log changed')
        if identity(source/'drive/RUSSELL.DAT') != recorded['runtime_after']:
            raise ValueError('DOS-generated runtime state changed')
        observed = {scope: [] for scope in expected}
        wanted = {signature: step for step, signature in expected['upper_320x100']}
        samples, index = {}, 0
        for video in recorded['videos']:
            path = source/'capture'/video['name']
            if any(identity(path)[key] != video[key] for key in ['name', 'size', 'sha256']):
                raise ValueError('original video bytes changed')
            with av.open(str(path)) as container:
                count = 0
                for local_index, frame in enumerate(container.decode(video=0)):
                    count += 1
                    if (frame.width, frame.height) != (640, 400):
                        continue
                    normalized = game_frame(frame.width, frame.height, rgb_bytes(frame))
                    signatures = scene_signatures(normalized)
                    for scope in expected:
                        observed[scope].append((index, signatures[scope]))
                    signature = signatures['upper_320x100']
                    if signature in wanted and wanted[signature] not in samples:
                        step = wanted[signature]
                        difference = pixel_difference(frames[step], normalized)
                        samples[step] = dict(native_control_step=step, game_video_frame=index,
                                             video=video['name'], local_video_frame=local_index,
                                             pts=frame.pts, time_base=str(frame.time_base),
                                             full_playfield_difference=difference)
                        dos_rgb, native_rgb = display_rgb(normalized), display_rgb(frames[step])
                        paired = b''.join(dos_rgb[y*960:(y+1)*960]+native_rgb[y*960:(y+1)*960] for y in range(160))
                        ppm = dest/f'pair-{step:06}.ppm'
                        save_ppm(ppm, 640, 160, paired)
                        png_from_ppm(ppm)
                    index += 1
                if count != video['decoded_frames']:
                    raise ValueError('decoded video frame count changed')
        if index != recorded['game_video_frames']:
            raise ValueError('game recording frame count changed')
        full, upper = (match_anchors(expected[s], observed[s]) for s in ['full_320x160', 'upper_320x100'])
        row = dict(run=number, settings_sha256=settings_hash,
                   status=upper['status'], scene_sha256=[s for _, s in expected['upper_320x100']],
                   full_playfield=full, upper_view=upper, game_video_frames=index,
                   samples=[samples[s] for s in ANCHORS if s in samples],
                   artifacts=[identity(p) for p in sorted(dest.glob('*.png'))])
        save_json(dest/'comparison.json', row)
        rows.append(row)
    result = dict(upper_view_status=repeat_verdict(rows), full_playfield_status=summary['status'],
                  observation_kind='video', settings_sha256=settings_hash,
                  normalization='verified exact 2x playfield duplicates; R5/G6/B5 codes on both sides',
                  upper_scope=[0, 0, 320, 100], full_scope=[0, 0, 320, 160],
                  side_by_side='left DOS, right native; normalized capture colors; playfield only',
                  native_trace=identity(native/'trace.jsonl'),
                  native_run=identity(native/'run.json'),
                  native_pixel_exact_captures=native_captures,
                  runtime_seed=runtime_seed, runs=rows)
    save_json(output/'summary.json', result)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--recording', type=Path, required=True)
    parser.add_argument('--native', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    if (ROOT/'assets/local').resolve() not in output.parents:
        raise ValueError('original-derived comparisons must stay in ignored assets/local/')
    result = compare(args.recording.resolve(), args.native.resolve(), output)
    print(json.dumps({k: result[k] for k in ['upper_view_status', 'full_playfield_status', 'observation_kind']}))


if __name__ == '__main__':
    main()
