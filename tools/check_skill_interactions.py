#!/usr/bin/env python3
"""Compare the complete assignment matrix and mixed SDL replays to Pascal."""
import argparse
from contextlib import nullcontext
import hashlib
import itertools
import json
from pathlib import Path
import re
import subprocess
import tempfile

from check_nuke import ADAPTERS as NUKE_ADAPTERS, reference_run, replay_text, run_native
from check_basher import synthetic_masks as bashing_masks
from check_miner import synthetic_masks as mining_masks
from state_replay import ROOT, digest

ACTIONS = ['Walking', 'Jumping', 'Falling', 'Splatting', 'Digging', 'Bashing',
           'Mining', 'Building', 'Shrugging', 'Blocking', 'OhNo', 'Exploding',
           'Climbing', 'Hoisting', 'Floating']
SKILLS = ['Climber', 'Floater', 'Bomber', 'Blocker', 'Builder', 'Basher', 'Miner', 'Digger']
VARIANTS = ['normal', 'empty', 'athlete', 'armed', 'steel_below', 'steel_front',
            'oneway_left', 'oneway_right', 'overlap', 'head_above', 'head_edge']
GOLDEN = ROOT / 'tools/fixtures/skill_interactions_golden.json'
ADAPTERS = ['build_interactions_reference.py', 'assignment_probe.pas'] + NUKE_ADAPTERS
CONFIG = dict(width=640, height=160, minutes=5, total=10, rate=50,
              entrances=[[40, 40]], pause_glitch=True, faller_three=True, steps=420)
CASES = {
    'all_eight_same_boundary': [(80, i, 'Assign' + skill, 0) for i, skill in enumerate(
        ['Climber', 'Floater', 'Bomber', 'Builder', 'Basher', 'Miner', 'Digger', 'Blocker'])],
    'blocker_rejects_replacements': [(80, i, 'Assign' + skill, 0) for i, skill in enumerate(
        ['Blocker', 'Builder', 'Basher', 'Miner', 'Digger', 'Climber', 'Floater', 'Bomber'])],
    'athlete_timer_job_chain': [(80, i, 'Assign' + skill, 0) for i, skill in enumerate(
        ['Climber', 'Floater', 'Bomber', 'Builder', 'Digger', 'Basher', 'Miner', 'Builder'])],
    'paused_job_chain': [(80, 0, 'Pause', 0), (80, 1, 'AssignBuilder', 0),
                         (80, 2, 'AssignFloater', 0), (80, 3, 'AssignBomber', 0),
                         (90, 0, 'AssignDigger', 0), (100, 0, 'Resume', 0)],
    'digger_frees_blocker': [(81, 0, 'AssignBlocker', 0), (105, 0, 'AssignDigger', 1)],
    'bombed_blocker_and_follower': [(80, 0, 'AssignBlocker', 0), (105, 0, 'AssignBomber', 0),
                                   (130, 0, 'AssignBuilder', 2)],
    'builder_support_and_miner': [(80, 0, 'AssignBuilder', 0), (105, 0, 'AssignFloater', 1),
                                 (106, 0, 'AssignBuilder', 1), (120, 0, 'AssignMiner', 0),
                                 (150, 0, 'AssignDigger', 2)],
    'two_bombers_different_jobs': [(80, 0, 'AssignBuilder', 0), (81, 0, 'AssignBomber', 0),
                                   (105, 0, 'AssignDigger', 1), (106, 0, 'AssignBomber', 1)],
    'permanent_duplicates': [(80, i, 'Assign' + skill, 0) for i, skill in enumerate(
        ['Climber', 'Floater', 'Climber', 'Floater', 'Bomber', 'Bomber', 'Basher', 'Basher'])],
    'shrugger_climber_once': [(80, 0, 'AssignBuilder', 0), (275, 0, 'AssignClimber', 0),
                             (276, 0, 'AssignFloater', 0), (277, 0, 'AssignBomber', 0)],
    'shrugger_once_replaced': [(80, 0, 'AssignBuilder', 0), (275, 0, 'AssignClimber', 0),
                              (275, 1, 'AssignBuilder', 0)],
    'climbing_with_bomber': [(80, 0, 'AssignClimber', 0), (80, 1, 'AssignFloater', 0),
                             (160, 0, 'AssignBomber', 0)],
    'blocker_and_opposing_jobs': [(80, 0, 'AssignBlocker', 0), (105, 0, 'AssignMiner', 1),
                                 (130, 0, 'AssignBasher', 2), (155, 0, 'AssignDigger', 3)],
    'mixed_nuke_pause': [(80, 0, 'AssignClimber', 0), (80, 1, 'AssignFloater', 0),
                         (80, 2, 'AssignBomber', 0), (80, 3, 'AssignBuilder', 0),
                         (105, 0, 'AssignBlocker', 1), (120, 0, 'Pause', 0),
                         (120, 1, 'Nuke', 0), (130, 0, 'Resume', 0), (130, 1, 'Nuke', 0)],
}


def matrix_keys():
    return list(itertools.product(ACTIONS, SKILLS, [-1, 1], VARIANTS))


def matrix_digest(rows):
    expected = matrix_keys()
    if len(rows) != len(expected):
        raise ValueError('assignment matrix length differs')
    booleans = ['accepted', 'end', 'new_digger', 'climber', 'floater', 'walking_once']
    integers = ['remaining', 'y', 'frame', 'fallen', 'bricks', 'float_index', 'timer']
    fields = set(['initial', 'skill', 'direction', 'variant', 'action', 'map_hash'] + booleans + integers)
    for key, row in zip(expected, rows):
        if set(row) != fields or tuple(row[k] for k in ['initial', 'skill', 'direction', 'variant']) != key:
            raise ValueError('assignment matrix keys/schema differ')
        if type(row['direction']) is not int:
            raise ValueError('assignment matrix direction type differs')
        if any(type(row[k]) is not bool for k in booleans):
            raise ValueError('assignment matrix boolean type differs')
        if any(type(row[k]) is not int or (k != 'y' and row[k] < 0) for k in integers):
            raise ValueError('assignment matrix integer type/range differs')
        if (row['action'] not in ACTIONS or type(row['map_hash']) is not str
                or not re.fullmatch('[0-9a-f]{16}', row['map_hash'])):
            raise ValueError('assignment matrix action/hash differs')
    return digest(rows)


def compare_states(actual, expected, name):
    if len(actual) != len(expected):
        raise AssertionError(f'{name} length differs: {len(actual)} != {len(expected)}')
    for step, (a, b) in enumerate(zip(actual, expected)):
        if a != b:
            raise AssertionError(f'{name} first difference at boundary {step}: {a} != {b}')


def assert_exercised(name, states):
    events = {e for s in states for e in s['events']}
    required = {
        'paused_job_chain': {'Building', 'Digging', 'Exploding'},
        'bombed_blocker_and_follower': {'Blocking', 'Building', 'Exploding'},
        'builder_support_and_miner': {'Building', 'Mining', 'Digging'},
        'two_bombers_different_jobs': {'Building', 'Digging', 'Exploding'},
        'climbing_with_bomber': {'Climbing', 'Exploding'},
        'blocker_and_opposing_jobs': {'Blocking', 'Mining', 'Bashing', 'Digging'},
    }.get(name, set())
    actions = {a['action'] for s in states for a in s['actors']}
    if not required <= actions:
        raise AssertionError(f'{name} did not exercise actions: {required - actions}')
    if name == 'all_eight_same_boundary':
        if not all(f'Input:80:{i}:true' in events for i in range(8)):
            raise AssertionError('eight-skill chain did not accept every assignment')
    if name == 'blocker_rejects_replacements':
        if not all(f'Input:80:{i}:false' in events for i in range(1, 7)):
            raise AssertionError('Blocker replacement rejection not exercised')
    if name == 'permanent_duplicates':
        if not all(f'Input:80:{i}:false' in events for i in [2, 3, 5, 7]):
            raise AssertionError('duplicate permanent/timer/job rejection not exercised')
    if name == 'shrugger_climber_once':
        if not any(a[2] for s in states for a in s['climber']['actors']):
            raise AssertionError('shrugger animation bug not exercised')
    if name == 'digger_frees_blocker':
        fields = [s['blocker']['fields'] for s in states]
        if not any(fields) or not any(not f for f in fields[110:]):
            raise AssertionError('Digger did not release Blocker field')
    if name == 'mixed_nuke_pause':
        if 'Input:120:1:false' not in events or not states[-1]['nuke']['is_nuked']:
            raise AssertionError('paused Nuke rejection/resume not exercised')


def check(args, root):
    hashes = {name: hashlib.sha256((ROOT / 'tools/lemmix' / name).read_bytes()).hexdigest() for name in ADAPTERS}
    frozen = None if args.record else json.loads(GOLDEN.read_text())
    if frozen and (frozen['adapter_sha256'] != hashes or frozen['config'] != CONFIG or set(frozen['cases']) != set(CASES)):
        raise ValueError('reference adapter/config/cases changed; execute reference again')
    native = subprocess.run(['cargo', 'run', '--quiet', '--locked', '--offline', '-p',
                             'lemmings-core', '--example', 'skill_matrix'],
                            cwd=ROOT, capture_output=True, text=True, check=True)
    matrix = [json.loads(line) for line in native.stdout.splitlines()]
    matrix_hash = matrix_digest(matrix)
    (root / 'native-matrix.jsonl').write_text(native.stdout)
    terrain = bytes(int((y >= 100 and x < 500) or (60 <= y < 100 and 160 <= x < 170))
                    for y in range(160) for x in range(640))
    paths = []
    for name, data in [('mask', terrain), ('objects', bytes(len(terrain))),
                       ('bash-masks', bashing_masks()), ('mine-masks', mining_masks()),
                       ('raw-map', bytes([128]) * (415 * 47)), ('explosion-mask', bytes([255]) * 44)]:
        path = root / name
        path.write_bytes(data)
        paths.append(path)
    if args.reference:
        script = '640 160 5 10 50 1 1 1 0\n10 10 10 10 10 10 10 10 1 1\n40 40\n0\n'
        result = subprocess.run([str(args.reference.resolve()), *map(str, paths), '--assignments'],
                                input=script, capture_output=True, text=True, check=True)
        expected = [json.loads(line) for line in result.stdout.splitlines()]
        matrix_digest(expected)
        compare_states(matrix, expected, 'assignment matrix')
        (root / 'reference-matrix.jsonl').write_text(result.stdout)
    if frozen and matrix_hash != frozen['matrix_sha256']:
        raise AssertionError('recorded assignment matrix differs')
    rows = {}
    snapshots = captures = 0
    for name, inputs in CASES.items():
        if frozen and [list(i) for i in inputs] != frozen['cases'][name]['inputs']:
            raise ValueError('case inputs changed: ' + name)
        replay = root / (name + '.txt')
        replay.write_text(replay_text(inputs))
        expected = reference_run(args.reference, CONFIG, *paths, inputs) if args.reference else None
        if expected is not None:
            (root / (name + '-reference.json')).write_text(json.dumps(expected))
        for render in [1, 7]:
            capture_context = nullcontext(root) if args.evidence else tempfile.TemporaryDirectory(dir=root)
            with capture_context as capture_root:
                states, count = run_native(args.binary, Path(capture_root) / (name + '-' + str(render)), replay, render)
            if expected is not None:
                compare_states(states, expected, name)
            assert_exercised(name, states)
            signature = digest(states)
            if frozen and signature != frozen['cases'][name]['snapshots_sha256']:
                raise AssertionError('recorded reference differs: ' + name)
            if name in rows and signature != rows[name]['snapshots_sha256']:
                raise AssertionError('render-dependent state: ' + name)
            rows[name] = dict(snapshots_sha256=signature, steps=len(states), inputs=inputs)
            snapshots += len(states)
            captures += count
    if args.record:
        provenance = json.loads(args.reference.with_name('reference.provenance.json').read_text())
        GOLDEN.write_text(json.dumps(dict(format='g07-interactions-v1', reference=provenance,
                                         adapter_sha256=hashes, config=CONFIG, matrix_sha256=matrix_hash,
                                         cases=rows), indent=2) + '\n')
    summary = dict(status='PASS', matrix_cases=len(matrix), cases=len(rows),
                          replays=len(rows) * 2, snapshots=snapshots, pixel_exact_captures=captures,
                          matrix_sha256=matrix_hash, reference_executed=args.reference is not None,
                          scope='single stable-ID assignments and mixed synthetic job/terrain/object/timer replay; not full DOS')
    (root / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(summary))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/release/lemmings-desktop')
    parser.add_argument('--reference', type=Path)
    parser.add_argument('--record', action='store_true')
    parser.add_argument('--evidence', type=Path)
    args = parser.parse_args()
    if args.record and not args.reference:
        raise ValueError('record requires actual reference execution')
    if args.evidence:
        root = args.evidence.resolve()
        if not root.is_relative_to(ROOT / 'assets/local'):
            raise ValueError('local evidence required')
        root.mkdir()
        check(args, root)
    else:
        with tempfile.TemporaryDirectory(prefix='lemmings-interactions-') as directory:
            check(args, Path(directory))


if __name__ == '__main__':
    main()
