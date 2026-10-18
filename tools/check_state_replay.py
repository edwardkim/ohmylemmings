#!/usr/bin/env python3
"""Offline Rust regression against hashes recorded from executed Pascal methods."""
import argparse
import hashlib
import json
from pathlib import Path
import tempfile

from state_replay import ROOT, canonical, compare, digest, run_engine

GOLDEN = ROOT/'tools/fixtures/state_replay_golden.json'


def cases():
    base = dict(width=96, height=160, minutes=5, total=10, rate=50, entrances=[[0, 0]],
                pause_glitch=True, faller_three=True, steps=340, inputs=[], level='synthetic')
    pause = [dict(control_step=16, sequence=0, command='Pause'),
             dict(control_step=34, sequence=0, command='Resume')]
    rate = [dict(control_step=54, sequence=0, command='SetReleaseRate', rate=99)]
    for shape in ['floor', 'fatal', 'empty', 'stair', 'wall', 'head']:
        mask = bytes(int((y >= 60 if shape in ['floor', 'wall'] else
                          y >= 100 if shape == 'fatal' else
                          y >= 56 or (48 <= x < 64 and y >= 53) if shape == 'stair' else
                          y >= 4 if shape == 'head' else False) or
                         (shape == 'wall' and 40 <= x < 48 and y >= 32))
                     for y in range(160) for x in range(96))
        for name, inputs in [('none', []), ('rr99', rate), ('pause', pause)]:
            for glitch in [False, True]:
                case_id = f'{shape}-{name}-glitch{int(glitch)}'
                yield case_id, base | dict(level=case_id, inputs=inputs, pause_glitch=glitch), mask
    # Exercise birth order, invalid rates, duplicate Pause, same-boundary ordering,
    # zero entrances/cap and the alternate FallerStartsWith3 setting.
    for count in range(5):
        inputs = [dict(control_step=0, sequence=0, command='SetReleaseRate', rate=49),
                  dict(control_step=16, sequence=0, command='Pause'),
                  dict(control_step=16, sequence=1, command='Pause'),
                  dict(control_step=34, sequence=0, command='Resume'),
                  dict(control_step=53, sequence=0, command='SetReleaseRate', rate=98),
                  dict(control_step=54, sequence=0, command='SetReleaseRate', rate=255)]
        name = f'entrances-{count}'
        yield name, base | dict(level=name, entrances=[[i*16, 0] for i in range(count)],
                               faller_three=False, inputs=inputs), bytes(96*160)
    yield 'zero-total', base | dict(level='zero-total', total=0), bytes(96*160)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference', help='compiled Pascal harness; omitted in offline CI')
    parser.add_argument('--record', type=Path, help='create a new golden file from actual reference execution')
    args = parser.parse_args()
    if args.record and not args.reference:
        parser.error('--record requires --reference')
    expected = json.loads(GOLDEN.read_text()) if not args.record else None
    if expected:
        for field, path in [('adapter_sha256', ROOT/'tools/lemmix/harness.pas'),
                            ('builder_sha256', ROOT/'tools/lemmix/build_reference.py')]:
            if expected['provenance'][field] != hashlib.sha256(path.read_bytes()).hexdigest():
                raise AssertionError('reference adapter changed; execute and record the reference again')
    rows = []
    reference_provenance = None
    with tempfile.TemporaryDirectory(prefix='lemmings-replay-') as directory:
        mask_path = Path(directory)/'terrain.bin'
        for name, config, mask in cases():
            mask_path.write_bytes(mask)
            reference = run_engine(config, mask_path, args.reference) if args.reference else None
            if reference:
                reference_provenance = reference['header']['provenance']
            signature = None
            for pattern in ['single', 'burst', 'jitter']:
                rust = run_engine(config, mask_path, 'rust', pattern)
                current = digest(rust['snapshots'])
                if reference and compare(reference, rust)['status'] != 'MATCH_SUPPORTED':
                    raise AssertionError(f'{name}/{pattern}: {compare(reference, rust)}')
                if signature is not None and signature != current:
                    raise AssertionError(f'{name}: render-frequency dependent')
                signature = current
            rows.append(dict(case=name, config_sha256=digest(config), mask_sha256=hashlib.sha256(mask).hexdigest(),
                             snapshots_sha256=signature, snapshots=config['steps']+1))
    if args.record:
        record = dict(format='g05-reference-golden-v1', provenance=reference_provenance, cases=rows,
                      signature=digest(rows))
        with args.record.open('xb') as output:
            output.write(canonical(record))
    elif expected['cases'] != rows or expected['signature'] != digest(rows):
        raise AssertionError('Rust snapshots differ from recorded Pascal reference execution')
    print(json.dumps(dict(cases=len(rows), replays=len(rows)*3, snapshots=sum(r['snapshots'] for r in rows)*3,
                          signature=digest(rows), reference_executed=bool(args.reference))))


if __name__ == '__main__':
    main()
