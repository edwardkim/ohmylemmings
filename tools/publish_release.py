#!/usr/bin/env python3
"""Promote a successful same-tree CI build to a versioned, verified GitHub release."""
import argparse
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
from urllib.request import urlopen
import zipfile

ROOT=Path(__file__).resolve().parents[1]
if __package__ in (None,''):sys.path.insert(0,str(ROOT))
from tools.release_version import check_repository, package_name, TARGETS
from tools.release_package import encoded, require, sha, verify_package
from tools.web_build import source_bundle
from tools.windows_package import verify as verify_windows
from tools.macos_package import verify as verify_macos

JOBS={'Rust native','Python tooling','WASM libraries','Web browser','Windows full game',
      'macOS universal app (Apple Silicon)','macOS universal app (Intel)'}
ARTIFACTS={'linux-x86_64':'development-linux-x86_64','web':'development-web',
           'windows-x86_64':'development-windows-x86_64','macos-universal2':'development-macos-universal2'}
EVIDENCE={'linux-x86_64':'linux-package-check','web':'web-package-check',
          'windows-x86_64':'windows-package-check','macos-universal2':'macos-package-check'}


def command(*args):
    return subprocess.check_output(list(map(str,args)),cwd=ROOT,text=True).strip()


def api(path):return json.loads(command('gh','api',path))


def tree(commit):return command('git','rev-parse',commit+'^{tree}')


def validate_run(run,jobs,repository,build_tree,release_tree):
    require(run['name']=='CI' and run['conclusion']=='success' and run['head_repository']['full_name']==repository,
            'Expected successful CI from this repository')
    require(build_tree==release_tree,'Build source tree differs from release tag')
    require({row['name'] for row in jobs}==JOBS and len(jobs)==len(JOBS)
            and all(row['conclusion']=='success' for row in jobs),'All seven CI jobs must pass')


def validate_inventory(rows,version):
    require(len(rows)==len(TARGETS) and {row['target'] for row in rows}==set(TARGETS)
            and all(row['version']==version for row in rows),'All four packages must share the release version')


def verify_sources(raw,expected):
    expected={p['path']:p for p in expected}
    with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as archive:
        entries=archive.getmembers()
        require(all(e.isfile() for e in entries),'Non-file source member')
        require(len(entries)==len(expected)+1 and {e.name for e in entries}==set(expected)|{'SOURCE-MANIFEST.json'},
                'Corresponding source inventory differs from release tree')
        pins=json.load(archive.extractfile('SOURCE-MANIFEST.json'))
        require(len(pins)==len(expected) and {p['path'] for p in pins}==set(expected),'Source manifest inventory differs')
        for pin in pins:
            data=archive.extractfile(pin['path']).read();known=expected[pin['path']]
            require(len(data)==known['bytes']==pin['bytes'] and sha(data)==known['sha256']==pin['sha256'],
                    'Corresponding source differs from release tree: '+pin['path'])
    return len(expected)


def prepare(tag,run_id,output):
    version=check_repository(tag=tag)['version']
    require(not command('git','status','--porcelain','--untracked-files=no'),'Commit tracked changes before release preparation')
    head=command('git','rev-parse','HEAD');release_tree=tree('HEAD')
    repository=json.loads(command('gh','repo','view','--json','nameWithOwner'))['nameWithOwner']
    if run_id is None:
        candidates=json.loads(command('gh','run','list','--workflow','CI','--status','success','--limit','30',
                                      '--json','databaseId,headSha'))
        for candidate in candidates:
            try:matches=tree(candidate['headSha'])==release_tree
            except subprocess.CalledProcessError:continue
            if matches:run_id=candidate['databaseId'];break
        require(run_id is not None,'No successful same-tree CI run found')
    run=api(f'repos/{repository}/actions/runs/{run_id}')
    jobs=api(f'repos/{repository}/actions/runs/{run_id}/jobs?per_page=100')['jobs']
    validate_run(run,jobs,repository,tree(run['head_sha']),release_tree)
    artifacts=api(f'repos/{repository}/actions/runs/{run_id}/artifacts?per_page=100')['artifacts']
    output.mkdir(parents=True)
    files={};packages=[];builds=[]
    with tempfile.TemporaryDirectory(dir=output,prefix='build-inputs-') as temporary:
        temporary=Path(temporary)
        source_bundle(temporary/'expected-source.tar.gz')
        with tarfile.open(temporary/'expected-source.tar.gz') as archive:
            expected=json.load(archive.extractfile('SOURCE-MANIFEST.json'))
        for target,name in ARTIFACTS.items():
            rows=[row for row in artifacts if row['name']==name and not row['expired']]
            require(len(rows)==1,'Missing or ambiguous CI artifact: '+name);artifact=rows[0]
            wrapper=temporary/(target+'.zip')
            with wrapper.open('xb') as stream:
                subprocess.run(['gh','api',f'repos/{repository}/actions/artifacts/{artifact["id"]}/zip'],
                               cwd=ROOT,stdout=stream,check=True)
            require('sha256:'+sha(wrapper.read_bytes())==artifact['digest'],'CI artifact digest differs')
            installed=temporary/target
            with zipfile.ZipFile(wrapper) as archive:
                require(all(not Path(n).is_absolute() and '..' not in Path(n).parts and '\\' not in n for n in archive.namelist()),
                        'Invalid artifact paths')
                archive.extractall(installed)
            path=installed/'dist'/package_name(target,version)
            if target=='windows-x86_64':report=verify_windows(path)
            elif target=='macos-universal2':report=verify_macos(path)
            else:report=verify_package(path)
            if path.suffix=='.zip':
                with zipfile.ZipFile(path) as archive:
                    manifest=json.loads(archive.read('PACKAGE-MANIFEST.json'))
                    source=archive.read('audio-source.tar.gz' if target=='web' else 'source.tar.gz')
            else:
                with tarfile.open(path) as archive:
                    manifest=json.load(archive.extractfile('PACKAGE-MANIFEST.json'))
                    source=archive.extractfile('source.tar.gz').read()
            require(manifest['version']==version and manifest['original_game_data'] is True,'Package version/data differs')
            count=verify_sources(source,expected)
            evidence=json.loads((installed/EVIDENCE[target]/'summary.json').read_text())
            smoke=evidence.get('smoke',evidence)
            require(evidence['package']['sha256']==report['sha256'] and smoke['status']=='PASS'
                    and smoke['original_levels_loaded']==120,'CI install evidence does not match package')
            files[path.name]=path.read_bytes()
            files[path.name+'.sha256']=Path(str(path)+'.sha256').read_bytes()
            if target=='linux-x86_64':files[f'ohmylemmings-{version}-source.tar.gz']=source
            packages.append(dict(target=target,version=version,path=path.name,sha256=report['sha256'],source_files=count))
            builds.append(dict(target=target,artifact_id=artifact['id'],artifact_digest=artifact['digest']))
    validate_inventory(packages,version)
    notes=next((ROOT/'mydocs/releases').glob(f'issue-*_v{version}.md'))
    files['PLAY.md']=notes.read_bytes()
    files['RELEASE-MANIFEST.json']=encoded(dict(schema_version=1,tag=tag,version=version,repository=repository,release_commit=head,
        source_tree=release_tree,ci_run=run_id,ci_commit=run['head_sha'],ci_url=run['html_url'],
        packages=packages,artifacts=builds,files=[dict(path=n,bytes=len(b),sha256=sha(b)) for n,b in sorted(files.items())]))
    files['SHA256SUMS']=''.join(f'{sha(b)}  {n}\n' for n,b in sorted(files.items())).encode()
    for name,data in files.items():(output/name).write_bytes(data)
    print(json.dumps(dict(status='PREPARED',tag=tag,ci_run=run_id,files=len(files),output=str(output))))


def checked_files(directory):
    info=json.loads((directory/'RELEASE-MANIFEST.json').read_text())
    version=check_repository(tag=info['tag'])['version']
    require(info['version']==version and info['release_commit']==command('git','rev-parse','HEAD')
            and info['source_tree']==tree('HEAD'),'Prepared release differs from current checkout')
    validate_inventory(info['packages'],version)
    expected={pin['path']:pin for pin in info['files']}
    require(set(expected)=={package_name(t,version) for t in TARGETS}
            |{package_name(t,version)+'.sha256' for t in TARGETS}
            |{f'ohmylemmings-{version}-source.tar.gz','PLAY.md'},'Unexpected release inventory')
    files={}
    for name,pin in expected.items():
        data=(directory/name).read_bytes()
        require(len(data)==pin['bytes'] and sha(data)==pin['sha256'],'Prepared file changed: '+name)
        files[name]=data
    files['RELEASE-MANIFEST.json']=(directory/'RELEASE-MANIFEST.json').read_bytes()
    sums=''.join(f'{sha(b)}  {n}\n' for n,b in sorted(files.items())).encode()
    require((directory/'SHA256SUMS').read_bytes()==sums,'Prepared checksums differ')
    files['SHA256SUMS']=sums
    return info,files


def publish(directory):
    info,files=checked_files(directory);tag=info['tag']
    require(command('git','cat-file','-t',tag)=='tag','Release requires an annotated tag')
    require(command('git','rev-parse',tag+'^{}')==info['release_commit'],'Tag target differs from prepared release')
    require(command('git','merge-base','--is-ancestor',tag,'origin/main')=='','Release tag must be on main')
    remote=command('git','ls-remote','--tags','origin','refs/tags/'+tag)
    require(remote.split()[0]==command('git','rev-parse',tag),'Remote annotated tag differs')
    prerelease=check_repository(tag=tag)['prerelease']
    command('gh','release','create',tag,'--draft','--verify-tag','--title','Oh My Lemmings '+info['version'],
            '--notes-file',directory/'PLAY.md',*(['--prerelease'] if prerelease else []),
            *[directory/n for n in sorted(files)])
    with tempfile.TemporaryDirectory(dir=directory,prefix='verify-draft-') as temporary:
        command('gh','release','download',tag,'--dir',temporary)
        require({p.name for p in Path(temporary).iterdir()}==set(files),'Draft asset inventory differs')
        for name,data in files.items():require((Path(temporary)/name).read_bytes()==data,'Uploaded draft differs: '+name)
    command('gh','release','edit',tag,'--draft=false','--latest='+str(not prerelease).lower())
    release=json.loads(command('gh','release','view',tag,'--json','url,isDraft,isPrerelease,tagName,assets'))
    require(not release['isDraft'] and release['tagName']==tag and release['isPrerelease']==prerelease,'Release publication differs')
    require({asset['name'] for asset in release['assets']}==set(files),'Published assets differ')
    # Download through public URLs without gh credentials, as a player would.
    assets=api(f'repos/{info["repository"]}/releases/tags/{tag}')['assets']
    for asset in assets:
        with urlopen(asset['browser_download_url'],timeout=60) as response:download=response.read(len(files[asset['name']])+1)
        require(download==files[asset['name']],'Public release download differs: '+asset['name'])
    (directory/'published.json').write_bytes(encoded(dict(status='PUBLISHED',**release)))
    print(json.dumps(dict(status='PUBLISHED',url=release['url'],tag=tag,assets=len(files),anonymous_downloads='PASS')))


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command',choices=['prepare','publish'])
    parser.add_argument('--tag')
    parser.add_argument('--run',type=int)
    parser.add_argument('--directory',type=Path,required=True)
    args=parser.parse_args()
    if args.command=='prepare':
        if args.tag is None:parser.error('--tag is required')
        prepare(args.tag,args.run,args.directory.resolve())
    else:publish(args.directory.resolve())
