#!/usr/bin/env python3
"""Compare Nuke cascade/spawn/timer states to executed Pascal."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile

from check_blocker import collect as collect_blocker
from check_bomber import validate_bomber_history
from check_climber import validate_climber_history
from check_floater import validate_floater_history
from check_desktop import verify_pixels
import os
from check_builder import replay_text as skill_replay_text

def replay_text(inputs):
    return ''.join(f'{s} {q} Nuke\n' if c=='Nuke' else skill_replay_text([(s,q,c,v)]) for s,q,c,v in inputs)
from check_basher import synthetic_masks as bashing_masks
from check_miner import synthetic_masks as mining_masks
from state_replay import ROOT, digest

GOLDEN=ROOT/'tools/fixtures/nuke_golden.json'
ADAPTERS=['build_nuke_reference.py','build_floater_reference.py','build_climber_reference.py','build_bomber_reference.py','build_blocker_reference.py','build_builder_reference.py','build_miner_reference.py',
          'build_basher_reference.py','build_digger_reference.py','build_reference.py','harness.pas']
CASES={
    'before_birth':[(0,0,'Nuke',0)],
    'birth_boundary':[(53,0,'Nuke',0)],
    'three_running':[(120,0,'Nuke',0)],
    'duplicates':[(120,0,'Nuke',0),(120,1,'Nuke',0),(150,0,'Nuke',0)],
    'pause_reject_resume':[(120,0,'Pause',0),(120,1,'Nuke',0),(130,0,'Resume',0),(130,1,'Nuke',0),(140,0,'Pause',0),(150,0,'Resume',0)],
    'blocker_then_nuke':[(80,0,'AssignBlocker',0),(120,0,'Nuke',0)],
    'running_bomber':[(80,0,'AssignBomber',0),(120,0,'Nuke',0)],
    'ohno_rearm':[(80,0,'AssignBomber',0),(160,0,'Nuke',0)],
    'permanent_athlete':[(80,0,'AssignClimber',0),(81,0,'AssignFloater',0),(120,0,'Nuke',0)],
    'changing_jobs':[(80,0,'AssignDigger',0),(100,0,'AssignBuilder',0),(110,0,'AssignBasher',0),(120,0,'Nuke',0)]
}



def validate_nuke_history(states,details):
    if len(states)!=len(details):raise ValueError('missing Nuke boundary')
    for state,row in zip(states,details):
        if set(row)!={'is_nuked','assigning','next_index','max_lemmings'}:raise ValueError('malformed Nuke row')
        if any(type(row[k]) is not bool for k in ['is_nuked','assigning']):raise ValueError('invalid Nuke flags')
        if any(type(row[k]) is not int or not 0<=row[k]<=2**32-1 for k in ['next_index','max_lemmings']):raise ValueError('invalid Nuke counters')
        if row['next_index']>state['released'] or row['max_lemmings']<state['released']:raise ValueError('impossible Nuke counters')
        if not row['is_nuked'] and (row['assigning'] or row['next_index']):raise ValueError('scan before Nuke')
        if not row['is_nuked'] and any(a['action'] in ['OhNo','Exploding'] and b[1] for a,b in zip(state['actors'],state['bomber']['actors'])):raise ValueError('rearmed terminal timer before Nuke')
        state['nuke']=row


def collect(directory):
    states=collect_blocker(directory)
    def rows(name):return [json.loads(s) for s in (directory/(name+'.jsonl')).read_text().splitlines()]
    validate_bomber_history(states,rows('bomber'),allow_nuke=True)
    validate_climber_history(states,rows('climber'))
    validate_floater_history(states,rows('floater'))
    validate_nuke_history(states,rows('nuke'))
    return states

def reference_run(executable,config,mask,objects,bash_masks,mine_masks,raw_map,explosion_mask,inputs,inventory=(10,10,10,10,10,10,10,10)):
    header=[config[k] for k in ['width','height','minutes','total','rate']]
    header += [len(config['entrances']),int(config['pause_glitch']),int(config['faller_three']),config['steps']]
    script=' '.join(map(str,header))+'\n'+' '.join(map(str,inventory))+' 1 '+str(int(config.get('nuke_glitch',True)))+'\n'
    script+=''.join(f'{x} {y}\n' for x,y in config['entrances'])+str(len(inputs))+'\n'
    codes=dict(Pause=0,Resume=1,SetReleaseRate=2,AssignDigger=3,AssignBasher=4,AssignMiner=5,AssignBuilder=6,AssignBlocker=7,AssignBomber=8,AssignClimber=9,AssignFloater=10,Nuke=11)
    script+=''.join(f'{s} {q} {codes[c]} {v}\n' for s,q,c,v in inputs)
    result=subprocess.run([str(executable.resolve()),str(mask),str(objects),str(bash_masks),str(mine_masks),str(raw_map),str(explosion_mask)],
                          input=script,capture_output=True,text=True,check=True)
    return [json.loads(line) for line in result.stdout.splitlines()]


def run_native(binary,directory,replay,render,*,steps=420,assets=None,level=2,fast=True,pacing=None):
    command=[str(binary.resolve()),'--steps',str(steps),'--render-every',str(render),'--capture',str(directory),'--replay',str(replay)]
    command+=['--assets',str(assets),'--level',str(level)] if assets else ['--synthetic-nuke']
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
    terrain=bytes(int((y>=100 and x<500) or (60<=y<100 and 160<=x<170)) for y in range(160) for x in range(640))
    rows={};snapshots=captures=0
    with tempfile.TemporaryDirectory(prefix='lemmings-nuke-') as temporary:
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
                if not states[-1]['nuke']['is_nuked'] or states[-1]['nuke']['assigning']:raise AssertionError('Nuke scan incomplete')
                if name in ['before_birth','birth_boundary'] and states[-1]['released']!=0:raise AssertionError('birth after early Nuke')
                if name=='ohno_rearm' and not any(a['action']=='OhNo' and b[1]>0 for state in states for a,b in zip(state['actors'],state['bomber']['actors'])):raise AssertionError('OhNo was not rearmed')
                rows[name]=dict(snapshots_sha256=signature,steps=len(states),inputs=inputs)
                snapshots+=len(states);captures+=count
    if args.record:
        provenance=json.loads(args.reference.with_name('reference.provenance.json').read_text())
        GOLDEN.write_text(json.dumps(dict(format='g07-nuke-v1',reference=provenance,adapter_sha256=hashes,cases=rows),indent=2)+'\n')
    print(json.dumps(dict(status='PASS',cases=len(rows),replays=len(rows)*2,snapshots=snapshots,pixel_exact_captures=captures,
                         reference_executed=args.reference is not None,scope='Nuke after eight-job motion, spawn suppression, sequential timers, OhNo rearming and glitch maximum; not whole DOS')))


if __name__=='__main__':main()
