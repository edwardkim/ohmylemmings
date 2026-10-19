#!/usr/bin/env python3
"""Observe original DOS hazards through normal passwords/keys and read-only RAM."""
import argparse
import json
from pathlib import Path
import threading
import time
import capture_dos
from capture_dos import MAPPER,ROOT,identity,record,save_json,sha
from observe_nuke_dos import NukeController,decode_nuke
from observe_dos import Observer
from reference_assets import verify_assets

# Candidates generated from the pinned legacy Lemmings.js algorithm, then
# accepted only when the original game exposes the matching level state.
PROFILES={'fire':(9,'cajjmdljcl',576,80,0x4000), 'trap':(17,'cajjlflbdv',608,50,0), 'water':(29,'mflcajnndw',864,80,0x1000)}


def decode_hazards(data):
    result=decode_nuke(data)
    result['saved_u8']=data[0x46]
    # Original trap dispatch uses original object slot ah and stride28. The
    # frame byte DS:17cb+slot*28+2 is also used by the existing object observer.
    result['trap_frames']=[result['object_frames_u8'][slot] for slot in [5,6,9,12]]
    return result


def trapped(a):
    return a['removed']==255 and a['action_bits'] in [0,4,8] and a['y']<=179


class HazardObserver(Observer):
    def __init__(self,process,exe,output):
        self.previous=None
        def decode(data):
            if len(data)==0x2200:
                self.previous=decode_hazards(data);return self.previous
            end=time.monotonic()+.5
            while process.poll() is None and time.monotonic()<end:time.sleep(.005)
            if data or process.poll() is None or self.previous is None:raise ValueError('incomplete live DOS hazard sample')
            return self.previous
        super().__init__(process,exe,output,decoder=decode)


class HazardController(NukeController):
    def __init__(self,process,exe,output,xdotool,profile):
        self.observer=HazardObserver(process,exe,output/'memory-observation.json')
        self.process,self.output,self.xdotool,self.profile=process,output,xdotool,profile
        self.events,self.error=[],None;self.start=time.monotonic_ns()
        self.thread=threading.Thread(target=self.control,daemon=True);self.thread.start()
    def control(self):
        try:
            level,code,scroll,total,bits=PROFILES[self.profile]
            self.wait(lambda s:5<=s['startup_counter_u8']<20 and s['scroll_x']==scroll and s['max_lemmings_u8']==total,timeout=65)
            self.command('keydown','a');time.sleep(1.2);self.command('keyup','a')
            self.wait(lambda s:s['cursor_y']>=170)
            self.wait(lambda s:sum(trapped(a) for a in s['actors'])>=2 if self.profile=='trap' else any(a['action_bits']==bits and a['removed']==255 for a in s['actors']),timeout=30)
        except Exception as error:self.error=str(error)


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--dosbox',type=Path,required=True);p.add_argument('--xdotool',type=Path,required=True)
    p.add_argument('--profile',choices=PROFILES,required=True);p.add_argument('--output',type=Path,required=True)
    args=p.parse_args();args.output=args.output.resolve()
    if not args.output.is_relative_to(ROOT/'assets/local'):raise ValueError('local DOS evidence required')
    manifest=json.loads((ROOT/'assets/manifests/dos_original.json').read_text());verify_assets(manifest,ROOT/manifest['local_directory'])
    seed=(ROOT/'assets/local/issue-28-dos-proof-2/bootstrap/drive/RUSSELL.DAT').read_bytes()
    if sha(seed)!='82abd5f27dfa78c0329f7f0ff67755725f28a92e5d8c96f642a3e7df77e333bf':raise ValueError('documented runtime seed differs')
    runtime=b'\0'+seed[1:];level,code,scroll,total,bits=PROFILES[args.profile]
    capture_dos.MAPPER=MAPPER.replace('hand_video "key 118"','hand_video "key 292"')+'\nkey_f2 "key 283"\n'+''.join(f'key_{c} "key {ord(c)}"\n' for c in sorted(set(code)-{'a'}))
    procedure=[('2',500),('Return',7000),('F4',5000),('3',3000),('F2',1500)]
    procedure += [(c,100) for c in code]+[('Return',3000),('s',200),('F1',5000),('s',200),('F11',300),('space',2000),('wait',20000),('s',200),('F11',200)]
    record(args,args.output,manifest,runtime,procedure=procedure,direct_window_keys=True,terminate_after_procedure=True,
        observer=lambda process:HazardController(process,(ROOT/manifest['local_directory']/'VGALEMMI.EXE').read_bytes(),args.output,args.xdotool,args.profile))
    verify_assets(manifest,ROOT/manifest['local_directory'])
    samples=json.loads((args.output/'memory-observation.json').read_text())['samples']
    playing=[s for s in samples if s['scroll_x']==scroll and s['max_lemmings_u8']==total]
    if not playing:raise ValueError('original level state not observed')
    if args.profile=='trap':
        deaths=max(sum(trapped(a) for a in s['actors']) for s in playing)
        frames=sorted({f for s in playing for f in s['trap_frames']})
        if deaths<2 or 14 not in frames:raise ValueError('trap death/reuse frames absent')
    else:
        actors=[a for s in playing for a in s['actors'] if a['action_bits']==bits]
        frames=sorted({a['frame'] for a in actors if not a['removed']})
        required=list(range(16 if args.profile=='water' else 14))
        if frames!=required:raise ValueError('incomplete hazard animation observation')
        deaths=max(sum(a['action_bits']==bits and a['removed']==255 for a in s['actors']) for s in playing)
        if not deaths:raise ValueError('hazard removal absent')
    summary=dict(status='OBSERVED_HAZARD',profile=args.profile,level=level,samples=len(samples),frames=frames,max_deaths=deaths,
        raw_removed_frame=0 if args.profile=='water' else 14 if args.profile=='fire' else 'action frame retained',
        limitation='asynchronous read-only observations and normal password/key input; no atomic/full DOS state or result assertion')
    save_json(args.output/'summary.json',summary)
    save_json(args.output/'provenance.json',dict(controller=identity(Path(__file__)),dosbox=identity(args.dosbox),original_files=manifest['files'],runtime_seed_sha256=sha(runtime),mapper=identity(args.output/'mapper.map'),config=identity(args.output/'dosbox.conf'),procedure=procedure,candidate_code=code,validated_level_state=dict(scroll=scroll,max_lemmings=total)))
    print(json.dumps(summary))


if __name__=='__main__':main()
