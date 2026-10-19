#!/usr/bin/env python3
"""Compare eight-job Floater ability/opening/landing states to executed Pascal."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile

from check_climber import collect as collect_climber
from check_desktop import verify_pixels
import os
from check_builder import replay_text
from check_basher import synthetic_masks as bashing_masks
from check_miner import synthetic_masks as mining_masks
from state_replay import ROOT, digest

GOLDEN=ROOT/'tools/fixtures/floater_golden.json'
ADAPTERS=['build_floater_reference.py','build_climber_reference.py','build_bomber_reference.py','build_blocker_reference.py','build_builder_reference.py','build_miner_reference.py',
          'build_basher_reference.py','build_digger_reference.py','build_reference.py','harness.pas']
CASES={
    'early_falling':[(54,0,'AssignFloater',0)],
    'walking_before_drop':[(80,0,'AssignFloater',0)],
    'duplicate_absent':[(54,0,'AssignFloater',0),(80,0,'AssignFloater',0),(80,1,'AssignFloater',999)],
    'pause_assign_resume':[(80,0,'Pause',0),(80,1,'AssignFloater',0),(90,0,'Resume',0)],
    'athlete':[(54,0,'AssignClimber',0),(55,0,'AssignFloater',0)],
    'floater_changes_jobs':[(54,0,'AssignFloater',0),(110,0,'AssignBuilder',0),(120,0,'AssignDigger',0),(130,0,'AssignMiner',0),(140,0,'AssignBasher',0),(145,0,'AssignBuilder',0)],
    'bomber_while_floating':[(54,0,'AssignFloater',0),(80,0,'AssignBomber',0)],
    'two_floaters':[(54,0,'AssignFloater',0),(100,0,'AssignFloater',1)],
    'blocked_assignment':[(80,0,'AssignBlocker',0),(90,0,'AssignFloater',0),(100,0,'AssignBomber',0)],
    'climber_then_late_floater':[(100,0,'AssignClimber',0),(120,0,'AssignFloater',0)]
}


def validate_floater_history(states,details):
    if len(states)!=len(details):raise ValueError('missing Floater boundary')
    for state,row in zip(states,details):
        if type(row['remaining']) is not int or not 0<=row['remaining']<=65535:raise ValueError('invalid Floater count')
        if len(row['actors'])!=len(state['actors']):raise ValueError('missing Floater actor')
        for actor,entry in zip(state['actors'],row['actors']):
            if (type(entry) is not list or len(entry)!=3 or type(entry[0]) is not int or entry[0]!=actor['id'] or type(entry[1]) is not bool or type(entry[2]) is not int or not 0<=entry[2]<=15):raise ValueError('invalid Floater actor state')
            if actor['action']=='Floating' and not entry[1]:raise ValueError('floating without ability')
            if entry[2] and not entry[1]:raise ValueError('float table state without ability')
        state['floater']=row


def collect(directory):
    states=collect_climber(directory)
    details=[json.loads(s) for s in (directory/'floater.jsonl').read_text().splitlines()]
    validate_floater_history(states,details)
    return states

def reference_run(executable,config,mask,objects,bash_masks,mine_masks,raw_map,explosion_mask,inputs,inventory=(10,10,10,10,10,10,10,10)):
    header=[config[k] for k in ['width','height','minutes','total','rate']]
    header += [len(config['entrances']),int(config['pause_glitch']),int(config['faller_three']),config['steps']]
    script=' '.join(map(str,header))+'\n'+' '.join(map(str,inventory))+' 1\n'
    script+=''.join(f'{x} {y}\n' for x,y in config['entrances'])+str(len(inputs))+'\n'
    codes=dict(Pause=0,Resume=1,SetReleaseRate=2,AssignDigger=3,AssignBasher=4,AssignMiner=5,AssignBuilder=6,AssignBlocker=7,AssignBomber=8,AssignClimber=9,AssignFloater=10)
    script+=''.join(f'{s} {q} {codes[c]} {v}\n' for s,q,c,v in inputs)
    result=subprocess.run([str(executable.resolve()),str(mask),str(objects),str(bash_masks),str(mine_masks),str(raw_map),str(explosion_mask)],
                          input=script,capture_output=True,text=True,check=True)
    return [json.loads(line) for line in result.stdout.splitlines()]


def run_native(binary,directory,replay,render,*,steps=420,assets=None,level=2,fast=True,pacing=None):
    command=[str(binary.resolve()),'--steps',str(steps),'--render-every',str(render),'--capture',str(directory),'--replay',str(replay)]
    command+=['--assets',str(assets),'--level',str(level)] if assets else ['--synthetic-floater']
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
    config=dict(width=640,height=160,minutes=5,total=10,rate=50,entrances=[[40,0]],pause_glitch=True,faller_three=True,steps=420)
    terrain=bytes(int((y>=140 and x<500) or (60<=y<140 and x<120) or (90<=y<140 and 200<=x<210)) for y in range(160) for x in range(640))
    rows={};snapshots=captures=0
    with tempfile.TemporaryDirectory(prefix='lemmings-floater-') as temporary:
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
                if name in ['early_falling','walking_before_drop','athlete'] and not any(a['action']=='Floating' for state in states for a in state['actors']):raise AssertionError('case never floated')
                if name=='bomber_while_floating' and not any(previous['actors'] and current['actors'] and previous['actors'][0]['action']=='Floating' and current['actors'][0]['action']=='Exploding' for previous,current in zip(states,states[1:])):raise AssertionError('countdown never expired while floating')
                rows[name]=dict(snapshots_sha256=signature,steps=len(states),inputs=inputs)
                snapshots+=len(states);captures+=count
    if args.record:
        provenance=json.loads(args.reference.with_name('reference.provenance.json').read_text())
        GOLDEN.write_text(json.dumps(dict(format='g07-floater-v1',reference=provenance,adapter_sha256=hashes,cases=rows),indent=2)+'\n')
    print(json.dumps(dict(status='PASS',cases=len(rows),replays=len(rows)*2,snapshots=snapshots,pixel_exact_captures=captures,
                         reference_executed=args.reference is not None,scope='eight jobs, both permanent abilities, sixteen-stage floating/opening/landing and Bomber expiry; not whole DOS')))


if __name__=='__main__':main()
