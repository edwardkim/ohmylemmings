#!/usr/bin/env python3
"""Compare deterministic PCM: direct OPL delivery, streaming Rust and actual WASM."""
import argparse
import array
import functools
import hashlib
from http.server import SimpleHTTPRequestHandler,ThreadingHTTPServer
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import threading
try:
    from .state_replay import ROOT,verified_assets
    from .check_adlib_stream import synthetic
    from .web_build import MANIFEST,TOOLS,verify_bindgen
except ImportError:
    from state_replay import ROOT,verified_assets
    from check_adlib_stream import synthetic
    from web_build import MANIFEST,TOOLS,verify_bindgen

FIXTURE=ROOT/'tools/tests/fixtures/audio_pcm.json'
BINARY=ROOT/'target/release/lemmings-audio-probe'

def guard_original(directory):
    if not directory.resolve().is_relative_to((ROOT/'assets/local').resolve()):raise ValueError('original PCM/evidence must remain ignored')

def synthetic_image(channels=1):
    data=bytearray(synthetic(channels=channels,bar=[0xc1,0xe3,35,0x82,0x81,0x84,0xfe,0x86,30,0x87,28,0x80],effect=[0xc1,0xef,18,0x85]))
    instrument=bytes([0xf2,0xf2,0x05,0x05,0x21,0x21,0,0,0,0x06,0,0,0,0,1,0])
    for base,count in [(3200,31),(21731,16)]:
        for i in range(count):data[base+i*16:base+(i+1)*16]=instrument
    data[2215:2343]=bytes(128)
    data[2343:2345]=(0x8298).to_bytes(2,'little')
    return bytes(data)

def synthetic_cases():
    cases=[]
    for rate in [8000,44100,48000,49716,96000,192000]:
        for num,den,label in [(2190197,31250,'vga'),(596591,8192,'pit')]:
            for voices in [1,8]:
                cases.append(dict(name=f'{label}-{rate}-{voices}',image=f'synthetic-{voices}.bin',rate=rate,num=num,den=den,actions=f'm0:{rate//5},e0:{rate//10},s:{rate//5},r20:{rate//5}'))
    return cases

def pcm_info(raw):
    if len(raw)%2:raise ValueError('odd PCM byte count')
    samples=array.array('h');samples.frombytes(raw)
    if sys.byteorder!='little':samples.byteswap()
    return dict(frames=len(samples),sha256=hashlib.sha256(raw).hexdigest(),peak=max(map(abs,samples),default=0),nonzero=sum(s!=0 for s in samples))

def validate_pcm(raw,expected):
    actual=pcm_info(raw)
    if actual!=expected:raise ValueError('PCM bytes or metrics differ')
    return actual

def validate_browser(result,cases):
    if result.get('status')!='PASS' or type(result.get('cases')) is not list or len(result['cases'])!=len(cases):raise ValueError('incomplete browser PCM result')
    for actual,case in zip(result['cases'],cases,strict=True):
        expected=dict(name=case['name'],frames=case['expected']['frames'],sha256=case['expected']['sha256'])
        if type(actual.get('frames')) is not int or actual!=expected:raise ValueError('browser PCM case differs')

def build(wasm=False,directory=None):
    subprocess.run(['cargo','build','-p','lemmings-audio-probe','--bin','lemmings-audio-probe','--release','--locked','--offline'],cwd=ROOT,check=True)
    if wasm:
        subprocess.run(['cargo','build','-p','lemmings-audio-probe','--lib','--target','wasm32-unknown-unknown','--release','--locked','--offline'],cwd=ROOT,check=True)
        manifest=json.loads(MANIFEST.read_text());tool=verify_bindgen(manifest,TOOLS)
        subprocess.run([str(tool),'--target','web','--out-dir',str(directory/'pkg'),'--out-name','audio_probe',str(ROOT/'target/wasm32-unknown-unknown/release/lemmings_audio_probe.wasm')],cwd=ROOT,check=True)

def native(directory,case,kind='pcm',partitions='8192'):
    return subprocess.check_output([str(BINARY),str(directory/case['image']),str(case['rate']),str(case['num']),str(case['den']),case['actions'],partitions,kind],cwd=ROOT)

def check_browser(directory,cases,browser=None):
    for name in ['check.html','check.js']:shutil.copyfile(ROOT/'tools/audio'/name,directory/name)
    (directory/'expected.json').write_text(json.dumps(dict(cases=cases),separators=(',',':'))+'\n')
    ready=threading.Event();result={}
    class Handler(SimpleHTTPRequestHandler):
        def log_message(self,*args):pass
        def do_POST(self):
            if self.path!='/__result':self.send_error(404);return
            size=int(self.headers.get('Content-Length','0'))
            if not 0<size<100000:self.send_error(413);return
            try:
                result.update(json.loads(self.rfile.read(size)));self.send_response(200);self.end_headers();self.wfile.write(b'OK');ready.set()
            except (ValueError,TypeError):self.send_error(400)
    browser=browser or next((shutil.which(x) for x in ['google-chrome','chromium','chromium-browser'] if shutil.which(x)),None)
    if not browser:raise ValueError('Chrome/Chromium required for actual audio WASM check')
    with ThreadingHTTPServer(('127.0.0.1',0),functools.partial(Handler,directory=directory)) as server:
        thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
        with tempfile.TemporaryDirectory(prefix='lemmings-audio-browser-') as profile,(directory/'browser.log').open('w') as log:
            process=subprocess.Popen([browser,'--headless','--no-first-run','--no-default-browser-check','--disable-gpu',f'--user-data-dir={profile}',f'http://127.0.0.1:{server.server_port}/check.html'],stdout=log,stderr=subprocess.STDOUT)
            try:
                for _ in range(180):
                    if ready.wait(1):break
                    if process.poll() is not None:raise ValueError('audio browser exited')
                if not ready.is_set():raise TimeoutError('audio browser timeout')
            finally:
                if process.poll() is None:
                    process.terminate()
                    try:process.wait(timeout=10)
                    except subprocess.TimeoutExpired:process.kill();process.wait()
                server.shutdown();thread.join(timeout=5)
    validate_browser(result,cases);(directory/'browser-result.json').write_text(json.dumps(result,indent=2)+'\n')
    return dict(status='PASS',cases=len(cases),frames=sum(c['expected']['frames'] for c in cases),browser=subprocess.check_output([browser,'--version'],text=True).strip())

def check(directory,*,wasm=False,original_assets=None,write_fixture=False,browser=None):
    if original_assets:
        guard_original(directory)
        if write_fixture:raise ValueError('original PCM cannot enter public synthetic fixture')
        original=verified_assets(original_assets.resolve())
    directory.mkdir(parents=True,exist_ok=False);build(wasm,directory)
    for voices in [1,8]:(directory/f'synthetic-{voices}.bin').write_bytes(synthetic_image(voices))
    cases=synthetic_cases()
    if original_assets:
        subprocess.run(['cargo','build','-p','lemmings-resources','--example','unpack_dat','--release','--locked','--offline'],cwd=ROOT,check=True)
        subprocess.run([str(ROOT/'target/release/examples/unpack_dat'),str(original_assets/'ADLIB.DAT'),str(directory/'decoded')],check=True,capture_output=True)
        raw=(directory/'decoded/part-000.bin').read_bytes()
        if hashlib.sha256(raw).hexdigest()!='ad4f33eb383c60b5a67945987f27580fc0540e38558d4c158efe1a672db5f490':raise ValueError('original ADLIB image differs')
        for command,count in [('m',21),('e',18)]:
            for i in range(count):cases.append(dict(name=f'original-{command}{i}',image='decoded/part-000.bin',rate=22050,num=2190197,den=31250,actions=f'{command}{i}:66150'))
    fixture={} if write_fixture else json.loads(FIXTURE.read_text())['cases'];generated={};rows=[]
    for case in cases:
        raw=native(directory,case,'direct');expected=pcm_info(raw)
        if not case['name'].startswith('original'):
            if write_fixture:generated[case['name']]=expected
            elif expected!=fixture[case['name']]:raise ValueError('fixed direct PCM fixture differs: '+case['name'])
        validate_pcm(native(directory,case,'pcm','8192'),expected);validate_pcm(native(directory,case,'pcm','1,127,512,3,2048,7,8192'),expected)
        if expected['nonzero']==0:raise ValueError('silent case: '+case['name'])
        case['expected']=expected;rows.append(dict(name=case['name'],**expected))
    if write_fixture:
        if original_assets:raise ValueError('original PCM cannot enter public synthetic fixture')
        FIXTURE.write_text(json.dumps(dict(engine='Nuked-OPL3-1.8/rust-0.1.0-no_std',scope='Unmodified OPL core, direct absolute rational delivery; synthetic data only; not hardware PCM',cases=generated),indent=2)+'\n')
    result=dict(status='PASS',engine='Nuked-OPL3-1.8/rust-0.1.0-no_std',native_cases=len(cases),native_deliveries=len(cases)*3,frames=sum(x['frames'] for x in rows),cases=rows,pcm_device_verified=False)
    if wasm:result['wasm']=check_browser(directory,cases,browser)
    if original_assets:
        if verified_assets(original_assets.resolve())!=original:raise ValueError('originals changed')
        result.update(original_files=len(original),original_music=21,original_effects=18)
    (directory/'summary.json').write_text(json.dumps(result,indent=2)+'\n');return result

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--wasm',action='store_true');parser.add_argument('--evidence',type=Path);parser.add_argument('--original-assets',type=Path);parser.add_argument('--write-fixture',action='store_true');args=parser.parse_args()
    if args.original_assets and not args.evidence:parser.error('original comparison requires ignored --evidence')
    if args.evidence:result=check(args.evidence.resolve(),wasm=args.wasm,original_assets=args.original_assets,write_fixture=args.write_fixture)
    else:
        with tempfile.TemporaryDirectory(prefix='lemmings-audio-') as p:result=check(Path(p)/'evidence',wasm=args.wasm,write_fixture=args.write_fixture)
    print(json.dumps({k:v for k,v in result.items() if k!='cases'}))

if __name__=='__main__':main()
