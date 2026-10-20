#!/usr/bin/env python3
"""Original-backed game cues/commands and native/actual-WASM PCM, offline by default."""
import argparse
import functools
import hashlib
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import threading
try:
    from .check_audio import ROOT, BINARY, build, synthetic_image, native, pcm_info
except ImportError:
    from check_audio import ROOT, BINARY, build, synthetic_image, native, pcm_info

FIXTURE=ROOT/'tools/tests/fixtures/dos_sound.json'
GOLDEN=ROOT/'tools/tests/fixtures/game_sound.json'
PCM_NAMES=['startup-56','splat-exit','assign-basher','expiry-0-255','builder-3','trap-11']
RATE=44100


def uint(value, maximum):
    return type(value) is int and 0 <= value <= maximum


def validate_fixture(data):
    if data.get('status')!='PASS' or data.get('exe_sha256')!='4f21e94d53b7ea814ade802b4a37d42481da3876e31fb5a23090e19c9600f21c':
        raise ValueError('original sound oracle identity differs')
    rows=data['effect_cases'];music=data['music_cases']
    if len(rows)!=97 or len({r['name'] for r in rows})!=97 or len(music)!=170:
        raise ValueError('original sound oracle coverage differs')
    for row in rows:
        if set(row)!={'name','begin','stop','calls','state_flags','frame','timer'}:
            raise ValueError('oracle must contain metadata only')
        validate_requests(row,row['calls'])
        if not all(uint(row[k],65535 if k in ['begin','stop','state_flags'] else 255) for k in ['begin','stop','state_flags','frame','timer']):
            raise ValueError('invalid original trace metadata')
    coverage=set()
    for row in music:
        if set(row)!={'special','advance','cursor','next_cursor','raw_music'} or type(row['advance']) is not bool or not uint(row['special'],4) or not uint(row['cursor'],16) or not uint(row['next_cursor'],16) or not uint(row['raw_music'],21) or row['raw_music']==0:
            raise ValueError('invalid original music case')
        coverage.add((row['special'],row['advance'],row['cursor']))
    if len(coverage)!=170 or sorted(data['normal_music']+data['special_music'])!=list(range(1,22)):
        raise ValueError('missing original music combinations')


def validate_requests(actual, expected):
    calls=actual['calls']
    if type(calls) is not list or any(type(v) is not list or len(v)!=2 or not all(uint(x,65535) for x in v) for v in calls) or calls!=expected:
        raise ValueError('original sound request mismatch: '+actual['name'])


def validate_trace(data, fixture):
    expected={r['name']:r for r in fixture['effect_cases']}
    rows=data['cases'];names=[r['name'] for r in rows]
    if len(names)!=len(set(names)) or set(names)!=set(expected)|{'splat-exit'}:
        raise ValueError('incomplete game sound scenarios')
    for row in rows:
        calls=([[97,0x408],[97,0x410]] if row['name']=='splat-exit' else
               [v for v in expected[row['name']]['calls'] if v[1]>>8 in [3,4]])
        validate_requests(row,calls)
        batch=row['sounds']
        if batch['dropped']!=0 or batch['sequence_exhausted'] or type(batch['reset']) is not bool:
            raise ValueError('lost synthetic sound events')
        previous=-1
        for event in batch['events']:
            if not uint(event['sequence'],2**64-1) or event['sequence']<=previous:
                raise ValueError('unordered game sound events')
            previous=event['sequence']
        pcm_actions(row)
    if data['music_cases']!=fixture['music_cases']:
        raise ValueError('original music cursor/selection mismatch')
    lifecycle=data['lifecycle']
    if not lifecycle['failed_preserved'] or lifecycle['next']['generation']!=1 or lifecycle['retry']['generation']!=1 or not lifecycle['next']['reset'] or not lifecycle['retry']['reset']:
        raise ValueError('sound generation/reset mismatch')
    if lifecycle['next']['music']!=6 or lifecycle['retry']['music']!=5 or lifecycle['old']['events'][-1]['command']!=['stop']:
        raise ValueError('result music/stop mismatch')


def pcm_actions(case):
    start=case['start_control_step'];steps=case['steps']
    if not uint(start,2**64-1) or not uint(steps,1000) or steps==0:
        raise ValueError('invalid game sound frame interval')
    end=(steps+2)*RATE*156250//2190197
    items=[];previous=start
    for event in case['sounds']['events']:
        step=event['control_step'];command=event['command']
        if not uint(step,2**64-1) or not start<=step<start+steps or step<previous:
            raise ValueError('game sound command outside interval or unordered')
        previous=step
        if command is None:continue
        if type(command) is not list or not command:
            raise ValueError('invalid game sound command')
        if command[0] in ['music','effect'] and len(command)==2 and uint(command[1],20 if command[0]=='music' else 17):
            label=('m' if command[0]=='music' else 'e')+str(command[1])
        elif command==['stop']:label='s'
        else:raise ValueError('invalid game sound command')
        items.append(((step-start)*RATE*156250//2190197,label))
    actions=[]
    if not items:return 'n:'+str(end)
    if items[0][0]:actions.append('n:'+str(items[0][0]))
    for i,(frame,label) in enumerate(items):
        until=items[i+1][0] if i+1<len(items) else end
        actions.append(label+':'+str(until-frame))
    return ','.join(actions)


def validate_browser(actual, expected):
    if actual.get('status')!='PASS' or actual.get('trace_sha256')!=expected['trace_sha256'] or actual.get('pcm')!=expected['pcm']:
        raise ValueError('incomplete or different actual WASM game sound/PCM')
    if any(type(r.get('frames')) is not int for r in actual['pcm']):
        raise ValueError('invalid browser PCM counts')


def browser_check(directory, expected):
    for name in ['game-check.html','game-check.js']:
        shutil.copyfile(ROOT/'tools/audio'/name,directory/name)
    (directory/'expected.json').write_text(json.dumps(expected,separators=(',',':'))+'\n')
    ready=threading.Event();result={}
    class Handler(SimpleHTTPRequestHandler):
        def log_message(self,*args):pass
        def do_POST(self):
            if self.path!='/__result':self.send_error(404);return
            size=int(self.headers.get('Content-Length','0'))
            if not 0<size<100000:self.send_error(413);return
            try:result.update(json.loads(self.rfile.read(size)))
            except (ValueError,TypeError):self.send_error(400);return
            self.send_response(200);self.end_headers();self.wfile.write(b'OK');ready.set()
    browser=next((shutil.which(x) for x in ['google-chrome','chromium','chromium-browser'] if shutil.which(x)),None)
    if not browser:raise ValueError('Chrome/Chromium required')
    with ThreadingHTTPServer(('127.0.0.1',0),functools.partial(Handler,directory=directory)) as server:
        thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
        with tempfile.TemporaryDirectory(prefix='lemmings-game-sound-browser-') as profile,(directory/'browser.log').open('w') as log:
            process=subprocess.Popen([browser,'--headless','--no-first-run','--no-default-browser-check','--disable-gpu',f'--user-data-dir={profile}',f'http://127.0.0.1:{server.server_port}/game-check.html'],stdout=log,stderr=subprocess.STDOUT)
            try:
                for _ in range(180):
                    if ready.wait(1):break
                    if process.poll() is not None:raise ValueError('game sound browser exited')
                if not ready.is_set():raise TimeoutError('game sound browser timeout')
            finally:
                if process.poll() is None:
                    process.terminate()
                    try:process.wait(timeout=10)
                    except subprocess.TimeoutExpired:process.kill();process.wait()
                server.shutdown();thread.join(timeout=5)
    (directory/'browser-result.json').write_text(json.dumps(result,indent=2)+'\n')
    validate_browser(result,expected)
    return dict(status='PASS',browser=subprocess.check_output([browser,'--version'],text=True).strip())


def check(directory,wasm=False,write_fixture=False):
    fixture=json.loads(FIXTURE.read_text());validate_fixture(fixture)
    directory.mkdir(parents=True,exist_ok=False);build(wasm,directory)
    raw=subprocess.check_output([str(BINARY),'game-sound','1']).strip()
    data=json.loads(raw);validate_trace(data,fixture)
    other=subprocess.check_output([str(BINARY),'game-sound','7,3,1000']).strip()
    if raw!=other:raise ValueError('game sound delivery partition mismatch')
    trace_sha=hashlib.sha256(raw).hexdigest()
    (directory/'game-trace.json').write_bytes(raw+b'\n')
    (directory/'synthetic.bin').write_bytes(synthetic_image(8))
    cases={r['name']:r for r in data['cases']};pcm=[]
    for name in PCM_NAMES:
        sequence=pcm_actions(cases[name]);case=dict(image='synthetic.bin',rate=RATE,num=2190197,den=31250,actions=sequence)
        reference=native(directory,case,'direct');info=pcm_info(reference)
        for partitions in ['8192','1,127,512,3,2048,7,8192']:
            if native(directory,case,'pcm',partitions)!=reference:raise ValueError('game commands to PCM mismatch: '+name)
        if not info['nonzero']:raise ValueError('silent game command PCM: '+name)
        pcm.append(dict(name=name,frames=info['frames'],sha256=info['sha256']))
    expected=dict(trace_sha256=trace_sha,pcm=pcm)
    if write_fixture:GOLDEN.write_text(json.dumps(dict(scope='Synthetic game trace and PCM only; original x86 oracle metadata separately pinned in dos_sound.json',**expected),indent=2)+'\n')
    else:
        pinned=json.loads(GOLDEN.read_text())
        if any(pinned[k]!=v for k,v in expected.items()):raise ValueError('fixed synthetic game sound/PCM golden differs')
    result=dict(status='PASS',game_cases=len(data['cases']),original_effect_branches=97,original_music_combinations=170,trace_sha256=trace_sha,pcm_cases=len(pcm),pcm_frames=sum(v['frames'] for v in pcm),device_verified=False)
    if wasm:result['wasm']=browser_check(directory,expected)
    (directory/'summary.json').write_text(json.dumps(result,indent=2)+'\n');return result


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--evidence',type=Path);p.add_argument('--wasm',action='store_true');p.add_argument('--write-fixture',action='store_true');args=p.parse_args()
    if args.evidence:result=check(args.evidence.resolve(),args.wasm,args.write_fixture)
    else:
        with tempfile.TemporaryDirectory(prefix='lemmings-game-sound-') as temp:result=check(Path(temp)/'evidence',args.wasm,args.write_fixture)
    print(json.dumps(result))


if __name__=='__main__':main()
