#!/usr/bin/env python3
"""Create verified full-game or original-free development packages from already built Linux/Web inputs."""
import argparse
import gzip
import hashlib
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
from tools.web_build import source_bundle
from tools.package_assets import GAME_FILES, game_pins
from tools.reference_assets import checked_bytes
from tools.release_version import workspace_version, package_name
from tools.hd_delivery import delivery_files, verify_delivery
from tools.app_icons import icon_bytes, MANIFEST as ICON_MANIFEST

WEB_FILES = (
    'index.html', 'app.js', 'audio.js', 'local-music.js', 'audio-license.html', 'controls.js',
    'host.js', 'frame-feedback.js', 'hd.js', 'hd-loader.js', 'progressive-hd.js', 'download.js', 'bundled.js', 'style.css', 'pkg/lemmings_web.js', 'pkg/lemmings_web_bg.wasm',
)
COMMON_FILES = {'README.md', 'REBUILD.md', 'LGPL-2.1-or-later.txt'}
PROJECT_LICENSE_FILES = {'LICENSE', 'THIRD_PARTY_NOTICES.txt'}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def encoded(data):
    return (json.dumps(data, indent=2, sort_keys=True) + '\n').encode()


def regular(path):
    require(path.is_file() and not path.is_symlink()
            and not any(parent.is_symlink() for parent in path.parents),
            f'Missing or linked package input: {path}')
    data = path.read_bytes()
    require(bool(data), f'Empty package input: {path}')
    return data


def project_license_files(root):
    return {name: regular(root / name) for name in sorted(PROJECT_LICENSE_FILES)}


def verify_project_license(files, manifest, root, prefix=''):
    # Legacy public v0.1.0 did not grant a project license. Do not relabel it.
    if 'project_license' not in manifest:
        return False
    require(manifest['project_license'] == 'MIT', 'Unsupported project license')
    for name, data in project_license_files(root).items():
        require(files.get(prefix + name) == data, 'Project/third-party license differs')
    return True


def archive_bytes(files, target):
    output = io.BytesIO()
    if target == 'web':
        with zipfile.ZipFile(output, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
            for name, data in sorted(files.items()):
                entry = zipfile.ZipInfo(name, (1980, 1, 1, 0, 0, 0))
                entry.create_system = 3
                entry.external_attr = (0o100644 << 16)
                entry.compress_type = zipfile.ZIP_DEFLATED
                archive.writestr(entry, data, compresslevel=9)
    else:
        with gzip.GzipFile(filename='', mode='wb', fileobj=output, mtime=0) as compressed:
            with tarfile.open(fileobj=compressed, mode='w') as archive:
                for name, data in sorted(files.items()):
                    entry = tarfile.TarInfo(name)
                    entry.mode = 0o755 if name == 'lemmings-desktop' else 0o644
                    entry.size = len(data)
                    archive.addfile(entry, io.BytesIO(data))
    return output.getvalue()


def create_package(kind, runtime, output, root=ROOT, assets=None):
    require(kind in ('linux', 'web'), 'Supported package targets: linux, web')
    target = 'linux-x86_64' if kind == 'linux' else 'web'
    version = workspace_version(root)
    name = package_name(target, version)
    if assets is None: name = name.replace(f'{version}-', f'{version}-dev-', 1)
    path = output / name
    sidecar = Path(str(path) + '.sha256')
    if path.exists() or sidecar.exists():
        raise FileExistsError(f'Package evidence already exists: {path}')
    files = project_license_files(root)
    if kind == 'web':
        for name in WEB_FILES:
            files[name] = regular(runtime / name)
            if not name.startswith('pkg/'):
                require(files[name] == regular(root / 'apps/web/www' / name), f'Stale Web runtime: {name}')
        if (root/'assets/manifests/remaster_pack.json').exists():
            require((runtime/'hd-assets.hdp').is_file(),'HD runtime pack is missing; rebuild the Web runtime')
        if (runtime/'hd-assets.hdp').exists() and not (runtime/'hd-assets/catalog.hdi').exists():
            files['hd-assets.hdp']=regular(runtime/'hd-assets.hdp')
        if (runtime/'hd-assets/catalog.hdi').exists():
            from tools.hd_catalog import runtime_catalog
            expected=runtime_catalog(root,regular(runtime/'hd-assets.hdp'))
            for name,data in expected.items():
                require(regular(runtime/'hd-assets'/name)==data,'HD catalog member differs')
                if not (root/'assets/manifests/remaster_pack.json').is_file():files['hd-assets/'+name]=data
            del expected
        require(files['pkg/lemmings_web_bg.wasm'][:8] == b'\0asm\1\0\0\0', 'Invalid WebAssembly header')
    else:
        binary = regular(runtime)
        require(len(binary) >= 64 and binary[:6] == b'\x7fELF\x02\x01'
                and int.from_bytes(binary[18:20], 'little') == 62, 'Expected a Linux x86_64 ELF executable')
        files['lemmings-desktop'] = binary
        if (root/ICON_MANIFEST).is_file():
            files['app-icon.png'] = icon_bytes(root, 'app-icon.png')
    hd_remaster = (root/'assets/manifests/remaster_pack.json').is_file()
    if kind == 'web':
        files['index.html'] = files['index.html'].replace(b'<head>', b'<head>\n    <meta name="game-version" content="'+version.encode()+b'">', 1)
    if hd_remaster:
        hd_files=delivery_files(root,web=kind=='web')
        if kind=='web':
            for name,data in hd_files.items():
                if name=='HD-WEB.json' or name.startswith('web-hd/'):
                    require(regular(runtime/name)==data,'Stale Web HD transport: '+name)
        files.update(hd_files)
        if kind == 'web':
            require(files['index.html'].count(b'<head>') == 1, 'Expected one Web document head')
            files['index.html'] = files['index.html'].replace(b'<head>', b'<head>\n    <meta name="game-presentation" content="hd">', 1)
            files['index.html'] = files['index.html'].replace(b'<head>', b'<head>\n    <meta name="game-hd-stream" content="./HD-WEB.json">', 1)
    if assets is not None:
        if kind == 'web':
            require(files['index.html'].count(b'<head>') == 1, 'Expected one Web document head')
            files['index.html'] = files['index.html'].replace(b'<head>', b'<head>\n    <meta name="game-data" content="./GAME-DATA.json">', 1)
        original, pins = game_pins(root)
        for pin in pins:
            files['data/' + pin['name']] = checked_bytes(regular(assets / pin['name']), pin, pin['name'])
        files['GAME-DATA.json'] = encoded(dict(profile=original['profile'], original_game_data=True,
            levels=120, files=pins, sources=original.get('sources', [])))
    files['LGPL-2.1-or-later.txt'] = regular(root / 'crates/lemmings-opl/LICENSE')
    source_name = 'audio-source.tar.gz' if kind == 'web' else 'source.tar.gz'
    with tempfile.TemporaryDirectory() as temporary:
        source_path = Path(temporary) / source_name
        source = source_bundle(source_path, root)
        files[source_name] = source_path.read_bytes()
    if kind == 'web':
        files['audio-source.json'] = encoded(dict(**source, license='LGPL-2.1-or-later',
            license_sha256=sha(files['LGPL-2.1-or-later.txt']),
            wasm_sha256=sha(files['pkg/lemmings_web_bg.wasm']), notice='audio-license.html', archive=source_name))
    launch = ('Serve this extracted folder over HTTP, for example: python3 -m http.server 8080\n'
              'Open http://localhost:8080/ in a current WASM-capable browser.\n'
              'Choose practice mode; sound needs a user click. Select owned DOS files for original levels.\n'
              'Use the visible pause, restart and sound controls.\n') if kind == 'web' else (
              'Linux x86_64: build qualification uses Ubuntu 24.04.\n'
              'Install the SDL2 runtime (Ubuntu: sudo apt-get install libsdl2-2.0-0).\n'
              './lemmings-desktop --synthetic-outcome win\n'
              './lemmings-desktop --assets /path/to/owned/dos-files --difficulty fun --level 1\n'
              'Space pauses; R restarts on results; Enter continues; S toggles sound; Esc exits.\n'
              'Run ./lemmings-desktop --help for options.\n')
    if assets is not None:
        launch = (
            'Serve the entire extracted folder over HTTP: python3 -m http.server 8080\n'
            'Open http://localhost:8080/ (or your hosted subdirectory) in a WASM-capable browser.\n'
            'Fun 01 loads automatically. Click the sound button to enable music/effects.\n'
            'Choose any of four difficulties and levels 1-30. Keep data/ and GAME-DATA.json.\n'
        ) if kind == 'web' else (
            'Linux x86_64: built on Ubuntu 24.04; requires glibc 2.39+ and SDL2 runtime.\n'
            'Ubuntu: sudo apt-get install libsdl2-2.0-0\n'
            'Extract everything, keep data/ beside the executable, then run ./lemmings-desktop\n'
            'Fun 01 starts automatically, including when launched from another directory.\n'
            'Other levels: ./lemmings-desktop --difficulty tricky --level 1\n'
            'Space pauses; R restarts on results; Enter continues; S toggles sound; Esc exits.\n'
        )
    data_notice = ('Original graphics, all 120 levels, music and effects are included in data/.\n'
                   'GAME-DATA.json records original provenance and hashes; the project LGPL notice does not license DOS data.\n'
                   if assets is not None else
                   'Original DOS assets are not included. Practice graphics and music are synthetic.\n')
    if hd_remaster:
        launch += ('HD presentation starts automatically. Keep hd-assets/, HD-ASSETS.json and HD-PRESENTATION.\n'
                   'Use the Web presentation selector or native F8 to switch; native --original starts original mode.\n')
    files['README.md'] = (
        f'# Oh My Lemmings {version} package ({target})\n\n' + launch + '\n'
        + data_notice +
        'Performance qualification on a faster host is pending (#113); 60fps is not guaranteed.\n'
        'Project-authored contributions are MIT; see LICENSE and THIRD_PARTY_NOTICES.txt.\n'
        'Third-party terms remain in force; original DOS data and marks are outside the MIT grant.\n'
        'The OPL synthesizer is LGPL-2.1-or-later: Copyright (C) 2013-2020 Nuke.YKT;\n'
        'Copyright (C) 2026 Tony Gies (Rust port). See LGPL-2.1-or-later.txt and REBUILD.md.\n'
        'Project: https://github.com/edwardkim/ohmylemmings\n'
    ).encode()
    files['REBUILD.md'] = (
        '# Corresponding source and relinking\n\n'
        f'Extract {source_name} into a separate folder. It includes current application/library\n'
        'sources, pinned build files, notices, modifications and SOURCE-MANIFEST.json hashes.\n'
        'Read crates/lemmings-opl/README.md and UPSTREAM.patch. Library modification, relinking\n'
        'and reverse engineering for debugging such modifications are permitted by the LGPL.\n'
        'Install Rust 1.93.1 and Python 3.12+; Linux builds need SDL2 development libraries.\n'
        'Run cargo fetch --locked, then cargo build -p lemmings-desktop --release --locked --offline.\n'
        'Web: rustup target add wasm32-unknown-unknown; python3 tools/web_build.py fetch; make web.\n'
        'Replace the executable or the generated Web pkg/JS files with your rebuilt outputs.\n'
        'No signing restriction or installation lock is applied by these development packages.\n'
    ).encode()
    files['PACKAGE-MANIFEST.json'] = encoded(dict(schema_version=1, version=version, target=target, project_license='MIT',
        development=True, performance_qualification_deferred=True, original_game_data=assets is not None, hd_remaster=hd_remaster,
        source_archive=source_name, source=source,
        files=[dict(path=name, bytes=len(data), sha256=sha(data)) for name, data in sorted(files.items())]))
    files['SHA256SUMS'] = ''.join(f'{sha(data)}  {name}\n' for name, data in sorted(files.items())).encode()
    data = archive_bytes(files, target)
    output.mkdir(parents=True, exist_ok=True)
    with path.open('xb') as stream:
        stream.write(data)
    with sidecar.open('xb') as stream:
        stream.write(f'{sha(data)}  {path.name}\n'.encode())
    verify_package(path, root=root)
    return path


def verify_package(path, root=ROOT):
    from tools.detached_source import verify_external
    external = verify_external(path, root, verify_package)
    if external is not None:
        return external
    data = regular(path)
    require(regular(Path(str(path) + '.sha256')).decode() == f'{sha(data)}  {path.name}\n', 'Archive checksum differs')
    if path.suffix == '.zip':
        with zipfile.ZipFile(io.BytesIO(data)) as archive:
            names = archive.namelist()
            require(all(not entry.is_dir() and entry.external_attr >> 16 == 0o100644 for entry in archive.infolist()), 'Invalid ZIP member type/mode')
            files = {name: archive.read(name) for name in names}
    else:
        with tarfile.open(fileobj=io.BytesIO(data), mode='r:gz') as archive:
            names = archive.getnames()
            require(all(entry.isfile() and entry.mode == (0o755 if entry.name == 'lemmings-desktop' else 0o644)
                        for entry in archive.getmembers()), 'Invalid tar member type/mode')
            files = {name: archive.extractfile(name).read() for name in names}
    require(len(names) == len(set(names)) and all(not Path(n).is_absolute() and '..' not in Path(n).parts for n in names), 'Invalid archive paths')
    manifest = json.loads(files['PACKAGE-MANIFEST.json'])
    target = manifest['target']
    require(target in ('web', 'linux-x86_64') and manifest['schema_version'] == 1
            and manifest['development'] is True and manifest['performance_qualification_deferred'] is True, 'Invalid development package manifest')
    source_name = 'audio-source.tar.gz' if target == 'web' else 'source.tar.gz'
    allowed = COMMON_FILES | {source_name, 'PACKAGE-MANIFEST.json', 'SHA256SUMS'}
    if verify_project_license(files, manifest, root):
        allowed |= PROJECT_LICENSE_FILES
    allowed |= set(WEB_FILES) | {'audio-source.json'} if target == 'web' else {'lemmings-desktop'}
    if target == 'linux-x86_64' and 'app-icon.png' in files:
        require(files['app-icon.png'] == icon_bytes(root, 'app-icon.png'), 'Linux app icon differs')
        allowed.add('app-icon.png')
    if manifest.get('hd_remaster', False):
        allowed |= verify_delivery(files, root,web=target=='web')
        if target == 'web':
            require(b'<meta name="game-presentation" content="hd">' in files['index.html'], 'Missing HD presentation marker')
            require(b'<meta name="game-hd-stream" content="./HD-WEB.json">' in files['index.html'], 'Missing progressive HD marker')
    elif target=='web' and ('hd-assets.hdp' in files or 'hd-assets/catalog.hdi' in files):
        try:from .hd_assets import runtime_pack
        except ImportError:from hd_assets import runtime_pack
        base_hd=runtime_pack(root=root)
        if 'hd-assets.hdp' in files:
            require(files['hd-assets.hdp']==base_hd,'HD runtime pack differs')
            allowed.add('hd-assets.hdp')
        if 'hd-assets/catalog.hdi' in files:
            from tools.hd_catalog import runtime_catalog
            for name,member in runtime_catalog(root,base_hd).items():
                require(files.get('hd-assets/'+name)==member,'HD catalog member differs')
                allowed.add('hd-assets/'+name)
    full = manifest.get('original_game_data', False)
    require(type(full) is bool, 'Invalid original data marker')
    if full:
        allowed |= {'GAME-DATA.json'} | {'data/' + name for name in GAME_FILES}
    require(set(files) == allowed, 'Unexpected or missing package files')
    if full:
        if target == 'web':
            require(b'<meta name="game-data" content="./GAME-DATA.json">' in files['index.html'], 'Missing required game-data marker')
        original, game = game_pins(root)
        info = json.loads(files['GAME-DATA.json'])
        require(info == dict(profile=original['profile'], original_game_data=True, levels=120,
                            files=game, sources=original.get('sources', [])), 'Original provenance differs')
        for pin in game:
            checked_bytes(files['data/' + pin['name']], pin, pin['name'])
    pins = manifest['files']
    require(len(pins) == len({pin['path'] for pin in pins})
            and {pin['path'] for pin in pins} == allowed - {'PACKAGE-MANIFEST.json', 'SHA256SUMS'}, 'Invalid manifest inventory')
    for pin in pins:
        require(len(files[pin['path']]) == pin['bytes'] and sha(files[pin['path']]) == pin['sha256'], 'Package file checksum differs')
    expected = ''.join(f'{sha(value)}  {name}\n' for name, value in sorted(files.items()) if name != 'SHA256SUMS')
    require(files['SHA256SUMS'].decode() == expected, 'Internal checksums differ')
    require(manifest['source_archive'] == source_name and sha(files[source_name]) == manifest['source']['archive_sha256']
            and len(files[source_name]) == manifest['source']['archive_bytes'], 'Source archive pin differs')
    with tarfile.open(fileobj=io.BytesIO(files[source_name]), mode='r:gz') as source:
        entries = source.getmembers()
        require(all(entry.isfile() and not Path(entry.name).is_absolute() and '..' not in Path(entry.name).parts
                    and not entry.name.startswith(('assets/local/', 'target/')) for entry in entries), 'Invalid source archive')
        pins = json.load(source.extractfile('SOURCE-MANIFEST.json'))
        require(len(entries) == len(pins) + 1 and len(pins) == manifest['source']['source_files']
                and {e.name for e in entries} == {p['path'] for p in pins} | {'SOURCE-MANIFEST.json'}, 'Source inventory differs')
        for pin in pins:
            value = source.extractfile(pin['path']).read()
            require(len(value) == pin['bytes'] and sha(value) == pin['sha256'], 'Source checksum differs')
    if target == 'web':
        info = json.loads(files['audio-source.json'])
        require(info['archive_sha256'] == manifest['source']['archive_sha256']
                and info['wasm_sha256'] == sha(files['pkg/lemmings_web_bg.wasm'])
                and info['license_sha256'] == sha(files['LGPL-2.1-or-later.txt']), 'Web source/license link differs')
    return dict(status='PASS', target=target, development=True, performance_qualification_deferred=True,
                original_game_data=full, game_files=len(GAME_FILES) if full else 0,
                archive_bytes=len(data), sha256=sha(data), files=len(files), source_files=len(pins))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['linux', 'web', 'verify'])
    parser.add_argument('--runtime', type=Path)
    parser.add_argument('--output', type=Path, default=ROOT / 'assets/local/dist')
    parser.add_argument('--archive', type=Path)
    parser.add_argument('--assets', type=Path, default=ROOT / 'assets/local/dos-original')
    parser.add_argument('--development', action='store_true', help='Explicit original-free practice package')
    args = parser.parse_args()
    if args.command == 'verify':
        if args.archive is None:
            parser.error('--archive is required')
        print(json.dumps(verify_package(args.archive)))
    else:
        if args.runtime is None:
            parser.error('--runtime is required')
        path = create_package(args.command, args.runtime, args.output, assets=None if args.development else args.assets)
        print(json.dumps(dict(archive=str(path), **verify_package(path))))


if __name__ == '__main__':
    main()
