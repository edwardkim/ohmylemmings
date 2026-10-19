#!/usr/bin/env python3
"""Compare deterministic water/fire/trap replays with executed pinned functions."""
import argparse
from contextlib import nullcontext
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

from check_exit import ADAPTERS as EXIT_ADAPTERS, SKILLS, collect as collect_exit
from check_nuke import replay_text
from check_desktop import verify_pixels
from check_skill_interactions import compare_states
from state_replay import ROOT, digest

ADAPTERS = ['build_hazards_reference.py'] + EXIT_ADAPTERS
GOLDEN = ROOT / 'tools/fixtures/hazards_golden.json'
CASES = {
    'water_expiry': (143, False, [(54,0,'AssignBomber',0)]),
    'fire_expiry': (144, False, [(54,0,'AssignBomber',0)]),
    'water': (133, False, []),
    'fire': (134, False, []),
    'water_assignments': (133, False, [(103,i,'Assign'+skill,0) for i,skill in enumerate(SKILLS)]),
    'fire_assignments': (134, False, [(103,i,'Assign'+skill,0) for i,skill in enumerate(SKILLS)]),
    'trap_fast': (0, False, []),
    'trap_busy': (1, False, []),
    'trap_pause': (0, False, [(103,0,'Pause',0),(125,0,'Resume',0)]),
    'trap_continuous': (2, False, []),
    'airborne_water': (133, True, []),
    'airborne_fire': (134, True, []),
    'trap_nuke': (0, False, [(103,0,'Nuke',0)]),
    'water_nuke': (133, False, [(103,0,'Nuke',0)]),
}


def validate_hazards(states, rows, metadata):
    if len(metadata) != 48 or len(states) != len(rows): raise ValueError('missing hazard boundaries/metadata')
    slots = {}
    for i in range(16):
        count,first,continuous = metadata[i*3:i*3+3]
        if continuous > 1 or (count and first >= count) or (not count and (first or continuous)):
            raise ValueError('invalid trap metadata')
        if count: slots[i] = count
    previous = [0,0,0]
    for state,row in zip(states,rows):
        if set(row) != {'traps','drowned','vaporized','trapped'} or type(row['traps']) is not list:
            raise ValueError('invalid hazard schema')
        ids = []
        for trap in row['traps']:
            if type(trap) is not list or len(trap) != 3: raise ValueError('invalid trap row')
            i,frame,triggered = trap
            if type(i) is not int or type(frame) is not int or type(triggered) is not bool or i not in slots or not 0 <= frame < slots[i]:
                raise ValueError('invalid trap state')
            ids.append(i)
        if ids != sorted(slots): raise ValueError('missing/duplicate/unordered trap IDs')
        wanted = [0,0,0]
        for a in state['actors']:
            reason = a['removed']
            if reason in ['Drowned','Vaporized']:
                index = int(reason == 'Vaporized')
                if a['action'] != ['Drowning','Vaporizing'][index] or a['frame'] != [15,13][index] or a['end_of_animation'] is not True:
                    raise ValueError('death before terminal animation completed')
                wanted[index] += 1
            if reason == 'Trapped':
                if a['action'] in ['Exiting','Drowning','Vaporizing','Exploding']:
                    raise ValueError('trap killed an action which does not check objects')
                wanted[2] += 1
        actual = [row[k] for k in ['drowned','vaporized','trapped']]
        if any(type(x) is not int for x in actual) or actual != wanted or any(a<b for a,b in zip(actual,previous)):
            raise ValueError('inconsistent death counts')
        previous = actual
        state['hazards'] = row


def collect(directory):
    states = collect_exit(directory)
    rows = [json.loads(line) for line in (directory/'hazards.jsonl').read_text().splitlines()]
    validate_hazards(states,rows,(directory/'traps.bin').read_bytes())
    return states


def reference_run(executable,config,directory,inputs):
    header = [config[k] for k in ['width','height','minutes','total','rate']]
    header += [len(config['entrances']),int(config['pause_glitch']),int(config['faller_three']),config['steps']]
    inventory = [config.get(k,10) for k in ['diggers','bashers','miners','builders','blockers','bombers','climbers','floaters']]
    script = ' '.join(map(str,header))+'\n'+' '.join(map(str,inventory))+' 1 '+str(int(config.get('nuke_glitch',True)))+'\n'
    script += ''.join(f'{x} {y}\n' for x,y in config['entrances'])+str(len(inputs))+'\n'
    codes = dict(Pause=0,Resume=1,SetReleaseRate=2,AssignDigger=3,AssignBasher=4,AssignMiner=5,AssignBuilder=6,AssignBlocker=7,AssignBomber=8,AssignClimber=9,AssignFloater=10,Nuke=11)
    script += ''.join(f'{s} {q} {codes[c]} {v}\n' for s,q,c,v in inputs)
    paths = ['terrain-initial.bin','objects.bin','bash-masks.bin','mine-masks.bin','object-map-initial.bin','explosion-mask.bin','traps.bin']
    result = subprocess.run([str(executable.resolve()),*[str(directory/p) for p in paths]],input=script,check=True,text=True,capture_output=True)
    return [json.loads(line) for line in result.stdout.splitlines()]


def run_native(binary,directory,replay,render,*,kind=133,drop=False,steps=420,assets=None,level=1,fast=True):
    modes = {143:'--synthetic-water-late',144:'--synthetic-fire-late',133:'--synthetic-water',134:'--synthetic-fire',0:'--synthetic-trap',1:'--synthetic-trap-slow',2:'--synthetic-trap-continuous'}
    command = [str(binary.resolve()),'--steps',str(steps),'--render-every',str(render),'--capture',str(directory),'--replay',str(replay)]
    command += ['--assets',str(assets),'--level',str(level)] if assets else [modes[kind]+('-drop' if drop else '')]
    if fast: command += ['--fast']
    subprocess.run(command,check=True,text=True,capture_output=True,env=dict(os.environ,SDL_VIDEODRIVER=os.environ.get('SDL_VIDEODRIVER','dummy')))
    return collect(directory),verify_pixels(directory)


def exercised(name,states):
    final = states[-1]
    if final['exit']['saved'] != 0: raise AssertionError('hazard counted as rescue')
    if 'trap' in name:
        n = final['hazards']['trapped']
        if name == 'trap_nuke':
            if n != final['released'] or n != 2: raise AssertionError('Nuke/trap cohort differs')
        elif (name == 'trap_busy' and not 0 < n < 10) or (name != 'trap_busy' and n != 10):
            raise AssertionError('trap activation/reuse not exercised: '+name)
    elif name.endswith('_expiry'):
        actions = [s['actors'][0]['action'] for s in states if s['actors']]
        if 'Exploding' not in actions or 'OhNo' in actions or final['actors'][0]['removed'] != 'Exploded':
            raise AssertionError('hazard timer expiry failed to explode immediately')
        expected_action = 'Drowning' if name.startswith('water') else 'Vaporizing'
        if expected_action not in actions: raise AssertionError('timer did not expire inside hazard')
    elif final['hazards']['drowned']+final['hazards']['vaporized'] != final['released']:
        raise AssertionError('terminal hazard cohort did not die')
    if name.endswith('assignments'):
        accepted = 3 if name.startswith('water') else 2
        events = {e for s in states for e in s['events']}
        for i in range(8):
            if f'Input:103:{i}:'+('true' if i<accepted else 'false') not in events:
                raise AssertionError('dying actor assignment rule not exercised')
    if name == 'trap_pause':
        if states[104]['hazards'] != states[125]['hazards']: raise AssertionError('trap advanced during pause')
    if name.startswith('airborne'):
        entry = next(s for s in states if s['actors'] and s['actors'][0]['action'] in ['Drowning','Vaporizing'])
        if entry['actors'][0]['y'] >= 140: raise AssertionError('airborne trigger was ignored')
    if name == 'water_nuke' and (not final['nuke']['is_nuked'] or final['released'] != 2):
        raise AssertionError('Nuke did not freeze the expected cohort')


def check(args,root):
    frozen = None if args.record else json.loads(GOLDEN.read_text())
    hashes = {n:hashlib.sha256((ROOT/'tools/lemmix'/n).read_bytes()).hexdigest() for n in ADAPTERS}
    if frozen and (hashes != frozen['adapter_sha256'] or set(CASES) != set(frozen['cases'])):
        raise ValueError('hazard adapter/cases changed; execute reference again')
    rows = {};snapshots=captures=0
    for name,(kind,drop,inputs) in CASES.items():
        replay = root/(name+'.txt');replay.write_text(replay_text(inputs))
        expected = None
        for render in [1,7]:
            context = nullcontext(root) if args.evidence else tempfile.TemporaryDirectory(dir=root)
            with context as directory:
                directory = Path(directory)/(name+'-'+str(render))
                states,count = run_native(args.binary,directory,replay,render,kind=kind,drop=drop)
                config = json.loads((directory/'config.json').read_text()) | dict(steps=420)
                if args.reference and expected is None:
                    expected = reference_run(args.reference,config,directory,inputs)
                    (root/(name+'-reference.json')).write_text(json.dumps(expected))
            if expected is not None: compare_states(states,expected,name)
            exercised(name,states)
            signature = digest(states)
            if frozen and (frozen['cases'][name]['snapshots_sha256'] != signature or frozen['cases'][name]['inputs'] != [list(i) for i in inputs]):
                raise AssertionError('recorded hazard result/scenario changed: '+name)
            if name in rows and signature != rows[name]['snapshots_sha256']: raise AssertionError('render-dependent hazard state')
            rows[name] = dict(kind=kind,drop=drop,inputs=inputs,snapshots_sha256=signature,snapshots=len(states),final=states[-1]['hazards'])
            snapshots += len(states);captures += count
    if args.record:
        provenance=json.loads(args.reference.with_name('reference.provenance.json').read_text())
        GOLDEN.write_text(json.dumps(dict(format='g08-hazards-v1',reference=provenance,adapter_sha256=hashes,cases=rows),indent=2)+'\n')
    summary=dict(status='PASS',cases=len(rows),replays=len(rows)*2,snapshots=snapshots,pixel_exact_captures=captures,reference_executed=args.reference is not None,scope='water/fire deaths, trap arbitration/reuse, jobs/timers/Pause/Nuke; not whole DOS')
    (root/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary',type=Path,default=ROOT/'target/release/lemmings-desktop')
    p.add_argument('--reference',type=Path);p.add_argument('--record',action='store_true');p.add_argument('--evidence',type=Path)
    args=p.parse_args()
    if args.record and not args.reference: raise ValueError('record requires actual reference execution')
    if args.evidence:
        root=args.evidence.resolve()
        if not root.is_relative_to(ROOT/'assets/local'): raise ValueError('local evidence required')
        root.mkdir();check(args,root)
    else:
        with tempfile.TemporaryDirectory(prefix='lemmings-hazards-') as directory: check(args,Path(directory))


if __name__=='__main__': main()
