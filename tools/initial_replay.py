#!/usr/bin/env python3
"""Audit 120 original initial replays in the actual SDL executable and WASM.

Original/derived evidence is restricted to assets/local. Default --smoke uses
invented files. Initial replay evidence never certifies level solutions.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import functools
import hashlib
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
import os
import platform
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile
import threading
import time
sys.path.insert(0,str(Path(__file__).resolve().parent))
try:
    from .web_build import ROOT, build
    from .state_replay import verified_assets
    from .web_fixture import bank
except ImportError:
    from web_build import ROOT, build
    from state_replay import verified_assets
    from web_fixture import bank

RANKS=['fun','tricky','taxing','mayhem']
VARIANTS=[(1,30),(7,60),(13,144)]
sha=lambda b:hashlib.sha256(b).hexdigest()
canonical=lambda value:json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False)


def catalog_ids(rows):
    wanted=[f'{rank}-{n:02}' for rank in RANKS for n in range(1,31)]
    if not isinstance(rows,list) or any(not isinstance(r,dict) or not isinstance(r.get('id'),str) for r in rows):raise ValueError('invalid catalog row')
    actual=[r['id'] for r in rows]
    if len(actual)!=len(set(actual)) or set(actual)!=set(wanted):raise ValueError('catalog requires all120 distinct level IDs')
    return wanted


def first_difference(a,b,path='$'):
    if type(a) is not type(b):return path
    if isinstance(a,dict):
        for key in sorted(set(a)|set(b)):
            at=path+'.'+str(key)
            if key not in a or key not in b:return at
            difference=first_difference(a[key],b[key],at)
            if difference:return difference
    elif isinstance(a,list):
        if len(a)!=len(b):return path+'.length'
        for i,(left,right) in enumerate(zip(a,b)):
            difference=first_difference(left,right,f'{path}[{i}]')
            if difference:return difference
    elif a!=b:return path
    return None


def exact(stream,size):
    data=bytearray()
    while len(data)<size:
        chunk=stream.read(size-len(data))
        if not chunk:raise ValueError('truncated frame stream')
        data.extend(chunk)
    return bytes(data)


def frame_stream(stream,frame_size=320*200*3):
    previous=-1
    while True:
        probe,size=struct.unpack('<QI',exact(stream,12))
        if probe==2**64-1:
            if size:raise ValueError('invalid frame terminator')
            return
        if probe<=previous or size!=frame_size:raise ValueError('invalid frame order/size')
        previous=probe
        yield probe,exact(stream,size)


def validate_rows(rows,steps):
    if len(rows)!=steps+1:raise ValueError('missing probe boundary')
    frozen=None
    for probe,row in enumerate(rows):
        if row['probe']!=probe:raise ValueError('missing/duplicate probe boundary')
        state=row['state'];step=state['session']['control_step']
        if frozen is not None:
            if first_difference(frozen,{k:v for k,v in row.items() if k!='probe'}):raise ValueError(f'frozen state/map changed at probe{probe}')
        elif step!=probe:raise ValueError(f'wrong control boundary{probe}')
        if state['completion']['phase']!='Playing':frozen={k:v for k,v in row.items() if k!='probe'}


def file_identity(path):
    data=Path(path).read_bytes();return dict(path=str(path),bytes=len(data),sha256=sha(data))


def jsonl(path):return [json.loads(line) for line in Path(path).read_text().splitlines()]


def elapsed_chunks(rate,steps):
    # > required wall time, consumed only until the final control boundary.
    count=(steps+2)*rate//14+2*rate
    return [((i+1)*1_000_000//rate)-(i*1_000_000//rate) for i in range(count)]


def native_case(root,assets,entry,steps,replay=None):
    level_id=entry['id'];rank,number=level_id.split('-');runs=[];baseline=None;frames=None;initial=None
    replay=replay or root/'inputs.txt';input_text=replay.read_text()
    for render,rate in VARIANTS:
        dest=root/f'{level_id}-r{render}';elapsed=root/f'{level_id}-r{render}.us'
        elapsed.write_text(''.join(f'{n}\n' for n in elapsed_chunks(rate,steps)))
        command=[str(ROOT/'target/release/lemmings-desktop'),'--assets',str(assets),'--difficulty',rank,'--level',str(int(number)),'--steps',str(steps),'--fast','--render-every',str(render),'--capture',str(dest),'--replay',str(replay),'--elapsed-replay',str(elapsed),'--capture-stream']
        frame_hashes={}
        with (root/f'{level_id}-r{render}.stderr').open('wb') as error:
            process=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=error,env=dict(os.environ,SDL_VIDEODRIVER='dummy'))
            try:
                for probe,rgb in frame_stream(process.stdout):
                    frame_hashes[str(probe)]=sha(rgb)
                    if render==1 and probe==0:(root/f'{level_id}-initial.rgb').write_bytes(rgb)
                summary=json.loads(process.stdout.read())
                if process.wait(timeout=10):raise ValueError(f'{level_id}: SDL process failed')
            finally:
                if process.poll() is None:process.kill();process.wait()
                process.stdout.close()
        if summary['video_driver']!='dummy' or summary['renderer']!='software':raise ValueError('actual SDL software renderer required')
        text=(dest/'full.jsonl').read_text().splitlines();states=[json.loads(s) for s in text]
        terrain=bytearray((dest/'terrain-initial.bin').read_bytes());bricks=bytearray(len(terrain));objects=bytearray((dest/'object-map-initial.bin').read_bytes());rows=[]
        digs=jsonl(dest/'digger.jsonl');builders=jsonl(dest/'builder.jsonl');blockers=jsonl(dest/'blocker.jsonl')
        if not(len(states)==len(digs)==len(builders)==len(blockers)==steps+1):raise ValueError('missing SDL state/delta boundaries')
        for probe,state in enumerate(states):
            for i in digs[probe]['cleared']:terrain[i]=0;bricks[i]=0
            for i in builders[probe]['filled']:terrain[i]=1;bricks[i]=1
            for i,value in blockers[probe]['changed']:objects[i]=value
            rows.append(dict(probe=probe,state=state,terrain=sha(terrain),bricks=sha(bricks),objects=sha(objects)))
        validate_rows(rows,steps)
        for name,data in [('terrain',terrain),('brick',bricks),('object-map',objects)]:
            if bytes(data)!=(dest/f'{name}-final.bin').read_bytes():raise ValueError(f'{level_id}: final {name} differs from recorded deltas')
        compact=[[sha(s.encode()),r['terrain'],r['bricks'],r['objects']] for s,r in zip(text,rows)]
        if baseline is None:
            baseline=compact;frames=frame_hashes
            if set(frames)!={str(n) for n in range(steps+1)}:raise ValueError('dense SDL replay did not render all boundaries')
            initial=dict(info=json.loads((dest/'asset-info.json').read_text()),materials=sha((dest/'objects.bin').read_bytes()),camera=jsonl(dest/'pointer.jsonl')[0]['camera'])
            (root/f'{level_id}-materials.bin').write_bytes((dest/'objects.bin').read_bytes())
        else:
            difference=first_difference(baseline,compact)
            if difference:raise ValueError(f'{level_id}/render{render}: state/maps differ {difference}')
            for probe,value in frame_hashes.items():
                if frames.get(probe)!=value:raise ValueError(f'{level_id}/render{render}/probe{probe}: RGB differs')
        if (dest/'inputs.txt').read_text()!=input_text:raise ValueError('replay input was changed, dropped or queued after result')
        runs.append(dict(render_every=render,elapsed_rate=rate,probes=len(rows),frames=len(frame_hashes),state_map_sha256=sha(canonical(compact).encode()),frames_sha256=sha(canonical(frame_hashes).encode()),summary=summary,files=[file_identity(p) for p in sorted(dest.iterdir()) if p.is_file()]))
        print(f'SDL {level_id} r{render}: PASS {len(rows)} probes/{len(frame_hashes)} readbacks',flush=True)
    expected=dict(id=level_id,steps=steps,initial=initial,rows=baseline,frames=frames,input_text=input_text)
    (root/f'{level_id}.json').write_text(json.dumps(expected,separators=(',',':'))+'\n')
    return dict(id=level_id,runs=runs,initial=initial,expected=file_identity(root/f'{level_id}.json'))


def browser_check(directory,ids,files,steps,timeout=1800):
    build(directory)
    for name in ['initial_replay.js','initial_replay.html']:shutil.copyfile(ROOT/'tools/web'/name,directory/name)
    (directory/'initial-index.json').write_text(json.dumps(dict(ids=ids,files=files,steps=steps,variants=VARIANTS))+'\n')
    ready=threading.Event();results=[];failed=[];last=time.monotonic()
    class Handler(SimpleHTTPRequestHandler):
        def log_message(self,*args):pass
        def do_POST(self):
            nonlocal last
            size=int(self.headers.get('Content-Length','0'))
            if not 0<size<2_000_000:self.send_error(413);return
            data=json.loads(self.rfile.read(size));last=time.monotonic()
            if self.path=='/__initial/progress':
                results.append(data);(directory/'browser-progress.json').write_text(json.dumps(results,indent=2)+'\n');print('WASM',data.get('id'),data.get('render'),'PASS',flush=True)
            elif self.path=='/__initial/done':ready.set()
            elif self.path=='/__initial/fail':failed.append(data);ready.set()
            else:self.send_error(404);return
            self.send_response(200);self.end_headers();self.wfile.write(b'OK')
    browser=next((shutil.which(n) for n in ['google-chrome','chromium','chromium-browser'] if shutil.which(n)),None)
    if not browser:raise ValueError('Chromium/Chrome required')
    with ThreadingHTTPServer(('127.0.0.1',0),functools.partial(Handler,directory=directory)) as server:
        thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
        with tempfile.TemporaryDirectory(prefix='initial-replay-browser-') as profile,(directory/'browser.log').open('w') as log:
            process=subprocess.Popen([browser,'--headless','--disable-gpu','--no-first-run','--no-default-browser-check',f'--user-data-dir={profile}',f'http://127.0.0.1:{server.server_port}/initial_replay.html'],stdout=log,stderr=subprocess.STDOUT)
            try:
                start=time.monotonic()
                while not ready.wait(1):
                    if process.poll() is not None:raise RuntimeError('browser exited')
                    if time.monotonic()-start>timeout or time.monotonic()-last>120:raise TimeoutError('initial browser replay stalled/timed out')
            finally:
                process.terminate()
                try:process.wait(timeout=10)
                except subprocess.TimeoutExpired:process.kill();process.wait()
                server.shutdown();thread.join(timeout=5)
    if failed:
        (directory/'first-failure.json').write_text(json.dumps(failed[0],indent=2)+'\n');raise ValueError(failed[0])
    if len(results)!=len(ids)*3:raise ValueError('missing/duplicate browser runs')
    if {(r['id'],r['render']) for r in results}!={(i,v[0]) for i in ids for v in VARIANTS}:raise ValueError('browser run inventory differs')
    return dict(status='PASS',browser=subprocess.check_output([browser,'--version'],text=True).strip(),runs=results)


def run(directory,original_assets=None,workers=3,steps=340):
    directory=directory.resolve()
    if directory.exists():raise ValueError('evidence directory must be new')
    if not directory.is_relative_to(ROOT/'assets/local'):raise ValueError('evidence must remain in ignored assets/local')
    directory.mkdir(parents=True);native=directory/'native';native.mkdir();(native/'inputs.txt').write_bytes(b'')
    originals=verified_assets(original_assets) if original_assets else None
    catalog=json.loads((ROOT/'assets/manifests/dos_compatibility.json').read_text())['levels'];ids=catalog_ids(catalog)
    if not original_assets:ids=['fun-01','fun-22','tricky-14','taxing-15','mayhem-22']
    assets=original_assets or directory/'fixture'
    if not original_assets:
        assets.mkdir()
        for name,data in bank(win=True,special=True).items():(assets/name).write_bytes(data)
    subprocess.run(['cargo','build','-p','lemmings-desktop','--release','--locked','--offline'],cwd=ROOT,check=True)
    sources=[file_identity(p) for base in ['apps/desktop/src','apps/web/src','crates/lemmings-game/src','crates/lemmings-session/src','crates/lemmings-core/src','crates/lemmings-resources/src'] for p in sorted((ROOT/base).rglob('*.rs'))]+[file_identity(ROOT/'tools/initial_replay.py'),file_identity(ROOT/'tools/web/initial_replay.js')]
    provenance=dict(head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),sources=sources,desktop=file_identity(ROOT/'target/release/lemmings-desktop'),python=sys.version,os=platform.platform(),rust=subprocess.check_output(['rustc','--version'],text=True).strip(),profile='release',flags=['--locked','--offline'],pacing='dos-vga-5',steps=steps,input_sha256=sha(b''))
    (directory/'provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')
    reports=[]
    with ThreadPoolExecutor(max_workers=workers) as pool:
        futures={pool.submit(native_case,native,assets,entry,steps):entry['id'] for entry in catalog if entry['id'] in ids}
        try:
            for future in as_completed(futures):reports.append(future.result());(directory/'native-progress.json').write_text(json.dumps(reports,indent=2)+'\n')
        except Exception as error:
            failed_id=futures[future]
            for pending in futures:pending.cancel()
            (directory/'first-failure.json').write_text(json.dumps(dict(status='FAIL',id=failed_id,error=str(error)),indent=2)+'\n')
            raise
    files=[p.name for p in assets.iterdir() if p.is_file() and (p.suffix=='.DAT' or p.name=='VGALEMMI.EXE') and not p.name.startswith(('CGA','TGA','TGASPEC'))]
    # The bank handles its strict logical allowlist; copy only recognized VGA data.
    import re
    files=sorted(n for n in files if re.fullmatch(r'(MAIN|ODDTABLE|ADLIB|LEVEL00[0-9]|GROUND[0-4]O|VGAGR[0-4]|VGASPEC[0-3])\.DAT|VGALEMMI\.EXE',n))
    (directory/'files').mkdir()
    for name in files:shutil.copyfile(assets/name,directory/'files'/name)
    for level_id in ids:shutil.copyfile(native/f'{level_id}.json',directory/f'{level_id}.json')
    web=browser_check(directory,ids,files,steps)
    if original_assets:assert verified_assets(original_assets)==originals
    assert provenance['desktop']==file_identity(ROOT/'target/release/lemmings-desktop')
    provenance['wasm']=file_identity(directory/'pkg/lemmings_web_bg.wasm')
    (directory/'provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')
    reports.sort(key=lambda r:ids.index(r['id']))
    result=dict(status='PASS',scope='initial no-input probes only; no solutions, new all-level DOS observation or performance certification',original=bool(original_assets),levels=len(ids),steps=steps,variants=VARIANTS,native_runs=len(reports)*3,wasm_runs=len(web['runs']),native_probes=sum(x['probes'] for r in reports for x in r['runs']),wasm_probes=sum(x['probes'] for x in web['runs']),sdl_readbacks=sum(x['frames'] for r in reports for x in r['runs']),canvas_readbacks=sum(x['frames'] for x in web['runs']),native=reports,web=web,original_files=originals,input_sha256=sha(b''))
    (directory/'summary.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in ['native','web','original_files']}));return result


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--original-assets',type=Path);p.add_argument('--evidence',type=Path);p.add_argument('--smoke',action='store_true');p.add_argument('--workers',type=int,default=3);a=p.parse_args()
    if not 1<=a.workers<=4:p.error('--workers must be1..4')
    if bool(a.original_assets)==bool(a.smoke):p.error('choose --original-assets or --smoke')
    if a.original_assets and not a.evidence:p.error('original replay requires ignored --evidence')
    if a.evidence:run(a.evidence,a.original_assets.resolve() if a.original_assets else None,a.workers)
    else:
        (ROOT/'assets/local').mkdir(parents=True,exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='initial-smoke-',dir=ROOT/'assets/local') as d:run(Path(d)/'evidence',workers=a.workers)

if __name__=='__main__':main()
