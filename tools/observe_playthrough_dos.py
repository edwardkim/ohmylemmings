#!/usr/bin/env python3
"""Fresh DOS Fun01 complete win/loss, normal own-window keys and read-only RAM."""
import argparse
import json
from pathlib import Path
import time
import capture_dos
from capture_dos import ROOT,MAPPER,record,save_json,identity,sha
from observe_completion_dos import CompletionController
from observe_digger_dos import Controller
from observe_nuke_dos import NukeController
from observe_blocker_dos import make_runtime_seed
from reference_assets import verify_assets

SEED_SHA='04462851c5edc273303b6fd19b8ee3c47ef722b9c1803dfd5adbd0229041e313'


def validate_result(samples,profile):
    initial=[s for s in samples if s['playing_u8']==255 and 5<=s['startup_counter_u8']<35 and s['max_lemmings_u8']==10 and s['level_index_u8']==0]
    results=[s for s in samples if s['finish_requested_u8']==255 and s['playing_u8']==0]
    if not initial or not results:raise ValueError('Fun01 start/result absent')
    final=results[-1];win=profile=='win'
    expected=(255,10,100,10,10,1) if win else (0,0,0,10,3,0)
    values=tuple(final[k] for k in ['result_success_u8','saved_u8','rescued_percent_u8','target_percent_u8','max_lemmings_u8','level_index_u8'])
    if values!=expected:raise ValueError('DOS complete result differs')
    if win and not any(s.get('remaining_diggers')==9 and s.get('actor0_action_bits',0)&16 for s in samples):
        raise ValueError('DOS Digger input absent')
    return final


class JourneyController(CompletionController):
    # Direct each key only to this recording's DOSBox child window.
    command=NukeController.command
    def control(self):
        try:
            if self.profile=='win':
                Controller.control(self)
                if self.error:raise ValueError(self.error)
            else:
                self.wait(lambda s:5<=s['startup_counter_u8']<20 and s['playing_u8']==255 and s['level_index_u8']==0 and s['max_lemmings_u8']==10,timeout=65)
                self.wait(lambda s:118<=s['startup_counter_u8']<=120 and s['released_u8']==3,timeout=20)
                self.command('keydown','F12');time.sleep(.025);self.command('keyup','F12')
                self.wait(lambda s:121<=s['startup_counter_u8']<=124 and s['released_u8']==3,timeout=2)
                self.command('keydown','F12');time.sleep(.025);self.command('keyup','F12')
                self.wait(lambda s:s['max_lemmings_u8']==3)
            sample=self.wait(lambda s:s['finish_requested_u8']==255 and s['playing_u8']==0,timeout=100)
            self.events.append(dict(event='result_boundary',sample=sample))
            time.sleep(1);self.key('s');time.sleep(.5);self.key('F11');time.sleep(1)
        except Exception as error:self.error=str(error)
        finally:self.done.set()


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--dosbox',type=Path,required=True);p.add_argument('--xdotool',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True);p.add_argument('--profile',choices=['win','loss'],required=True)
    p.add_argument('--seed',type=Path,required=True);p.add_argument('--seed-sha256',default=SEED_SHA)
    a=p.parse_args();a.output=a.output.resolve()
    if not a.output.is_relative_to(ROOT/'assets/local'):raise ValueError('local DOS evidence required')
    manifest=json.loads((ROOT/'assets/manifests/dos_original.json').read_text());verify_assets(manifest,ROOT/manifest['local_directory'])
    original=a.seed.read_bytes();runtime=make_runtime_seed(original,a.seed_sha256)
    capture_dos.MAPPER=MAPPER.replace('hand_video "key 118"','hand_video "key 292"')+'\nkey_f10 "key 291"\nkey_f12 "key 293"\nkey_o "key 111"\nkey_q "key 113"\n'
    menu=[('2',500),('Return',7000),('F4',5000),('3',3000),('F1',5000),('s',200),('F11',300),('space',2000)]
    box=[]
    def observer(process):
        c=JourneyController(process,(ROOT/manifest['local_directory']/'VGALEMMI.EXE').read_bytes(),a.output,a.xdotool,a.profile)
        box.append(c);return c
    def procedure():
        yield from menu
        end=time.monotonic()+145
        while not box[0].done.is_set():
            if time.monotonic()>end:raise ValueError('DOS complete journey timed out')
            yield ('wait',100)
    record(a,a.output,manifest,runtime,procedure=procedure(),direct_window_keys=True,terminate_after_procedure=True,observer=observer)
    samples=json.loads((a.output/'memory-observation.json').read_text())['samples']
    final=validate_result(samples,a.profile)
    summary=dict(status='OBSERVED_RESULT',profile=a.profile,samples=len(samples),final=final,
        videos=[identity(p) for p in sorted((a.output/'capture').glob('*.avi'))],screenshots=[identity(p) for p in sorted((a.output/'capture').glob('*.png'))],
        limitation='normal keys and asynchronous read-only samples; no atomic DOS trace or native result-artwork equivalence')
    save_json(a.output/'summary.json',summary)
    save_json(a.output/'provenance.json',dict(controller=identity(Path(__file__)),base_controller=identity(Path('tools/observe_completion_dos.py')),
        own_window_controller=identity(Path('tools/observe_nuke_dos.py')),capture_driver=identity(Path(capture_dos.__file__)),
        dosbox=identity(a.dosbox),original_files=manifest['files'],generated_seed_sha256=sha(original),runtime_seed_sha256=sha(runtime),
        runtime_changed_offsets=[0],runtime_before=[9],runtime_after=[0],exe_or_original_dat_modified=False,
        mapper=identity(a.output/'mapper.map'),config=identity(a.output/'dosbox.conf'),menu=menu))
    verify_assets(manifest,ROOT/manifest['local_directory'])
    print(json.dumps(dict(status='OBSERVED_RESULT',profile=a.profile,samples=len(samples),final_percent=final['rescued_percent_u8'])))


if __name__=='__main__':main()
