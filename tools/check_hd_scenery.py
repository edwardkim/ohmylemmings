#!/usr/bin/env python3
"""Compare actual SDL scenery before/after without changing game contracts.

Includes five ordinary themes, four special scenes and saved Builder/Mayhem08
solutions. Original-derived evidence is restricted to ignored assets/local.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import struct
import subprocess
import tempfile
import zlib

ROOT = Path(__file__).resolve().parents[1]


def framebuffer_png(ppm):
    magic, dimensions, maximum, rgb = ppm.split(b'\n', 3)
    width, height = map(int, dimensions.split())
    if magic != b'P6' or maximum != b'255' or len(rgb) != width*height*3:
        raise ValueError('invalid native framebuffer')
    def chunk(kind, data):
        return struct.pack('>I', len(data))+kind+data+struct.pack('>I', zlib.crc32(kind+data))
    rows = b''.join(b'\0'+rgb[y*width*3:(y+1)*width*3] for y in range(height))
    return (b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0))
            +chunk(b'IDAT', zlib.compress(rows))+chunk(b'IEND', b''))


def solution(manifest, identity):
    data = json.loads((ROOT/'assets/manifests'/manifest).read_text())
    def walk(value):
        if isinstance(value, dict):
            if value.get('id') == identity and value.get('phase') == 'Won' and 'input_text' in value:
                return value
            for child in value.values():
                found = walk(child)
                if found:
                    return found
        elif isinstance(value, list):
            for child in value:
                found = walk(child)
                if found:
                    return found
        return None
    result = walk(data)
    if result is None:
        raise ValueError('saved solution missing: '+identity)
    if hashlib.sha256(result['input_text'].encode()).hexdigest() != result['input_sha256']:
        raise ValueError('saved solution hash differs: '+identity)
    return result


def check(before, after, assets, pack, output):
    if not output.is_relative_to(ROOT/'assets/local') or output.exists():
        raise ValueError('use a new assets/local output directory')
    output.mkdir(parents=True)
    identities = ['fun-01', 'fun-02', 'fun-03', 'fun-04', 'fun-06', 'fun-07',
                  'fun-22', 'tricky-14', 'taxing-15', 'mayhem-08', 'mayhem-22']
    cases = []
    for identity in identities:
        rank, number = identity.split('-')
        record = None
        if identity in ['fun-07', 'mayhem-08']:
            record = solution('solution_replay.json' if rank == 'fun' else 'solution_replay_mayhem.json', identity)
        replay = output/(identity+'.txt')
        replay.write_text(record['input_text'] if record else '60 0 AssignDigger 0\n' if identity == 'fun-01' else '')
        steps = record['steps'] if record else 1000 if identity == 'fun-01' else 120
        with tempfile.TemporaryDirectory(prefix=identity+'-', dir=output) as temporary:
            folders = []
            for name, binary in [('before', before), ('after', after)]:
                folder = Path(temporary)/name
                command = [str(binary), '--assets', str(assets), '--difficulty', rank, '--level', str(int(number)),
                           '--hd', '--hd-pack', str(pack), '--steps', str(steps), '--fast',
                           '--render-every', '200', '--capture', str(folder), '--replay', str(replay)]
                process = subprocess.run(command, env=dict(os.environ, SDL_VIDEODRIVER='dummy'),
                                         cwd=ROOT, capture_output=True, text=True, timeout=180)
                (output/f'{identity}-{name}.log').write_text(process.stdout+process.stderr)
                if process.returncode:
                    raise RuntimeError(f'Native {name} failed: {identity}')
                frames = sorted(folder.glob('*.ppm'))
                for label, frame in [('initial', frames[0]), ('detail', frames[min(2, len(frames)-1)])]:
                    (output/f'{identity}-{name}-{label}.png').write_bytes(framebuffer_png(frame.read_bytes()))
                folders.append(folder)
            left, right = folders
            protocol = sorted(p.name for p in left.iterdir() if p.suffix in {'.jsonl', '.bin', '.rgb', '.txt'})
            if set(protocol) != {p.name for p in right.iterdir() if p.suffix in {'.jsonl', '.bin', '.rgb', '.txt'}}:
                raise ValueError('capture protocol inventory differs: '+identity)
            for name in protocol:
                if (left/name).read_bytes() != (right/name).read_bytes():
                    raise ValueError(f'game contract changed: {identity}/{name}')
            states = (right/'full.jsonl').read_text().splitlines()
            final = json.loads(states[-1])['completion']
            if (record or identity == 'fun-01') and final['phase'] != 'Won':
                raise ValueError('saved solution no longer wins: '+identity)
            row = dict(id=identity, states=len(states), equal_protocol_files=len(protocol),
                       phase=final['phase'], rescued=final['rescued'],
                       bridge_pixels=sum((right/'brick-final.bin').read_bytes()),
                       initial_pixels_changed=(left/'frame-000000.ppm').read_bytes() != (right/'frame-000000.ppm').read_bytes())
            cases.append(row)
            print(json.dumps(row), flush=True)
    result = dict(status='PASS', cases=cases, same_game_inputs_masks_and_original_rgb=True,
                  scope='Actual SDL five-theme/four-special pairs and three saved solutions; not all-level completion or FPS qualification')
    (output/'summary.json').write_text(json.dumps(result, indent=2)+'\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['before', 'after', 'assets', 'pack', 'output']:
        parser.add_argument('--'+name, type=Path, required=True)
    args = parser.parse_args()
    check(*(getattr(args, n).resolve() for n in ['before', 'after', 'assets', 'pack', 'output']))


if __name__ == '__main__':
    main()
