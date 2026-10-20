#!/usr/bin/env python3
"""Fetch the pinned official bindgen CLI, build offline, or serve the web entry."""
import argparse
import functools
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
