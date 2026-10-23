#!/usr/bin/env python3
"""Pair actual SDL trap courses and preserve fixtures/states for WASM replay."""
import argparse,json,os
from pathlib import Path
import subprocess
try:
    from .trap_fixture import bank,PROFILES
except ImportError:
    import sys
    sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
    from tools.trap_fixture import bank,PROFILES
ROOT=Path(__file__).resolve().parents[1]

def check(binary,pack,output):
    output=output.resolve()
    if not output.is_relative_to(ROOT/'assets/local') or output.exists():raise ValueError('use a fresh directory under assets/local')
    output.mkdir(parents=True);cases=[]
    for theme,id,count,width,height,trigger,sound in PROFILES:
        name=f'trap-{theme}-{id}';directory=output/name;directory.mkdir();files=bank(theme,id)
        for filename,data in files.items():(directory/filename).write_bytes(data)
        for variant,commands in [('normal',[]),('pause',[(105,0,'Pause',0),(115,0,'Resume',0)])]:
            replay=output/f'{name}-{variant}.txt';replay.write_text(''.join(f'{step} {seq} {command}\n' for step,seq,command,value in commands));captures=[]
            for mode in ['original','hd']:
                capture=output/f'native-{theme}-{id}-{variant}-{mode}';captures.append(capture)
                args=[str(binary.resolve()),'--assets',str(directory),'--level','1','--pacing','dos','--steps','400','--render-every','400','--fast','--capture',str(capture),'--replay',str(replay)]
                if mode=='hd':args+=['--hd','--hd-pack',str(pack.resolve())]
                with (output/f'{name}-{variant}-{mode}.log').open('w') as log:subprocess.run(args,cwd=ROOT,env=dict(os.environ,SDL_VIDEODRIVER='dummy'),stdout=log,stderr=subprocess.STDOUT,check=True)
            a,b=captures;names=[p.name for p in a.iterdir() if p.suffix=='.jsonl' or p.name in ['terrain-final.bin','brick-final.bin','object-map-final.bin','inputs.txt']]
            for filename in names:
                if (a/filename).read_bytes()!=(b/filename).read_bytes():raise AssertionError((theme,id,variant,filename))
            states=[json.loads(s) for s in (b/'full.jsonl').read_text().splitlines()];final=states[-1];seen={t[1] for s in states for t in s['hazards']['traps']};active=sum(any(t[2] for t in s['hazards']['traps']) for s in states)
            if final['hazards']['trapped']<2 or final['exit']['saved']!=0 or final['hazards']['drowned'] or final['hazards']['vaporized']:raise AssertionError((name,variant,final['hazards']))
            if seen!=set(range(count)) or active==0:raise AssertionError(('missing phases',name,sorted(seen)))
            if variant=='pause':
                paused=[s['hazards']['traps'] for s in states if s['session']['paused']]
                if len(paused)!=10 or any(t!=paused[0] for t in paused):raise AssertionError(('pause advanced trap',name))
            cases.append(dict(theme=theme,id=id,count=count,variant=variant,commands=commands,directory=directory.name,files=sorted(files),states=states,trapped=final['hazards']['trapped'],active_states=active,identical_files=len(names)))
    (output/'cases.json').write_text(json.dumps(cases,separators=(',',':'))+'\n')
    report=dict(status='PASS',profiles=len(PROFILES),cases=len(cases),states=sum(len(r['states']) for r in cases),trapped=sum(r['trapped'] for r in cases),all_159_phases=True,mode_state_inputs_masks_sound_equal=True,scope='invented courses with original effect4 metadata; two independently animated instances after a non-trap, normal/Pause, actual SDL pairs; not original-level solutions or performance')
    (output/'summary.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,default=ROOT/'target/release/lemmings-desktop');p.add_argument('--pack',type=Path,default=ROOT/'assets/local/web/hd-assets');p.add_argument('--output',type=Path,required=True);a=p.parse_args();check(a.binary,a.pack,a.output)
