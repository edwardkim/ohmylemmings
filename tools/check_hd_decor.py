#!/usr/bin/env python3
"""Pair actual SDL decor courses and preserve fixtures/states for WASM replay."""
import argparse,json,os
from pathlib import Path
import subprocess
try:
    from .decor_fixture import bank,PROFILES
except ImportError:
    import sys
    sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
    from tools.decor_fixture import bank,PROFILES
ROOT=Path(__file__).resolve().parents[1]

def check(binary,pack,output):
    output=output.resolve()
    if not output.is_relative_to(ROOT/'assets/local') or output.exists():raise ValueError('use a fresh directory under assets/local')
    output.mkdir(parents=True);cases=[]
    for theme in range(5):
        name=f'decor-{theme}';directory=output/name;directory.mkdir();files=bank(theme)
        for filename,data in files.items():(directory/filename).write_bytes(data)
        for variant,commands in [('dig-pause',[(68,0,'AssignDigger',0),(130,0,'Pause',0),(140,0,'Resume',0)])]:
            replay=output/f'{name}-{variant}.txt';replay.write_text(''.join(f'{step} {seq} {command}'+(f' {value}' if command.startswith('Assign') else '')+'\n' for step,seq,command,value in commands));captures=[]
            for mode in ['original','hd']:
                capture=output/f'native-{theme}-{variant}-{mode}';captures.append(capture)
                args=[str(binary.resolve()),'--assets',str(directory),'--level','1','--pacing','dos','--steps','400','--render-every','400','--fast','--capture',str(capture),'--replay',str(replay)]
                if mode=='hd':args+=['--hd','--hd-pack',str(pack.resolve())]
                with (output/f'{name}-{variant}-{mode}.log').open('w') as log:subprocess.run(args,cwd=ROOT,env=dict(os.environ,SDL_VIDEODRIVER='dummy'),stdout=log,stderr=subprocess.STDOUT,check=True)
            a,b=captures;names=[p.name for p in a.iterdir() if p.suffix=='.jsonl' or p.name in ['terrain-final.bin','brick-final.bin','object-map-final.bin','inputs.txt']]
            for filename in names:
                if (a/filename).read_bytes()!=(b/filename).read_bytes():raise AssertionError((theme,variant,filename))
            states=[json.loads(s) for s in (b/'full.jsonl').read_text().splitlines()]
            if not any(a['action']=='Digging' for s in states for a in s['session']['actors']):raise AssertionError(('no digging',theme))
            if (b/'terrain-final.bin').read_bytes()==(b/'terrain-initial.bin').read_bytes():raise AssertionError(('no terrain change',theme))
            paused=[s for s in states if s['session']['paused']]
            if len(paused)!=10:raise AssertionError(('pause',theme,len(paused)))
            cases.append(dict(theme=theme,variant=variant,commands=commands,directory=directory.name,files=sorted(files),states=states,identical_files=len(names)))
    (output/'cases.json').write_text(json.dumps(cases,separators=(',',':'))+'\n')
    report=dict(status='PASS',profiles=len(PROFILES),cases=len(cases),states=sum(len(r['states']) for r in cases),mode_state_inputs_masks_sound_equal=True,scope='invented five-theme courses with original flag/one-way metadata, both directions and dig/Pause inputs, actual SDL pairs; not original-level solutions or performance')
    (output/'summary.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,default=ROOT/'target/release/lemmings-desktop');p.add_argument('--pack',type=Path,default=ROOT/'assets/local/web/hd-assets');p.add_argument('--output',type=Path,required=True);a=p.parse_args();check(a.binary,a.pack,a.output)
