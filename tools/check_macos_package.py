#!/usr/bin/env python3
"""Run the extracted original-game application on native macOS ARM64 or Intel."""
import argparse
import json
import os
from pathlib import Path
import platform
import re
import signal
import subprocess
import sys
import time

ROOT=Path(__file__).resolve().parents[1]
if __package__ in (None,''):sys.path.insert(0,str(ROOT))
from tools.macos_package import APP, EXE, LIB, verify
from tools.release_package import require, sha


def save(path,value):path.write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')


def check(archive,evidence,expected_arch):
    require(sys.platform=='darwin' and platform.machine()==expected_arch,'Native Mac architecture required')
    require(evidence.is_relative_to((ROOT/'assets/local').resolve()),'Keep original captures under assets/local')
    package=verify(archive);evidence.mkdir(parents=True)
    installed=evidence/'압축 해제 game folder';foreign=evidence/'다른 작업 folder';foreign.mkdir()
    subprocess.run(['ditto','-x','-k',str(archive),str(installed)],check=True)
    app=installed/APP;binary=installed/EXE;library=installed/LIB
    environment={k:v for k,v in os.environ.items() if not k.startswith('DYLD_') and k!='SDL_VIDEODRIVER'}
    environment.update(SDL_AUDIODRIVER='dummy',SDL_RENDER_DRIVER='software')
    subprocess.run(['codesign','--verify','--deep','--strict','--verbose=2',str(app)],check=True)
    signature=subprocess.run(['codesign','-d','--verbose=4',str(app)],capture_output=True,text=True,check=True)
    require('Signature=adhoc' in signature.stderr,'Expected documented ad-hoc signature')
    (evidence/'signature.log').write_text(signature.stderr)
    dependencies={}
    for path in [binary,library]:
        deps=subprocess.check_output(['otool','-L',str(path)],text=True)
        libraries=re.findall(r'^\s+(\S+) \(compatibility version',deps,re.M)
        require(libraries and all(p.startswith(('/System/Library/','/usr/lib/','@rpath/')) for p in libraries),'External/non-system runtime dependency')
        require(all(not p.startswith('@rpath/') or p=='@rpath/libSDL2-2.0.0.dylib' for p in libraries),'Unexpected rpath library')
        dependencies[path.name]=deps
    save(evidence/'dependencies.json',dependencies)

    # LaunchServices follows the same .app entry point as Finder, without game arguments.
    probe=evidence/'window-probe'
    subprocess.run(['swiftc',str(ROOT/'tools/macos_window.swift'),'-o',str(probe)],check=True)
    launch=subprocess.Popen(['open','-n','-W',str(app)],cwd=foreign,env=environment,
                            stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    pid=None
    try:
        deadline=time.monotonic()+25;visible=[]
        while time.monotonic()<deadline:
            rows=json.loads(subprocess.check_output([str(probe)],text=True))
            visible=[r for r in rows if r['windows']>0]
            if visible:break
            if launch.poll() is not None:break
            time.sleep(.2)
        require(visible,'No visible native window after opening the app without arguments')
        pid=visible[0]['pid'];time.sleep(1);os.kill(pid,0)
        save(evidence/'no-arguments.json',dict(status='PASS',visible=visible,arguments=[],launch_services=True))
    finally:
        if pid is not None:
            try:os.kill(pid,signal.SIGTERM)
            except ProcessLookupError:pass
        launch.wait(timeout=10)

    def execute(label,args,*,success=True,visible=False):
        env=dict(environment)
        if not visible:env['SDL_VIDEODRIVER']='dummy'
        result=subprocess.run([str(binary),'--original',*map(str,args)],cwd=foreign,env=env,text=True,
            capture_output=True,timeout=180)
        save(evidence/(label+'.json'),dict(arguments=list(map(str,args)),exit_code=result.returncode,stdout=result.stdout,stderr=result.stderr))
        require((result.returncode==0)==success,f'{label} exited {result.returncode}: {result.stderr}')
        return result

    initial=evidence/'original-first'
    from tools.check_hd_package import check as check_hd
    hd_environment=dict(environment,SDL_VIDEODRIVER='dummy')
    check_hd(binary,app/'Contents/Resources',foreign,hd_environment,evidence)
    result=execute('first',['--steps',2,'--fast','--capture',initial],visible=True)
    first=json.loads(result.stdout);config=json.loads((initial/'config.json').read_text())
    require(first['synthetic'] is False and config['difficulty']=='fun' and config['level']==1 and first['video_driver']=='cocoa','Original native Fun01 not loaded')
    from tools.check_desktop import verify_pixels
    verify_pixels(initial)
    logical=next(initial.glob('frame-*.rgb')).read_bytes()
    require(sha(logical)=='1539ca2afdcc9cfbe4fa5de51036c45b881014ec14a0ab030c191ca41f988c4d','Original Fun01 frame differs from verified Windows frame')
    import shutil
    shutil.copyfile(next(initial.glob('frame-*.ppm')),evidence/'original-fun01.ppm')
    levels=[]
    for difficulty in ['fun','tricky','taxing','mayhem']:
        for number in range(1,31):
            result=execute(f'level-{difficulty}-{number:02}',['--difficulty',difficulty,'--level',number,'--steps',1,'--fast'])
            require(json.loads(result.stdout)['synthetic'] is False,'Synthetic fallback')
            levels.append(dict(difficulty=difficulty,number=number,status='PASS'))
    save(evidence/'levels.json',levels)
    replay=evidence/'solution.txt';replay.write_text('60 0 AssignDigger 0\n')
    transitions=[]
    for action,target in [('restart',1),('continue',2)]:
        directory=evidence/('original-'+action)
        execute(action,['--steps',1000,'--fast','--render-every',7,'--replay',replay,'--result-actions',action,'--capture',directory])
        rows=[json.loads(line) for line in (directory/'transitions.jsonl').read_text().splitlines()]
        require(len(rows)==1 and rows[0]['target']==target,'Wrong restart/next target')
        require(json.loads((directory/'completion.jsonl').read_text().splitlines()[-1])['phase']=='Won','Saved Fun01 solution failed')
        attempt=directory/'attempt-0001';baseline=evidence/f'fresh-{target}'
        require(not (attempt/'inputs.txt').read_text(),'Old inputs leaked across result')
        execute(f'fresh-{target}',['--level',target,'--steps',1000,'--fast','--render-every',7,'--capture',baseline])
        from tools.check_completion import collect
        require(collect(attempt)==collect(baseline),'Restart/next differs from independent fresh attempt')
        require((attempt/'terrain-initial.bin').read_bytes()==(baseline/'terrain-initial.bin').read_bytes(),'Fresh terrain differs')
        transitions.append(dict(action=action,target=target,status='PASS'))
    result=execute('original-audio',['--steps',60])
    match=re.search(r'LEMMINGS_AUDIO submitted=(\d+) nonzero=(\d+) active=true original=true',result.stderr)
    require(match and int(match[1])>0 and int(match[2])>0,'Original ADLIB PCM output missing')
    audio=dict(submitted_frames=int(match[1]),nonzero_frames=int(match[2]),driver='dummy',original=True)

    missing=app/'Contents/Resources/data/MAIN.DAT';hidden=missing.with_suffix('.hidden')
    error_log=Path.home()/'Library/Logs/OhMyLemmings/last-error.log'
    previous=error_log.read_bytes() if error_log.exists() else None
    missing.rename(hidden)
    try:
        execute('missing-data',['--steps',1,'--fast'],success=False)
        require(error_log.exists() and 'MAIN.DAT' in error_log.read_text(),'Native startup error log missing')
        (evidence/'startup-error.log').write_bytes(error_log.read_bytes())
    finally:
        hidden.rename(missing)
        if previous is not None:error_log.write_bytes(previous)
        elif error_log.exists():error_log.unlink()
    subprocess.run(['codesign','--verify','--deep','--strict',str(app)],check=True)
    # Model the browser-download quarantine flag only on this disposable installed test copy.
    subprocess.run(['xattr','-w','com.apple.quarantine','0083;00000000;OhMyLemmingsPackageTest;',str(app)],check=True)
    gatekeeper=subprocess.run(['spctl','--assess','--type','execute','--verbose=4',str(app)],capture_output=True,text=True)
    require(gatekeeper.returncode!=0,'Ad-hoc preview unexpectedly passed Developer ID distribution assessment')
    save(evidence/'gatekeeper.json',dict(exit_code=gatekeeper.returncode,stdout=gatekeeper.stdout,stderr=gatekeeper.stderr,notarized=False))
    report=dict(status='PASS',package=package,source_head=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),
        native_architecture=platform.machine(),macos=platform.mac_ver()[0],original_levels_loaded=len(levels),
        launch_services_visible_window=True,unicode_space_install=True,foreign_cwd=True,
        original_frame_sha256=sha(logical),original_transitions=transitions,audio=audio,
        signing='ad-hoc',notarized=False,gatekeeper_assessment='rejected-as-expected',
        scope='Native hosted Mac functional packaging; Developer ID/notarization, physical listening, older deployment OS and 60fps qualification remain separate.')
    save(evidence/'summary.json',report);print(json.dumps(report))


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--archive',type=Path,required=True)
    p.add_argument('--evidence',type=Path,default=ROOT/'assets/local/macos-package-check')
    p.add_argument('--expected-arch',choices=['arm64','x86_64'],required=True);a=p.parse_args()
    check(a.archive.resolve(),a.evidence.resolve(),a.expected_arch)
