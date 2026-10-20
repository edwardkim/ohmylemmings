#!/usr/bin/env python3
"""Compare every AdLib catalog field with synthetic data or pinned TS initialization."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import tempfile
try:
    from .state_replay import ROOT,verified_assets
except ImportError:
    from state_replay import ROOT,verified_assets


def synthetic():
    data=bytearray(22125);music=[];effects=[]
    def word(at,value):struct.pack_into('<H',data,at,value)
    for i in range(21):
        h=2968+i*64;count=8 if i==20 else 2
        word(2926+i*2,h);word(h,0x1200+i);word(h+2,h+48-2926);data[h+4]=i%4;data[h+5]=count
        programs=[];bars=[]
        for c in range(count):
            p=h+24+c*2;bar=h+40+c;word(h+6+c*2,p-2926);word(p,bar-2926);data[bar]=c+5;programs.append(p);bars.append(bar)
        music.append(dict(index=i,header_offset=h,sample_rate_factor=0x1200+i,instrument_offset=h+48,wait_cycles=i%4,channel_offsets=programs,first_bar_offsets=bars))
    for i in range(18):
        offset=22124 if i==17 else 22025+i*4;word(21989+i*2,offset);effects.append(dict(index=i,data_offset=offset))
    expected=dict(music=music,effects=effects,level_tracks=[music[i%21]['header_offset'] for i in range(120)])
    return bytes(data),expected


def validate(actual,expected):
    def typed(value):
        if type(value) is int:return
        if type(value) is list:
            for v in value:typed(v)
            return
        if type(value) is dict:
            for v in value.values():typed(v)
            return
        raise ValueError('catalog fields must be integer values')
    typed(actual)
    if set(actual)!={'music','effects','level_tracks'} or [len(actual[k]) for k in ['music','effects','level_tracks']]!=[21,18,120]:raise ValueError('incomplete catalog')
    if actual!=expected:raise ValueError('catalog fields or displayed order differ')
    return dict(status='PASS',music=21,effects=18,music_channels=sum(len(m['channel_offsets']) for m in actual['music']),levels=120)


def build():
    subprocess.run(['cargo','build','-p','lemmings-resources','--example','inspect_adlib','--example','unpack_dat','--release','--locked','--offline'],cwd=ROOT,check=True)


def inspect(path,raw=False):
    command=[str(ROOT/'target/release/examples/inspect_adlib')]+(['--image'] if raw else [])+[str(path)]
    return json.loads(subprocess.check_output(command,text=True,cwd=ROOT))


def check_synthetic(directory):
    build();directory.mkdir(parents=True,exist_ok=True);data,expected=synthetic();path=directory/'synthetic.bin';path.write_bytes(data)
    actual=inspect(path,raw=True);summary=validate(actual,expected)
    for name,value in [('expected.json',expected),('rust.json',actual),('summary.json',summary)]:
        (directory/name).write_text(json.dumps(value,indent=2)+'\n')
    return summary


def check_original(assets,directory):
    if not directory.resolve().is_relative_to((ROOT/'assets/local').resolve()):raise ValueError('original evidence must remain ignored under assets/local')
    originals=verified_assets(assets.resolve())
    manifest=json.loads((ROOT/'assets/manifests/reference_sources.json').read_text());source=ROOT/manifest['local_directory'];sources=[]
    required={'lemmings-ts__public__data__config.json','lemmings-ts__src__game__resources__file__binary-reader.ts','lemmings-ts__src__game__resources__sound__sound-image-player.ts','lemmings-ts__src__game__resources__sound__sound-image-channel.ts'}
    for entry in manifest['files']:
        if entry['name'] not in required:continue
        raw=(source/entry['name']).read_bytes()
        if len(raw)!=entry['size'] or hashlib.sha256(raw).hexdigest()!=entry['sha256']:raise ValueError('pinned audio source mismatch: '+entry['name'])
        sources.append({k:entry[k] for k in ['name','size','sha256']})
    if {e['name'] for e in sources}!=required:raise ValueError('missing pinned initialization source')
    directory.mkdir(parents=True,exist_ok=False);build()
    subprocess.run([str(ROOT/'target/release/examples/unpack_dat'),str(assets/'ADLIB.DAT'),str(directory/'decoded')],check=True,capture_output=True,text=True)
    decoded=directory/'decoded/part-000.bin';raw=decoded.read_bytes();baseline=json.loads((ROOT/'assets/manifests/dos_compatibility.json').read_text())['audio']
    if len(raw)!=baseline['decoded_size'] or hashlib.sha256(raw).hexdigest()!=baseline['decoded_sha256']:raise ValueError('decoded AdLib baseline differs')
    reference=json.loads(subprocess.check_output(['node',str(ROOT/'tools/adlib_reference.mjs'),str(source),str(decoded)],text=True))
    expected={k:reference[k] for k in ['music','effects']};expected['level_tracks']=[expected['music'][i%21]['header_offset'] for i in range(120)]
    actual=inspect(assets/'ADLIB.DAT');summary=validate(actual,expected)
    for key in ['music','effects']:
        for new,old in zip(actual[key],baseline[key],strict=True):
            if any(new[k]!=v for k,v in old.items() if k!='id'):raise ValueError('existing catalog baseline differs')
    if verified_assets(assets.resolve())!=originals:raise ValueError('originals changed during inspection')
    summary.update(reference_scope=reference['scope'],configured_tracks=reference['configured_tracks'],adopted_tracks=reference['adopted_tracks'],decoded_sha256=baseline['decoded_sha256'],sources=sources,original_files=len(originals),dos_audio_verified=False,pcm_verified=False)
    for name,value in [('rust.json',actual),('reference.json',reference),('summary.json',summary)]:
        (directory/name).write_text(json.dumps(value,indent=2)+'\n')
    return summary


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--original-assets',type=Path);parser.add_argument('--evidence',type=Path);args=parser.parse_args()
    if args.original_assets:
        if args.evidence is None:parser.error('original verification requires ignored --evidence')
        summary=check_original(args.original_assets.resolve(),args.evidence.resolve())
    elif args.evidence:summary=check_synthetic(args.evidence.resolve())
    else:
        with tempfile.TemporaryDirectory(prefix='lemmings-adlib-') as p:summary=check_synthetic(Path(p))
    print(json.dumps(summary))

if __name__=='__main__':main()
