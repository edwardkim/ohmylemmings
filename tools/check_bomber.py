#!/usr/bin/env python3
"""Compare six-job Bomber countdown/mask/Blocker states to executed Pascal."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile

from check_blocker import collect as collect_blocker, run_native as blocker_native
from check_builder import replay_text
from check_basher import synthetic_masks as bashing_masks
from check_miner import synthetic_masks as mining_masks
from state_replay import ROOT, digest

GOLDEN=ROOT/'tools/fixtures/bomber_golden.json'
ADAPTERS=['build_bomber_reference.py','build_blocker_reference.py','build_builder_reference.py','build_miner_reference.py',
          'build_basher_reference.py','build_digger_reference.py','build_reference.py','harness.pas']
CASES={
    'walking_right':[(80,0,'AssignBomber',0)],
    'walking_left':[(200,0,'AssignBomber',0)],
    'duplicate_absent':[(80,0,'AssignBomber',0),(80,1,'AssignBomber',0),(80,2,'AssignBomber',999)],
    'pause_assign_resume':[(80,0,'Pause',0),(80,1,'AssignBomber',0),(90,0,'Resume',0)],
    'blocker_then_bomber':[(80,0,'AssignBlocker',0),(100,0,'AssignBomber',0)],
    'builder_then_bomber':[(80,0,'AssignBuilder',0),(100,0,'AssignBomber',0)],
    'countdown_changes_jobs':[(80,0,'AssignDigger',0),(81,0,'AssignBomber',0),(90,0,'AssignMiner',0),(95,0,'AssignBuilder',0),(100,0,'AssignBasher',0)],
    'two_bombers':[(80,0,'AssignBomber',0),(105,0,'AssignBomber',1)],
}


def validate_bomber_history(states,details):
    if len(states)!=len(details):raise ValueError('missing Bomber boundary')
    for state,row in zip(states,details):
        if type(row['remaining']) is not int or not 0<=row['remaining']<=65535:raise ValueError('invalid Bomber count')
        if len(row['actors'])!=len(state['actors']):raise ValueError('missing Bomber actor')
        for actor,entry in zip(state['actors'],row['actors']):
            if (type(entry) is not list or len(entry)!=4 or any(type(v) is not int for v in entry)
                or entry[0]!=actor['id'] or not 0<=entry[1]<=79 or not 0<=entry[2]<=52 or not 0<=entry[3]<=50):raise ValueError('invalid Bomber actor state')
            if actor['action'] in ['OhNo','Exploding','Splatting'] and entry[1]!=0:raise ValueError('timer survived terminal action')
            if (entry[2] or entry[3]) and actor['removed']!='Exploded':raise ValueError('particles without explosion')
        state['bomber']=row


def collect(directory):
    states=collect_blocker(directory)
    details=[json.loads(s) for s in (directory/'bomber.jsonl').read_text().splitlines()]
    validate_bomber_history(states,details)
    return states


def reference_run(executable,config,mask,objects,bash_masks,mine_masks,raw_map,explosion_mask,inputs,inventory=(10,10,10,10,10,10)):
    header=[config[k] for k in ['width','height','minutes','total','rate']]
    header += [len(config['entrances']),int(config['pause_glitch']),int(config['faller_three']),config['steps']]
    script=' '.join(map(str,header))+'\n'+' '.join(map(str,inventory))+' 1\n'
    script+=''.join(f'{x} {y}\n' for x,y in config['entrances'])+str(len(inputs))+'\n'
    codes=dict(Pause=0,Resume=1,SetReleaseRate=2,AssignDigger=3,AssignBasher=4,AssignMiner=5,AssignBuilder=6,AssignBlocker=7,AssignBomber=8)
    script+=''.join(f'{s} {q} {codes[c]} {v}\n' for s,q,c,v in inputs)
    result=subprocess.run([str(executable.resolve()),str(mask),str(objects),str(bash_masks),str(mine_masks),str(raw_map),str(explosion_mask)],
                          input=script,capture_output=True,text=True,check=True)
    return [json.loads(line) for line in result.stdout.splitlines()]


def run_native(binary,directory,replay,render,**kwargs):
    _,captures=blocker_native(binary,directory,replay,render,**kwargs)
    return collect(directory),captures


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary',type=Path,default=ROOT/'target/debug/lemmings-desktop')
    parser.add_argument('--reference',type=Path)
    parser.add_argument('--record',action='store_true')
    args=parser.parse_args()
    if args.record and not args.reference:raise ValueError('record requires actual reference execution')
    frozen=json.loads(GOLDEN.read_text()) if not args.record else None
    hashes={name:hashlib.sha256((ROOT/'tools/lemmix'/name).read_bytes()).hexdigest() for name in ADAPTERS}
    if frozen and hashes!=frozen['adapter_sha256']:raise ValueError('reference adapter changed; execute reference again')
    config=dict(width=640,height=160,minutes=5,total=10,rate=50,entrances=[[40,40]],pause_glitch=True,faller_three=True,steps=420)
    terrain=bytes(int((y>=100 and x<500) or (60<=y<100 and 160<=x<170)) for y in range(160) for x in range(640))
    rows={};snapshots=captures=0
    with tempfile.TemporaryDirectory(prefix='lemmings-bomber-') as temporary:
        root=Path(temporary)
        for name,data in [('mask',terrain),('objects',bytes(len(terrain))),('bash-masks',bashing_masks()),('mine-masks',mining_masks()),('raw-map',bytes([128])*(415*47)),('explosion-mask',bytes([255])*44)]:
            (root/name).write_bytes(data)
        for name,inputs in CASES.items():
            replay=root/(name+'.txt');replay.write_text(replay_text(inputs))
            expected=reference_run(args.reference,config,root/'mask',root/'objects',root/'bash-masks',root/'mine-masks',root/'raw-map',root/'explosion-mask',inputs) if args.reference else None
            for render in [1,7]:
                with tempfile.TemporaryDirectory(prefix=name+'-',dir=root) as case:
                    states,count=run_native(args.binary,Path(case)/'capture',replay,render)
                if expected is not None and states!=expected:
                    for step,(actual,wanted) in enumerate(zip(states,expected)):
                        if actual!=wanted:raise AssertionError(f'{name} first reference difference {step}: {actual} != {wanted}')
                    raise AssertionError('reference length differs')
                signature=digest(states)
                if frozen and signature!=frozen['cases'][name]['snapshots_sha256']:raise AssertionError('recorded reference differs: '+name)
                if name in rows and signature!=rows[name]['snapshots_sha256']:raise AssertionError('render-dependent state')
                if not any(a['removed']=='Exploded' for state in states for a in state['actors']):raise AssertionError('case never exploded')
                if name=='blocker_then_bomber' and states[-1]['blocker']['fields']:raise AssertionError('Blocker field not released')
                rows[name]=dict(snapshots_sha256=signature,steps=len(states),inputs=inputs)
                snapshots+=len(states);captures+=count
    if args.record:
        provenance=json.loads(args.reference.with_name('reference.provenance.json').read_text())
        GOLDEN.write_text(json.dumps(dict(format='g07-bomber-v1',reference=provenance,adapter_sha256=hashes,cases=rows),indent=2)+'\n')
    print(json.dumps(dict(status='PASS',cases=len(rows),replays=len(rows)*2,snapshots=snapshots,pixel_exact_captures=captures,
                         reference_executed=args.reference is not None,scope='six jobs, countdown/particles counters, explosion mask, Blocker restore and terrain; not whole DOS')))


if __name__=='__main__':main()
