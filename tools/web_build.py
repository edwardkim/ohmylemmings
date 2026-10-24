#!/usr/bin/env python3
"""Fetch the pinned official bindgen CLI, build offline, or serve the web entry."""
import argparse
import functools
import gzip
import hashlib
import io
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
try:
    from .reference_assets import checked_bytes, download_bytes
except ImportError:
    from reference_assets import checked_bytes, download_bytes

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / 'assets/manifests/web_tools.json'
TOOLS = ROOT / 'assets/local/web-tools'
OUTPUT = ROOT / 'assets/local/web'
SOURCE_REQUIRED = {
    'LICENSE', 'THIRD_PARTY_NOTICES.txt',
    'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'Makefile',
    'apps/web/Cargo.toml', 'apps/web/src/lib.rs',
    'crates/lemmings-opl/Cargo.toml', 'crates/lemmings-opl/LICENSE',
    'crates/lemmings-opl/README.md', 'crates/lemmings-opl/UPSTREAM.patch',
    'crates/lemmings-opl/src/lib.rs', 'crates/lemmings-opl/src/core.rs',
    'tools/web_build.py', 'assets/manifests/web_tools.json',
}


def runtime_art_sources(root):
    """Follow the same direct inputs as runtime_pack/runtime_catalog, not history pins."""
    result = set()
    seen = set()
    pending = ['assets/manifests/remaster_pack.json']
    while pending:
        name = pending.pop()
        if name in seen:
            continue
        seen.add(name)
        path = Path(name)
        if path.is_absolute() or '..' in path.parts or name.startswith(('assets/local/', 'target/')):
            raise ValueError('Invalid HD source path')
        if not (root / name).exists() and name == 'assets/manifests/remaster_pack.json':
            continue  # Synthetic/original-free source fixture.
        result.add(name)
        info = json.loads((root / name).read_text())
        for row in info.get('atlases', []) + info.get('provenance', []):
            source = row['path']
            if Path(source).is_absolute() or '..' in Path(source).parts or source.startswith(('assets/local/', 'target/')):
                raise ValueError('Invalid HD source path')
            if not (root / source).is_file():
                raise ValueError('Missing required HD source: ' + source)
            result.add(source)
        pending.extend(row['path'] for row in info.get('catalog_extensions', []))
    return result


def source_bundle(destination, root=ROOT):
    """Deliver public build source, including current modifications, never local data."""
    if (root / '.git').exists():
        tracked = subprocess.check_output(['git', '-C', str(root), 'ls-files', '-z'], text=True).split('\0')
        new = subprocess.check_output(['git', '-C', str(root), 'ls-files', '--others', '--exclude-standard', '-z', '--', 'apps', 'crates', 'tools'], text=True).split('\0')
    else:
        # Extracted source distributions are rebuildable without Git metadata.
        # Users may edit the library or add modules before relinking it.
        tracked = [r['path'] for r in json.loads((root / 'SOURCE-MANIFEST.json').read_text())]
        new = [p.relative_to(root).as_posix() for base in ['apps', 'crates', 'tools'] for p in (root / base).rglob('*') if p.is_file()]
    names = set(tracked)
    names.update(n for n in new if Path(n).suffix in {'.rs', '.toml', '.js', '.css', '.html', '.py'} or n.startswith('tools/tests/fixtures/') and n.endswith('.json'))
    files = {}; modes = {}
    art_sources = runtime_art_sources(root)
    try:
        from .release_version import workspace_version
    except ImportError:
        from release_version import workspace_version
    version = workspace_version(root)
    public_roots = {'README.en.md', 'AGENTS.md', 'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'Makefile', 'README.md', 'LICENSE', 'THIRD_PARTY_NOTICES.txt', '.gitignore', '.github', 'apps', 'crates', 'tools', 'third_party', 'assets', 'mydocs'}
    for name in sorted(names):
        p = Path(name)
        if not name or p.is_absolute() or '..' in p.parts or p.parts[0] not in public_roots or '.git' in p.parts:
            continue
        if name.startswith(('assets/local/', 'target/')) or any(part.startswith('.') for part in p.parts[1:]):
            continue
        if name.startswith('assets/manifests/') and name.endswith('.jsonl'):
            continue  # Historical measurement/replay logs are not build inputs.
        if name.startswith('assets/remaster/') and name not in art_sources:
            continue
        if name.startswith('mydocs/') and not (version and name.startswith('mydocs/releases/issue-') and name.endswith(f'_v{version}.md')):
            continue
        path = root / name
        if path.is_symlink() or not path.is_file():
            continue
        # Reject linked directories too: every archived byte comes from this tree.
        if any(parent.is_symlink() for parent in path.parents if parent != root and root in parent.parents):
            continue
        files[name] = path.read_bytes()
        modes[name] = 0o755 if path.stat().st_mode & 0o111 else 0o644
    if not SOURCE_REQUIRED.issubset(files):
        raise ValueError('Audio source/license/relink materials are incomplete')
    manifest = [dict(path=n, bytes=len(b), mode=modes[n], sha256=hashlib.sha256(b).hexdigest()) for n, b in files.items()]
    files['SOURCE-MANIFEST.json'] = (json.dumps(manifest, separators=(',', ':')) + '\n').encode()
    raw = io.BytesIO()
    with gzip.GzipFile(filename='', mode='wb', fileobj=raw, mtime=0) as compressed:
        with tarfile.open(fileobj=compressed, mode='w') as archive:
            for name, data in sorted(files.items()):
                entry = tarfile.TarInfo(name); entry.size = len(data); entry.mode = modes.get(name, 0o644)
                archive.addfile(entry, io.BytesIO(data))
    data = raw.getvalue(); destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = destination.with_suffix(destination.suffix + '.tmp'); temporary.write_bytes(data); temporary.replace(destination)
    return dict(archive_sha256=hashlib.sha256(data).hexdigest(), archive_bytes=len(data), source_files=len(manifest))


def audio_delivery(output, root=ROOT):
    for name in ['LICENSE', 'THIRD_PARTY_NOTICES.txt']:
        (output / name).write_bytes((root / name).read_bytes())
    row = source_bundle(output / 'audio-source.tar.gz', root)
    license_bytes = (root / 'crates/lemmings-opl/LICENSE').read_bytes()
    (output / 'LGPL-2.1-or-later.txt').write_bytes(license_bytes)
    row.update(license='LGPL-2.1-or-later', license_sha256=hashlib.sha256(license_bytes).hexdigest(),
        wasm_sha256=hashlib.sha256((output / 'pkg/lemmings_web_bg.wasm').read_bytes()).hexdigest(),
        notice='audio-license.html', archive='audio-source.tar.gz')
    (output / 'audio-source.json').write_text(json.dumps(row, indent=2) + '\n')
    return row


def verify_bindgen(manifest, destination):
    path = destination / 'wasm-bindgen'
    if not path.is_file():
        raise ValueError('Missing pinned wasm-bindgen; run python3 tools/web_build.py fetch')
    checked_bytes(path.read_bytes(), manifest['binary'], 'wasm-bindgen')
    return path


def install_bindgen(manifest, destination, download=download_bytes):
    data = download(manifest['archive']['url'], manifest['archive']['size'])
    checked_bytes(data, manifest['archive'], 'wasm-bindgen archive')
    with tarfile.open(fileobj=io.BytesIO(data), mode='r:gz') as archive:
        entries = [e for e in archive.getmembers() if e.name == manifest['binary']['member']]
        if len(entries) != 1 or not entries[0].isfile() or entries[0].size != manifest['binary']['size']:
            raise ValueError('Pinned wasm-bindgen member missing, duplicated, or not a regular file')
        binary = archive.extractfile(entries[0]).read(manifest['binary']['size'] + 1)
    checked_bytes(binary, manifest['binary'], 'wasm-bindgen')
    destination.mkdir(parents=True, exist_ok=True)
    temporary = destination / 'wasm-bindgen.tmp'
    temporary.write_bytes(binary); temporary.chmod(0o755)
    temporary.replace(destination / 'wasm-bindgen')
    return verify_bindgen(manifest, destination)


def build(output=OUTPUT):
    manifest = json.loads(MANIFEST.read_text())
    tool = verify_bindgen(manifest, TOOLS)
    version = subprocess.check_output([str(tool), '--version'], text=True).strip()
    if version != 'wasm-bindgen ' + manifest['version']:
        raise ValueError('wasm-bindgen version does not match the pinned crate')
    subprocess.run(['cargo', 'build', '-p', 'lemmings-web', '--lib', '--release', '--target', 'wasm32-unknown-unknown', '--locked', '--offline'], cwd=ROOT, check=True)
    output.mkdir(parents=True, exist_ok=True)
    subprocess.run([str(tool), '--target', 'web', '--out-dir', str(output / 'pkg'), '--out-name', 'lemmings_web', str(ROOT / 'target/wasm32-unknown-unknown/release/lemmings_web.wasm')], check=True)
    for source in (ROOT / 'apps/web/www').iterdir():
        if source.is_file(): shutil.copyfile(source, output / source.name)
    try:
        from .hd_assets import runtime_pack
    except ImportError:
        from hd_assets import runtime_pack
    data=runtime_pack(root=ROOT)
    (output/'hd-assets.hdp').write_bytes(data)
    try:from .hd_catalog import runtime_catalog,write_files
    except ImportError:from hd_catalog import runtime_catalog,write_files
    members=runtime_catalog(ROOT,data)
    write_files(members,output/'hd-assets')
    try:from .web_hd import delivery,write_delivery
    except ImportError:from web_hd import delivery,write_delivery
    write_delivery(delivery(members),output)
    audio_delivery(output)
    print(json.dumps(dict(status='PASS',bindgen=version,output=str(output))))
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['fetch', 'build', 'serve'])
    parser.add_argument('--port', type=int, default=8080)
    args = parser.parse_args()
    if args.command == 'fetch':
        manifest = json.loads(MANIFEST.read_text())
        path = install_bindgen(manifest, TOOLS)
        print(path)
    elif args.command == 'build': build()
    else:
        if not (OUTPUT / 'index.html').is_file():
            raise ValueError('Missing web output; run make web first')
        handler = functools.partial(SimpleHTTPRequestHandler, directory=OUTPUT)
        with ThreadingHTTPServer(('127.0.0.1', args.port), handler) as server:
            print(f'Web preview: http://127.0.0.1:{args.port}/', flush=True)
            server.serve_forever()


if __name__ == '__main__': main()
