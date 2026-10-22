#!/usr/bin/env python3
"""One-off #115 Windows feasibility experiment; not a release packaging command."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import zipfile

ROOT = Path(__file__).resolve().parents[1]
if __package__ in (None, ''):
    sys.path.insert(0, str(ROOT))
from tools.reference_assets import download_bytes
from tools.release_package import archive_bytes
from tools.web_build import source_bundle

BASE = ROOT / 'assets/local/windows-assessment'
SDL_URL = 'https://github.com/libsdl-org/SDL/releases/download/release-2.32.10/SDL2-devel-2.32.10-VC.zip'
SDL_SIZE = 7172966
SDL_SHA256 = 'af347939395a58b365846aaea27391e69f9ec9d4dd650d6ac40802159b418a6e'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def record(name, value):
    path = BASE / 'evidence' / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')


def prepare():
    data = download_bytes(SDL_URL, SDL_SIZE)
    if len(data) != SDL_SIZE or digest(data) != SDL_SHA256:
        raise ValueError('Pinned official SDL2 archive differs')
    BASE.mkdir(parents=True)
    path = BASE / 'SDL2-devel-2.32.10-VC.zip'
    path.write_bytes(data)
    with zipfile.ZipFile(path) as archive:
        # Copy only the pinned x64 inputs and license, never an entire archive tree.
        members = ['lib/x64/SDL2.dll', 'lib/x64/SDL2.lib', 'lib/x64/SDL2main.lib', 'LICENSE.txt']
        pins = []
        for name in members:
            source = 'SDL2-2.32.10/' + name
            content = archive.read(source)
            destination = BASE / 'sdl' / source
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(content)
            pins.append(dict(path=source, bytes=len(content), sha256=digest(content)))
    record('inputs.json', dict(url=SDL_URL, bytes=len(data), sha256=SDL_SHA256, files=pins))


def execute(binary, arguments, cwd, label):
    result = subprocess.run([str(binary), *arguments], cwd=cwd, text=True, encoding='utf-8',
        capture_output=True, timeout=45, env=dict(os.environ, SDL_VIDEODRIVER='dummy',
                                               SDL_AUDIODRIVER='dummy', SDL_RENDER_DRIVER='software'))
    record(label + '.json', dict(arguments=arguments, exit_code=result.returncode,
                                stdout=result.stdout, stderr=result.stderr))
    result.check_returncode()
    return result.stdout


def check():
    if os.name != 'nt':
        raise ValueError('This experiment requires an actual Windows runner')
    binary = ROOT / 'target/release/lemmings-desktop.exe'
    sdl = BASE / 'sdl/SDL2-2.32.10'
    files = {
        'lemmings-desktop.exe': binary.read_bytes(),
        'SDL2.dll': (sdl / 'lib/x64/SDL2.dll').read_bytes(),
        'SDL2-LICENSE.txt': (sdl / 'LICENSE.txt').read_bytes(),
        'LGPL-2.1-or-later.txt': (ROOT / 'crates/lemmings-opl/LICENSE').read_bytes(),
    }
    source = source_bundle(BASE / 'source.tar.gz')
    files['source.tar.gz'] = (BASE / 'source.tar.gz').read_bytes()
    files['README.txt'] = (
        'Windows packaging feasibility experiment for issue #115; not a final release.\n'
        'Keep SDL2.dll next to lemmings-desktop.exe.\n'
        'Practice: lemmings-desktop.exe --synthetic-outcome win\n'
        'Original levels: lemmings-desktop.exe --assets C:\\path\\to\\owned\\DOS-files\n'
        'Original DOS assets are not included.\n'
        'SDL2 uses its included zlib license. OPL uses LGPL-2.1-or-later.\n'
        'OPL: Copyright (C) 2013-2020 Nuke.YKT; Copyright (C) 2026 Tony Gies (Rust port).\n'
        'Extract source.tar.gz for complete application/library source and notices.\n'
        'Read crates/lemmings-opl/README.md and UPSTREAM.patch for modification/relinking.\n'
        'Install Rust 1.93.1/MSVC/Python; cargo fetch --locked; use the pinned SDL2\n'
        'MSVC x64 import libraries via LIB and copy SDL2.dll beside the rebuilt exe.\n'
        'Then cargo build -p lemmings-desktop --release --locked --offline.\n'
        'Library modification, relinking and reverse engineering to debug those\n'
        'modifications are permitted. No project-wide license is granted here.\n'
        'Performance, desktop human interaction and clean end-user installation\n'
        'are not certified by this hosted-runner experiment.\n'
    ).encode()
    flags = os.environ.get('RUSTFLAGS', '')
    files['README.txt'] += ('Build RUSTFLAGS: ' + (flags or '(default)') + '\n').encode()
    files['MANIFEST.json'] = (json.dumps(dict(source=source, files=[dict(path=n, bytes=len(b), sha256=digest(b))
        for n, b in sorted(files.items())]), indent=2) + '\n').encode()
    files['SHA256SUMS'] = ''.join(f'{digest(b)}  {n}\n' for n, b in sorted(files.items())).encode()
    trial = BASE / 'ohmylemmings-windows-assessment.zip'
    trial.write_bytes(archive_bytes(files, 'web'))
    # A separate path exercises spaces/Unicode and avoids the build tree's DLL search.
    installed = BASE / '설치 검사'
    with zipfile.ZipFile(trial) as archive:
        assert set(archive.namelist()) == set(files)
        for name, data in files.items():
            assert archive.read(name) == data
        archive.extractall(installed)
    executable = installed / 'lemmings-desktop.exe'
    help_text = execute(executable, ['--help'], installed, 'help')
    assert 'Usage: lemmings-desktop' in help_text
    smoke = installed / 'capture'
    execute(executable, ['--synthetic-outcome', 'win', '--steps', '1200', '--fast',
        '--render-every', '7', '--result-actions', 'restart,continue', '--capture', str(smoke)], installed, 'results')
    transitions = [json.loads(line) for line in (smoke / 'transitions.jsonl').read_text(encoding='utf-8').splitlines()]
    assert [row['action'].lower() for row in transitions] == ['restart', 'continue']
    phases = []
    for folder in [smoke, *sorted(smoke.glob('attempt-*'))]:
        rows = [json.loads(line) for line in (folder / 'completion.jsonl').read_text(encoding='utf-8').splitlines()]
        phases.append(rows[-1]['phase'])
    assert phases == ['Won', 'Won', 'Lost']
    audio = json.loads(execute(executable, ['--audio-probe'], installed, 'audio'))
    from tools.check_audio_output import validate_sdl
    validate_sdl(audio)
    # Record actual PE import requirements using the runner's MSVC tools.
    vswhere = Path(os.environ['ProgramFiles(x86)']) / 'Microsoft Visual Studio/Installer/vswhere.exe'
    studio = subprocess.check_output([str(vswhere), '-latest', '-products', '*', '-requires',
        'Microsoft.VisualStudio.Component.VC.Tools.x86.x64', '-property', 'installationPath'],
        text=True, encoding='utf-8').strip()
    dumpbin = sorted((Path(studio) / 'VC/Tools/MSVC').glob('*/bin/Hostx64/x64/dumpbin.exe'))[-1]
    dependencies = {}
    for name in ['lemmings-desktop.exe', 'SDL2.dll']:
        dependencies[name] = subprocess.check_output([str(dumpbin), '/DEPENDENTS', str(installed / name)],
                                                       text=True, encoding='utf-8', errors='replace')
    record('dependencies.json', dependencies)
    if '+crt-static' in flags:
        for text in dependencies.values():
            assert 'VCRUNTIME140' not in text.upper() and 'MSVCP140' not in text.upper()
            assert 'API-MS-WIN-CRT-' not in text.upper()
    summary = dict(status='PASS', target='x86_64-pc-windows-msvc',
        source_head=subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
        rust=subprocess.check_output(['rustc', '--version'], text=True).strip(),
        build_flags=flags,
        runner_os=os.environ.get('ImageOS'), runner_version=os.environ.get('ImageVersion'),
        archive_sha256=digest(trial.read_bytes()), source=source,
        files=[dict(path=n, bytes=len(b), sha256=digest(b)) for n, b in sorted(files.items())],
        outcomes=phases, audio=audio, unicode_space_install_path=True,
        scope='Hosted Windows build and extracted trial only; no real desktop/human audio/60fps qualification')
    record('summary.json', summary)
    print(json.dumps(summary))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['prepare', 'check'])
    arguments = parser.parse_args()
    try:
        prepare() if arguments.command == 'prepare' else check()
    except Exception as error:
        record('failure.json', dict(status='FAIL', command=arguments.command, error=repr(error)))
        raise
