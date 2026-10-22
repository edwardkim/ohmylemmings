#!/usr/bin/env python3
"""Build a Windows ZIP containing the executable and all original game resources."""
import argparse
import io
import json
from pathlib import Path
import sys
import tarfile
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]
if __package__ in (None, ''):
    sys.path.insert(0, str(ROOT))
from tools.reference_assets import checked_bytes, download_bytes
from tools.release_package import archive_bytes, encoded, regular, require, sha
from tools.web_build import source_bundle

GAME_FILES = tuple(sorted(['ADLIB.DAT', 'MAIN.DAT', 'ODDTABLE.DAT', 'VGALEMMI.EXE']
    + [f'LEVEL{i:03}.DAT' for i in range(10)] + [f'GROUND{i}O.DAT' for i in range(5)]
    + [f'VGAGR{i}.DAT' for i in range(5)] + [f'VGASPEC{i}.DAT' for i in range(4)]))
BASE_FILES = {'OhMyLemmings.exe', 'SDL2.dll', 'SDL2-LICENSE.txt', 'LGPL-2.1-or-later.txt',
              'source.tar.gz', 'README.txt', 'REBUILD.md', 'start-game.cmd',
              'GAME-DATA.json', 'PACKAGE-MANIFEST.json', 'SHA256SUMS'}


def manifest(root, name):
    return json.loads((root/'assets/manifests'/name).read_text(encoding='utf-8'))


def pe64(data, dll=False):
    require(len(data) >= 64 and data[:2] == b'MZ', 'Expected an x64 Windows PE image')
    offset = int.from_bytes(data[60:64], 'little')
    require(offset+26 <= len(data) and data[offset:offset+4] == b'PE\0\0'
        and int.from_bytes(data[offset+4:offset+6], 'little') == 0x8664
        and int.from_bytes(data[offset+24:offset+26], 'little') == 0x20b
        and bool(int.from_bytes(data[offset+22:offset+24], 'little') & 0x2000) == dll,
        'Expected an x64 PE32+ executable/DLL of the correct kind')


def fetch_sdl(destination, root=ROOT):
    data = manifest(root, 'windows_runtime.json')
    raw = checked_bytes(download_bytes(data['archive']['url'], data['archive']['size']), data['archive'], 'SDL2 archive')
    pending = {}
    with zipfile.ZipFile(io.BytesIO(raw)) as archive:
        for pin in data['files']:
            pending[pin['name']] = checked_bytes(archive.read(pin['member']), pin, pin['name'])
    destination.mkdir(parents=True, exist_ok=True)
    for name, content in pending.items():
        require(Path(name).name == name and '/' not in name and '\\' not in name, 'Invalid SDL filename')
        (destination/name).write_bytes(content)


def game_pins(root):
    data = manifest(root, 'dos_original.json')
    by_name = {row['name']: row for row in data['files']}
    require(all(name in by_name for name in GAME_FILES), 'Original-game manifest is incomplete')
    return data, [by_name[name] for name in GAME_FILES]


def create(runtime, assets, sdl, output, root=ROOT):
    import tomllib
    version = tomllib.loads((root/'Cargo.toml').read_text())['workspace']['package']['version']
    path = output/f'ohmylemmings-{version}-windows-x86_64.zip'
    sidecar = Path(str(path)+'.sha256')
    if path.exists() or sidecar.exists(): raise FileExistsError(path)
    files = {'OhMyLemmings.exe': regular(runtime/'lemmings-desktop.exe')}
    pe64(files['OhMyLemmings.exe'])
    runtime_manifest = manifest(root, 'windows_runtime.json')
    for name in ['SDL2.dll', 'SDL2-LICENSE.txt']:
        pin = next(row for row in runtime_manifest['files'] if row['name'] == name)
        files[name] = checked_bytes(regular(sdl/name), pin, name)
    pe64(files['SDL2.dll'], True)
    original, pins = game_pins(root)
    for pin in pins:
        files['data/'+pin['name']] = checked_bytes(regular(assets/pin['name']), pin, pin['name'])
    files['GAME-DATA.json'] = encoded(dict(profile=original['profile'], original_game_data=True,
        levels=120, files=pins, sources=original.get('sources', [])))
    files['LGPL-2.1-or-later.txt'] = regular(root/'crates/lemmings-opl/LICENSE')
    files['start-game.cmd'] = regular(root/'apps/desktop/windows/start-game.cmd')
    files['README.txt'] = (
        'Oh My Lemmings - Windows x64\r\n\r\n'
        'ZIP 파일을 모두 풀고 OhMyLemmings.exe를 더블클릭하세요.\r\n'
        '원본 그래픽/전체 120개 레벨/음악과 효과음 데이터가 data 폴더에 포함되어 있습니다.\r\n'
        'Fun 01 원본 게임이 바로 시작되며 별도 자산 다운로드나 경로 지정이 필요 없습니다.\r\n'
        'data 폴더와 SDL2.dll을 exe 옆에 유지하세요. ZIP 내부에서 바로 실행하지 마세요.\r\n'
        '문제가 생기면 start-game.cmd를 실행하세요. 오류/로그 경로를 확인할 수 있습니다.\r\n'
        'F/C/E/K/U/M/B/D: 기술 선택, 왼쪽 클릭: 할당, Space: 일시정지, S: 음소거/재개\r\n'
        '방향키: 화면/생성속도, 결과 화면 R: 재시작, Enter: 다음 레벨, Esc: 종료\r\n'
        '고급 선택: OhMyLemmings.exe --difficulty tricky --level 1\r\n'
        '선택 가능한 난이도: fun/tricky/taxing/mayhem, 각 1..30\r\n\r\n'
        'Windows MSVC CRT는 정적으로 링크하며 SDL2.dll은 동봉합니다.\r\n'
        'SDL2: 동봉 SDL2-LICENSE.txt. OPL: LGPL-2.1-or-later.txt.\r\n'
        'OPL Copyright (C) 2013-2020 Nuke.YKT; (C) 2026 Tony Gies (Rust port).\r\n'
        '게임 데이터 출처/해시: GAME-DATA.json. 대응 소스/수정/재링크: source.tar.gz 및 REBUILD.md.\r\n'
        '개발 프리뷰 빌드입니다. 성능 판정은 별도 환경에서 계속합니다(#113).\r\n'
    ).encode('utf-8')
    files['REBUILD.md'] = (
        '# Corresponding source and relinking\n\n'
        'Extract source.tar.gz; read crates/lemmings-opl/README.md and UPSTREAM.patch.\n'
        'It contains current application/library sources, pinned build files, notices and hashes.\n'
        'Install Rust 1.93.1/MSVC/Python 3.12+; cargo fetch --locked.\n'
        'python tools/windows_package.py fetch-sdl\n'
        'Set LIB to assets/local/windows-sdl and RUSTFLAGS=-C target-feature=+crt-static.\n'
        'cargo build -p lemmings-desktop --release --locked --offline\n'
        'Replace OhMyLemmings.exe with the rebuilt target/release/lemmings-desktop.exe.\n'
        'Keep data/ and SDL2.dll beside it. Library modification/relinking and reverse\n'
        'engineering to debug such modifications are permitted. No installation lock is applied.\n'
        'Game data is distributed separately in data/, outside the public build-source archive.\n'
    ).encode()
    with tempfile.TemporaryDirectory() as temporary:
        source = Path(temporary)/'source.tar.gz'; source_info = source_bundle(source, root)
        files['source.tar.gz'] = source.read_bytes()
    files['PACKAGE-MANIFEST.json'] = encoded(dict(schema_version=1, version=version, target='windows-x86_64',
        original_game_data=True, first_level=dict(difficulty='fun',number=1), static_crt=True,
        performance_qualification_deferred=True, source=source_info,
        files=[dict(path=n,bytes=len(b),sha256=sha(b)) for n,b in sorted(files.items())]))
    files['SHA256SUMS'] = ''.join(f'{sha(b)}  {n}\n' for n,b in sorted(files.items())).encode()
    data = archive_bytes(files,'web'); output.mkdir(parents=True, exist_ok=True)
    with path.open('xb') as stream: stream.write(data)
    with sidecar.open('xb') as stream: stream.write(f'{sha(data)}  {path.name}\n'.encode('ascii'))
    verify(path, root=root)
    return path


def verify(path, root=ROOT):
    raw = regular(path)
    require(regular(Path(str(path)+'.sha256')).decode() == f'{sha(raw)}  {path.name}\n', 'Archive checksum differs')
    with zipfile.ZipFile(io.BytesIO(raw)) as archive:
        names = archive.namelist()
        require(len(names) == len(set(names)) and set(names) == BASE_FILES | {'data/'+n for n in GAME_FILES}, 'Invalid Windows inventory')
        require(all(not i.is_dir() and i.external_attr >> 16 == 0o100644 for i in archive.infolist()), 'Invalid Windows ZIP file type')
        files = {n:archive.read(n) for n in names}
    package = json.loads(files['PACKAGE-MANIFEST.json'])
    require(package['schema_version'] == 1 and package['target'] == 'windows-x86_64'
        and package['original_game_data'] is True and package['static_crt'] is True, 'Windows package schema')
    pins = package['files']
    require(len(pins) == len(files)-2 and {p['path'] for p in pins} == set(files)-{'PACKAGE-MANIFEST.json','SHA256SUMS'}, 'Invalid file pins')
    for pin in pins:
        require(len(files[pin['path']]) == pin['bytes'] and sha(files[pin['path']]) == pin['sha256'], 'Internal file checksum differs')
    require(files['SHA256SUMS'].decode() == ''.join(f'{sha(b)}  {n}\n' for n,b in sorted(files.items()) if n!='SHA256SUMS'), 'Internal checksums differ')
    pe64(files['OhMyLemmings.exe']); pe64(files['SDL2.dll'], True)
    original, pins = game_pins(root)
    metadata = json.loads(files['GAME-DATA.json'])
    require(metadata == dict(profile=original['profile'], original_game_data=True,levels=120,files=pins,sources=original.get('sources',[])), 'Original data provenance differs')
    for pin in pins: checked_bytes(files['data/'+pin['name']],pin,pin['name'])
    for name in ['SDL2.dll','SDL2-LICENSE.txt']:
        pin = next(p for p in manifest(root,'windows_runtime.json')['files'] if p['name']==name)
        checked_bytes(files[name],pin,name)
    require(files['LGPL-2.1-or-later.txt'] == regular(root/'crates/lemmings-opl/LICENSE'), 'OPL license differs')
    require(files['start-game.cmd'] == regular(root/'apps/desktop/windows/start-game.cmd'), 'Launcher differs')
    require(sha(files['source.tar.gz']) == package['source']['archive_sha256'], 'Source archive differs')
    with tarfile.open(fileobj=io.BytesIO(files['source.tar.gz']),mode='r:gz') as archive:
        require(all(member.isfile() for member in archive.getmembers()), 'Source contains links or non-files')
        require(len(set(archive.getnames())) == len(archive.getnames()), 'Duplicate source members')
        source_pins = json.load(archive.extractfile('SOURCE-MANIFEST.json'))
        require(len(source_pins) == package['source']['source_files']
            and set(archive.getnames()) == {'SOURCE-MANIFEST.json'} | {pin['path'] for pin in source_pins}
            and len(archive.getnames()) == len(source_pins)+1, 'Source inventory differs')
        for pin in source_pins:
            require(not Path(pin['path']).is_absolute() and '..' not in Path(pin['path']).parts
                and not pin['path'].startswith(('assets/local/','target/')), 'Invalid source path')
            data = archive.extractfile(pin['path']).read()
            require(len(data)==pin['bytes'] and sha(data)==pin['sha256'], 'Source file checksum differs')
    return dict(status='PASS', target='windows-x86_64', original_game_data=True, game_files=len(GAME_FILES),
        source_files=len(source_pins), archive_bytes=len(raw), sha256=sha(raw))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command',choices=['fetch-sdl','create','verify'])
    parser.add_argument('--runtime',type=Path,default=ROOT/'target/release')
    parser.add_argument('--assets',type=Path,default=ROOT/'assets/local/dos-original')
    parser.add_argument('--sdl',type=Path,default=ROOT/'assets/local/windows-sdl')
    parser.add_argument('--output',type=Path,default=ROOT/'assets/local/dist')
    parser.add_argument('--archive',type=Path)
    args = parser.parse_args()
    if args.command=='fetch-sdl': fetch_sdl(args.sdl)
    elif args.command=='create':
        path=create(args.runtime,args.assets,args.sdl,args.output); print(json.dumps(dict(archive=str(path),**verify(path))))
    else:
        if args.archive is None: parser.error('--archive is required')
        print(json.dumps(verify(args.archive)))
