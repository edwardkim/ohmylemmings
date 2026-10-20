#!/usr/bin/env python3
"""Execute real Chromium WASM and compare every boundary with native Rust."""
import argparse
import functools
import hashlib
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile
import threading
try:
    from .web_build import ROOT, build
    from .web_fixture import bank
except ImportError:
    from web_build import ROOT, build
    from web_fixture import bank


def cases():
    rows = [dict(name=name,scene=name,pacing='reference',steps=steps,commands=commands)
            for name,steps,commands in [('win',220,[]),('loss',350,[]),('mixed',380,[[60,'Nuke',0]]),('particles',260,[[60,'AssignBomber',0]]),('zero',5,[])]]
    for skill in ['Climber','Floater','Bomber','Blocker','Builder','Basher','Miner','Digger']:
        rows.append(dict(name=skill,scene='builder',pacing='dos' if skill in ['Climber','Miner'] else 'reference',steps=160,commands=[[20,'SetReleaseRate',99],[80,'Assign'+skill,0],[90,'Pause',0],[96,'Resume',0]]))
    for skill,code in [('Climber','KeyC'),('Floater','KeyF'),('Bomber','KeyE'),('Blocker','KeyK'),('Builder','KeyU'),('Basher','KeyB'),('Miner','KeyM'),('Digger','KeyD')]:
        rows.append(dict(name='pointer-'+skill,scene='builder',pacing='reference',steps=120,commands=[[20,'SetReleaseRate',99],[80,'Key',code,0],[80,'Pointer','move',75,95,0,0,0],[80,'Pointer','down',75,95,2,2,1],[80,'Pointer','down',75,95,0,3,1],[82,'Release']]))
    rows.append(dict(name='panel-camera-focus',scene='builder',pacing='reference',steps=100,commands=[[0,'Key','Space',0],[0,'Key','Space',1],[1,'Key','Space',0],[2,'Pointer','down',24,185,0,1,1],[7,'Release'],[10,'Pointer','down',104,185,0,1,1],[11,'Pointer','down',280,185,0,1,1],[12,'Pointer','move',250,185,0,1,0],[13,'Release'],[14,'Key','Home',0],[15,'Key','End',0],[16,'Key','ArrowLeft',0],[17,'Key','ArrowRight',0],[80,'Pointer','down',184,185,0,1,1],[81,'Pointer','down',184,185,0,1,2]]))
    rows.append(dict(name='result-lifecycle',scene='win',pacing='reference',steps=650,interactive=True,commands=[[220,'Continue'],[550,'Continue'],[600,'Restart']]))
    rows.append(dict(name='byte-bank-lifecycle',scene='original:1',pacing='dos',steps=100,interactive=True,commands=[[10,'Key','KeyU',0],[10,'Pointer','down',280,185,0,1,1],[20,'Level',2],[40,'Restart'],[60,'Level',30]]))
    return rows


def exact(stream, size):
    data = stream.read(size)
    if len(data) != size: raise ValueError('truncated native web trace')
    return data


def native_rows(binary, test):
    commands = ''.join(' '.join(map(str,row))+'\n' for row in test['commands'])
    arguments=[str(binary),test['scene'],test['pacing'],str(test['steps'])]
    if test.get('asset_directory'): arguments.append(str(test['asset_directory']))
    if test.get('interactive'): arguments.append('--interactive')
    process = subprocess.Popen(arguments,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    try:
        process.stdin.write(commands.encode());process.stdin.close();rows=[]
        while header := process.stdout.read(4):
            values=[]
            for index in range(5):
                if index: header=exact(process.stdout,4)
                if len(header)!=4: raise ValueError('truncated native web trace header')
                size=struct.unpack('<I',header)[0]
                if size>2_000_000: raise ValueError('oversized native web trace record')
                values.append(exact(process.stdout,size))
            row=json.loads(values[0])
            for name,data in zip(['terrain','bricks','objects','frame'],values[1:]): row[name]=hashlib.sha256(data).hexdigest()
            rows.append(row)
        error=process.stderr.read().decode();status=process.wait(timeout=10)
        if status: raise RuntimeError(f'native web trace failed: {error}')
        if not rows: raise ValueError('empty native web trace')
        return rows
    finally:
        if process.poll() is None: process.kill();process.wait()
        for stream in [process.stdout,process.stderr]: stream.close()


def check(directory, browser=None, original_assets=None):
    directory=directory.resolve();directory.mkdir(parents=True,exist_ok=True)
    original_files=[]
    if original_assets:
        if not directory.is_relative_to(ROOT/'assets/local'):
            raise ValueError('original evidence must remain in ignored assets/local')
        try: from .state_replay import verified_assets
        except ImportError: from state_replay import verified_assets
        original_files=verified_assets(original_assets.resolve())
    build(directory)
    subprocess.run(['cargo','build','-p','lemmings-web','--bin','web_trace','--release','--locked','--offline'],cwd=ROOT,check=True)
    binary=ROOT/'target/release/web_trace'
    tests=cases()
    fixture=directory/'fixture';fixture.mkdir(exist_ok=True)
    for name,data in bank().items(): (fixture/name).write_bytes(data)
    for test in tests:
        if test['scene'].startswith('original:'): test['asset_directory']=fixture
        test['rows']=native_rows(binary,test)
        test.pop('asset_directory',None)
    selected=[]
    if original_assets:
        import re
        original=directory/'original';original.mkdir(exist_ok=True)
        for entry in original_files:
            name=entry['name']
            if re.fullmatch(r'(MAIN|ODDTABLE|LEVEL00[0-9]|GROUND[0-4]O|VGAGR[0-4])\.DAT|VGALEMMI\.EXE',name):
                shutil.copyfile(original_assets/name,original/name);selected.append(name)
        for level,code in [(1,'KeyD'),(2,'KeyU'),(3,'KeyC')]:
            test=dict(name=f'original-Fun{level:02}',scene=f'original:{level}',pacing='dos',steps=150,asset_directory=original,original=True,interactive=True,commands=[[10,'Key',code,0],[20,'Key','ArrowUp',0],[40,'Key','ArrowRight',0],[50,'Key','Home',0],[70,'Key','Space',0],[72,'Key','Space',0],[100,'Pointer','down',280,185,0,1,1],[110,'Release'],[120,'Restart']])
            probe=native_rows(binary,test)[80];actor=probe['state']['session']['actors'][0]
            map_x=max(208,min(310,208+(actor['x']+16)//16));camera=max(0,min(1280,(map_x-208)*16-176))
            test['commands'] += [[79,'Pointer','down',map_x,185,0,1,1],[79,'Release'],[80,'Pointer','move',actor['x']-camera,actor['y']-5,0,0,0],[80,'Pointer','down',actor['x']-camera,actor['y']-5,0,1,1],[81,'Release']]
            test['rows']=native_rows(binary,test);test.pop('asset_directory');tests.append(test)
    expected=directory/'expected.json';expected.write_text(json.dumps(dict(cases=tests,original_files=selected),separators=(',',':'))+'\n')
    for name in ['check.html','check.js','dom_check.js']: shutil.copyfile(ROOT/'tools/web'/name,directory/name)
    ready=threading.Event();result={}
    class Handler(SimpleHTTPRequestHandler):
        def log_message(self, *args): pass
        def do_POST(self):
            if self.path!='/__result': self.send_error(404);return
            size=int(self.headers.get('Content-Length','0'))
            if size>100000: self.send_error(413);return
            try:
                data=json.loads(self.rfile.read(size));result.update(data)
                self.send_response(200);self.end_headers();self.wfile.write(b'OK');ready.set()
            except (ValueError,TypeError): self.send_error(400)
    browser=browser or next((shutil.which(name) for name in ['google-chrome','chromium','chromium-browser'] if shutil.which(name)),None)
    if not browser: raise ValueError('Chrome/Chromium required for make check-web')
    with ThreadingHTTPServer(('127.0.0.1',0),functools.partial(Handler,directory=directory)) as server:
        thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
        url=f'http://127.0.0.1:{server.server_port}/check.html'
        with tempfile.TemporaryDirectory(prefix='lemmings-web-browser-') as profile, (directory/'browser.log').open('w') as log:
            command=[browser,'--headless','--no-first-run','--no-default-browser-check','--disable-gpu',f'--user-data-dir={profile}',url]
            process=subprocess.Popen(command,stdout=log,stderr=subprocess.STDOUT)
            try:
                for _ in range(180):
                    if ready.wait(1):break
                    if process.poll() is not None:raise RuntimeError(f'browser exited: see {directory}/browser.log')
                if not ready.is_set():raise TimeoutError(f'browser check timed out: see {directory}/browser.log')
            finally:
                if process.poll() is None:
                    process.terminate()
                    try: process.wait(timeout=10)
                    except subprocess.TimeoutExpired: process.kill();process.wait()
                server.shutdown();thread.join(timeout=5)
    result.update(browser=subprocess.check_output([browser,'--version'],text=True).strip(),expected_sha256=hashlib.sha256(expected.read_bytes()).hexdigest(),scope='real WASM versus native wrapper: every full diagnostic state and all terrain/brick/object bytes by SHA-256; all rendered RGB and opaque Canvas readback; synthetic/invented bytes plus explicitly selected verified originals' if original_assets else 'real WASM versus native wrapper: every full diagnostic state and all terrain/brick/object bytes by SHA-256; all rendered RGB and opaque Canvas readback; synthetic and invented file bytes only')
    (directory/'summary.json').write_text(json.dumps(result,indent=2)+'\n')
    if result.get('status')!='PASS':raise AssertionError(result)
    print(json.dumps({k:v for k,v in result.items() if k!='runs_detail'}));return result


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--evidence',type=Path);parser.add_argument('--browser')
    parser.add_argument('--original-assets',type=Path,help='optional verified local originals; evidence must stay under assets/local')
    args=parser.parse_args()
    if args.original_assets and not args.evidence: parser.error('--original-assets requires --evidence under assets/local')
    if args.evidence: check(args.evidence,args.browser,args.original_assets)
    else:
        with tempfile.TemporaryDirectory(prefix='lemmings-web-check-') as directory:check(Path(directory),args.browser)


if __name__=='__main__':main()
