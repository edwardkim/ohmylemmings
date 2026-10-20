#!/usr/bin/env python3
"""Run the actual product DOM through a complete synthetic journey, then replay natively."""
import argparse
import base64
import functools
import hashlib
from http.server import SimpleHTTPRequestHandler,ThreadingHTTPServer
import json
import os
from pathlib import Path
import shutil
import struct
import sys
import subprocess
import tempfile
import threading

# Existing native verification helpers also support direct CLI imports.
sys.path.insert(0,str(Path(__file__).resolve().parent))

try:
    from .web_build import ROOT,build
    from .check_web import exact
    from .web_journey import decode,expanded,validate_journey
    from .check_completion import collect
    from .check_desktop import verify_pixels
    from .state_replay import verified_assets
except ImportError:
    from web_build import ROOT,build
    from check_web import exact
    from web_journey import decode,expanded,validate_journey
    from check_completion import collect
    from check_desktop import verify_pixels
    from state_replay import verified_assets


def normalized(row):
    source=row['state'];state=dict(source['session'])
    for name in ['digger','builder','blocker','bomber','climber','floater','exit','hazards','completion','nuke']:state[name]=dict(source[name])
    state['basher']={'remaining':source['remaining_bashers']};state['miner']={'remaining':source['remaining_miners']}
    state['digger']['cleared']=[i for i,a,b in row['changes']['terrain'] if a==1 and b==0]
    state['builder']['filled']=[i for i,a,b in row['changes']['terrain'] if a==0 and b==1]
    state['blocker']['changed']=[[i,b] for i,a,b in row['changes']['objects']]
    return state


def compare_native(data,directory,assets=None,desktop=True):
    if data['original']:
        if assets is None or not directory.resolve().is_relative_to(ROOT/'assets/local'):raise ValueError('verified originals and ignored evidence required')
        verified_assets(assets)
    summary=validate_journey(data);reports=[]
    for index,attempt in enumerate(data['attempts']):
        folder=directory/f'attempt-{index:04}';folder.mkdir()
        rows=list(expanded(attempt));events=attempt['events'];steps=len(rows)-1
        scene=f'original:{attempt["level"]}' if assets else 'win:2' if index==3 else 'win'
        command=[str(ROOT/'target/release/web_trace'),scene,'dos' if assets else 'reference',str(steps)]
        if assets:command.append(str(assets))
        command+=['--interactive']
        script=''.join(' '.join(map(str,[e['step'],e['op'],*e['args']]))+'\n' for e in events)
        (folder/'events.txt').write_text(script);(folder/'inputs.txt').write_text(attempt['final']['inputs'])
        process=subprocess.Popen(command,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        captures=[];count=0
        try:
            process.stdin.write(script.encode());process.stdin.close()
            while header:=process.stdout.read(4):
                values=[]
                for field in range(5):
                    if field:header=exact(process.stdout,4)
                    if len(header)!=4:raise ValueError('truncated native header')
                    size=struct.unpack('<I',header)[0]
                    if size>2_000_000:raise ValueError('oversized native record')
                    values.append(exact(process.stdout,size))
                record=json.loads(values[0]);state=record['state']
                if 'capture' in record:
                    n=record['capture']
                    if n!=len(captures) or n>=len(attempt['frames']):raise ValueError('extra/missing native frame')
                    frame=attempt['frames'][n]
                    if frame['state']!=state or frame['view']!=record['view'] or decode(frame['rgb'])!=values[4]:raise ValueError(f'frame differs at attempt{index}/{n}')
                    captures.append(hashlib.sha256(values[4]).hexdigest())
                else:
                    if count>=len(rows) or state!=rows[count]['state']:raise ValueError(f'full native state differs at attempt{index}/{count}: {state} != {rows[count]["state"] if count<len(rows) else None}')
                    for name,value in zip(['terrain','bricks','objects'],values[1:4]):
                        if hashlib.sha256(value).hexdigest()!=rows[count][name]:raise ValueError(f'{name} differs at attempt{index}/{count}')
                        if count in (0,steps):
                            if value!=decode(attempt['initial' if count==0 else 'final']['maps'][name]):raise ValueError('initial/final bytes differ')
                            (folder/f'{name}-{"initial" if count==0 else "final"}.bin').write_bytes(value)
                    count+=1
            error=process.stderr.read().decode();status=process.wait(timeout=10)
            if status:raise ValueError('native replay failed: '+error)
        finally:
            if process.poll() is None:process.kill();process.wait()
            process.stdout.close();process.stderr.close()
        if count!=len(rows) or len(captures)!=len(attempt['frames']):raise ValueError('native replay incomplete')
        desktop_replays=[]
        if desktop:
            expected=[normalized(r) for r in attempt['rows']]
            for render in [1,7,13]:
                dest=folder/f'desktop-{render}'
                cmd=[str(ROOT/'target/release/lemmings-desktop'),'--steps',str(steps),'--render-every',str(render),'--capture',str(dest),'--replay',str(folder/'inputs.txt'),'--fast']
                cmd+=['--assets',str(assets),'--level',str(attempt['level'])] if assets else ['--synthetic-outcome','loss' if index==3 else 'win']
                subprocess.run(cmd,check=True,capture_output=True,text=True,env=dict(os.environ,SDL_VIDEODRIVER='dummy'),timeout=180)
                actual=collect(dest)
                if actual!=expected:
                    first=next((i for i,(a,b) in enumerate(zip(actual,expected)) if a!=b),min(len(actual),len(expected)))
                    raise ValueError(f'independent SDL state differs at attempt{index}/render{render}/{first}: {actual[first] if first<len(actual) else None} != {expected[first] if first<len(expected) else None}')
                for name,path in [('terrain','terrain-final.bin'),('bricks','brick-final.bin'),('objects','object-map-final.bin')]:
                    if (dest/path).read_bytes()!=decode(attempt['final']['maps'][name]):raise ValueError('independent SDL final map differs')
                readbacks=verify_pixels(dest)
                images=[]
                for path in sorted(dest.glob('frame-*'),key=lambda p:p.as_posix()):
                    raw=path.read_bytes();images.append(dict(path=path.name,bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest()));path.unlink()
                (dest/'verified-images.json').write_text(json.dumps(images,separators=(',',':'))+'\n')
                desktop_replays.append(dict(render_every=render,states=len(actual),pixel_exact_readbacks=readbacks))
        reports.append(dict(attempt=index,level=attempt['level'],states=count,frames=len(captures),final=attempt['final']['state']['completion'],inputs=attempt['final']['inputs'],desktop_replays=desktop_replays))
    summary.update(native_states=sum(r['states'] for r in reports),native_frames=sum(r['frames'] for r in reports),independent_desktop_replays=sum(len(r['desktop_replays']) for r in reports),attempts_detail=reports)
    if assets:verified_assets(assets)
    return summary


def run(directory):
    directory.mkdir(parents=True,exist_ok=True);build(directory)
    subprocess.run(['cargo','build','-p','lemmings-web','--bin','web_trace','-p','lemmings-desktop','--release','--locked','--offline'],cwd=ROOT,check=True)
    for name in ['journey_check.js','journey_recorder.js','journey_replay.js']:shutil.copyfile(ROOT/'tools/web'/name,directory/name)
    html=(directory/'index.html').read_text()
    shim='<script>window.__frames=new Map();let sequence=0;window.requestAnimationFrame=fn=>{const id=++sequence;__frames.set(id,fn);return id;};window.cancelAnimationFrame=id=>__frames.delete(id);</script>'
    html=html.replace('<script type="module" src="./app.js"></script>',shim+'<script type="module" src="./journey_check.js"></script>')
    (directory/'journey.html').write_text(html);ready=threading.Event();result={}
    class Handler(SimpleHTTPRequestHandler):
        def log_message(self,*args):pass
        def do_POST(self):
            if self.path!='/__journey':self.send_error(404);return
            size=int(self.headers.get('Content-Length','0'))
            if not 0<size<150_000_000:self.send_error(413);return
            try:
                data=json.loads(self.rfile.read(size));result.update(data);self.send_response(200);self.end_headers();self.wfile.write(b'OK');ready.set()
            except (ValueError,TypeError):self.send_error(400)
    browser=next((shutil.which(n) for n in ['google-chrome','chromium','chromium-browser'] if shutil.which(n)),None)
    if not browser:raise ValueError('Chrome/Chromium required')
    with ThreadingHTTPServer(('127.0.0.1',0),functools.partial(Handler,directory=directory)) as server:
        thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
        with tempfile.TemporaryDirectory(prefix='lemmings-journey-profile-') as profile,(directory/'browser.log').open('w') as log:
            process=subprocess.Popen([browser,'--headless','--disable-gpu','--no-first-run','--no-default-browser-check',f'--user-data-dir={profile}',f'http://127.0.0.1:{server.server_port}/journey.html'],stdout=log,stderr=subprocess.STDOUT)
            try:
                for _ in range(180):
                    if ready.wait(1):break
                    if process.poll() is not None:raise ValueError('browser exited')
                if not ready.is_set():raise TimeoutError('browser journey timed out')
            finally:
                process.terminate()
                try:process.wait(timeout=10)
                except subprocess.TimeoutExpired:process.kill();process.wait()
                server.shutdown();thread.join(timeout=5)
    if result.get('error'):raise ValueError(result)
    (directory/'journey.json').write_text(json.dumps(result,separators=(',',':'))+'\n')
    summary=compare_native(result,directory)
    replays=result.get('browser_replays',{})
    if replays.get('runs')!=12 or replays.get('states')!=3*summary['states']:raise ValueError('full browser partition replays missing')
    summary['browser_replays']=replays
    summary.update(browser=subprocess.check_output([browser,'--version'],text=True).strip(),scope='actual product DOM with controlled rAF timestamps; full WASM/native raw-input replay and three independent stable-ID desktop replays per attempt; synthetic only')
    (directory/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary));return summary


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--evidence',type=Path);args=parser.parse_args()
    if args.evidence:run(args.evidence.resolve())
    else:
        with tempfile.TemporaryDirectory(prefix='lemmings-web-journey-') as path:run(Path(path))

if __name__=='__main__':main()
