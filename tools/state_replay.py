#!/usr/bin/env python3
"""Headless G05 traces. Standard library only; never certifies full DOS behavior."""
import argparse
import base64
from functools import lru_cache
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
FORMAT = 'dos-compat-trace-v1'
SCOPE = 'g05-basic-v1'
UNVERIFIED = ['skills', 'abilities', 'objects', 'rescued', 'result', 'nuke', 'audio', 'music', 'DOS execution']
STATE_INTS = ['control_step', 'iteration', 'seconds_remaining', 'clock_frame',
              'glitch_pause_iterations', 'release_rate', 'spawn_countdown', 'released', 'alive', 'removed']
ACTOR_INTS = ['id', 'born_iteration', 'x', 'y', 'direction', 'frame', 'fallen']


def canonical(value):
    def check(v):
        if v is None or type(v) in (bool, int, str):
            return
        if type(v) is list:
            for item in v:
                check(item)
            return
        if type(v) is dict and all(type(k) is str for k in v):
            for item in v.values():
                check(item)
            return
        raise ValueError('canonical state permits only integers, booleans, strings, arrays, objects and null')
    check(value)
    return json.dumps(value, sort_keys=True, ensure_ascii=True, separators=(',', ':')).encode('ascii')


def digest(value):
    return hashlib.sha256(canonical(value)).hexdigest()


def integer(value, lo, hi, name):
    if type(value) is not int or not lo <= value <= hi:
        raise ValueError(f'{name}: expected integer {lo}..{hi}')


def validate_config(c):
    required = {'width', 'height', 'minutes', 'total', 'rate', 'entrances', 'pause_glitch',
                'faller_three', 'steps', 'inputs', 'level'}
    if type(c) is not dict or set(c) != required:
        raise ValueError('config fields differ from g05-basic-v1')
    for key, lo, hi in [('width', 1, 10000), ('height', 1, 10000), ('minutes', 0, 65535),
                        ('total', 0, 10000), ('rate', 0, 99), ('steps', 0, 100000)]:
        integer(c[key], lo, hi, key)
    if c['width'] * c['height'] > 10000000:
        raise ValueError('mask exceeds ten million cells')
    if type(c['level']) is not str or not c['level']:
        raise ValueError('level must identify the fixture')
    for key in ['pause_glitch', 'faller_three']:
        if type(c[key]) is not bool:
            raise ValueError(f'{key}: expected boolean')
    if type(c['entrances']) is not list or len(c['entrances']) > 4:
        raise ValueError('entrances: expected zero to four points')
    for point in c['entrances']:
        if type(point) is not list or len(point) != 2:
            raise ValueError('entrance: expected [x,y]')
        for coord in point:
            integer(coord, -10000, 10000, 'entrance coordinate')
    if type(c['inputs']) is not list:
        raise ValueError('inputs: expected list')
    previous = (-1, -1)
    for item in c['inputs']:
        if type(item) is not dict:
            raise ValueError('input: expected object')
        command = item.get('command')
        keys = {'control_step', 'sequence', 'command'}
        if command == 'SetReleaseRate':
            keys.add('rate')
            integer(item.get('rate'), 0, 255, 'input rate')
        elif command not in ['Pause', 'Resume']:
            raise ValueError(f'unsupported command: {command}')
        if set(item) != keys:
            raise ValueError('input argument fields differ')
        integer(item['control_step'], 0, c['steps'] - 1, 'input boundary')
        integer(item['sequence'], 0, 2**32 - 1, 'input sequence')
        key = item['control_step'], item['sequence']
        if key <= previous:
            raise ValueError('inputs must be strictly ordered by boundary and sequence')
        previous = key
    return c


def make_header(config, mask, provenance, input_files=None):
    validate_config(config)
    if len(mask) != config['width'] * config['height'] or any(v > 1 for v in mask):
        raise ValueError('mask must be exact row-major binary cells')
    if type(provenance) is not dict or type(provenance.get('engine')) is not str or not provenance['engine']:
        raise ValueError('engine provenance required')
    if input_files is None:
        input_files = []
    if type(input_files) is not list:
        raise ValueError('input_files must be a manifest list')
    previous = ''
    for entry in input_files:
        if type(entry) is not dict or set(entry) != {'name', 'size', 'sha256'}:
            raise ValueError('input file requires name, size and sha256')
        if type(entry['name']) is not str or entry['name'] <= previous:
            raise ValueError('input file names must be strictly ordered')
        integer(entry['size'], 0, 2**64-1, 'input file size')
        if type(entry['sha256']) is not str or len(entry['sha256']) != 64 or any(v not in '0123456789abcdef' for v in entry['sha256']):
            raise ValueError('invalid input file hash')
        previous = entry['name']
    return dict(profile='dos-original-vga', scope=SCOPE, coordinates='foot; entrance stored x-16',
                ticks_per_second=17, config=config, input_sha256=digest(config['inputs']),
                input_files=input_files,
                terrain=dict(width=config['width'], height=config['height'], sha256=hashlib.sha256(mask).hexdigest(),
                             cells_base64=base64.b64encode(mask).decode()),
                provenance=provenance, observation_kind='structured', unverified=UNVERIFIED)


def make_trace(header, snapshots):
    trace = dict(format=FORMAT, header=header, snapshots=snapshots)
    validate_trace(trace)
    return trace


def validate_trace(t):
    if set(t) != {'format', 'header', 'snapshots'} or t['format'] != FORMAT:
        raise ValueError('unknown trace format or fields')
    h = t['header']
    mask = base64.b64decode(h['terrain']['cells_base64'], validate=True)
    if h != make_header(h['config'], mask, h['provenance'], h['input_files']):
        raise ValueError('header identity, hash or scope invalid')
    c = h['config']
    if type(t['snapshots']) is not list or len(t['snapshots']) != c['steps'] + 1:
        raise ValueError('missing initial or post-boundary snapshots')
    previous_iteration = 0
    previous_released = 0
    for index, s in enumerate(t['snapshots']):
        expected_keys = set(STATE_INTS + ['paused', 'entrances_open', 'actors', 'events'])
        if type(s) is not dict or set(s) != expected_keys:
            raise ValueError('missing or unknown snapshot fields')
        for k in STATE_INTS:
            integer(s[k], 0, 2**64 - 1, k)
        for k, hi in [('clock_frame', 16), ('glitch_pause_iterations', 34), ('release_rate', 99), ('spawn_countdown', 53)]:
            integer(s[k], 0, hi, k)
        if type(s['paused']) is not bool or type(s['entrances_open']) is not bool:
            raise ValueError('snapshot flags must be booleans')
        expected_iteration = 0 if index == 0 else previous_iteration + (not s['paused'])
        if s['control_step'] != index or s['iteration'] != expected_iteration:
            raise ValueError('noncontiguous control/iteration counters')
        if not previous_released <= s['released'] <= c['total'] or s['released'] != len(s['actors']):
            raise ValueError('invalid birth count or missing actor records')
        for actor_id, a in enumerate(s['actors']):
            if set(a) != set(ACTOR_INTS + ['action', 'end_of_animation', 'removed']):
                raise ValueError('missing or unknown actor fields')
            for k in ACTOR_INTS:
                integer(a[k], -(2**31) if k in ['x', 'y', 'direction'] else 0, 2**64 - 1, k)
            if a['id'] != actor_id or a['born_iteration'] > s['iteration'] or a['direction'] not in [-1, 0, 1]:
                raise ValueError('invalid actor identity or direction')
            frames = {'Walking': 8, 'Jumping': 1, 'Falling': 4, 'Splatting': 16}
            if a['action'] not in frames or a['frame'] >= frames[a['action']]:
                raise ValueError('invalid action/frame')
            if type(a['end_of_animation']) is not bool or a['removed'] not in [None, 'BottomBoundary', 'SplattingFinished']:
                raise ValueError('invalid actor flags/removal')
            if (a['direction'] == 0) != (a['action'] == 'Splatting') or (a['action'] != 'Falling' and a['fallen'] != 0):
                raise ValueError('inconsistent actor direction or fall counter')
        removed = sum(a['removed'] is not None for a in s['actors'])
        if s['removed'] != removed or s['alive'] + removed != s['released']:
            raise ValueError('invalid actor totals')
        if type(s['events']) is not list or any(type(e) is not str for e in s['events']):
            raise ValueError('events must be ordered strings')
        previous_iteration, previous_released = s['iteration'], s['released']
    canonical(t)


def first_difference(a, b, path=''):
    if type(a) is not type(b):
        return path, a, b
    if type(a) is dict:
        for key in sorted(set(a) | set(b)):
            if key not in a or key not in b:
                return path + '/' + key, a.get(key), b.get(key)
            diff = first_difference(a[key], b[key], path + '/' + key)
            if diff:
                return diff
    elif type(a) is list:
        for i, (left, right) in enumerate(zip(a, b)):
            diff = first_difference(left, right, f'{path}/{i}')
            if diff:
                return diff
        if len(a) != len(b):
            return path + '/length', len(a), len(b)
    elif a != b:
        return path, a, b
    return None


def compare(reference, candidate):
    try:
        validate_trace(reference)
        validate_trace(candidate)
    except (ValueError, KeyError, TypeError, OverflowError) as error:
        return dict(status='UNVERIFIED', reason=str(error))
    a, b = reference['header'], candidate['header']
    if a['terrain'] != b['terrain']:
        ma, mb = (base64.b64decode(h['terrain']['cells_base64']) for h in [a, b])
        coordinate = None
        if (a['terrain']['width'], a['terrain']['height']) == (b['terrain']['width'], b['terrain']['height']):
            i = next(i for i, (x, y) in enumerate(zip(ma, mb)) if x != y)
            coordinate = [i % a['terrain']['width'], i // a['terrain']['width']]
        return dict(status='DIFFERENT', control_step=0, field='/header/terrain', coordinate=coordinate,
                    reference_mask_base64=a['terrain']['cells_base64'], candidate_mask_base64=b['terrain']['cells_base64'])
    for key in ['config', 'input_sha256', 'input_files']:
        if a[key] != b[key]:
            return dict(status='UNVERIFIED', reason=f'different {key}; initial state and input must match')
    diff = first_difference(reference['snapshots'], candidate['snapshots'], '/snapshots')
    if diff:
        path, left, right = diff
        index = int(path.split('/')[2])
        return dict(status='DIFFERENT', control_step=index, field=path, reference=left, candidate=right,
                    neighboring_snapshots=dict(reference=reference['snapshots'][max(0, index-1):index+2],
                                               candidate=candidate['snapshots'][max(0, index-1):index+2]),
                    terrain=a['terrain'])
    return dict(status='MATCH_SUPPORTED', snapshots=len(reference['snapshots']), unverified=UNVERIFIED)


def script(config, mask_path):
    c = validate_config(config)
    lines = [' '.join(map(str, [c['width'], c['height'], c['minutes'], c['total'], c['rate'],
                              len(c['entrances']), int(c['pause_glitch']), int(c['faller_three']), c['steps']]))]
    lines.extend(f'{x} {y}' for x, y in c['entrances'])
    lines.append(str(len(c['inputs'])))
    for i in c['inputs']:
        code = {'Pause': 0, 'Resume': 1, 'SetReleaseRate': 2}[i['command']]
        lines.append(f"{i['control_step']} {i['sequence']} {code} {i.get('rate', 0)}")
    return '\n'.join(lines) + '\n'


@lru_cache(maxsize=1)
def rust_version():
    return subprocess.check_output(['rustc', '--version'], text=True, cwd=ROOT).strip()


def run_engine(config, mask_path, engine, pattern='single', input_files=None):
    if engine == 'rust':
        command = ['cargo', 'run', '--quiet', '--locked', '--offline', '-p', 'lemmings-core',
                   '--example', 'trace_replay', '--', str(mask_path), pattern]
        provenance = dict(engine='rust', adapter='trace_replay', render_pattern=pattern,
                          compiler=rust_version(), build_profile='dev', cargo_flags=['--locked', '--offline'],
                          source_sha256=digest({str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
                                                for p in sorted((ROOT/'crates/lemmings-core/src').rglob('*.rs'))} |
                                               {'example': hashlib.sha256((ROOT/'crates/lemmings-core/examples/trace_replay.rs').read_bytes()).hexdigest()}))
    else:
        command = [str(Path(engine).resolve()), str(mask_path)]
        provenance = json.loads(Path(str(engine) + '.provenance.json').read_text())
        provenance['executable_sha256'] = hashlib.sha256(Path(engine).read_bytes()).hexdigest()
    result = subprocess.run(command, input=script(config, mask_path), text=True, capture_output=True, cwd=ROOT, check=True)
    return make_trace(make_header(config, Path(mask_path).read_bytes(), provenance, input_files),
                      [json.loads(line) for line in result.stdout.splitlines()])


def verified_assets(directory):
    manifest = json.loads((ROOT/'assets/manifests/dos_original.json').read_text())
    result = []
    for entry in sorted(manifest['files'], key=lambda e: e['name']):
        data = (directory/entry['name']).read_bytes()
        if len(data) != entry['size'] or hashlib.sha256(data).hexdigest() != entry['sha256']:
            raise ValueError(f"original input mismatch: {entry['name']}")
        result.append({k: entry[k] for k in ['name', 'size', 'sha256']})
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='action', required=True)
    run = sub.add_parser('run')
    run.add_argument('config', type=Path)
    run.add_argument('mask', type=Path)
    run.add_argument('output', type=Path)
    run.add_argument('--engine', default='rust')
    run.add_argument('--pattern', choices=['single', 'burst', 'jitter'], default='single')
    run.add_argument('--assets', type=Path, help='verify original files and include their hashes in the header')
    diff = sub.add_parser('compare')
    diff.add_argument('reference', type=Path)
    diff.add_argument('candidate', type=Path)
    diff.add_argument('--output', type=Path)
    args = parser.parse_args()
    try:
        if args.action == 'run':
            trace = run_engine(json.loads(args.config.read_text()), args.mask, args.engine, args.pattern,
                               verified_assets(args.assets) if args.assets else None)
            data = canonical(trace)
            with args.output.open('xb') as output:
                output.write(data)
            print(json.dumps(dict(sha256=hashlib.sha256(data).hexdigest(), snapshots=len(trace['snapshots']))))
            return 0
        result = compare(json.loads(args.reference.read_text()), json.loads(args.candidate.read_text()))
        data = canonical(result)
        if args.output:
            with args.output.open('xb') as output:
                output.write(data)
        # Full masks and neighboring states are retained in --output, not dumped to the terminal.
        print(json.dumps({k: v for k, v in result.items() if k not in
                          ['reference_mask_base64', 'candidate_mask_base64', 'neighboring_snapshots', 'terrain']}))
        return {'MATCH_SUPPORTED': 0, 'DIFFERENT': 1, 'UNVERIFIED': 2}[result['status']]
    except (ValueError, KeyError, TypeError, OSError, subprocess.CalledProcessError) as error:
        print(f'UNVERIFIED: {error}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    sys.exit(main())
