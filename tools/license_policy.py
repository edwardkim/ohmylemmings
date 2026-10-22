#!/usr/bin/env python3
"""Check the owner's MIT choice, unchanged upstream notices and locked dependency inventory."""
import hashlib
import json
from pathlib import Path
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def checked(root, pin):
    path = root / pin['path']
    require(not Path(pin['path']).is_absolute() and '..' not in Path(pin['path']).parts,
            'Unsafe license path')
    require(path.is_file() and not path.is_symlink()
            and not any(p.is_symlink() for p in path.parents), 'Missing or linked license material')
    data = path.read_bytes()
    require(len(data) == pin['bytes'] and hashlib.sha256(data).hexdigest() == pin['sha256'],
            'License bytes differ: ' + pin['path'])
    return data


def check(root=ROOT):
    info = json.loads((root / 'assets/manifests/license_inventory.json').read_text())
    require(info['schema_version'] == 1 and info['project_license'] == 'MIT', 'Invalid license policy')
    checked(root, info['project_license_file'])
    bundle = checked(root, info['notice_bundle'])
    paths = [pin['path'] for pin in info['notices']]
    require(len(paths) == len(set(paths)), 'Duplicate notice inventory')
    actual = {p.relative_to(root).as_posix() for p in (root / 'third_party').rglob('*')
              if p.is_file() and p.name != 'README.md'}
    require(actual == set(paths), 'Unreviewed or missing third-party notices')
    for pin in info['notices']:
        require(checked(root, pin) in bundle, 'License missing from delivered notice bundle')
    for pin in info['retained_library_notices']:
        checked(root, pin)
    references = json.loads((root / 'assets/manifests/reference_sources.json').read_text())['files']
    for name in ['lemmix__Licence.txt', 'lemmings-ts__LICENSE']:
        pin = next(p for p in info['notices'] if p['path'] == 'third_party/' + name)
        reference = next(p for p in references if p['name'] == name)
        require(pin['sha256'] == reference['sha256'] and pin['bytes'] == reference['size'],
                'Preserve the locked upstream reference notice')
    lock = tomllib.loads((root / 'Cargo.lock').read_text())['package']
    expected = {(p['name'], p['version'], p['checksum']) for p in lock if 'source' in p}
    recorded = {(p['name'], p['version'], p['checksum']) for p in info['cargo_dependencies']}
    require(len(recorded) == len(info['cargo_dependencies']) and expected == recorded,
            'Review license inventory after changing Cargo.lock')
    for package in info['cargo_dependencies']:
        require(package['license'] and package['notices'] and set(package['notices']) <= set(paths),
                'Dependency license material is missing')
    workspace = tomllib.loads((root / 'Cargo.toml').read_text())['workspace']
    require(workspace['package']['license'] == 'MIT', 'Workspace must declare MIT')
    inherited = 0
    for member in workspace['members']:
        package = tomllib.loads((root / member / 'Cargo.toml').read_text())['package']
        if member == 'crates/lemmings-opl':
            require(package['license'] == 'LGPL-2.1-or-later', 'Preserve the derived OPL license')
        else:
            require(package['license'] == {'workspace': True}, 'Inherit MIT for project crate: ' + member)
            inherited += 1
    return dict(status='PASS', project_license='MIT', inherited_crates=inherited,
                cargo_dependencies=len(recorded), third_party_notices=len(paths),
                scope='Recorded license/source delivery; original DOS data and third-party terms remain separate')


if __name__ == '__main__':
    print(json.dumps(check()))
