#!/usr/bin/env python3
"""Strict full-playfield Nuke image anchors; DOS state remains a limited observation."""
import argparse
import json
from pathlib import Path

from capture_dos import identity, rgb_bytes, save_json, sha
from dos_observation import game_frame, match_anchors
from reference_assets import ROOT, verify_assets
from verify_native_view import png_from_ppm


from compare_builder_dos import repeat_builder_verdict as repeat_nuke_verdict


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
    labels = [121,122,123,199,200,216,220,260]
    expected = [(label, sha(game_frame(320,200,(args.native/f'frame-{label:06}.rgb').read_bytes()))) for label in labels]
    rows = []
    for number, directory in enumerate(args.recording, 1):
        verify_assets(manifest, directory/'drive')
        provenance = json.loads((directory/'provenance.json').read_text())
        if (identity(directory/'dosbox.conf') != provenance['config']
                or identity(directory/'mapper.map') != provenance['mapper']
                or provenance['original_files'] != manifest['files']):
            raise ValueError('DOS execution evidence changed')
        protocol = dict(dosbox=provenance['dosbox'], seed=provenance['runtime_seed_sha256'],
                        procedure=provenance['procedure'], mapper=provenance['mapper'],
                        controller_sha256=provenance['controller_sha256'],candidate=provenance['level_candidate'],
                        capture_driver_sha256=provenance['capture_driver_sha256'],termination=provenance['termination'],
                        config=(directory/'dosbox.conf').read_text().replace(str(directory.resolve()), '<RUN>'))
        protocol_sha = sha(json.dumps(protocol, sort_keys=True, separators=(',', ':')).encode())
        observation = json.loads((directory/'memory-observation.json').read_text())
        controls = json.loads((directory/'control-events.json').read_text())
        if observation['error'] or controls['error']: raise ValueError('failed DOS observation')
        samples = observation['samples']
        request=[e for e in controls['events'] if e['command']==['keydown','F12']]
        if len(request)!=2 or request[0]['sample']['released_u8']!=3 or request[1]['sample']['startup_counter_u8']!=121:raise ValueError('Nuke double-key boundary not observed')
        if not any(s['max_lemmings_u8']==3 and s['nuke_assigning_u8']==255 and s['nuke_next_index_u8']==1 and s['actors'][0]['timer']==79 for s in samples):raise ValueError('first Nuke timer not observed')
        if not any(len(s['actors'])==3 and [a['timer'] for a in s['actors']]==[77,78,79] for s in samples):raise ValueError('sequential timers not observed')
        if not any(s['nuke_assigning_u8']==0 and s['nuke_next_index_u8']==3 and s['released_u8']==3 and s['max_lemmings_u8']==3 for s in samples):raise ValueError('Nuke scan completion absent')
        if not any(len(s['actors'])==3 and all(a['removed']==255 for a in s['actors']) for s in samples):raise ValueError('three removals absent')
        if any(s['remaining_bombers']!=5 for s in samples if s['max_lemmings_u8']==3):raise ValueError('Nuke consumed inventory')
        indices, videos, selected = [], [], set()
        index = 0
        post_anchor_frames = 0
        output = args.output/f'run-{number}'
        output.mkdir()
        for path in sorted((directory/'capture').glob('*.avi')):
            with av.open(str(path)) as container:
                count = 0
                for frame in container.decode(video=0):
                    count += 1
                    if (frame.width,frame.height)!=(640,400): continue
                    # The requested evidence ends at anchor260, before the
                    # unsupported result screen. Preserve/count later frames
                    # without applying gameplay-only 2x rules to result text.
                    if len(selected)==len(labels):
                        post_anchor_frames += 1
                        continue
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
        result.update(recording=str(directory), game_video_frames=index, post_anchor_frames_excluded=post_anchor_frames, videos=videos,
                      input_samples=[r['sample'] for r in request], protocol_sha256=protocol_sha)
        save_json(output/'comparison.json',result)
        rows.append(result)
        print(json.dumps(dict(run=number,status=result['status'],frames=index)),flush=True)
    save_json(args.output/'summary.json', dict(status=repeat_nuke_verdict(rows),
              scope='sequential arming/countdown/expiry/explosion/particles full320x160 through anchor260; later result frames excluded, no cursor mask/shift/color tolerance', runs=rows))


if __name__ == '__main__':
    main()
