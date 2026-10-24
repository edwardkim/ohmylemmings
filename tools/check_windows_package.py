#!/usr/bin/env python3
"""Verify a fresh full-game ZIP on actual Windows, without repository game assets."""
import argparse
import ctypes
from ctypes import wintypes
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
import zipfile

ROOT = Path(__file__).resolve().parents[1]
if __package__ in (None, ''):
    sys.path.insert(0, str(ROOT))
from tools.windows_package import verify
from tools.release_package import require, sha


def save(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')


def window_for(pid):
    user = ctypes.windll.user32
    found = []
    callback_type = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)

    @callback_type
    def visit(window, _):
        owner = wintypes.DWORD()
        user.GetWindowThreadProcessId(window, ctypes.byref(owner))
        if owner.value == pid and user.IsWindowVisible(window):
            title = ctypes.create_unicode_buffer(512)
            user.GetWindowTextW(window, title, len(title))
            if title.value.startswith('Oh My Lemmings'):
                found.append(title.value)
        return True

    user.EnumWindows(visit, 0)
    return found


def check(archive, evidence):
    require(os.name == 'nt', 'Actual Windows is required')
    require(evidence.is_relative_to((ROOT/'assets/local').resolve()), 'Keep captures under assets/local')
    package = verify(archive)
    evidence.mkdir(parents=True)
    installed = evidence/'압축 해제 game folder'
    foreign = evidence/'다른 작업 folder'
    foreign.mkdir()
    with zipfile.ZipFile(archive) as zipped:
        zipped.extractall(installed)  # Inventory, file types and all hashes verified above.
    binary = installed/'OhMyLemmings.exe'
    environment = dict(os.environ, SDL_VIDEODRIVER='dummy', SDL_AUDIODRIVER='dummy',
                       SDL_RENDER_DRIVER='software', LOCALAPPDATA=str(evidence/'user data'))

    def execute(label, arguments, *, success=True, launcher=False):
        command = [str(binary), '--original', *map(str, arguments)]
        if launcher:
            # /S strips the outer quotes, preserving a quoted script path containing spaces.
            command = f'cmd.exe /d /s /c ""{installed / "start-game.cmd"}" ' + subprocess.list2cmdline(list(map(str, arguments))) + '"'
        result = subprocess.run(command, cwd=foreign, env=environment, input='\n',
                                text=True, encoding='utf-8', errors='replace',
                                capture_output=True, timeout=180)
        save(evidence/(label+'.json'), dict(arguments=list(map(str, arguments)),
             exit_code=result.returncode, stdout=result.stdout, stderr=result.stderr))
        require((result.returncode == 0) == success, f'{label}: unexpected exit {result.returncode}: {result.stderr}')
        return result

    from tools.check_local_music import check as check_local_music
    check_local_music(binary, evidence)

    # Genuine zero-argument launch with the normal Windows video driver/window.
    visible_env = dict(environment)
    del visible_env['SDL_VIDEODRIVER']
    with (evidence/'no-arguments.log').open('w', encoding='utf-8') as log:
        process = subprocess.Popen([str(binary)], cwd=foreign, env=visible_env, stdout=log, stderr=log)
        try:
            deadline = time.monotonic()+15
            titles = []
            while process.poll() is None and time.monotonic() < deadline:
                titles = window_for(process.pid)
                if titles:
                    break
                time.sleep(0.1)
            require(titles and process.poll() is None, 'Zero-argument launch did not create a visible game window')
            time.sleep(1)
            require(process.poll() is None, 'Zero-argument game exited immediately')
            save(evidence/'no-arguments.json', dict(status='PASS', visible_window=titles, arguments=[], foreign_cwd=True))
        finally:
            if process.poll() is None:
                process.terminate()
            process.wait(timeout=10)

    initial = evidence/'original-first'
    from tools.check_hd_package import check as check_hd
    check_hd(binary, installed, foreign, environment, evidence)
    result = execute('first', ['--steps', 2, '--fast', '--capture', initial])
    first = json.loads(result.stdout)
    config = json.loads((initial/'config.json').read_text())
    require(first['synthetic'] is False and config['difficulty'] == 'fun' and config['level'] == 1,
            'Default launch must load original Fun01')
    logical = next(initial.glob('frame-*.rgb')).read_bytes()
    require(len(logical) == 320*200*3 and len(set(logical[i:i+3] for i in range(0,len(logical),3))) > 12,
            'Original graphics/panel capture is missing')
    # Include one displayable screenshot without retaining every intermediate capture in the artifact.
    from tools.check_desktop import verify_pixels
    verify_pixels(initial)
    import shutil
    shutil.copyfile(next(initial.glob('frame-*.ppm')), evidence/'original-fun01.ppm')

    levels = []
    for difficulty in ['fun', 'tricky', 'taxing', 'mayhem']:
        for number in range(1,31):
            result = execute(f'level-{difficulty}-{number:02}',
                ['--difficulty', difficulty, '--level', number, '--steps', 1, '--fast'])
            require(json.loads(result.stdout)['synthetic'] is False, 'Synthetic level fallback')
            levels.append(dict(difficulty=difficulty, number=number, status='PASS'))
    save(evidence/'levels.json', levels)

    replay = evidence/'solution.txt'
    replay.write_text('60 0 AssignDigger 0\n', encoding='utf-8')
    transitions = []
    for action, target in [('restart',1), ('continue',2)]:
        directory = evidence/('original-'+action)
        execute(action, ['--steps',1000,'--fast','--render-every',7,'--replay',replay,
                         '--result-actions',action,'--capture',directory])
        rows = [json.loads(line) for line in (directory/'transitions.jsonl').read_text().splitlines()]
        require(len(rows)==1 and rows[0]['target']==target, 'Wrong result transition target')
        phase = json.loads((directory/'completion.jsonl').read_text().splitlines()[-1])
        require(phase['phase']=='Won', 'Original recorded solution did not win')
        attempt = directory/'attempt-0001'
        require(not (attempt/'inputs.txt').read_text(), 'Replay leaked into new attempt')
        baseline = evidence/f'fresh-{target}'
        execute(f'fresh-{target}', ['--level',target,'--steps',1000,'--fast','--render-every',7,'--capture',baseline])
        from tools.check_completion import collect
        require(collect(attempt)==collect(baseline), 'Restart/next differs from a fresh level')
        require((attempt/'terrain-initial.bin').read_bytes()==(baseline/'terrain-initial.bin').read_bytes(),
                'Fresh terrain differs')
        transitions.append(dict(action=action,target=target,status='PASS'))

    result = execute('original-audio', ['--steps',60])
    match = re.search(r'LEMMINGS_AUDIO submitted=(\d+) nonzero=(\d+) active=true original=true',result.stderr)
    require(match and int(match[1])>0 and int(match[2])>0, 'Original ADLIB did not produce nonzero SDL PCM')
    audio = dict(submitted_frames=int(match[1]), nonzero_frames=int(match[2]), driver='dummy', original=True)
    execute('launcher', ['--steps',2,'--fast'], launcher=True)
    run_log = Path(environment['LOCALAPPDATA'])/'OhMyLemmings/last-run.log'
    require('"synthetic":false' in run_log.read_text(encoding='utf-8'), 'Launcher did not run the original game')

    missing = installed/'data/MAIN.DAT'
    renamed = missing.with_suffix('.hidden')
    missing.rename(renamed)
    try:
        execute('missing-data', ['--steps',1,'--fast'], success=False)
        error_log = Path(environment['LOCALAPPDATA'])/'OhMyLemmings/last-error.log'
        require(error_log.exists() and 'Could not start' in error_log.read_text(), 'Startup error log missing')
        execute('launcher-missing-data', ['--steps',1,'--fast'], success=False, launcher=True)
    finally:
        renamed.rename(missing)
    dll = installed/'SDL2.dll'
    dll.rename(installed/'SDL2.hidden')
    try:
        execute('launcher-missing-dll', [], success=False, launcher=True)
        require('SDL2.dll is missing' in run_log.read_text(), 'Missing DLL diagnosis absent')
    finally:
        (installed/'SDL2.hidden').rename(dll)

    vswhere = Path(os.environ['ProgramFiles(x86)'])/'Microsoft Visual Studio/Installer/vswhere.exe'
    studio = subprocess.check_output([str(vswhere),'-latest','-products','*','-requires',
        'Microsoft.VisualStudio.Component.VC.Tools.x86.x64','-property','installationPath'],text=True).strip()
    dumpbin = sorted((Path(studio)/'VC/Tools/MSVC').glob('*/bin/Hostx64/x64/dumpbin.exe'))[-1]
    dependencies = {}
    for name in ['OhMyLemmings.exe','SDL2.dll']:
        content = subprocess.check_output([str(dumpbin),'/DEPENDENTS',str(installed/name)],text=True)
        require(not any(word in content.upper() for word in ['VCRUNTIME140','MSVCP140','API-MS-WIN-CRT-']),
                'Unexpected external MSVC/UCRT runtime')
        dependencies[name] = content
    save(evidence/'dependencies.json', dependencies)
    report = dict(status='PASS',package=package,source_head=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),
        runner_os=os.environ.get('ImageOS'),runner_version=os.environ.get('ImageVersion'),
        rust=subprocess.check_output(['rustc','--version'],text=True).strip(), build_flags=os.environ.get('RUSTFLAGS'),
        original_levels_loaded=len(levels), first_frame_sha256=sha(logical),
        visible_no_argument_window=True, unicode_space_install=True, foreign_cwd=True,
        original_transitions=transitions, audio=audio,
        scope='Actual hosted Windows, original artwork/data/SDL PCM; physical sound and 60fps qualification remain separate.')
    save(evidence/'summary.json', report)
    print(json.dumps(report))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive',type=Path,required=True)
    parser.add_argument('--evidence',type=Path,default=ROOT/'assets/local/windows-package-check')
    args = parser.parse_args()
    check(args.archive.resolve(), args.evidence.resolve())
