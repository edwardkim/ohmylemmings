#!/usr/bin/env python3
"""Strict full-playfield Basher image anchors; DOS state remains a limited observation."""
import argparse
import json
from pathlib import Path

from capture_dos import identity, rgb_bytes, save_json, sha
from dos_observation import game_frame, match_anchors
from reference_assets import ROOT, verify_assets
from verify_native_view import png_from_ppm


def main():
    import av
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--recording', type=Path, nargs='+', required=True)
    parser.add_argument('--native', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if not args.output.resolve().is_relative_to(ROOT/'assets/local'):
        raise ValueError('comparison evidence must stay in assets/local')
    if av.__version__ != '14.2.0': raise ValueError('pinned PyAV required')
    manifest = json.loads((ROOT/'assets/manifests/dos_original.json').read_text())
    verify_assets(manifest, ROOT/manifest['local_directory'])
    args.output.mkdir()
    # After moving the actual DOS pointer into the HUD: full playfield, no masks.
    labels = [220, 260, 340]
    expected = [(label, sha(game_frame(320,200,(args.native/f'frame-{label:06}.rgb').read_bytes()))) for label in labels]
    rows = []
    for number, directory in enumerate(args.recording, 1):
        observation = json.loads((directory/'memory-observation.json').read_text())
        controls = json.loads((directory/'control-events.json').read_text())
        if observation['error'] or controls['error']: raise ValueError('failed DOS observation')
        samples = observation['samples']
        request = [e for e in controls['events'] if e['command']==['keydown','space']]
        if len(request)!=1 or request[0]['sample']['selected_skill']!=7 or request[0]['sample']['actor0_action_bits']!=0:
            raise ValueError('walking Basher input not observed')
        if not any(s['remaining_bashers']==49 and s['actor0_action_bits']&512 for s in samples):
            raise ValueError('Basher acceptance absent')
        indices, videos, selected = [], [], set()
        index = 0
        output = args.output/f'run-{number}'
        output.mkdir()
        for path in sorted((directory/'capture').glob('*.avi')):
            with av.open(str(path)) as container:
                count = 0
                for frame in container.decode(video=0):
                    count += 1
                    if (frame.width,frame.height)!=(640,400): continue
                    rgb = rgb_bytes(frame)
                    signature = sha(game_frame(640,400,rgb))
                    indices.append((index,signature))
                    for label, wanted in expected:
                        if wanted==signature and label not in selected:
                            selected.add(label)
                            native=(args.native/f'frame-{label:06}.rgb').read_bytes()
                            # Exact 2x verification occurred above. Make a viewable
                            # left DOS/right native pair, without altering pixels.
                            dos=b''.join(b''.join(rgb[(y*2*640+x*2)*3:(y*2*640+x*2)*3+3] for x in range(320)) for y in range(160))
                            pair=b''.join(dos[y*960:(y+1)*960]+native[y*960:(y+1)*960] for y in range(160))
                            ppm=output/f'pair-{label:06}.ppm'
                            ppm.write_bytes(b'P6\n640 160\n255\n'+pair)
                            png_from_ppm(ppm)
                    index += 1
                videos.append(identity(path) | dict(decoded_frames=count))
        result=match_anchors(expected, indices)
        # Generic G06 unverified list still includes skill/internal-state ticks:
        # these image matches and numeric observations certify no full state.
        result.update(recording=str(directory), game_video_frames=index, videos=videos,
                      input_sample=request[0]['sample'])
        save_json(output/'comparison.json',result)
        rows.append(result)
        print(json.dumps(dict(run=number,status=result['status'],frames=index)),flush=True)
    save_json(args.output/'summary.json', dict(status='MATCH_FULL_PLAYFIELD' if all(r['status']=='MATCH_VISUAL_ANCHORS' for r in rows) else 'UNVERIFIED',
              scope='three post-assignment full 320x160 image anchors; no cursor mask/shift/color tolerance', runs=rows))


if __name__ == '__main__':
    main()
