#!/usr/bin/env python3
"""Observe DOS win, Nuke loss and one-minute timeout with normal keys/read-only RAM."""
import argparse
import json
from pathlib import Path
import threading
import time
import capture_dos
from capture_dos import MAPPER,ROOT,identity,record,save_json,sha
from observe_digger_dos import Controller
from observe_exit_dos import decode_exit
from observe_dos import Observer
from reference_assets import verify_assets


def decode_completion(data):
    result=decode_exit(data)
    digits=bytes(data[i] for i in [0x20a7,0x20a9,0x20aa])
    result.update(clock_digits=digits.decode('ascii',errors='replace'),
        seconds_remaining=(digits[0]-48)*60+(digits[1]-48)*10+digits[2]-48 if all(48<=d<=57 for d in digits) else None,
        playing_u8=data[0x2a],finish_requested_u8=data[0x4f],out_with_particles_u8=data[0x45],
        rescued_percent_u8=data[0x40],target_percent_u8=data[0x41],result_success_u8=data[0x1f6d],level_index_u8=data[0x1f6b])
    return result


class CompletionObserver(Observer):
    def __init__(self,process,exe,output):
        self.previous=None
        def decode(data):
            if len(data)==0x2200:self.previous=decode_completion(data);return self.previous
            end=time.monotonic()+.5
            while process.poll() is None and time.monotonic()<end:time.sleep(.005)
            if data or process.poll() is None or self.previous is None:raise ValueError('incomplete DOS result sample')
            return self.previous
        super().__init__(process,exe,output,decoder=decode)


class CompletionController(Controller):
    def __init__(self,process,exe,output,xdotool,profile):
        self.observer=CompletionObserver(process,exe,output/'memory-observation.json')
        self.process,self.output,self.xdotool,self.profile=process,output,xdotool,profile
        self.events,self.error=[],None;self.start=time.monotonic_ns();self.done=threading.Event()
        self.thread=threading.Thread(target=self.control,daemon=True);self.thread.start()
    def control(self):
        try:
            if self.profile=='win':
                Controller.control(self)
                if self.error:raise ValueError(self.error)
            else:
                self.wait(lambda s:5<=s['startup_counter_u8']<20 and s['playing_u8']==255 and
                    (s['level_index_u8']==105 and s['seconds_remaining']==60 if self.profile=='timeout' else s['scroll_x']==640 and s['max_lemmings_u8']==10),timeout=65)
                if self.profile=='loss':
                    self.wait(lambda s:s['startup_counter_u8']==118 and s['released_u8']==3,timeout=20)
                    self.command('keydown','F12');time.sleep(.025);self.command('keyup','F12')
                    self.wait(lambda s:s['startup_counter_u8']==121 and s['released_u8']==3,timeout=2)
                    self.command('keydown','F12');time.sleep(.025);self.command('keyup','F12')
                    self.wait(lambda s:s['max_lemmings_u8']==3)
            sample=self.wait(lambda s:s['finish_requested_u8']==255 and s['playing_u8']==0,timeout=100)
            self.events.append(dict(event='result_boundary',sample=sample))
            time.sleep(2)
            self.key('F11');time.sleep(1);self.key('s');time.sleep(1);self.key('F11')
        except Exception as error:self.error=str(error)
        finally:self.done.set()


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--dosbox',type=Path,required=True);p.add_argument('--xdotool',type=Path,required=True)
    p.add_argument('--profile',choices=['win','loss','timeout'],required=True);p.add_argument('--output',type=Path,required=True)
    args=p.parse_args();args.output=args.output.resolve()
    if not args.output.is_relative_to(ROOT/'assets/local'):raise ValueError('local DOS evidence required')
    manifest=json.loads((ROOT/'assets/manifests/dos_original.json').read_text());verify_assets(manifest,ROOT/manifest['local_directory'])
    seed=(ROOT/'assets/local/issue-28-dos-proof-2/bootstrap/drive/RUSSELL.DAT').read_bytes()
    if sha(seed)!='82abd5f27dfa78c0329f7f0ff67755725f28a92e5d8c96f642a3e7df77e333bf':raise ValueError('runtime seed differs')
    runtime=b'\0'+seed[1:]
    # Mayhem16 candidate from pinned legacy generator; acceptance requires DOS
    # level index105 and original one-minute clock, not the generator alone.
    code='ijjmdmgkip'
    capture_dos.MAPPER=MAPPER.replace('hand_video "key 118"','hand_video "key 292"')+'\nkey_f10 "key 291"\nkey_f12 "key 293"\nkey_o "key 111"\nkey_q "key 113"\n'
    menu=[('2',500),('Return',7000),('F4',5000),('3',3000)]
    if args.profile=='timeout':
        capture_dos.MAPPER+='\nkey_f2 "key 283"\n'+''.join(f'key_{c} "key {ord(c)}"\n' for c in sorted(set(code)-{'a','p'}))
        menu += [('F2',1500)]+[(c,100) for c in code]+[('Return',3000)]
    menu += [('F1',5000),('space',2000)]
    box=[]
    def observer(process):
        controller=CompletionController(process,(ROOT/manifest['local_directory']/'VGALEMMI.EXE').read_bytes(),args.output,args.xdotool,args.profile)
        box.append(controller);return controller
    def procedure():
        yield from menu
        end=time.monotonic()+140
        while not box[0].done.is_set():
            if time.monotonic()>end:raise ValueError('DOS result protocol timed out')
            yield ('wait',100)
    record(args,args.output,manifest,runtime,procedure=procedure(),direct_window_keys=True,terminate_after_procedure=True,observer=observer)
    verify_assets(manifest,ROOT/manifest['local_directory'])
    samples=json.loads((args.output/'memory-observation.json').read_text())['samples']
    results=[s for s in samples if s['finish_requested_u8']==255 and s['playing_u8']==0]
    if not results:raise ValueError('no original result observed')
    final=results[-1];expected_success=args.profile=='win'
    if final['result_success_u8']!=255*expected_success:raise ValueError('original result flag differs')
    if final['rescued_percent_u8']<final['target_percent_u8'] if expected_success else final['rescued_percent_u8']>=final['target_percent_u8']:
        raise ValueError('original integer percentage comparison differs')
    if args.profile=='win' and (final['saved_u8'],final['rescued_percent_u8'])!=(10,100):raise ValueError('original full rescue absent')
    if args.profile=='loss' and (final['max_lemmings_u8'],final['rescued_percent_u8'])!=(3,0):raise ValueError('original Nuke result absent')
    if args.profile=='timeout' and final['seconds_remaining']!=0:raise ValueError('original timeout absent')
    summary=dict(status='OBSERVED_RESULT',profile=args.profile,samples=len(samples),final=final,
        screenshots=[identity(p) for p in sorted((args.output/'capture').glob('*.png'))],
        limitation='normal keyboard and asynchronous read-only RAM/result screenshots; normalized final values, not atomic DOS trace or result renderer equivalence')
    save_json(args.output/'summary.json',summary)
    save_json(args.output/'provenance.json',dict(controller=identity(Path(__file__)),dosbox=identity(args.dosbox),original_files=manifest['files'],runtime_seed_sha256=sha(runtime),mapper=identity(args.output/'mapper.map'),config=identity(args.output/'dosbox.conf'),menu=menu))
    print(json.dumps({k:v for k,v in summary.items() if k!='final'}))


if __name__=='__main__':main()
