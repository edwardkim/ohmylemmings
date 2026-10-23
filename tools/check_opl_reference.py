#!/usr/bin/env python3
"""Exact mono PCM comparison against pinned C Nuked-OPL3 buffered delivery, offline by default."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import tempfile
try:
    from .check_audio import ROOT,synthetic_image,build,native,pcm_info,guard_original
    from .state_replay import verified_assets
except ImportError:
    from check_audio import ROOT,synthetic_image,build,native,pcm_info,guard_original
    from state_replay import verified_assets

MANIFEST=ROOT/'assets/manifests/opl_reference.json'

def verify_sources():
    manifest=json.loads(MANIFEST.read_text())
    for row in manifest['files']:
        raw=(ROOT/row['path']).read_bytes()
        if len(raw)!=row['size'] or hashlib.sha256(raw).hexdigest()!=row['sha256']:raise ValueError('pinned C source differs')
    return manifest

def timeline(rows,rate,frames):
    events=[];expected=0
    for row in rows:
        if type(row['step']) is not int or row['step']!=expected:raise ValueError('nonconsecutive driver calls')
        frame=expected*rate*31250//2190197;expected+=1
        if frame>=frames:break
        for reg,value in row['writes']:
            if type(reg) is not int or type(value) is not int or not 0<=reg<=255 or not 0<=value<=255:raise ValueError('invalid OPL register/value')
            events.append((frame,reg,value))
    calls=(frames*2190197+rate*31250-1)//(rate*31250)
    if expected<calls:raise ValueError('truncated driver timeline')
    return struct.pack('<III',rate,frames,len(events))+b''.join(struct.pack('<IBB',*e) for e in events)

def check(directory,original_assets=None):
    manifest=verify_sources()
    if original_assets:guard_original(directory);original=verified_assets(original_assets.resolve())
    directory.mkdir(parents=True,exist_ok=False);build()
    subprocess.run(['cargo','build','-p','lemmings-resources','--example','trace_adlib','--example','unpack_dat','--release','--locked','--offline'],cwd=ROOT,check=True)
    binary=directory/'c-reference'
    subprocess.run(['cc','-std=c99','-O2','-Wall','-Wextra','-Werror','-I',str(ROOT/'tools/audio/reference'),str(ROOT/'tools/audio/nuked_reference.c'),str(ROOT/'tools/audio/reference/opl3.c'),'-o',str(binary)],check=True)
    cases=[]
    for wave in range(4):
        data=bytearray(synthetic_image());data[3100:3105]=bytes([0xc1,0xff,35,0x82,0x80]);data[3206]=wave;data[3207]=wave
        path=directory/f'wave-{wave}.bin';path.write_bytes(data)
        for rate in [8000,44100,48000,49716,96000,192000]:cases.append(dict(name=f'wave-{wave}-{rate}',image=path.name,rate=rate,num=2190197,den=31250,actions=f'm0:{rate*2}',command='m0'))
    if original_assets:
        subprocess.run([str(ROOT/'target/release/examples/unpack_dat'),str(original_assets/'ADLIB.DAT'),str(directory/'decoded')],check=True,capture_output=True)
        raw=(directory/'decoded/part-000.bin').read_bytes()
        if hashlib.sha256(raw).hexdigest()!='ad4f33eb383c60b5a67945987f27580fc0540e38558d4c158efe1a672db5f490':raise ValueError('original image differs')
        for cmd,count in [('m',21),('e',18)]:
            for i in range(count):cases.append(dict(name=f'original-{cmd}{i}',image='decoded/part-000.bin',rate=44100,num=2190197,den=31250,actions=f'{cmd}{i}:88200',command=f'{cmd}{i}'))
    rows=[]
    for case in cases:
        calls=(case['rate']*2*2190197+case['rate']*31250-1)//(case['rate']*31250)
        trace=subprocess.check_output([str(ROOT/'target/release/examples/trace_adlib'),str(directory/case['image']),case['command']+':'+str(calls)],text=True)
        data=timeline([json.loads(x) for x in trace.splitlines()],case['rate'],case['rate']*2)
        reference=subprocess.check_output([str(binary)],input=data)
        actual=native(directory,case,'pcm','1,127,512,3,2048,7,8192')
        if reference!=actual:raise ValueError('Rust/C PCM mismatch: '+case['name'])
        info=pcm_info(actual)
        if not info['nonzero']:raise ValueError('silent reference: '+case['name'])
        rows.append(dict(name=case['name'],**info))
        if original_assets:(directory/(case['name']+'.pcm')).write_bytes(actual)
    result=dict(status='PASS',scope='Exact mono left PCM; fixed Rust core/stream scheduler versus separate-process C1.8 buffered delivery; not hardware/device listening',cases=rows,case_count=len(rows),frames=sum(v['frames'] for v in rows),c_commit=manifest['commit'],compiler=subprocess.check_output(['cc','--version'],text=True).splitlines()[0])
    if original_assets:
        if verified_assets(original_assets.resolve())!=original:raise ValueError('originals changed')
        result.update(original_files=len(original),music=21,effects=18)
    (directory/'summary.json').write_text(json.dumps(result,indent=2)+'\n');return result

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--evidence',type=Path);p.add_argument('--original-assets',type=Path);args=p.parse_args()
    if args.original_assets and not args.evidence:p.error('original comparison requires ignored --evidence')
    if args.evidence:result=check(args.evidence.resolve(),args.original_assets)
    else:
        with tempfile.TemporaryDirectory(prefix='lemmings-opl-reference-') as t:result=check(Path(t)/'evidence')
    print(json.dumps({k:v for k,v in result.items() if k!='cases'}))

if __name__=='__main__':main()
