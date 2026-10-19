#!/usr/bin/env python3
"""Compare five-job replays, raw object-map histories and cached fields to Pascal."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile

from check_builder import collect as collect_builder, replay_text, run_native as builder_native
from check_basher import synthetic_masks as bashing_masks
from check_miner import synthetic_masks as mining_masks
from state_replay import ROOT, digest

GOLDEN=ROOT/'tools/fixtures/blocker_golden.json'
ADAPTERS=['build_blocker_reference.py','build_builder_reference.py','build_miner_reference.py',
          'build_basher_reference.py','build_digger_reference.py','build_reference.py','harness.pas']
CASES={
    'right_field':[(80,0,'AssignBlocker',0)],
    'left_field':[(200,0,'AssignBlocker',0)],
    'duplicate_absent_falling':[(54,0,'AssignBlocker',0),(80,0,'AssignBlocker',0),(80,1,'AssignBlocker',0),(80,2,'AssignBlocker',999)],
    'pause_assign_resume':[(80,0,'Pause',0),(80,1,'AssignBlocker',0),(90,0,'Resume',0)],
    'builder_then_blocker':[(80,0,'AssignBuilder',0),(100,0,'AssignBlocker',0),(110,0,'AssignDigger',0),(111,0,'AssignBasher',0),(112,0,'AssignMiner',0)],
    'two_fields':[(80,0,'AssignBlocker',0),(105,0,'AssignBlocker',1),(130,0,'AssignBlocker',1)],
    'support_digger':[(81,0,'AssignBlocker',0),(105,0,'AssignDigger',1)],
    'miner_then_blocker':[(80,0,'AssignMiner',0),(81,0,'AssignBlocker',0)],
}


def map_signature(data):
    result=0xcbf29ce484222325
    for value in data:result=((result^value)*0x100000001b3)&0xffffffffffffffff
    return f'{result:016x}'


def validate_map_history(rows,initial,final):
    if not rows:raise ValueError('missing object map boundaries')
    data=bytearray(initial);signature=map_signature(data)
    for step,row in enumerate(rows):
        previous=-1
        for item in row['changed']:
            if (type(item) is not list or len(item)!=2 or type(item[0]) is not int
                    or type(item[1]) is not int):raise ValueError('invalid object change')
            index,value=item
            if (step==0 or not previous<index<len(data) or not 0<=value<=255
                    or data[index]==value):raise ValueError('invalid or repeated object change')
            data[index]=value;previous=index
        if row['changed']:signature=map_signature(data)
        if row['object_map_fnv1a64']!=signature:raise ValueError('object map boundary hash differs')
    if bytes(data)!=final:raise ValueError('object map history differs from final bytes')


def collect(directory):
    states=collect_builder(directory)
    details=[json.loads(s) for s in (directory/'blocker.jsonl').read_text().splitlines()]
    if len(states)!=len(details):raise ValueError('missing Blocker boundary')
    validate_map_history(details,(directory/'object-map-initial.bin').read_bytes(),(directory/'object-map-final.bin').read_bytes())
    for state,detail in zip(states,details):state['blocker']=detail
    return states


def reference_run(executable,config,mask,objects,bash_masks,mine_masks,raw_map,inputs,inventory=(10,10,10,10,10)):
    header=[config[k] for k in ['width','height','minutes','total','rate']]
    header += [len(config['entrances']),int(config['pause_glitch']),int(config['faller_three']),config['steps']]
    script=' '.join(map(str,header))+'\n'+' '.join(map(str,inventory))+' 1\n'
    script+=''.join(f'{x} {y}\n' for x,y in config['entrances'])+str(len(inputs))+'\n'
    codes=dict(Pause=0,Resume=1,SetReleaseRate=2,AssignDigger=3,AssignBasher=4,AssignMiner=5,AssignBuilder=6,AssignBlocker=7)
    script+=''.join(f'{s} {q} {codes[c]} {v}\n' for s,q,c,v in inputs)
    result=subprocess.run([str(executable.resolve()),str(mask),str(objects),str(bash_masks),str(mine_masks),str(raw_map)],
                          input=script,capture_output=True,text=True,check=True)
    return [json.loads(line) for line in result.stdout.splitlines()]


def run_native(binary,directory,replay,render,**kwargs):
    _,captures=builder_native(binary,directory,replay,render,**kwargs)
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
    with tempfile.TemporaryDirectory(prefix='lemmings-blocker-') as temporary:
        root=Path(temporary)
        for name,data in [('mask',terrain),('objects',bytes(len(terrain))),('bash-masks',bashing_masks()),('mine-masks',mining_masks()),('raw-map',bytes([128])*(415*47))]:
            (root/name).write_bytes(data)
        for name,inputs in CASES.items():
            replay=root/(name+'.txt');replay.write_text(replay_text(inputs))
            expected=reference_run(args.reference,config,root/'mask',root/'objects',root/'bash-masks',root/'mine-masks',root/'raw-map',inputs) if args.reference else None
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
                if not any(a['action']=='Blocking' for s in states for a in s['actors']):raise AssertionError('case never exercised Blocker')
                if name=='two_fields' and max(len(s['blocker']['fields']) for s in states)<2:raise AssertionError('two fields not exercised')
                if name=='support_digger' and states[-1]['blocker']['fields']:raise AssertionError('support loss never restored the field')
                rows[name]=dict(snapshots_sha256=signature,steps=len(states),inputs=inputs)
                snapshots+=len(states);captures+=count
    if args.record:
        provenance=json.loads(args.reference.with_name('reference.provenance.json').read_text())
        GOLDEN.write_text(json.dumps(dict(format='g07-blocker-v1',reference=provenance,adapter_sha256=hashes,cases=rows),indent=2)+'\n')
    print(json.dumps(dict(status='PASS',cases=len(rows),replays=len(rows)*2,snapshots=snapshots,pixel_exact_captures=captures,
                         reference_executed=args.reference is not None,scope='five jobs, raw object-map changes/cache/force effects and terrain; not whole DOS')))


if __name__=='__main__':main()
