#!/usr/bin/env python3
"""Build and verify a universal2 macOS full-game app with a private SDL runtime."""
import argparse
import io
import json
from pathlib import Path
import plistlib
import struct
import subprocess
import sys
import tarfile
import tempfile
import zipfile

ROOT=Path(__file__).resolve().parents[1]
if __package__ in (None,''):sys.path.insert(0,str(ROOT))
from tools.reference_assets import checked_bytes, download_bytes
from tools.release_package import encoded, regular, require, sha, project_license_files, verify_project_license, PROJECT_LICENSE_FILES
from tools.web_build import source_bundle
from tools.release_version import workspace_version, package_name, parse_version
from tools.windows_package import GAME_FILES, game_pins, manifest

APP='OhMyLemmings.app'
EXE=APP+'/Contents/MacOS/OhMyLemmings'
LIB=APP+'/Contents/Frameworks/libSDL2-2.0.0.dylib'
RES=APP+'/Contents/Resources/'
SIGNATURE=APP+'/Contents/_CodeSignature/CodeResources'
APP_FILES={EXE,LIB,APP+'/Contents/Info.plist',SIGNATURE,RES+'GAME-DATA.json',
           RES+'SDL2-LICENSE.txt',RES+'LGPL-2.1-or-later.txt'} | {RES+'data/'+n for n in GAME_FILES}
LICENSE_APP_FILES={RES+n for n in PROJECT_LICENSE_FILES}
OUTSIDE={'README.txt','REBUILD.md','source.tar.gz','PACKAGE-MANIFEST.json','SHA256SUMS'}


def universal_arches(data,kind=2):
    require(len(data)>=48 and struct.unpack_from('>II',data)==(0xcafebabe,2),'Expected universal2 Mach-O')
    slices=[];cpus=set()
    for index in range(2):
        cpu,_,offset,size,alignment=struct.unpack_from('>IIIII',data,8+index*20)
        require(cpu in (0x1000007,0x100000c) and cpu not in cpus and alignment<=30
                and offset>=48 and offset%(1<<alignment)==0 and size>=32 and offset+size<=len(data),
                'Invalid universal2 slice')
        header=struct.unpack_from('<IIII',data,offset)
        require(header[0]==0xfeedfacf and header[1]==cpu and header[3]==kind,'Wrong Mach-O slice/type')
        cpus.add(cpu);slices.append((offset,offset+size))
    require(slices[0][1]<=slices[1][0] or slices[1][1]<=slices[0][0],'Overlapping Mach-O slices')
    return ['arm64','x86_64']


def prepare_sdl(destination,root=ROOT):
    require(sys.platform=='darwin','SDL macOS build requires an actual Mac SDK')
    pin=manifest(root,'macos_runtime.json')
    raw=checked_bytes(download_bytes(pin['archive']['url'],pin['archive']['size']),pin['archive'],'SDL source')
    destination.mkdir(parents=True,exist_ok=True)
    archive=destination/'SDL2-source.tar.gz';archive.write_bytes(raw)
    with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as source:
        source.extractall(destination/'source',filter='data')
    source=destination/'source'/('SDL2-'+pin['version'])
    license=checked_bytes(regular(source/'LICENSE.txt'),pin['license'],'SDL license')
    subprocess.run(['cmake','-S',str(source),'-B',str(destination/'build'),
        '-DCMAKE_BUILD_TYPE=Release','-DCMAKE_POLICY_VERSION_MINIMUM=3.5',
        '-DCMAKE_OSX_ARCHITECTURES=arm64;x86_64','-DCMAKE_OSX_DEPLOYMENT_TARGET=13.0',
        '-DCMAKE_INSTALL_PREFIX='+str(destination/'install'),'-DCMAKE_INSTALL_NAME_DIR=@rpath',
        '-DSDL_SHARED=ON','-DSDL_STATIC=OFF','-DSDL_TEST=OFF','-DSDL_TESTS=OFF'],check=True)
    subprocess.run(['cmake','--build',str(destination/'build'),'--parallel','3'],check=True)
    subprocess.run(['cmake','--install',str(destination/'build')],check=True)
    library=regular(destination/'install/lib/libSDL2-2.0.0.dylib');universal_arches(library,6)
    (destination/'libSDL2-2.0.0.dylib').write_bytes(library)
    (destination/'SDL2-LICENSE.txt').write_bytes(license)


def app_info(version):
    return dict(CFBundleExecutable='OhMyLemmings',CFBundleIdentifier='com.github.edwardkim.ohmylemmings',
        CFBundleName='Oh My Lemmings',CFBundleDisplayName='Oh My Lemmings',CFBundlePackageType='APPL',
        CFBundleShortVersionString=parse_version(version)['base'],CFBundleVersion=parse_version(version)['base'],
        CFBundleGetInfoString='Oh My Lemmings '+version,LSMinimumSystemVersion='13.0',
        NSHighResolutionCapable=True,NSPrincipalClass='NSApplication')


def stage(binary,assets,sdl,destination,root=ROOT):
    app=destination/APP
    if app.exists():raise FileExistsError(app)
    version=workspace_version(root)
    files={EXE:regular(binary),LIB:regular(sdl/'libSDL2-2.0.0.dylib'),
           APP+'/Contents/Info.plist':plistlib.dumps(app_info(version)),
           RES+'SDL2-LICENSE.txt':checked_bytes(regular(sdl/'SDL2-LICENSE.txt'),manifest(root,'macos_runtime.json')['license'],'SDL license'),
           RES+'LGPL-2.1-or-later.txt':regular(root/'crates/lemmings-opl/LICENSE')}
    files.update({RES+n:data for n,data in project_license_files(root).items()})
    universal_arches(files[EXE]);universal_arches(files[LIB],6)
    original,pins=game_pins(root)
    files[RES+'GAME-DATA.json']=encoded(dict(profile=original['profile'],original_game_data=True,levels=120,files=pins,sources=original.get('sources',[])))
    for pin in pins:files[RES+'data/'+pin['name']]=checked_bytes(regular(assets/pin['name']),pin,pin['name'])
    for name,data in files.items():
        path=destination/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(data)
        path.chmod(0o755 if name in (EXE,LIB) else 0o644)
    return app


def validate_app(files,version,root):
    universal_arches(files[EXE]);universal_arches(files[LIB],6)
    require(plistlib.loads(files[APP+'/Contents/Info.plist'])==app_info(version),'Application Info.plist differs')
    original,pins=game_pins(root)
    require(json.loads(files[RES+'GAME-DATA.json'])==dict(profile=original['profile'],original_game_data=True,levels=120,files=pins,sources=original.get('sources',[])),'Mac game provenance differs')
    for pin in pins:checked_bytes(files[RES+'data/'+pin['name']],pin,pin['name'])
    checked_bytes(files[RES+'SDL2-LICENSE.txt'],manifest(root,'macos_runtime.json')['license'],'SDL license')
    require(files[RES+'LGPL-2.1-or-later.txt']==regular(root/'crates/lemmings-opl/LICENSE'),'OPL license differs')


def create(app,output,root=ROOT):
    version=workspace_version(root)
    path=output/package_name('macos-universal2',version);sidecar=Path(str(path)+'.sha256')
    if path.exists() or sidecar.exists():raise FileExistsError(path)
    require(app.name==APP and not app.is_symlink(),'Invalid application directory')
    require(all(not p.is_symlink() for p in app.rglob('*')),'Application contains links')
    names={APP+'/'+p.relative_to(app).as_posix() for p in app.rglob('*') if not p.is_dir()}
    require(names==APP_FILES|LICENSE_APP_FILES,'Unexpected/missing app files or signature')
    files={n:regular(app.parent/n) for n in sorted(names)}
    validate_app(files,version,root)
    verify_project_license(files,dict(project_license='MIT'),root,RES)
    files['README.txt']=(
        'Oh My Lemmings - macOS universal2 (Apple Silicon / Intel, macOS 13+)\n\n'
        'ZIP을 모두 풀고 OhMyLemmings.app을 Applications에 복사한 뒤 여세요.\n'
        '원본 그래픽·120개 레벨·음원이 포함되며 Fun01을 바로 시작합니다. SDL 설치는 필요 없습니다.\n'
        '현재 시험 패키지는 ad-hoc 서명이며 Apple Developer ID/공증은 아직 적용하지 않았습니다.\n'
        '최초 다운로드 실행이 차단되면 앱을 열어 본 뒤 시스템 설정 > 개인정보 보호 및 보안 > 그래도 열기를 사용합니다.\n'
        'Apple 안내: https://support.apple.com/102445\n'
        '앱 전체를 유지하세요. 오류 로그: ~/Library/Logs/OhMyLemmings/last-error.log\n'
        'F/C/E/K/U/M/B/D: 기술 선택, 왼쪽 클릭: 할당, Space: 일시정지, S: 소리, Esc: 종료\n'
        '결과 화면 R: 재시작, Enter: 다음 레벨. 고급: Contents/MacOS/OhMyLemmings --difficulty tricky --level 1\n'
        'SDL2-LICENSE.txt / LGPL-2.1-or-later.txt / GAME-DATA.json은 앱 Contents/Resources에 있습니다.\n'
        '자체 코드는 MIT(LICENSE), 제3자 고지는 THIRD_PARTY_NOTICES.txt를 따릅니다. 원본 게임 데이터/상표는 MIT 대상이 아닙니다.\n'
        'OPL Copyright (C) 2013-2020 Nuke.YKT; (C) 2026 Tony Gies (Rust port).\n'
        '대응 소스와 수정·재링크 안내: source.tar.gz 및 REBUILD.md. 성능 판정은 #113에서 별도로 진행합니다.\n').encode()
    files['REBUILD.md']=(
        '# Corresponding source and relinking\n\n'
        'Extract source.tar.gz; install Rust 1.93.1, Python 3.12+, Xcode SDK and CMake.\n'
        'Read crates/lemmings-opl/README.md and UPSTREAM.patch.\n'
        'Use tools/macos_package.py prepare-sdl and the pinned macOS CI build commands.\n'
        'Build both aarch64-apple-darwin/x86_64-apple-darwin, merge with lipo, stage the app,\n'
        'sign its dylib and app, and create the ZIP. Read mydocs/reports/issue-120_macos_full_game.md.\n'
        'Keep original data separate from public source. Library modification/relinking and reverse\n'
        'engineering to debug those modifications are permitted. Re-sign your modified local app.\n'
        'Developer ID signing/notarization requires your own Apple account and credentials.\n').encode()
    with tempfile.TemporaryDirectory() as temporary:
        source=Path(temporary)/'source.tar.gz';info=source_bundle(source,root);files['source.tar.gz']=source.read_bytes()
    files['PACKAGE-MANIFEST.json']=encoded(dict(schema_version=1,version=version,target='macos-universal2',project_license='MIT',
        original_game_data=True,architectures=['arm64','x86_64'],minimum_macos='13.0',
        signing='ad-hoc',notarized=False,source=info,
        files=[dict(path=n,bytes=len(b),sha256=sha(b)) for n,b in sorted(files.items())]))
    files['SHA256SUMS']=''.join(f'{sha(b)}  {n}\n' for n,b in sorted(files.items())).encode()
    output.mkdir(parents=True,exist_ok=True)
    with path.open('xb') as stream:
        with zipfile.ZipFile(stream,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=9) as z:
            for name,data in sorted(files.items()):
                entry=zipfile.ZipInfo(name,(1980,1,1,0,0,0));entry.create_system=3
                entry.external_attr=(0o100755 if name in (EXE,LIB) else 0o100644)<<16
                entry.compress_type=zipfile.ZIP_DEFLATED;z.writestr(entry,data,compresslevel=9)
    sidecar.write_bytes(f'{sha(path.read_bytes())}  {path.name}\n'.encode())
    verify(path,root=root)
    return path


def verify(path,root=ROOT):
    raw=regular(path);require(regular(Path(str(path)+'.sha256'))==f'{sha(raw)}  {path.name}\n'.encode(),'ZIP checksum differs')
    with zipfile.ZipFile(io.BytesIO(raw)) as z:
        package=json.loads(z.read('PACKAGE-MANIFEST.json'))
        licensed=LICENSE_APP_FILES if 'project_license' in package else set()
        names=z.namelist();require(len(names)==len(set(names)) and set(names)==APP_FILES|OUTSIDE|licensed,'Invalid Mac ZIP inventory')
        for info in z.infolist():
            require(info.external_attr>>16==(0o100755 if info.filename in (EXE,LIB) else 0o100644),'Invalid Mac file permissions/type')
        files={n:z.read(n) for n in names}
    package=json.loads(files['PACKAGE-MANIFEST.json'])
    verify_project_license(files,package,root,RES)
    require(package['schema_version']==1 and package['target']=='macos-universal2' and package['original_game_data'] is True
            and package['signing']=='ad-hoc' and package['notarized'] is False,'Invalid Mac package schema')
    pins=package['files'];require(len(pins)==len(files)-2 and {p['path'] for p in pins}==set(files)-{'PACKAGE-MANIFEST.json','SHA256SUMS'},'Invalid Mac pins')
    for pin in pins:require(len(files[pin['path']])==pin['bytes'] and sha(files[pin['path']])==pin['sha256'],'Mac file checksum differs')
    require(files['SHA256SUMS']==''.join(f'{sha(b)}  {n}\n' for n,b in sorted(files.items()) if n!='SHA256SUMS').encode(),'Internal Mac checksums differ')
    validate_app(files,package['version'],root)
    require(sha(files['source.tar.gz'])==package['source']['archive_sha256'],'Source archive differs')
    with tarfile.open(fileobj=io.BytesIO(files['source.tar.gz']),mode='r:gz') as source:
        require(all(m.isfile() for m in source.getmembers()),'Source contains links/non-files')
        pins=json.load(source.extractfile('SOURCE-MANIFEST.json'))
        require(len(pins)==package['source']['source_files'] and len(source.getnames())==len(pins)+1
                and set(source.getnames())=={'SOURCE-MANIFEST.json'}|{p['path'] for p in pins},'Source inventory differs')
        for pin in pins:
            require(not Path(pin['path']).is_absolute() and '..' not in Path(pin['path']).parts
                    and not pin['path'].startswith(('assets/local/','target/')),'Invalid source path')
            data=source.extractfile(pin['path']).read()
            require(len(data)==pin['bytes'] and sha(data)==pin['sha256'],'Source checksum differs')
    return dict(status='PASS',target='macos-universal2',architectures=['arm64','x86_64'],game_files=28,
                source_files=len(pins),notarized=False,archive_bytes=len(raw),sha256=sha(raw))


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('command',choices=['prepare-sdl','stage','create','verify'])
    p.add_argument('--sdl',type=Path,default=ROOT/'assets/local/macos-sdl')
    p.add_argument('--binary',type=Path,default=ROOT/'target/universal/OhMyLemmings')
    p.add_argument('--assets',type=Path,default=ROOT/'assets/local/dos-original')
    p.add_argument('--stage',type=Path,default=ROOT/'assets/local/macos-app')
    p.add_argument('--output',type=Path,default=ROOT/'assets/local/dist');p.add_argument('--archive',type=Path)
    a=p.parse_args()
    if a.command=='prepare-sdl':prepare_sdl(a.sdl.resolve())
    elif a.command=='stage':print(stage(a.binary,a.assets,a.sdl,a.stage))
    elif a.command=='create':print(create(a.stage/APP,a.output))
    else:
        if a.archive is None:p.error('--archive is required')
        print(json.dumps(verify(a.archive)))
