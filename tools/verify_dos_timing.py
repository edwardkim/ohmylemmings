#!/usr/bin/env python3
"""Verify VGA/PIT timing paths using recorded original video and optional RAM polls.

This checks image-anchor intervals and documented counters, not complete DOS state.
PyAV 14.2.0 is optional for offline tests but required for this original-data tool.
"""
import argparse
from fractions import Fraction
import json
from pathlib import Path

from capture_dos import identity, rgb_bytes, save_json
from dos_observation import game_frame, match_anchors, scene_signatures
from dos_timing import check_cadence, control_rate
from reference_assets import ROOT, verify_assets


def inspect(directory, native, manifest):
    import av
    verify_assets(manifest, directory/'drive')
    config = (directory/'dosbox.conf').read_text()
    command = next(line for line in config.splitlines() if line.startswith('VGALEMMI '))
    # Original parser's -x sets DS:200e, which bypasses VGA sync in IRQ0 and
    # uses PIT divisor 0x4000. GUI PC2 leaves it clear. Do not equate their names.
    unlocked = '-x' in command.split()
    expected = [(i, scene_signatures(game_frame(320, 200, (native/f'frame-{i:06}.rgb').read_bytes()))['upper_320x100'])
                for i in [35, 54, 60, 100, 170, 340]]
    observed, videos, index, rates = [], [], 0, set()
    for path in sorted((directory/'capture').glob('*.avi')):
        with av.open(str(path)) as container:
            stream = container.streams.video[0]
            video = identity(path) | dict(rate=str(stream.average_rate), frames=0,
                                          width=stream.width, height=stream.height)
            for frame in container.decode(video=0):
                video['frames'] += 1
                if (frame.width, frame.height) != (640, 400):
                    continue
                rates.add(stream.average_rate)
                observed.append((index, scene_signatures(game_frame(frame.width, frame.height, rgb_bytes(frame)))['upper_320x100']))
                index += 1
            videos.append(video)
    matches = match_anchors(expected, observed)
    if matches['status'] != 'MATCH_VISUAL_ANCHORS' or len(rates) != 1:
        raise ValueError('all six moving image anchors and one VGA video rate required')
    rate = Fraction(next(iter(rates)))
    anchors = [(a['native_control_step'], a['dos_frames'][0]) for a in matches['anchors']]
    cadence = check_cadence(anchors, rate, control_rate(unlocked, 16384 if unlocked else 1, rate))
    row = dict(directory=str(directory.relative_to(ROOT)), command=command,
               config=identity(directory/'dosbox.conf'), mapper=identity(directory/'mapper.map'),
               inputs=identity(directory/'host-inputs.json'), videos=videos,
               game_frames=index, image_anchors=matches, cadence=cadence,
               timing_path='free-running PIT/5' if unlocked else 'VGA-synchronised IRQ/5')
    path = directory/'memory-observation.json'
    if path.exists():
        raw = json.loads(path.read_text())
        if raw['error'] or not raw['location']:
            raise ValueError('failed memory observation cannot certify timing fields')
        game = [s for s in raw['samples'] if 1 <= s['released_u8'] <= 10]
        flags = {tuple(s['pc_flags']) for s in game}
        divisors = sorted({s['pit_divisor_u16'] for s in game})
        if len(flags) != 1 or any((s['pc_flags'][4] == 255) != unlocked for s in game):
            raise ValueError('observed timer flag differs from the declared command path')
        if unlocked and divisors != [16384]:
            raise ValueError('unlocked PIT divisor differs from original code')
        first_birth = next(s for s in raw['samples'] if s['released_u8'] == 1)
        prewrap = [s for s in raw['samples'] if s['released_u8'] < 3]
        stable = {s['startup_counter_u8']: s for s in prewrap}
        if (first_birth['startup_counter_u8'], stable[36]['object_frames_u8'][0],
                stable[55]['object_frames_u8'][2]) != (55, 2, 2):
            raise ValueError('DOS entrance/birth/continuous frame mapping differs')
        row['memory'] = dict(file=identity(path), samples=len(raw['samples']),
                             poll_interval_ns=raw['poll_interval_ns'], termination=raw['termination'],
                             pc_flags=list(next(iter(flags))), pit_divisors=divisors,
                             first_birth_startup_counter=55, native_birth_image_label=54,
                             entrance_frame_at_counter_36=2, flag_frame_at_counter_55=2,
                             limitation='asynchronous raw polls; not an atomic full-state trace')
    return row


def main():
    import av
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--recording', type=Path, nargs='+', required=True)
    parser.add_argument('--native', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    if (ROOT/'assets/local').resolve() not in output.parents:
        raise ValueError('original observation output must stay in ignored assets/local/')
    if av.__version__ != '14.2.0':
        raise ValueError('PyAV 14.2.0 required')
    output.mkdir()
    manifest = json.loads((ROOT/'assets/manifests/dos_original.json').read_text())
    rows = []
    for directory in args.recording:
        row = inspect(directory.resolve(), args.native.resolve(), manifest)
        rows.append(row)
        save_json(output/f'run-{len(rows)}.json', row)
        print(json.dumps(dict(run=row['directory'], status=row['cadence']['status'], path=row['timing_path'])), flush=True)
    result = dict(format='dos-timer-path-evidence-v1', tracking_issue=30,
                  video_decoder=av.__version__, runs=rows,
                  status='OBSERVED_TIMER_PATHS',
                  cadence_matches=sum(r['cadence']['status'] == 'MATCH_VIDEO_QUANTIZATION' for r in rows),
                  cadence_differences=[r['directory'] for r in rows if r['cadence']['status'] != 'MATCH_VIDEO_QUANTIZATION'],
                  unverified=['complete DOS state', 'other DOS revisions/hardware', 'audio'])
    save_json(output/'summary.json', result)


if __name__ == '__main__':
    main()
