#!/usr/bin/env python3
"""Single version/tag/package naming contract; see the #124 release policy."""
import argparse
import json
from pathlib import Path
import re
import sys
import tomllib

ROOT=Path(__file__).resolve().parents[1]
VERSION=re.compile(r'(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-(alpha|beta|rc)\.([1-9][0-9]*))?')
TARGETS={'linux-x86_64':'tar.gz','web':'zip','windows-x86_64':'zip','macos-universal2':'zip'}


def parse_version(value):
    match=VERSION.fullmatch(value)
    if not match:raise ValueError(f'Invalid release version: {value!r}; use X.Y.Z or X.Y.Z-alpha/beta/rc.N')
    return dict(version=value,base='.'.join(match.groups()[:3]),prerelease=match[4] is not None)


def workspace_version(root=ROOT):
    value=tomllib.loads((root/'Cargo.toml').read_text())['workspace']['package']['version']
    return parse_version(value)['version']


def package_name(target,version):
    parse_version(version)
    if target not in TARGETS:raise ValueError(f'Unknown package target: {target}')
    return f'ohmylemmings-{version}-{target}.{TARGETS[target]}'


def check_repository(root=ROOT,tag=None):
    workspace=tomllib.loads((root/'Cargo.toml').read_text())['workspace']
    version=workspace_version(root)
    if tag is not None and tag!='v'+version:raise ValueError(f'Tag {tag!r} must equal v{version}')
    lock=tomllib.loads((root/'Cargo.lock').read_text())['package']
    for member in workspace['members']:
        package=tomllib.loads((root/member/'Cargo.toml').read_text())['package']
        if package.get('version')!={'workspace':True}:raise ValueError(f'{member}: inherit workspace version')
        rows=[row for row in lock if row['name']==package['name'] and 'source' not in row]
        if len(rows)!=1 or rows[0]['version']!=version:raise ValueError(f'{member}: stale Cargo.lock version')
    if len(list((root/'mydocs/releases').glob(f'issue-*_v{version}.md')))!=1:
        raise ValueError(f'Expected one issue-linked release note for {version}')
    return dict(status='PASS',**parse_version(version),tag='v'+version,workspace_members=len(workspace['members']))


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command',choices=['version','check','path'])
    parser.add_argument('--tag')
    parser.add_argument('--target',choices=list(TARGETS))
    parser.add_argument('--directory',type=Path,default=ROOT/'assets/local/dist')
    args=parser.parse_args()
    if args.command=='version':print(workspace_version())
    elif args.command=='check':print(json.dumps(check_repository(tag=args.tag)))
    else:
        if args.target is None:parser.error('--target is required')
        print(args.directory/package_name(args.target,workspace_version()))
