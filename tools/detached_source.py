"""Separate corresponding source at publication, preserving every runtime byte/mode."""
import gzip
import hashlib
import io
import json
from pathlib import Path
import tarfile
import tempfile
import zipfile

RELEASE_BASE = 'https://github.com/edwardkim/ohmylemmings/releases/download/'


def require(ok, message):
    if not ok:
        raise ValueError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def encoded(value):
    return (json.dumps(value, indent=2, sort_keys=True) + '\n').encode()


def source_location(version):
    from tools.release_version import parse_version
    parse_version(version)
    name = f'ohmylemmings-{version}-source.tar.gz'
    return name, f'{RELEASE_BASE}v{version}/{name}'


def read_package(path):
    require(path.is_file() and not path.is_symlink() and not any(p.is_symlink() for p in path.parents),
            'Missing or linked package input')
    files, modes = {}, {}
    if path.suffix == '.zip':
        with zipfile.ZipFile(path) as archive:
            entries = [(i.filename, i.external_attr >> 16, archive.read(i)) for i in archive.infolist()]
    else:
        with tarfile.open(path, 'r:gz') as archive:
            require(all(i.isfile() for i in archive.getmembers()), 'Non-file archive member')
            entries = [(i.name, 0o100000 | i.mode, archive.extractfile(i).read()) for i in archive.getmembers()]
    for name, mode, data in entries:
        require(name not in files and not Path(name).is_absolute() and '..' not in Path(name).parts
                and '\\' not in name and mode in (0o100644, 0o100755), 'Invalid archive member')
        files[name], modes[name] = data, mode
    return files, modes


def write_package(path, files, modes):
    output = io.BytesIO()
    if path.suffix == '.zip':
        with zipfile.ZipFile(output, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
            for name, data in sorted(files.items()):
                entry = zipfile.ZipInfo(name, (1980, 1, 1, 0, 0, 0))
                entry.create_system = 3
                entry.external_attr = modes.get(name, 0o100644) << 16
                entry.compress_type = zipfile.ZIP_DEFLATED
                archive.writestr(entry, data, compresslevel=9)
    else:
        with gzip.GzipFile(filename='', fileobj=output, mode='wb', mtime=0) as stream:
            with tarfile.open(fileobj=stream, mode='w') as archive:
                for name, data in sorted(files.items()):
                    entry = tarfile.TarInfo(name)
                    entry.size, entry.mode = len(data), modes.get(name, 0o100644) & 0o777
                    archive.addfile(entry, io.BytesIO(data))
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(output.getvalue())
    Path(str(path) + '.sha256').write_text(f'{digest(output.getvalue())}  {path.name}\n')


def repin(files, manifest):
    manifest['files'] = [dict(path=n, bytes=len(b), sha256=digest(b)) for n, b in sorted(files.items())
                         if n not in ('PACKAGE-MANIFEST.json', 'SHA256SUMS')]
    files['PACKAGE-MANIFEST.json'] = encoded(manifest)
    files['SHA256SUMS'] = ''.join(f'{digest(b)}  {n}\n' for n, b in sorted(files.items()) if n != 'SHA256SUMS').encode()


def source_bytes(path, files, manifest):
    """Offline verification requires the sibling source asset; never fetch arbitrary URLs."""
    row = manifest['source_delivery']
    name, url = source_location(manifest['version'])
    require(row == dict(mode='external', path=name, url=url,
                        bytes=manifest['source']['archive_bytes'], sha256=manifest['source']['archive_sha256']),
            'Invalid external source location or pin')
    local = path.parent / name
    require(local.is_file() and not local.is_symlink(), 'Missing corresponding source asset: ' + name)
    raw = local.read_bytes()
    require(len(raw) == row['bytes'] and digest(raw) == row['sha256'], 'External source checksum differs')
    require(url.encode() in files['REBUILD.md'], 'Missing external source instructions')
    if manifest['target'] == 'web':
        info = json.loads(files['audio-source.json'])
        require(info['archive'] == url and info['archive_sha256'] == row['sha256']
                and info['archive_bytes'] == row['bytes'] and url.encode() in files['audio-license.html'],
                'Web external source link differs')
    return raw


def source_identity(raw):
    """Check actual source bytes; compression headers and host modes are not content."""
    with tarfile.open(fileobj=io.BytesIO(raw), mode='r:gz') as archive:
        entries = archive.getmembers()
        require(all(e.isfile() and not Path(e.name).is_absolute() and '..' not in Path(e.name).parts
                    and not e.name.startswith(('assets/local/', 'target/')) for e in entries), 'Invalid source member')
        pins = json.load(archive.extractfile('SOURCE-MANIFEST.json'))
        names = {p['path'] for p in pins}
        require(len(names) == len(pins) and len(entries) == len(names) + 1
                and {e.name for e in entries} == names | {'SOURCE-MANIFEST.json'}, 'Source inventory differs')
        result = {}
        for pin in pins:
            data = archive.extractfile(pin['path']).read()
            require(len(data) == pin['bytes'] and digest(data) == pin['sha256'], 'Source content checksum differs')
            result[pin['path']] = (len(data), digest(data))
        return result


def detach(path, output, *, canonical_source=None):
    """Called only after the embedded CI package has passed its full verifier/smoke."""
    require(not output.exists() and not Path(str(output) + '.sha256').exists(), 'Output already exists')
    files, modes = read_package(path)
    manifest = json.loads(files['PACKAGE-MANIFEST.json'])
    require('source_delivery' not in manifest, 'Package source is already external')
    embedded = 'audio-source.tar.gz' if manifest['target'] == 'web' else 'source.tar.gz'
    raw = files.pop(embedded)
    require(digest(raw) == manifest['source']['archive_sha256'] and len(raw) == manifest['source']['archive_bytes'],
            'Embedded source checksum differs')
    if canonical_source is not None:
        identity = source_identity(raw)
        require(identity == source_identity(canonical_source), 'Platform source contents differ')
        raw = canonical_source
        manifest['source'] = dict(archive_sha256=digest(raw), archive_bytes=len(raw), source_files=len(identity))
    name, url = source_location(manifest['version'])
    manifest['source_delivery'] = dict(mode='external', path=name, url=url, bytes=len(raw), sha256=digest(raw))
    notice = ('Corresponding source is a separate download (required to modify/relink):\n'
              f'{url}\nSHA-256: {digest(raw)}\nBytes: {len(raw)}\n'
              f'Download and rename it to {embedded} before following the extraction instructions.\n\n').encode()
    for member in ['REBUILD.md', 'README.md', 'README.txt']:
        if member in files:
            files[member] = notice + files[member]
    if manifest['target'] == 'web':
        info = json.loads(files['audio-source.json']);info['archive'] = url
        info.update(manifest['source'])
        files['audio-source.json'] = encoded(info)
        # A visible link is also supplied if a synthetic fixture has no HTML anchor.
        files['audio-license.html'] = files['audio-license.html'].replace(b'./audio-source.tar.gz', url.encode())
        if url.encode() not in files['audio-license.html']:
            files['audio-license.html'] += f'\n<a href="{url}">Corresponding source</a>\n'.encode()
    repin(files, manifest)
    output.parent.mkdir(parents=True, exist_ok=True)
    source = output.parent / name
    require(not source.exists() or source.read_bytes() == raw, 'Conflicting corresponding source asset')
    source.write_bytes(raw)
    write_package(output, files, modes)
    return output


def verify_external(path, root, verifier):
    """Validate detached pins, then reuse each platform's full existing validator."""
    if path.suffix == '.zip':
        with zipfile.ZipFile(path) as archive:
            manifest = json.loads(archive.read('PACKAGE-MANIFEST.json'))
    else:
        with tarfile.open(path, 'r:gz') as archive:
            manifest = json.load(archive.extractfile('PACKAGE-MANIFEST.json'))
    if 'source_delivery' not in manifest:
        return None
    files, modes = read_package(path)
    raw = path.read_bytes()
    require(Path(str(path) + '.sha256').read_text() == f'{digest(raw)}  {path.name}\n', 'Archive checksum differs')
    pins = manifest['files']
    require(len(pins) == len(files) - 2 and {p['path'] for p in pins} == set(files) - {'PACKAGE-MANIFEST.json', 'SHA256SUMS'},
            'Detached package inventory differs')
    for pin in pins:
        value = files[pin['path']]
        require(len(value) == pin['bytes'] and digest(value) == pin['sha256'], 'Detached package checksum differs')
    require(files['SHA256SUMS'] == ''.join(f'{digest(b)}  {n}\n' for n, b in sorted(files.items()) if n != 'SHA256SUMS').encode(),
            'Detached internal checksums differ')
    source = source_bytes(path, files, manifest)
    embedded = 'audio-source.tar.gz' if manifest['target'] == 'web' else 'source.tar.gz'
    require(embedded not in files, 'Redundant embedded source')
    files[embedded] = source
    del manifest['source_delivery']
    repin(files, manifest)
    with tempfile.TemporaryDirectory() as temp:
        hydrated = Path(temp) / path.name
        write_package(hydrated, files, modes)
        report = verifier(hydrated, root=root)
    report.update(archive_bytes=len(raw), sha256=digest(raw), source_delivery='external')
    if 'files' in report:
        report['files'] -= 1
    return report
