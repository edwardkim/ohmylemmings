#!/usr/bin/env python3
"""Promote the latest public stable Web release to GitHub Pages without rebuilding it."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
from urllib.request import Request, urlopen
import zipfile
import io

ROOT = Path(__file__).resolve().parents[1]
if __package__ in (None, ''):
    sys.path.insert(0, str(ROOT))
from tools.release_package import encoded, require, sha, verify_package
from tools.release_version import package_name, parse_version

REPOSITORY = 'edwardkim/ohmylemmings'
RELEASES = f'https://github.com/{REPOSITORY}/releases'
MAX_DOWNLOAD = 256 * 1024 * 1024
MAX_SITE = 512 * 1024 * 1024
MAX_FILES = 256


def stable_version(tag):
    require(tag.startswith('v'), 'Expected an explicit vX.Y.Z tag')
    version = parse_version(tag[1:])
    require(not version['prerelease'], 'Pages requires a stable version')
    return version['version']


def asset_rows(tag, release):
    version = stable_version(tag)
    require(release['tag_name'] == tag and release['draft'] is False
            and release['prerelease'] is False, 'Expected a public stable release')
    require(release['html_url'] == f'{RELEASES}/tag/{tag}', 'Release origin differs')
    names = [row['name'] for row in release['assets']]
    require(len(names) == len(set(names)), 'Duplicate release assets')
    expected = [package_name('web', version), package_name('web', version) + '.sha256', 'RELEASE-MANIFEST.json']
    rows = {row['name']: row for row in release['assets'] if row['name'] in expected}
    require(set(rows) == set(expected), 'Missing Web release assets')
    for name, row in rows.items():
        require(row['browser_download_url'] == f'{RELEASES}/download/{tag}/{name}', 'Asset origin differs')
        require(type(row['size']) is int and 0 < row['size'] <= MAX_DOWNLOAD
                and re.fullmatch(r'sha256:[0-9a-f]{64}', row.get('digest') or ''), 'Invalid asset size/digest')
    return rows


def stage(tag, release, latest_tag, inputs, output, root=ROOT):
    version = stable_version(tag)
    require(tag == latest_tag, 'Only the latest published stable release may replace Pages')
    rows = asset_rows(tag, release)
    for name, row in rows.items():
        data = inputs[name]
        require(len(data) == row['size'] and 'sha256:' + sha(data) == row['digest'], 'Public asset digest differs')
    manifest = json.loads(inputs['RELEASE-MANIFEST.json'])
    require(manifest['schema_version'] == 1 and manifest['repository'] == REPOSITORY
            and manifest['version'] == version and manifest['tag'] == tag, 'Release manifest identity differs')
    for field in ['release_commit', 'source_tree']:
        require(re.fullmatch('[0-9a-f]{40}', manifest[field]), 'Invalid release source identity')
    pins = {row['path']: row for row in manifest['files']}
    require(len(pins) == len(manifest['files']), 'Duplicate manifest files')
    name = package_name('web', version)
    for filename in [name, name + '.sha256']:
        data = inputs[filename]
        require(filename in pins and pins[filename]['bytes'] == len(data)
                and pins[filename]['sha256'] == sha(data), 'Release manifest checksum differs')
    packages = [row for row in manifest['packages'] if row['target'] == 'web']
    require(len(packages) == 1 and packages[0]['version'] == version
            and packages[0]['path'] == name and packages[0]['sha256'] == sha(inputs[name]), 'Web package identity differs')
    # Bound expansion before the full package verifier reads archive members.
    with zipfile.ZipFile(io.BytesIO(inputs[name])) as archive:
        require(len(archive.infolist()) <= MAX_FILES and sum(row.file_size for row in archive.infolist()) <= MAX_SITE,
                'Web package exceeds the Pages staging budget')
    with tempfile.TemporaryDirectory() as temporary:
        path = Path(temporary) / name
        path.write_bytes(inputs[name])
        Path(str(path) + '.sha256').write_bytes(inputs[name + '.sha256'])
        checked = verify_package(path, root=root)
    require(checked['target'] == 'web' and checked['original_game_data'] and checked['game_files'] == 28,
            'Pages requires the full original game package')
    with zipfile.ZipFile(io.BytesIO(inputs[name])) as archive:
        package = json.loads(archive.read('PACKAGE-MANIFEST.json'))
        require(package['version'] == version, 'Internal package version differs')
        # verify_package has checked the complete allowlist, paths, types and all file hashes.
        output.mkdir(parents=True, exist_ok=False)
        for filename in archive.namelist():
            path = output / filename
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(archive.read(filename))
    provenance = dict(schema_version=1, version=version, tag=tag, release_url=release['html_url'],
                      release_commit=manifest['release_commit'], source_tree=manifest['source_tree'],
                      archive=name, archive_sha256=checked['sha256'], files=checked['files'])
    (output / '.nojekyll').write_bytes(b'')
    (output / 'deployment.json').write_bytes(encoded(provenance))
    return provenance


def api(path):
    return json.loads(subprocess.check_output(['gh', 'api', f'repos/{REPOSITORY}/{path}'], text=True))


def latest():
    return api('releases/latest')['tag_name']


def download(row):
    request = Request(row['browser_download_url'], headers={'User-Agent': 'ohmylemmings-pages'})
    with urlopen(request, timeout=60) as response:
        data = response.read(MAX_DOWNLOAD + 1)
    require(len(data) <= MAX_DOWNLOAD, 'Public download exceeds staging budget')
    return data


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tag', required=True)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--check-latest', action='store_true')
    args = parser.parse_args()
    stable_version(args.tag)
    current = latest()
    require(current == args.tag, 'Requested tag is no longer the latest stable release')
    if args.check_latest:
        print(json.dumps(dict(status='PASS', tag=current)))
        return
    if args.output is None:
        parser.error('--output is required for staging')
    release = api(f'releases/tags/{args.tag}')
    rows = asset_rows(args.tag, release)
    inputs = {name: download(row) for name, row in rows.items()}
    print(json.dumps(stage(args.tag, release, current, inputs, args.output)))


if __name__ == '__main__':
    main()
