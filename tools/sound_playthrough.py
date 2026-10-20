#!/usr/bin/env python3
"""Integrated game/audio replay; original bytes and listening files stay local."""
import argparse
import array
import functools
import hashlib
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
import math
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import threading
import wave
try:
    from .web_build import ROOT, build
    from .state_replay import verified_assets
except ImportError:
    from web_build import ROOT, build
    from state_replay import verified_assets


def guard_original(directory):
    if not directory.resolve().is_relative_to((ROOT/'assets/local').resolve()):
        raise ValueError('original sound evidence must remain ignored')


def metrics(raw, rate):
    if type(rate) is not int or not 8000<=rate<=192000 or len(raw)%2:
        raise ValueError('invalid mono i16 PCM/rate')
    values=array.array('h');values.frombytes(raw)
    if sys.byteorder!='little':values.byteswap()
    return dict(frames=len(values),rate=rate,seconds=len(values)/rate,
        peak=max(map(abs,values),default=0),nonzero=sum(x!=0 for x in values),
        rms=math.sqrt(sum(x*x for x in values)/len(values)) if values else 0,
        sha256=hashlib.sha256(raw).hexdigest())


def write_wave(path, raw, rate=44100):
    info=metrics(raw,rate)
    with wave.open(str(path),'wb') as output:
        output.setnchannels(1);output.setsampwidth(2);output.setframerate(rate);output.writeframes(raw)
    return info


def left_channel(raw, channels):
    if type(channels) is not int or not 1<=channels<=8 or len(raw)%(2*channels):
        raise ValueError('invalid packed i16 channel data')
    return b''.join(raw[i:i+2] for i in range(0,len(raw),channels*2))


def extract_dos_audio(capture, directory):
    """Optional PyAV decoder; retain exact first-channel i16 mixer samples."""
    guard_original(directory)
    import av
    directory.mkdir(parents=True,exist_ok=False);rows=[]
    for source in sorted(capture.glob('*.avi')):
        with av.open(str(source)) as container:
            if not container.streams.audio:continue
            stream=container.streams.audio[0];rate=stream.codec_context.sample_rate;raw=bytearray()
            if stream.codec_context.name!='pcm_s16le':raise ValueError('unexpected DOS mixer codec')
            for frame in container.decode(audio=0):
                channels=len(frame.layout.channels)
                if frame.format.name!='s16':raise ValueError('unexpected packed DOS mixer samples')
                raw+=left_channel(bytes(frame.planes[0])[:frame.samples*channels*2],channels)
            info=write_wave(directory/(source.stem+'.wav'),raw,rate)
            video=container.streams.video[0]
            rows.append(dict(source=source.name,source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),video=[video.width,video.height],**info))
    if not any(r['video']==[640,400] and r['seconds']>10 and r['nonzero']>1000 for r in rows):
        raise ValueError('missing audible DOS game recording')
    summary=dict(status='PASS',decoder=av.__version__,channel='exact first i16 channel; no gain, normalization, resampling or downmix',clips=rows,physical_listening_verified=False)
    (directory/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');return summary


def validate_trace(rows, original=False):
    if len(rows)!=580:raise ValueError('incomplete integrated trace')
    commands=set();previous={};phases=set();base=int(original)
    for wall,row in enumerate(rows,1):
        batch=row['sounds'];generation=base+int(wall>=360)
        if row['wall']!=wall or row['end']!=wall*44100*156250//2190197:
            raise ValueError('audio sample/control clock mismatch')
        if batch['generation']!=generation or batch['dropped']!=0 or batch['sequence_exhausted']:
            raise ValueError('lost sound commands or generation')
        phases.add(row['state']['completion']['phase'])
        for event in batch['events']:
            sequence=event['sequence']
            if type(sequence) is not int or sequence<=previous.get(generation,-1) or event.get('mapping_error'):
                raise ValueError('unordered/unmapped sound event')
            previous[generation]=sequence
            if event['command']:commands.add(tuple(event['command']))
    if not {('music',5),('effect',0),('effect',1),('effect',2),('effect',3),('stop',)}<=commands:
        raise ValueError('missing startup/selection/assignment/nuke/result sounds')
    if original and not {('effect',4),('effect',11)}<=commands:
        raise ValueError('missing original Nuke sounds')
    if rows[358]['state']['completion']['phase'] not in (['Lost'] if original else ['Won','Lost']):
        raise ValueError('Nuke did not reach a frozen loss before restart')
    return dict(steps=len(rows),frames=rows[-1]['end'],commands=sorted(commands),phases=sorted(phases),generations=[base,base+1])


def browser(directory, expected):
    ready=threading.Event();result={}
    class Handler(SimpleHTTPRequestHandler):
        def log_message(self,*args):pass
        def do_POST(self):
            size=int(self.headers.get('Content-Length','0'))
            if not 0<size<8_000_000:self.send_error(413);return
            raw=self.rfile.read(size)
            if self.path=='/__pcm':
                (directory/'browser.pcm').write_bytes(raw)
                value=dict(equal=raw==(directory/'native/native.pcm').read_bytes(),**metrics(raw,44100))
            elif self.path=='/__sha':value=dict(sha256=hashlib.sha256(raw).hexdigest())
            elif self.path=='/__result':
                if size>100000:self.send_error(413);return
                result.update(json.loads(raw));value={};ready.set()
            else:self.send_error(404);return
            self.send_response(200);self.end_headers();self.wfile.write(json.dumps(value).encode())
    executable=next((shutil.which(n) for n in ['google-chrome','chromium','chromium-browser'] if shutil.which(n)),None)
    if not executable:raise ValueError('Chromium required')
    with ThreadingHTTPServer(('127.0.0.1',0),functools.partial(Handler,directory=directory)) as server:
        thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
        with tempfile.TemporaryDirectory(prefix='lemmings-sound-journey-') as profile,(directory/'browser.log').open('w') as log:
            process=subprocess.Popen([executable,'--headless','--disable-gpu','--no-first-run','--no-default-browser-check',f'--user-data-dir={profile}',f'http://127.0.0.1:{server.server_port}/playthrough-check.html'],stdout=log,stderr=log)
            try:
                for _ in range(180):
                    if ready.wait(1):break
                    if process.poll() is not None:raise ValueError('browser exited')
                if not ready.is_set():raise TimeoutError('sound playthrough timed out')
            finally:
                process.terminate()
                try:process.wait(timeout=10)
                except subprocess.TimeoutExpired:process.kill();process.wait()
                server.shutdown();thread.join(timeout=5)
    (directory/'browser-result.json').write_text(json.dumps(result,indent=2)+'\n')
    if result.get('status')!='PASS' or result.get('steps')!=580 or result.get('pcm')!=expected:
        raise ValueError(result)
    return result


def run(directory, original=None):
    if original:
        guard_original(directory);before=verified_assets(original)
    directory.mkdir(parents=True,exist_ok=False)
    subprocess.run(['cargo','build','-p','lemmings-web','--bin','sound_playthrough','--release','--locked','--offline'],cwd=ROOT,check=True)
    command=[str(ROOT/'target/release/sound_playthrough'),str(directory/'native')]
    if original:command.append(str(original))
    subprocess.run(command,cwd=ROOT,check=True)
    rows=json.loads((directory/'native/trace.json').read_text());summary=validate_trace(rows,bool(original))
    raw=(directory/'native/native.pcm').read_bytes();info=write_wave(directory/'native.wav',raw)
    if info['frames']!=summary['frames'] or not info['nonzero']:raise ValueError('missing integrated PCM')
    build(directory)
    for name in ['playthrough-check.html','playthrough-check.js']:
        shutil.copyfile(ROOT/'tools/audio'/name,directory/name)
    selected=[]
    if original:
        (directory/'original').mkdir()
        for p in sorted(original.iterdir()):
            if p.name in ['MAIN.DAT','ODDTABLE.DAT','VGALEMMI.EXE','ADLIB.DAT'] or re.fullmatch(r'LEVEL00[0-9]\.DAT|GROUND[0-4]O\.DAT|VGAGR[0-4]\.DAT',p.name):
                shutil.copyfile(p,directory/'original'/p.name);selected.append(p.name)
    images={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in (directory/'native').iterdir() if p.suffix in ['.rgb','.terrain','.objects']}
    (directory/'expected.json').write_text(json.dumps(dict(original_files=selected,pcm=info,images=images),separators=(',',':'))+'\n')
    wasm=browser(directory,info)
    write_wave(directory/'browser.wav',(directory/'browser.pcm').read_bytes())
    if original and verified_assets(original)!=before:raise ValueError('originals changed')
    summary.update(status='PASS',original_files=len(before) if original else 0,pcm=info,wasm=wasm,physical_listening_verified=False)
    (directory/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');return summary


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--evidence',type=Path);parser.add_argument('--original-assets',type=Path);parser.add_argument('--dos-capture',type=Path);args=parser.parse_args()
    if args.dos_capture:
        if not args.evidence or args.original_assets:parser.error('DOS extraction requires its own ignored --evidence')
        print(json.dumps(extract_dos_audio(args.dos_capture.resolve(),args.evidence.resolve())));return
    if args.original_assets and not args.evidence:parser.error('original sound requires ignored --evidence')
    if args.evidence:result=run(args.evidence.resolve(),args.original_assets.resolve() if args.original_assets else None)
    else:
        with tempfile.TemporaryDirectory(prefix='lemmings-sound-playthrough-') as path:result=run(Path(path)/'run')
    print(json.dumps(result))


if __name__=='__main__':main()
