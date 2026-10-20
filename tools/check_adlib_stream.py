#!/usr/bin/env python3
"""Offline synthetic ADLIB streams; optional original x86 driver/state comparison."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import tempfile
try:
    from .state_replay import ROOT, verified_assets
except ImportError:
    from state_replay import ROOT, verified_assets

FIXTURE=ROOT/'tools/tests/fixtures/adlib_stream.json'
BINARY=ROOT/'target/release/examples/trace_adlib'

def canonical(value):
    return json.dumps(value,sort_keys=True,separators=(',',':')).encode()

def digest(value):
    return hashlib.sha256(canonical(value)).hexdigest()

def synthetic(*,channels=1,wait=2,bar=None,effect=None):
    b=bytearray(22125)
    def word(p,v):struct.pack_into('<H',b,p,v)
    for i in range(21):word(2926+i*2,2968)
    word(2968,0x4300);word(2970,3200-2926);b[2972]=wait;b[2973]=channels
    for c in range(channels):
        p=3000+c*6;word(2974+c*2,p-2926);word(p,3100-2926);word(p+2,0);word(p+4,p-2926)
    bar=bar or [0xc1,0xe2,10,0x80]
    b[3100:3100+len(bar)]=bytes(bar)
    for i in range(18):word(21989+i*2,22025)
    effect=effect or [0xc1,0xe2,10,0x85]
    b[22025:22025+len(effect)]=bytes(effect)
    inst=bytes([1,2,3,4,5,6,7,8,0,9,10,11,0xc3,0,1,0])
    for i in range(31):b[3200+i*16:3216+i*16]=inst
    for i in range(16):b[21731+i*16:21747+i*16]=inst
    for i in range(128):b[2215+i]=i&63
    b[2727:2823]=bytes([2]*96);b[2823:2919]=bytes(96);word(2343,0x8123)
    return bytes(b)

def scenarios():
    crossing=bytearray(synthetic(effect=[0xc1,0xe2,0,0x85]));crossing[21739]=246;crossing[2727+250]=255;crossing[2823+250]=31
    struct.pack_into('<H',crossing,2343+15*32,0x0123)
    return [
        ('loop',synthetic(),'m0:90'),
        ('controls',synthetic(bar=[0xdf,0xe1,0x84,0xfe,0x86,10,0x82,0x87,12,0x81,0xb0,127,0x80]),'m0:100'),
        ('rest',synthetic(bar=[0xe2,0x81,0xc1,10,0x85]),'m0:20'),
        ('song-end',synthetic(channels=3,bar=[0xe0,0x83]),'m0:5'),
        ('eight-channels',synthetic(channels=8),'m20:32'),
        ('zero-divider',synthetic(wait=0),'m0:520'),
        ('effect-slots',synthetic(effect=[0xc1,0xff,10,0x85]),'e0:1,e1:1,e2:1,e3:1,e4:40'),
        ('restart-stop',synthetic(),'m0:9,e0:2,m20:9,s:3,m0:10'),
        ('effect-crossing',bytes(crossing),'e14:8'),
        ('volume-abi',synthetic(channels=2,bar=[0xc1,0xe2,10,0xbf,0xff,0x81,0x80]),'m0:25'),
    ]

def build():
    subprocess.run(['cargo','build','-p','lemmings-resources','--example','trace_adlib','--release','--locked','--offline'],cwd=ROOT,check=True)

def actions(sequence):
    for item in sequence.split(','):
        command,count=item.split(':');yield command,int(count)

def trace(path,sequence):
    p=subprocess.Popen([str(BINARY),str(path),sequence],cwd=ROOT,stdout=subprocess.PIPE,text=True)
    try:
        for line in p.stdout:yield json.loads(line)
        if p.wait()!=0:raise ValueError('Rust ADLIB trace failed')
    finally:
        if p.poll() is None:p.terminate();p.wait()
        p.stdout.close()

def record(state):
    state=state.copy();writes=state.pop('writes');return [digest(state),writes]

def validate_record(actual,expected):
    def ints(v):
        if type(v) is int:return
        if type(v) is list:
            for x in v:ints(x)
            return
        if type(v) is dict:
            for x in v.values():ints(x)
            return
        raise ValueError('trace fields must be integers')
    ints(actual)
    if record(actual)!=expected:raise ValueError('state or ordered register stream differs at step '+str(actual.get('step')))

def check_synthetic(directory):
    fixture=json.loads(FIXTURE.read_text());summaries=[]
    if fixture['scope']!='original x86-16 driver over synthetic data; DX=0; no PCM or PIT':raise ValueError('fixture scope differs')
    for name,data,sequence in scenarios():
        expected=fixture['scenarios'][name]
        if expected['image_sha256']!=hashlib.sha256(data).hexdigest() or expected['actions']!=sequence:raise ValueError('synthetic fixture input differs')
        path=directory/(name+'.bin');path.write_bytes(data);count=0
        for count,(actual,want) in enumerate(zip(trace(path,sequence),expected['records'],strict=True),1):validate_record(actual,want)
        summaries.append(dict(name=name,steps=count,writes=sum(len(x[1]) for x in expected['records'])))
    return dict(status='PASS',scope=fixture['scope'],scenarios=summaries,steps=sum(x['steps'] for x in summaries),writes=sum(x['writes'] for x in summaries),fixture_sha256=hashlib.sha256(FIXTURE.read_bytes()).hexdigest())

def reference_synthetic(data,driver):
    # Keep the executable private. Only original-produced synthetic trace hashes
    # and synthetic register values enter the committed offline fixture.
    b=bytearray(data);b[:1452]=driver[:1452]
    for i,(a,c) in enumerate([(0,3),(1,4),(2,5),(8,11),(9,12),(10,13),(16,19),(17,20),(18,21)]):
        b[1452+i*20:1472+i*20]=bytes([0,0,0,0,0,a,c,i,0xb0+i,0,0,0,0,0,0,0,0,0,0,0])
    struct.pack_into('<H',b,0x8a3,0x388);b[0xb68:0xb6e]=bytes([0,0,9,0,1,0]);return bytes(b)

def create_fixture(driver_path,output):
    from adlib_driver_reference import Driver
    driver=driver_path.read_bytes();Driver(driver) # Verify original before executing synthetic overlays.
    result=dict(scope='original x86-16 driver over synthetic data; DX=0; no PCM or PIT',scenarios={})
    for name,data,sequence in scenarios():
        oracle=Driver(reference_synthetic(data,driver),verify=False);records=[]
        for command,count in actions(sequence):
            oracle.request(command)
            for _ in range(count):records.append(record(oracle.read()))
        result['scenarios'][name]=dict(image_sha256=hashlib.sha256(data).hexdigest(),actions=sequence,records=records)
    output.write_text(json.dumps(result,indent=2)+'\n')

def check_original(assets,directory):
    from adlib_driver_reference import Driver
    if not directory.resolve().is_relative_to((ROOT/'assets/local').resolve()):raise ValueError('original evidence must remain ignored under assets/local')
    original=verified_assets(assets.resolve());directory.mkdir(parents=True,exist_ok=False)
    subprocess.run(['cargo','build','-p','lemmings-resources','--example','unpack_dat','--release','--locked','--offline'],cwd=ROOT,check=True)
    subprocess.run([str(ROOT/'target/release/examples/unpack_dat'),str(assets/'ADLIB.DAT'),str(directory/'decoded')],check=True,capture_output=True)
    path=directory/'decoded/part-000.bin';data=path.read_bytes();Driver(data)
    scenarios_to_check=[('music-'+str(i),'m'+str(i)+':40000') for i in range(21)]+[('effect-'+str(i),'e'+str(i)+':512') for i in range(18)]
    scenarios_to_check += [('mixed','m20:200,e0:1,e1:1,e2:1,e3:1,e4:256,m0:500,s:3,m0:500'),('restart','m0:1000,m0:1000,s:1,m20:1000')]
    summaries=[]
    for name,sequence in scenarios_to_check:
        oracle=Driver(data);events=iter(actions(sequence));command,remaining=next(events);oracle.request(command)
        state_hash=hashlib.sha256();stream_hash=hashlib.sha256();steps=0;writes=0
        iterator=trace(path,sequence)
        try:
            for actual in iterator:
                expected=oracle.read();validate_record(actual,record(expected));state_hash.update(canonical(expected));stream_hash.update(canonical(expected['writes']));steps+=1;writes+=len(expected['writes'])
                remaining-=1
                if remaining==0:
                    event=next(events,None)
                    if event:command,remaining=event;oracle.request(command)
                if name.startswith('music-'):
                    active=[i for i,c in enumerate(expected['channels']) if c[16]==1]
                    if not active or all(oracle.channel_loops[i]>=2 for i in active):break
                elif name.startswith('effect-') and not any(c[16] for c in expected['channels']):break
        finally:iterator.close()
        if name.startswith('music-') and active and not all(oracle.channel_loops[i]>=2 for i in active):raise ValueError('music loop bound not reached: '+name)
        if name.startswith('effect-') and any(c[16] for c in expected['channels']):raise ValueError('effect did not terminate: '+name)
        summary=dict(name=name,actions=sequence,steps=steps,writes=writes,channel_loops=oracle.channel_loops,stops=oracle.stops,opcodes={hex(k):v for k,v in sorted(oracle.opcodes.items())},state_sha256=state_hash.hexdigest(),stream_sha256=stream_hash.hexdigest())
        summaries.append(summary);(directory/(name+'.json')).write_text(json.dumps(summary,indent=2)+'\n');print(name,steps,'steps',writes,'writes',flush=True)
    if verified_assets(assets.resolve())!=original:raise ValueError('originals changed')
    result=dict(status='PASS',scope='original x86-16 driver; every channel/global/flag/register and ordered write per call; DX=0',music=21,effects=18,scenarios=summaries,steps=sum(x['steps'] for x in summaries),writes=sum(x['writes'] for x in summaries),original_files=len(original),image_sha256=hashlib.sha256(data).hexdigest(),unicorn='2.1.4',pcm_verified=False,pit_verified=False)
    (directory/'summary.json').write_text(json.dumps(result,indent=2)+'\n');return result

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--original-assets',type=Path);parser.add_argument('--evidence',type=Path);parser.add_argument('--fixture-driver',type=Path);parser.add_argument('--fixture-output',type=Path);args=parser.parse_args()
    if args.fixture_driver:
        if not args.fixture_output:parser.error('--fixture-output is required')
        create_fixture(args.fixture_driver,args.fixture_output);return
    build()
    if args.original_assets:
        if not args.evidence:parser.error('original verification requires --evidence')
        result=check_original(args.original_assets,args.evidence)
    else:
        with tempfile.TemporaryDirectory(prefix='lemmings-adlib-stream-') as p:result=check_synthetic(Path(p))
    print(json.dumps(result))

if __name__=='__main__':main()
