#!/usr/bin/env python3
"""Compare seven-job Climber permanent ability/animation states to executed Pascal."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile

from check_bomber import collect as collect_bomber
from check_desktop import verify_pixels
import os
from check_builder import replay_text
from check_basher import synthetic_masks as bashing_masks
from check_miner import synthetic_masks as mining_masks
from state_replay import ROOT, digest

GOLDEN=ROOT/'tools/fixtures/climber_golden.json'
ADAPTERS=['build_climber_reference.py','build_bomber_reference.py','build_blocker_reference.py','build_builder_reference.py','build_miner_reference.py',
          'build_basher_reference.py','build_digger_reference.py','build_reference.py','harness.pas']
CASES={
    'walking_right':[(80,0,'AssignClimber',0)],
    'walking_left':[(200,0,'AssignClimber',0)],
    'falling_duplicate_absent':[(54,0,'AssignClimber',0),(80,0,'AssignClimber',0),(80,1,'AssignClimber',999)],
    'pause_assign_resume':[(80,0,'Pause',0),(80,1,'AssignClimber',0),(90,0,'Resume',0)],
    'builder_then_climber':[(80,0,'AssignBuilder',0),(100,0,'AssignClimber',0)],
    'shrugger_bug':[(80,0,'AssignBuilder',0),(275,0,'AssignClimber',0)],
    'climber_then_bomber':[(80,0,'AssignClimber',0),(100,0,'AssignBomber',0)],
    'ability_changes_jobs':[(80,0,'AssignClimber',0),(81,0,'AssignDigger',0),(90,0,'AssignMiner',0),(95,0,'AssignBuilder',0),(100,0,'AssignBasher',0)],
    'two_climbers':[(80,0,'AssignClimber',0),(105,0,'AssignClimber',1)],
    'blocked_assignment':[(80,0,'AssignBlocker',0),(100,0,'AssignClimber',0),(110,0,'AssignBomber',0)]
}


def validate_climber_history(states,details):
    if len(states)!=len(details):raise ValueError('missing Climber boundary')
    for state,row in zip(states,details):
        if type(row['remaining']) is not int or not 0<=row['remaining']<=65535:raise ValueError('invalid Climber count')
        if len(row['actors'])!=len(state['actors']):raise ValueError('missing Climber actor')
        for actor,entry in zip(state['actors'],row['actors']):
            if (type(entry) is not list or len(entry)!=3 or type(entry[0]) is not int or entry[0]!=actor['id'] or any(type(v) is not bool for v in entry[1:])):raise ValueError('invalid Climber actor state')
            if actor['action'] in ['Climbing','Hoisting'] and not entry[1]:raise ValueError('climbing without ability')
            if entry[2] and (not entry[1] or actor['action']!='Walking'):raise ValueError('invalid stale Walking animation')
        state['climber']=row


def collect(directory):
    states=collect_bomber(directory)
    details=[json.loads(s) for s in (directory/'climber.jsonl').read_text().splitlines()]
    validate_climber_history(states,details)
    return states

def reference_run(executable,config,mask,objects,bash_masks,mine_masks,raw_map,explosion_mask,inputs,inventory=(10,10,10,10,10,10,10)):
    header=[config[k] for k in ['width','height','minutes','total','rate']]
    header += [len(config['entrances']),int(config['pause_glitch']),int(config['faller_three']),config['steps']]
    script=' '.join(map(str,header))+'\n'+' '.join(map(str,inventory))+' 1\n'
    script+=''.join(f'{x} {y}\n' for x,y in config['entrances'])+str(len(inputs))+'\n'
    codes=dict(Pause=0,Resume=1,SetReleaseRate=2,AssignDigger=3,AssignBasher=4,AssignMiner=5,AssignBuilder=6,AssignBlocker=7,AssignBomber=8,AssignClimber=9)
    script+=''.join(f'{s} {q} {codes[c]} {v}\n' for s,q,c,v in inputs)
    result=subprocess.run([str(executable.resolve()),str(mask),str(objects),str(bash_masks),str(mine_masks),str(raw_map),str(explosion_mask)],
                          input=script,capture_output=True,text=True,check=True)
    return [json.loads(line) for line in result.stdout.splitlines()]


def run_native(binary,directory,replay,render,*,steps=420,assets=None,level=4,fast=True,pacing=None):
    command=[str(binary.resolve()),'--steps',str(steps),'--render-every',str(render),'--capture',str(directory),'--replay',str(replay)]
    command+=['--assets',str(assets),'--level',str(level)] if assets else ['--synthetic-climber']
    if fast:command+=['--fast']
    if pacing:command+=['--pacing',pacing]
    subprocess.run(command,env=dict(os.environ,SDL_VIDEODRIVER=os.environ.get('SDL_VIDEODRIVER','dummy')),check=True,capture_output=True,text=True)
    return collect(directory),verify_pixels(directory)

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
    terrain=bytes(int((y>=100 and x<500) or (60<=y<100 and (160<=x<170 or 20<=x<30))) for y in range(160) for x in range(640))
    rows={};snapshots=captures=0
    with tempfile.TemporaryDirectory(prefix='lemmings-climber-') as temporary:
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
                if name in ['walking_right','walking_left'] and not any(a['action']=='Climbing' for state in states for a in state['actors']):raise AssertionError('case never climbed')
                if name=='walking_right' and not any(a['action']=='Hoisting' for state in states for a in state['actors']):raise AssertionError('case never hoisted')
                if name=='shrugger_bug' and not any(a[2] for state in states for a in state['climber']['actors']):raise AssertionError('shrugger bug not exercised')
                rows[name]=dict(snapshots_sha256=signature,steps=len(states),inputs=inputs)
                snapshots+=len(states);captures+=count
    if args.record:
        provenance=json.loads(args.reference.with_name('reference.provenance.json').read_text())
        GOLDEN.write_text(json.dumps(dict(format='g07-climber-v1',reference=provenance,adapter_sha256=hashes,cases=rows),indent=2)+'\n')
    print(json.dumps(dict(status='PASS',cases=len(rows),replays=len(rows)*2,snapshots=snapshots,pixel_exact_captures=captures,
                         reference_executed=args.reference is not None,scope='seven jobs, permanent ability, climbing/hoisting and original shrugger bug; not whole DOS')))


if __name__=='__main__':main()
