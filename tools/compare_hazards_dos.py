#!/usr/bin/env python3
"""Compare exact water/fire/trap image anchors; never infer atomic DOS state from video."""
import argparse
import json
from pathlib import Path

from capture_dos import game_frame, identity, rgb_bytes, save_json, sha
from dos_observation import match_anchors
from reference_assets import ROOT, verify_assets
from verify_native_view import png_from_ppm


def main():
    import av
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--profile',choices=['water','fire','trap'],required=True)
    p.add_argument('--recording', type=Path, nargs='+', required=True)
    p.add_argument('--native', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    args = p.parse_args()
    if not args.output.resolve().is_relative_to(ROOT / 'assets/local') or av.__version__ != '14.2.0':
        raise ValueError('local evidence and pinned PyAV required')
    manifest = json.loads((ROOT / 'assets/manifests/dos_original.json').read_text())
    verify_assets(manifest, ROOT / manifest['local_directory'])
    states = [json.loads(line) for line in (args.native / 'trace.jsonl').read_text().splitlines()]
    rows = [json.loads(line) for line in (args.native / 'hazards.jsonl').read_text().splitlines()]
    if args.profile == 'trap':
        triggers = [i for i in range(1,len(rows)) if rows[i]['trapped'] > rows[i-1]['trapped']][:3]
        if len(triggers) < 3: raise ValueError('three native trap uses required')
        labels = sorted(set(triggers + [i for i in range(triggers[0],triggers[2])
            if any(frame==14 and active for _,frame,active in rows[i]['traps'])]))
    else:
        action = 'Drowning' if args.profile == 'water' else 'Vaporizing'
        first = next(i for i,state in enumerate(states) if state['actors'] and state['actors'][0]['action']==action)
        duration = 16 if args.profile == 'water' else 14
        labels = [first,first+7,first+duration-1,first+duration]
    expected = [(step, sha(game_frame(320, 200, (args.native / f'frame-{step:06}.rgb').read_bytes()))) for step in labels]
    args.output.mkdir()
    results = []
    for number, directory in enumerate(args.recording, 1):
        observation = json.loads((directory / 'memory-observation.json').read_text())
        controls = json.loads((directory / 'control-events.json').read_text())
        if observation['error'] or controls['error']:
            raise ValueError('failed DOS observation')
        output = args.output / f'run-{number}'
        output.mkdir()
        observed, videos, selected = [], [], set()
        for path in sorted((directory / 'capture').glob('*.avi')):
            count = 0
            with av.open(str(path)) as container:
                for frame in container.decode(video=0):
                    count += 1
                    if (frame.width, frame.height) != (640, 400):
                        continue
                    rgb = rgb_bytes(frame)
                    signature = sha(game_frame(640, 400, rgb))
                    observed.append((len(observed), signature))
                    for step, wanted in expected:
                        if signature == wanted and step not in selected:
                            selected.add(step)
                            native = (args.native / f'frame-{step:06}.rgb').read_bytes()
                            # Side-by-side raw image; no mask, shift or tolerance.
                            dos = b''.join(b''.join(rgb[(y * 2 * 640 + x * 2) * 3:(y * 2 * 640 + x * 2) * 3 + 3]
                                                  for x in range(320)) for y in range(160))
                            pair = b''.join(dos[y * 960:(y + 1) * 960] + native[y * 960:(y + 1) * 960] for y in range(160))
                            ppm = output / f'pair-{step:06}.ppm'
                            ppm.write_bytes(b'P6\n640 160\n255\n' + pair)
                            png_from_ppm(ppm)
            videos.append(identity(path) | dict(decoded_frames=count))
        result = match_anchors(expected, observed)
        result.update(recording=str(directory), game_video_frames=len(observed), videos=videos)
        save_json(output / 'comparison.json', result)
        results.append(result)
        print(json.dumps(dict(run=number, status=result['status'], frames=len(observed))), flush=True)
    save_json(args.output / 'summary.json', dict(status='MATCH_FULL_PLAYFIELD' if all(r['status'] == 'MATCH_VISUAL_ANCHORS' for r in results) else 'UNVERIFIED',
        profile=args.profile, native_labels=labels, scope='ordered full320x160 hazard anchors; exact 2x/RGB565 normalization; no cursor mask, shift, tolerance or DOS tick inference', runs=results))


if __name__ == '__main__':
    main()
