"""Functional checks of the installed original game, without an asset path argument."""
import json
import os
from pathlib import Path
import re
import shutil
import subprocess

from tools.release_package import require, sha


def check(directory, evidence):
    binary = directory / 'lemmings-desktop'
    foreign = evidence / '다른 작업 folder'
    foreign.mkdir()
    env = dict(os.environ, SDL_VIDEODRIVER='dummy', SDL_AUDIODRIVER='dummy', SDL_RENDER_DRIVER='software')
    from tools.check_hd_package import check as check_hd
    hd = check_hd(binary, directory, foreign, env, evidence)

    def execute(label, arguments, success=True):
        result = subprocess.run([str(binary), '--original', *map(str, arguments)], cwd=foreign, env=env,
                                capture_output=True, text=True, timeout=90)
        (evidence / (label + '.log')).write_text(result.stdout + result.stderr)
        require((result.returncode == 0) == success, f'{label}: {result.stderr}')
        return result

    initial = evidence / 'original-first'
    first = json.loads(execute('first', ['--steps', 2, '--fast', '--capture', initial]).stdout)
    config = json.loads((initial / 'config.json').read_text())
    require(not first['synthetic'] and config['difficulty'] == 'fun' and config['level'] == 1,
            'Default launch did not load original Fun01')
    from tools.check_desktop import verify_pixels
    verify_pixels(initial)
    logical = (initial / 'frame-000000.rgb').read_bytes()
    require(sha(logical) == '1539ca2afdcc9cfbe4fa5de51036c45b881014ec14a0ab030c191ca41f988c4d',
            'Packaged original Fun01 pixels differ from Windows/Mac baseline')
    shutil.copyfile(initial / 'frame-000000.ppm', evidence / 'original-fun01.ppm')
    levels = 0
    for rank in ['fun', 'tricky', 'taxing', 'mayhem']:
        for number in range(1, 31):
            result = execute(f'level-{rank}-{number:02}', ['--difficulty', rank, '--level', number, '--steps', 1, '--fast'])
            require(not json.loads(result.stdout)['synthetic'], 'Synthetic level fallback')
            levels += 1
    replay = evidence / 'solution.txt'
    replay.write_text('60 0 AssignDigger 0\n')
    transitions = []
    from tools.check_completion import collect
    for action, target in [('restart', 1), ('continue', 2)]:
        capture = evidence / ('original-' + action)
        execute(action, ['--steps', 1000, '--fast', '--render-every', 1000, '--replay', replay,
                         '--result-actions', action, '--capture', capture])
        rows = [json.loads(row) for row in (capture / 'transitions.jsonl').read_text().splitlines()]
        require(len(rows) == 1 and rows[0]['target'] == target, 'Wrong result transition')
        require(collect(capture)[-1]['completion']['phase'] == 'Won', 'Fun01 solution failed')
        attempt = capture / 'attempt-0001'
        require(not (attempt / 'inputs.txt').read_text(), 'Replay leaked into new attempt')
        fresh = evidence / f'fresh-{target}'
        execute(f'fresh-{target}', ['--level', target, '--steps', 1000, '--fast', '--render-every', 1000, '--capture', fresh])
        require(collect(attempt) == collect(fresh), 'Restart/continue differs from a fresh level')
        require((attempt / 'terrain-initial.bin').read_bytes() == (fresh / 'terrain-initial.bin').read_bytes(), 'Terrain differs')
        transitions.append(dict(action=action, target=target, status='PASS'))
    result = execute('original-audio', ['--steps', 60])
    match = re.search(r'LEMMINGS_AUDIO submitted=(\d+) nonzero=(\d+) active=true original=true', result.stderr)
    require(match and int(match[1]) > 0 and int(match[2]) > 0, 'Original SDL PCM missing')
    missing = directory / 'data/MAIN.DAT'
    missing.rename(missing.with_suffix('.hidden'))
    try:
        result = execute('missing-data', ['--steps', 1, '--fast'], success=False)
        require('MAIN.DAT' in result.stderr, 'Missing data error does not name the file')
    finally:
        missing.with_suffix('.hidden').rename(missing)
    dependencies = subprocess.check_output(['ldd', str(binary)], text=True)
    require('not found' not in dependencies and 'libSDL2' in dependencies, 'Unresolved Linux runtime')
    (evidence / 'dependencies.log').write_text(dependencies)
    return dict(status='PASS', hd=hd, original_levels_loaded=levels, first_frame_sha256=sha(logical),
                foreign_cwd=True, unicode_space_install=True, transitions=transitions,
                audio=dict(submitted=int(match[1]), nonzero=int(match[2]), driver='dummy'),
                scope='Extracted full game; SDL dummy video/audio and pixel readback; no physical audibility or performance qualification')
