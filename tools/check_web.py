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
except ImportError:
    from web_build import ROOT, build


def cases():
    rows = [dict(name=name,scene=name,pacing='reference',steps=steps,commands=commands)
            for name,steps,commands in [('win',220,[]),('loss',350,[]),('mixed',380,[[60,'Nuke',0]]),('particles',260,[[60,'AssignBomber',0]]),('zero',5,[])]]
    for skill in ['Climber','Floater','Bomber','Blocker','Builder','Basher','Miner','Digger']:
        rows.append(dict(name=skill,scene='builder',pacing='dos' if skill in ['Climber','Miner'] else 'reference',steps=160,commands=[[20,'SetReleaseRate',99],[80,'Assign'+skill,0],[90,'Pause',0],[96,'Resume',0]]))
    return rows


def exact(stream, size):
    data = stream.read(size)
    if len(data) != size: raise ValueError('truncated native web trace')
    return data


def native_rows(binary, test):
    commands = ''.join(f'{step} {name} {value}\n' for step,name,value in test['commands'])
    process = subprocess.Popen([str(binary),test['scene'],test['pacing'],str(test['steps'])],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
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
            state=json.loads(values[0]);row=dict(state=state)
            for name,data in zip(['terrain','bricks','objects','frame'],values[1:]): row[name]=hashlib.sha256(data).hexdigest()
            rows.append(row)
        error=process.stderr.read().decode();status=process.wait(timeout=10)
        if status: raise RuntimeError(f'native web trace failed: {error}')
        if not rows: raise ValueError('empty native web trace')
        return rows
    finally:
        if process.poll() is None: process.kill();process.wait()
        for stream in [process.stdout,process.stderr]: stream.close()


def check(directory, browser=None):
    directory=directory.resolve();directory.mkdir(parents=True,exist_ok=True)
    build(directory)
    subprocess.run(['cargo','build','-p','lemmings-web','--bin','web_trace','--release','--locked','--offline'],cwd=ROOT,check=True)
    binary=ROOT/'target/release/web_trace'
    tests=cases()
    for test in tests: test['rows']=native_rows(binary,test)
    expected=directory/'expected.json';expected.write_text(json.dumps(dict(cases=tests),separators=(',',':'))+'\n')
    for name in ['check.html','check.js']: shutil.copyfile(ROOT/'tools/web'/name,directory/name)
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
    result.update(browser=subprocess.check_output([browser,'--version'],text=True).strip(),expected_sha256=hashlib.sha256(expected.read_bytes()).hexdigest(),scope='real WASM versus native wrapper: every full diagnostic state and all terrain/brick/object bytes by SHA-256; all rendered RGB and opaque Canvas readback; synthetic only')
    (directory/'summary.json').write_text(json.dumps(result,indent=2)+'\n')
    if result.get('status')!='PASS':raise AssertionError(result)
    print(json.dumps({k:v for k,v in result.items() if k!='runs_detail'}));return result


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--evidence',type=Path);parser.add_argument('--browser')
    args=parser.parse_args()
    if args.evidence: check(args.evidence,args.browser)
    else:
        with tempfile.TemporaryDirectory(prefix='lemmings-web-check-') as directory:check(Path(directory),args.browser)


if __name__=='__main__':main()
