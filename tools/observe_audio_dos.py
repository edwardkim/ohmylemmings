#!/usr/bin/env python3
"""Optional owned DOSBox run: read AdLib selection/VGA IRQ cadence; no listening."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
from types import SimpleNamespace
from capture_dos import SETTINGS,PROCEDURE,identity,record,save_json,sha
from observe_dos import Observer
from audio_timing import cadence, decode_audio_sample
from state_replay import ROOT,verified_assets


def observe(directory,dosbox,xdotool,cycles=3000):
    if not directory.resolve().is_relative_to((ROOT/'assets/local').resolve()):raise ValueError('DOS evidence must stay ignored')
    before=verified_assets(ROOT/'assets/local/dos-original');manifest=json.loads((ROOT/'assets/manifests/dos_original.json').read_text());directory.mkdir(exist_ok=False)
    args=SimpleNamespace(dosbox=dosbox,xdotool=xdotool)
    generated=record(args,directory/'bootstrap',manifest,direct_window_keys=True)
    expected=json.loads((ROOT/'assets/manifests/dos_observation.json').read_text())['runtime_seed']['generated_sha256']
    # The original writes a 256-byte DS window, including mutable memory
    # beyond the filenames. Its full hash varies with this DOS runtime.
    # Validate the generated counter/immutable filename prefix against EXE,
    # preserve the whole measured seed, and change only the disk counter.
    exe=(ROOT/'assets/local/dos-original/VGALEMMI.EXE').read_bytes()
    prefix_end=generated.index(b'russell.dat\0')+len(b'russell.dat\0')
    static=exe[0x120+0x9a90+0x252a:0x120+0x9a90+0x252a+256]
    if len(generated)!=256 or generated[0]!=9 or generated[1:prefix_end]!=static[1:prefix_end]:raise ValueError('generated runtime counter/filename prefix differs from original EXE')
    runtime=b'\0'+generated[1:]
    settings=SETTINGS.replace('VGALEMMI /v','VGALEMMI /v -o').replace('cycles=fixed 10000',f'cycles=fixed {cycles}')
    record(args,directory/'run',manifest,runtime,settings=settings,procedure=PROCEDURE,direct_window_keys=True,observer=lambda process:Observer(process,exe,directory/'memory.json',decoder=decode_audio_sample))
    observation=json.loads((directory/'memory.json').read_text())
    if observation['error'] or not observation['location']:raise ValueError('failed RAM observation')
    result=cadence(observation['samples']);result.update(status='PASS',command='VGALEMMI /v -o',cycles=cycles,dosbox=identity(dosbox),xdotool=identity(xdotool),original_files=len(before),runtime_seed_sha256=sha(runtime),changed_runtime_offsets=[0],exe_or_original_dat_modified=False,generated_runtime_sha256=sha(generated),historical_generated_sha256=expected,runtime_seed_limitation='Newly generated original DS window; mutable trailing bytes differ from historical machine. Only byte0 disk-check counter is seeded; original key disk remains unverified.')
    music_ids=sorted({s['audio_music_driver_id_u8'] for s in observation['samples'] if 1<=s['released_u8']<=10})
    if music_ids != [6]:raise ValueError('actual Fun01 music selection differs from original normal table')
    result['selected_music_driver_ids']=music_ids
    import av
    rates=[]
    for path in sorted((directory/'run/capture').glob('*.avi')):
        with av.open(str(path)) as container:
            stream=container.streams.video[0]
            if (stream.width,stream.height)==(640,400):rates.append(str(stream.average_rate))
    if not rates or set(rates)!={'2190197/31250'}:raise ValueError('recorded game VGA rate differs')
    result['video_rates']=rates
    if verified_assets(ROOT/'assets/local/dos-original')!=before:raise ValueError('originals changed')
    save_json(directory/'summary.json',result);return result

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--evidence',type=Path,required=True);parser.add_argument('--cycles',type=int,default=3000);args=parser.parse_args()
    if not 1000<=args.cycles<=100000:parser.error('cycles must be 1000..100000')
    if not args.evidence.resolve().is_relative_to((ROOT/'assets/local').resolve()):parser.error('DOS evidence must stay ignored')
    if args.evidence.exists():parser.error('evidence directory must be new')
    dosbox,xdotool=shutil.which('dosbox'),shutil.which('xdotool')
    if not dosbox or not xdotool or not shutil.which('Xvfb'):parser.error('DOSBox, xdotool and Xvfb required')
    read_fd,write_fd=os.pipe()
    with tempfile.TemporaryDirectory(prefix='lemmings-dos-xvfb-') as runtime, (Path(runtime)/'xvfb.log').open('w') as log:
        xvfb=subprocess.Popen(['Xvfb','-displayfd',str(write_fd),'-screen','0','1280x1024x24','-nolisten','tcp'],pass_fds=[write_fd],stdout=log,stderr=log);os.close(write_fd)
        try:
            display=os.read(read_fd,32).decode().strip();os.close(read_fd)
            if not display.isdigit():raise ValueError('private X display startup failed')
            os.environ['DISPLAY']=':'+display
            result=observe(args.evidence.resolve(),Path(dosbox),Path(xdotool),args.cycles);print(json.dumps(result))
        finally:
            xvfb.terminate();xvfb.wait(timeout=10);log.flush()
            if args.evidence.is_dir():shutil.copyfile(Path(runtime)/'xvfb.log',args.evidence/'xvfb.log')

if __name__=='__main__':main()
