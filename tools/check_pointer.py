#!/usr/bin/env python3
"""Offline pointer arbitration golden and selected-ID SDL replay regression."""
import argparse
from contextlib import nullcontext
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

from check_basher import synthetic_masks as bashing_masks
from check_miner import synthetic_masks as mining_masks
from check_hazards import collect
from check_desktop import verify_pixels
from check_skill_interactions import ADAPTERS as BASE_ADAPTERS
from state_replay import ROOT, digest

GOLDEN=ROOT/'tools/fixtures/pointer_golden.json'
ADAPTERS=['build_pointer_reference.py','pointer_probe.pas']+BASE_ADAPTERS
CASES={
    'fallback':('--synthetic-builder','80 0 AssignBuilder 0\n105 0 AssignPointer Builder 0 1 false\n'),
    'remote':('--synthetic-builder','80 0 AssignBuilder 0\n105 0 AssignPointer Bomber 1 - true\n'),
    'trap_ghost':('--synthetic-trap','105 0 AssignPointer Blocker 0 - true\n'),
    'pause_remote':('--synthetic-builder','80 0 Pause\n80 1 AssignBuilder 0\n90 0 AssignPointer Bomber 0 - true\n100 0 Resume\n'),
}


def probe_shape(text):
    lines=text.splitlines()
    if len(lines)!=48494:raise ValueError('missing pointer probe cases')
    for i,line in enumerate(lines):
        w=line.split()
        if i<44604:
            if len(w)!=5 or w[0]!='H' or int(w[1])!=i:raise ValueError('malformed hit case')
            count,primary,secondary=map(int,w[2:])
            if not 0<=count<=2 or primary not in [-1,0,1] or secondary not in [-1,0,1]:raise ValueError('invalid hit result')
        elif i<44606:
            if len(w)!=6 or w[:2]!=['R',str(i-44604)] or w[2:5]!=['0','1','true' if i==44605 else 'false']:
                raise ValueError('remote assignment was not exercised')
        elif len(w)!=4 or w[:2]!=['F',str(i-44606)] or int(w[2]) not in [-1,0,1] or int(w[3]) not in [0,1,2]:
            raise ValueError('malformed fallback case')
    return hashlib.sha256(text.encode()).hexdigest()


def check(args,root):
    hashes={name:hashlib.sha256((ROOT/'tools/lemmix'/name).read_bytes()).hexdigest() for name in ADAPTERS}
    hashes['pointer_probe.rs']=hashlib.sha256((ROOT/'crates/lemmings-core/examples/pointer_probe.rs').read_bytes()).hexdigest()
    frozen=None if args.record else json.loads(GOLDEN.read_text())
    if frozen and frozen['adapter_sha256']!=hashes:raise ValueError('probe changed; execute reference again')
    probe=subprocess.run(['cargo','run','--quiet','--locked','--offline','-p','lemmings-core','--example','pointer_probe'],
                         cwd=ROOT,check=True,capture_output=True,text=True).stdout
    signature=probe_shape(probe);(root/'native-probe.txt').write_text(probe)
    if args.reference:
        paths=[]
        for name,data in [('mask',bytes(640*160)),('objects',bytes(640*160)),('bash',bashing_masks()),
                          ('mine',mining_masks()),('raw',bytes([128])*(415*47)),('explosion',bytes([255])*44)]:
            path=root/name;path.write_bytes(data);paths.append(str(path.resolve()))
        result=subprocess.run([str(args.reference.resolve()),*paths,'--pointer'],
            input='640 160 5 10 50 1 1 1 0\n10 10 10 10 10 10 10 10 1 1\n40 40\n0\n',check=True,capture_output=True,text=True)
        (root/'reference-probe.txt').write_text(result.stdout)
        if probe_shape(result.stdout)!=signature:raise AssertionError('executed reference differs')
    if frozen and frozen['probe_sha256']!=signature:raise AssertionError('pointer golden differs')
    rows=[]
    for case,(flag,script) in CASES.items():
        replay=root/(case+'.txt');replay.write_text(script);prior=None
        for every in [1,7]:
            folder=root/f'{case}-{every}'
            subprocess.run([str(args.binary.resolve()),flag,'--steps','340','--fast','--render-every',str(every),
                '--capture',str(folder),'--replay',str(replay)],env=dict(os.environ,SDL_VIDEODRIVER='dummy'),
                check=True,text=True,capture_output=True,timeout=30)
            states=collect(folder)
            if prior is not None and states!=prior:raise AssertionError('render frequency changed selected-ID replay')
            prior=states;events={e for s in states for e in s['events']}
            target='Input:90:0:true' if case=='pause_remote' else 'Input:105:0:true'
            if target not in events:raise AssertionError('selected assignment rejected: '+case)
            if case=='trap_ghost':
                ghost=states[106]['actors'][0]
                if ghost['removed']!='Trapped' or ghost['action']!='Blocking':raise AssertionError('missing trap ghost')
                if any(s['actors'][0]!=ghost for s in states[106:]):raise AssertionError('ghost advanced')
                maps=[s['blocker']['object_map_fnv1a64'] for s in states[106:]]
                if len(set(maps))!=1 or maps[0]==states[0]['blocker']['object_map_fnv1a64']:raise AssertionError('ghost field was restored or not placed')
            rows.append(dict(case=case,render_every=every,states=len(states),sha256=digest(states),captures=verify_pixels(folder)))
    record=dict(adapter_sha256=hashes,probe_sha256=signature,hit_cases=44604,remote_cases=2,fallback_cases=3888,
                replay_scope='selected-ID regression; independent reference comparison is the probe only')
    if args.record:GOLDEN.write_text(json.dumps(record,indent=2)+'\n')
    summary=dict(status='PASS',reference_executed=args.reference is not None,**record,runs=rows)
    (root/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    return summary


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary',type=Path,default=ROOT/'target/release/lemmings-desktop')
    p.add_argument('--reference',type=Path)
    p.add_argument('--record',action='store_true')
    p.add_argument('--evidence',type=Path)
    a=p.parse_args()
    if a.record and not a.reference:raise ValueError('record requires reference execution')
    if a.evidence:a.evidence.mkdir()
    with nullcontext(str(a.evidence)) if a.evidence else tempfile.TemporaryDirectory(prefix='lemmings-pointer-') as folder:
        print(json.dumps(check(a,Path(folder))))


if __name__=='__main__':main()
