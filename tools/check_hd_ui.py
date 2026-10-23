#!/usr/bin/env python3
"""Pair actual SDL HD/original UI courses and retain states for WASM replay."""
import argparse,json,os,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
CASES=[('play','builder',130,[(80,'AssignBuilder',0)]),('paused','builder',100,[(80,'Pause',0)]),
       ('countdown','builder',130,[(80,'AssignBomber',0)]),('nuke','builder',120,[(80,'Nuke',0)]),
       ('won','win',500,[]),('lost','loss',500,[]),('time-up','zero',1,[])]
def check(binary,pack,output):
    output=output.resolve()
    if not output.is_relative_to(ROOT/'assets/local') or output.exists():raise ValueError('fresh evidence under assets/local required')
    output.mkdir(parents=True);rows=[]
    for name,scene,steps,commands in CASES:
        replay=output/(name+'.txt');replay.write_text(''.join(f'{n} 0 {c}'+(f' {v}' if c.startswith('Assign') else '')+'\n' for n,c,v in commands))
        captures=[]
        for mode in ['original','hd']:
            target=output/(name+'-'+mode);captures.append(target)
            args=[str(binary.resolve()),'--synthetic-builder'] if scene=='builder' else [str(binary.resolve()),'--synthetic-outcome',scene]
            args+=['--steps',str(steps),'--render-every','400','--fast','--capture',str(target)]
            if commands:args+=['--replay',str(replay)]
            if mode=='hd':args+=['--hd','--hd-pack',str(pack.resolve())]
            with (output/(name+'-'+mode+'.log')).open('w') as log:subprocess.run(args,cwd=ROOT,env=dict(os.environ,SDL_VIDEODRIVER='dummy'),stdout=log,stderr=subprocess.STDOUT,check=True)
        plain,hd=captures
        for path in plain.iterdir():
            if path.suffix=='.jsonl' or path.name in ['terrain-final.bin','brick-final.bin','object-map-final.bin','inputs.txt']:
                if path.read_bytes()!=(hd/path.name).read_bytes():raise AssertionError((name,path.name))
        states=[json.loads(s) for s in (hd/'full.jsonl').read_text().splitlines()]
        if name=='paused' and not states[-1]['session']['paused']:raise AssertionError('Pause not exercised')
        if name in ['won','lost','time-up'] and states[-1]['completion']['phase'] not in ['Won','Lost']:raise AssertionError('result not exercised')
        rows.append(dict(name=name,scene=scene,commands=commands,states=states))
    (output/'cases.json').write_text(json.dumps(rows,separators=(',',':'))+'\n')
    report=dict(status='PASS',cases=len(rows),states=sum(len(r['states']) for r in rows),mode_state_masks_inputs_sound_equal=True,scope='actual SDL synthetic UI courses; not original-level completion or performance')
    (output/'summary.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,default=ROOT/'target/release/lemmings-desktop');p.add_argument('--pack',type=Path,default=ROOT/'assets/local/web/hd-assets');p.add_argument('--output',type=Path,required=True);a=p.parse_args();check(a.binary,a.pack,a.output)
