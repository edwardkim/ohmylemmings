#!/usr/bin/env python3
"""Bounded SDL queue and actual Web Audio/WASM, offline and original-free by default."""
import argparse
import functools
import hashlib
from http.server import SimpleHTTPRequestHandler,ThreadingHTTPServer
import ipaddress
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import threading
try:
    from .web_build import ROOT,build
    from .check_audio import pcm_info
    from .state_replay import verified_assets
except ImportError:
    from web_build import ROOT,build
    from check_audio import pcm_info
    from state_replay import verified_assets
FIXTURE=ROOT/'tools/tests/fixtures/audio_output.json'
DESKTOP=ROOT/'target/release/lemmings-desktop'
TRACE=ROOT/'target/release/audio_trace'


def uint(value,maximum=2**64-1):return type(value) is int and 0<=value<=maximum

def guard_original(directory):
    if not directory.resolve().is_relative_to((ROOT/'assets/local').resolve()):raise ValueError('original audio evidence must remain ignored')

def validate_sdl(row):
    if row.get('status')!='PASS' or row.get('driver')!='dummy' or row.get('mute_hide_generation') is not True:raise ValueError('SDL production device path incomplete')
    for key in ['queued_bytes','consumed_bytes','cleared_bytes','queue_limit_frames','nonzero_frames','submitted_frames']:
        if not uint(row.get(key)):raise ValueError('invalid SDL count')
    if not 0<row['consumed_bytes']<=row['queued_bytes']<=6144 or row['cleared_bytes']!=0 or row['queue_limit_frames']!=3072 or not 0<row['nonzero_frames']<=row['submitted_frames']:raise ValueError('SDL queue/PCM contract failed')

def validate_fixture(data):
    rows=data['cases']
    if len(rows)!=6 or {r['name'] for r in rows}!={f'{rate}-{volume}' for rate in [44100,48000] for volume in [0,33,100]}:raise ValueError('missing audio output cases')
    for row in rows:
        if not uint(row['frames']) or row['frames']==0 or not uint(row['peak'],32768) or not uint(row['nonzero'],row['frames']):raise ValueError('invalid output PCM metrics')
        if row['name'].endswith('-0') and (row['nonzero'] or row['peak']):raise ValueError('zero volume is not silent')
        if not row['name'].endswith('-0') and row['nonzero']==0:raise ValueError('audible output is silent')
        for key in ['pcm_sha256','state_sha256','rgb_sha256','terrain_sha256','objects_sha256']:
            if not re.fullmatch('[0-9a-f]{64}',row[key]):raise ValueError('invalid output digest')

def validate_browser(row,expected,secure):
    if row.get('status')!='PASS' or row.get('offline_context') is not True or row.get('worklet_required') is not False or row.get('secure_context') is not secure:raise ValueError('actual Web Audio context evidence incomplete')
    if row.get('source_delivery')!={'license':True,'archive':True,'wasm_pair':True}:raise ValueError('source/license/WASM delivery incomplete')
    want=[dict(name=r['name'],frames=r['frames'],sha256=r['pcm_sha256']) for r in expected]
    if row.get('cases')!=want:raise ValueError('native/actual WASM output differs')
    if row.get('mocks')!={k:True for k in ['bounded','blocked','activation_race','mute_hide_resume','volume','clock_jump','context_recreation']}:raise ValueError('activation/device recovery coverage incomplete')
    live=row['live']
    if live.get('state')!='running' or not uint(live.get('nonzero'),2048) or live['nonzero']==0 or not uint(live.get('queued_nodes'),8) or type(live.get('queue_seconds')) not in [int,float] or not 0<=live['queue_seconds']<=0.1:raise ValueError('live AudioContext queue/graph failed')

def sdl_check(directory,binary):
    env=dict(os.environ,SDL_AUDIODRIVER='dummy',SDL_VIDEODRIVER='dummy',SDL_RENDER_DRIVER='software')
    result=subprocess.run([str(binary),'--audio-probe'],env=env,capture_output=True,text=True,timeout=20,check=True)
    (directory/'sdl-probe.log').write_text(result.stderr);row=json.loads(result.stdout);validate_sdl(row)
    bad=dict(env,SDL_AUDIODRIVER='lemmings-intentionally-unavailable')
    failed=subprocess.run([str(binary),'--synthetic','--steps','2'],env=bad,capture_output=True,text=True,timeout=20,check=True)
    if 'Audio unavailable:' not in failed.stderr:raise ValueError('missing device was not reported')
    (directory/'sdl-unavailable.log').write_text(failed.stderr+failed.stdout)
    for name,extra in [('sound',[]),('muted',['--mute'])]:
        subprocess.run([str(binary),'--synthetic-builder','--steps','60','--capture',str(directory/name),*extra],env=env,capture_output=True,text=True,timeout=30,check=True)
    names=['terrain-final.bin','brick-final.bin','object-map-final.bin','inputs.txt','frame-000060.rgb']
    names += [p.name for p in (directory/'sound').glob('*.jsonl')]
    for name in names:
        if (directory/'sound'/name).read_bytes()!=(directory/'muted'/name).read_bytes():raise ValueError('SDL sound changed game/capture: '+name)
    for value in ['-1','101','0.5','NaN']:
        invalid=subprocess.run([str(binary),'--synthetic','--steps','1','--volume',value],env=env,capture_output=True,text=True,timeout=5)
        if invalid.returncode==0:raise ValueError('invalid volume accepted')
    return dict(**row,gameplay_unchanged=True,compared_files=len(names),missing_device_recoverable=True)

def native_cases(directory,original=None):
    subprocess.run(['cargo','build','-p','lemmings-web','--bin','audio_trace','--release','--locked','--offline'],cwd=ROOT,check=True)
    command=[str(TRACE),str(directory/'native')]
    if original:command.append(str(original))
    subprocess.run(command,cwd=ROOT,check=True)
    rows=json.loads((directory/'native/native.json').read_text())['cases'];public=[]
    for row in rows:
        name=row['name'];info=pcm_info((directory/'native'/f'{name}.pcm').read_bytes())
        row.update(frames=info['frames'],pcm_sha256=info['sha256'])
        for extension in ['rgb','terrain','objects']:row[extension+'_sha256']=hashlib.sha256((directory/'native'/f'{name}.{extension}').read_bytes()).hexdigest()
        state=json.dumps(row['state'],separators=(',',':')).encode()
        public.append(dict(name=name,frames=info['frames'],pcm_sha256=info['sha256'],peak=info['peak'],nonzero=info['nonzero'],state_sha256=hashlib.sha256(state).hexdigest(),**{k:v for k,v in row.items() if k.endswith('_sha256') and k!='pcm_sha256'}))
    return rows,dict(scope='Original-free composed practice tones through owned output/continuations; fixed native PCM/state/map/RGB',cases=public)

def browser_check(directory,rows,host,original=None):
    build(directory)
    for name in ['output-check.html','output-check.js']:shutil.copyfile(ROOT/'tools/audio'/name,directory/name)
    selected=[]
    if original:
        (directory/'original').mkdir()
        for p in sorted(original.iterdir()):
            if p.name in ['MAIN.DAT','ODDTABLE.DAT','VGALEMMI.EXE','ADLIB.DAT'] or re.fullmatch(r'LEVEL00[0-9]\.DAT|GROUND[0-4]O\.DAT|VGAGR[0-4]\.DAT',p.name):shutil.copyfile(p,directory/'original'/p.name);selected.append(p.name)
    (directory/'expected.json').write_text(json.dumps(dict(cases=rows,original_files=selected),separators=(',',':'))+'\n')
    ready=threading.Event();result={}
    class Handler(SimpleHTTPRequestHandler):
        def log_message(self,*args):pass
        def do_POST(self):
            size=int(self.headers.get('Content-Length','0'))
            if not 0<size<=8*1024*1024:self.send_error(413);return
            if self.path=='/__sha':
                digest=hashlib.sha256(self.rfile.read(size)).hexdigest().encode();self.send_response(200);self.end_headers();self.wfile.write(digest);return
            if self.path!='/__result':self.send_error(404);return
            if size>100000:self.send_error(413);return
            try:result.update(json.loads(self.rfile.read(size)))
            except (ValueError,TypeError):self.send_error(400);return
            self.send_response(200);self.end_headers();self.wfile.write(b'OK');ready.set()
    browser=next((shutil.which(n) for n in ['google-chrome','chromium','chromium-browser'] if shutil.which(n)),None)
    if not browser:raise ValueError('Chrome/Chromium required')
    address=ipaddress.IPv4Address(host);secure=address.is_loopback
    with ThreadingHTTPServer((str(address),0),functools.partial(Handler,directory=directory)) as server:
        worker=threading.Thread(target=server.serve_forever,daemon=True);worker.start()
        with tempfile.TemporaryDirectory(prefix='lemmings-device-browser-') as profile,(directory/'browser.log').open('w') as log:
            # This flag tests rendering without automating a user gesture. Blocked
            # resume/retry/races are separately exercised; LAN UI gesture is manual.
            command=[browser,'--headless','--no-first-run','--no-default-browser-check','--disable-gpu','--autoplay-policy=no-user-gesture-required',f'--user-data-dir={profile}',f'http://{address}:{server.server_port}/output-check.html']
            process=subprocess.Popen(command,stdout=log,stderr=subprocess.STDOUT)
            try:
                for _ in range(180):
                    if ready.wait(1):break
                    if process.poll() is not None:raise ValueError('audio output browser exited')
                if not ready.is_set():raise TimeoutError('audio output browser timed out')
            finally:
                if process.poll() is None:
                    process.terminate()
                    try:process.wait(timeout=10)
                    except subprocess.TimeoutExpired:process.kill();process.wait()
                server.shutdown();worker.join(timeout=5)
    (directory/'browser-result.json').write_text(json.dumps(result,indent=2)+'\n');validate_browser(result,rows,secure)
    return dict(status='PASS',cases=len(rows),frames=sum(r['frames'] for r in rows),secure_context=secure,offline_context=True,source_delivery=result['source_delivery'],live=result['live'],browser=subprocess.check_output([browser,'--version'],text=True).strip(),autoplay_policy='headless no-user-gesture-required; separate blocked/race mocks; trusted gesture checked separately')

def check(directory,*,wasm=False,binary=DESKTOP,original=None,host='127.0.0.1',write_fixture=False):
    if original:
        guard_original(directory)
        if write_fixture:raise ValueError('original PCM cannot enter public fixture')
        before=verified_assets(original)
    directory.mkdir(parents=True,exist_ok=False)
    rows,pinned=native_cases(directory,original)
    if not original:
        validate_fixture(pinned)
        if write_fixture:FIXTURE.write_text(json.dumps(pinned,indent=2)+'\n')
        elif pinned!=json.loads(FIXTURE.read_text()):raise ValueError('fixed output PCM/state/map/RGB differs')
    result=dict(status='PASS',native_cases=len(rows),native_frames=sum(r['frames'] for r in rows),simulation_unchanged=True,physical_listening_verified=False)
    if wasm:result['wasm']=browser_check(directory,rows,host,original)
    else:result['sdl']=sdl_check(directory,binary)
    if original:
        if before!=verified_assets(original):raise ValueError('original assets changed')
        result['original_files']=len(before)
    (directory/'summary.json').write_text(json.dumps(result,indent=2)+'\n');return result

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--evidence',type=Path);p.add_argument('--wasm',action='store_true');p.add_argument('--binary',type=Path,default=DESKTOP);p.add_argument('--original-assets',type=Path);p.add_argument('--host',default='127.0.0.1');p.add_argument('--write-fixture',action='store_true');a=p.parse_args()
    if a.original_assets and (not a.wasm or not a.evidence):p.error('original audio requires --wasm and ignored --evidence')
    if a.evidence:result=check(a.evidence.resolve(),wasm=a.wasm,binary=a.binary.resolve(),original=a.original_assets.resolve() if a.original_assets else None,host=a.host,write_fixture=a.write_fixture)
    else:
        with tempfile.TemporaryDirectory(prefix='lemmings-output-') as temp:result=check(Path(temp)/'evidence',wasm=a.wasm,binary=a.binary.resolve(),host=a.host,write_fixture=a.write_fixture)
    print(json.dumps(result))

if __name__=='__main__':main()
