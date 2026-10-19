#!/usr/bin/env python3
"""Check time, integer percentages and frozen results against executed references."""
import argparse
from contextlib import nullcontext
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
from check_hazards import ADAPTERS as HAZARD_ADAPTERS, collect as collect_hazards
from check_nuke import replay_text
from check_desktop import verify_pixels
from check_skill_interactions import compare_states
from state_replay import ROOT, digest

ADAPTERS = ['build_completion_reference.py'] + HAZARD_ADAPTERS
GOLDEN = ROOT/'tools/fixtures/completion_golden.json'
CASES = {
    'win': ('win',[]),
    'loss': ('loss',[]),
    'clock': ('clock',[]),
    'clock_pause_glitch': ('clock',[(16,0,'Pause',0),(34,0,'Resume',0)]),
    'paused_exit': ('win',[(103,0,'Pause',0),(125,0,'Resume',0)]),
    'particles': ('particles',[(60,0,'AssignBomber',0)]),
    'particles_pause': ('particles',[(60,0,'AssignBomber',0),(180,0,'Pause',0),(200,0,'Resume',0)]),
    'nuke_mixed': ('mixed',[(60,0,'Nuke',0)]),
    'early_nuke': ('win',[(0,0,'Nuke',0)]),
    'zero_time': ('zero',[]),
    'after_finish': ('win',[(800,0,'Nuke',0),(900,0,'AssignBomber',0)]),
}


def validate_completion(states,rows,config):
    keys={'phase','reason','original_total','lemming_count','required','rescued','target_percent','rescued_percent'}
    numeric=keys-{'phase','reason'}
    if not states or len(states)!=len(rows): raise ValueError('missing completion boundaries')
    previous=None
    for state,row in zip(states,rows):
        if type(row) is not dict or set(row)!=keys or any(type(row[k]) is not int or row[k]<0 for k in numeric):
            raise ValueError('invalid completion schema')
        if previous is not None and previous['completion']['phase']!='Playing':
            old={k:v for k,v in previous.items() if k!='completion'}
            current={k:v for k,v in state.items() if k!='completion'}
            if row!=previous['completion'] or old!=current:raise ValueError('frozen result changed')
        total=config['total'];required=config['required'];maximum=state['nuke']['max_lemmings'];saved=state['exit']['saved']
        done=saved*100//maximum if maximum else 0;target=required*100//total if total else 0
        if (row['original_total'],row['required'],row['lemming_count'],row['rescued'],row['target_percent'],row['rescued_percent'])!=(total,required,maximum,saved,target,done):
            raise ValueError('invalid original target/current denominator/counts')
        if row['phase'] not in ['Playing','Won','Lost']:raise ValueError('invalid completion phase')
        delay=max((a[2] for a in state['bomber']['actors']),default=0)
        exhausted=delay==0 and (state['removed']>=maximum or (state['nuke']['is_nuked'] and state['alive']==0))
        expired=previous is not None and previous['seconds_remaining']==0
        # Initial boundary zero is a snapshot, not a call to the completion check.
        should_end=previous is not None and (exhausted or expired)
        if row['phase']=='Playing':
            if row['reason'] is not None or should_end:raise ValueError('missing or premature result')
        else:
            reason='PopulationExhausted' if exhausted else 'TimeExpired' if expired else None
            if not should_end or row['reason']!=reason or (row['phase']=='Won')!=(done>=target):
                raise ValueError('invalid finish reason/result/order')
        state['completion']=row;previous=state


def collect(directory):
    states=collect_hazards(directory)
    rows=[json.loads(line) for line in (directory/'completion.jsonl').read_text().splitlines()]
    validate_completion(states,rows,json.loads((directory/'config.json').read_text()))
    return states


def reference_run(executable,config,directory,inputs):
    header=[config[k] for k in ['width','height','minutes','total','rate']]
    header += [len(config['entrances']),int(config['pause_glitch']),int(config['faller_three']),config['steps']]
    inventory=[config.get(k,10) for k in ['diggers','bashers','miners','builders','blockers','bombers','climbers','floaters']]
    script=' '.join(map(str,header))+'\n'+' '.join(map(str,inventory))+' 1 '+str(int(config.get('nuke_glitch',True)))+'\n'
    script+=''.join(f'{x} {y}\n' for x,y in config['entrances'])+str(len(inputs))+'\n'
    codes=dict(Pause=0,Resume=1,SetReleaseRate=2,AssignDigger=3,AssignBasher=4,AssignMiner=5,AssignBuilder=6,AssignBlocker=7,AssignBomber=8,AssignClimber=9,AssignFloater=10,Nuke=11)
    script+=''.join(f'{s} {q} {codes[c]} {v}\n' for s,q,c,v in inputs)
    (directory/'completion-config.txt').write_text(str(config['required'])+'\n')
    paths=['terrain-initial.bin','objects.bin','bash-masks.bin','mine-masks.bin','object-map-initial.bin','explosion-mask.bin','traps.bin','completion-config.txt']
    result=subprocess.run([str(executable.resolve()),*[str(directory/p) for p in paths]],input=script,check=True,text=True,capture_output=True)
    return [json.loads(line) for line in result.stdout.splitlines()]


def run_native(binary,directory,replay,render,*,kind='win',steps=1200,assets=None,level=1,fast=True):
    command=[str(binary.resolve()),'--steps',str(steps),'--render-every',str(render),'--capture',str(directory),'--replay',str(replay)]
    command += ['--assets',str(assets),'--level',str(level)] if assets else ['--synthetic-outcome',kind]
    if fast:command+=['--fast']
    subprocess.run(command,check=True,text=True,capture_output=True,env=dict(os.environ,SDL_VIDEODRIVER=os.environ.get('SDL_VIDEODRIVER','dummy')))
    return collect(directory),verify_pixels(directory)


def exercised(name,states):
    final=states[-1];r=final['completion']
    if r['phase']=='Playing':raise AssertionError('case did not finish')
    if name=='nuke_mixed' and (r['rescued'],r['lemming_count'],r['target_percent'],r['rescued_percent'],r['phase'])!=(1,2,50,50,'Won'):
        raise AssertionError('Nuke denominator case did not exercise one-of-two rescue')
    if name.startswith('clock') and (r['reason']!='TimeExpired' or final['iteration'] not in [1021,1039]):
        raise AssertionError('clock boundary not exercised')
    if name.startswith('particles'):
        if not any(s['removed']==1 and s['completion']['phase']=='Playing' and s['bomber']['actors'][0][2]>0 for s in states):
            raise AssertionError('particle delay absent')
        if final['bomber']['actors'][0][2]!=0:raise AssertionError('particles not completed')
    if name=='after_finish' and any(e.startswith('Input:') for s in states for e in s['events']):raise AssertionError('post-finish command applied')


def check(args,root):
    hashes={name:hashlib.sha256((ROOT/'tools/lemmix'/name).read_bytes()).hexdigest() for name in ADAPTERS}
    frozen=None if args.record else json.loads(GOLDEN.read_text())
    if frozen and (frozen['adapter_sha256']!=hashes or set(frozen['cases'])!=set(CASES)):raise ValueError('completion adapters/cases changed')
    rows={};snapshots=captures=0
    for name,(kind,inputs) in CASES.items():
        replay=root/(name+'.txt');replay.write_text(replay_text(inputs));expected=None
        for render in [1,7]:
            context=nullcontext(root) if args.evidence else tempfile.TemporaryDirectory(dir=root)
            with context as directory:
                directory=Path(directory)/(name+'-'+str(render))
                states,count=run_native(args.binary,directory,replay,render,kind=kind)
                config=json.loads((directory/'config.json').read_text()) | dict(steps=1200)
                if args.reference and expected is None:
                    expected=reference_run(args.reference,config,directory,inputs)
                    if args.evidence:(root/(name+'-reference.json')).write_text(json.dumps(expected))
            if expected is not None:compare_states(states,expected,name)
            exercised(name,states);signature=digest(states)
            if frozen and frozen['cases'][name]['snapshots_sha256']!=signature:raise AssertionError('completion golden changed: '+name)
            if name in rows and rows[name]['snapshots_sha256']!=signature:raise AssertionError('render-dependent completion')
            rows[name]=dict(kind=kind,inputs=inputs,snapshots=len(states),snapshots_sha256=signature,final=states[-1]['completion'],final_iteration=states[-1]['iteration'])
            snapshots+=len(states);captures+=count
    if args.record:
        provenance=json.loads(args.reference.with_name('reference.provenance.json').read_text())
        GOLDEN.write_text(json.dumps(dict(format='g08-completion-v1',reference=provenance,adapter_sha256=hashes,cases=rows),indent=2)+'\n')
    summary=dict(status='PASS',cases=len(rows),replays=len(rows)*2,snapshots=snapshots,pixel_exact_captures=captures,reference_executed=args.reference is not None,scope='time/percentages/results with DOS boundary adapter; not result screens or whole DOS')
    (root/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary',type=Path,default=ROOT/'target/release/lemmings-desktop')
    p.add_argument('--reference',type=Path);p.add_argument('--record',action='store_true');p.add_argument('--evidence',type=Path)
    args=p.parse_args()
    if args.record and not args.reference:raise ValueError('record requires actual reference execution')
    if args.evidence:
        root=args.evidence.resolve()
        if not root.is_relative_to(ROOT/'assets/local'):raise ValueError('local evidence required')
        root.mkdir();check(args,root)
    else:
        with tempfile.TemporaryDirectory(prefix='lemmings-completion-') as directory:check(args,Path(directory))


if __name__=='__main__':main()
