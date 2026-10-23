#!/usr/bin/env python3
"""Pair actual SDL original/HD hazard courses; write fixtures for WASM replay."""
import argparse,json,os
from pathlib import Path
import subprocess
try:
    from .hazard_fixture import bank,PROFILES
except ImportError:
    import sys
    sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
    from tools.hazard_fixture import bank,PROFILES
ROOT=Path(__file__).resolve().parents[1]

def check(binary,pack,output):
    output=output.resolve()
    if not output.is_relative_to(ROOT/'assets/local') or output.exists():raise ValueError('use a fresh directory under assets/local')
    output.mkdir(parents=True);cases=[]
    for theme,id,count,width,height,effect,trigger in PROFILES:
        name=f'hazard-{theme}-{id}';directory=output/name;directory.mkdir();files=bank(theme,id)
        for filename,data in files.items():(directory/filename).write_bytes(data)
        captures=[]
        for mode in ['original','hd']:
            capture=output/f'native-{theme}-{id}-{mode}';captures.append(capture)
            args=[str(binary.resolve()),'--assets',str(directory),'--level','1','--pacing','dos','--steps','300','--render-every','300','--fast','--capture',str(capture)]
            if mode=='hd':args+=['--hd','--hd-pack',str(pack.resolve())]
            with (output/f'{name}-{mode}.log').open('w') as log:subprocess.run(args,cwd=ROOT,env=dict(os.environ,SDL_VIDEODRIVER='dummy'),stdout=log,stderr=subprocess.STDOUT,check=True)
        a,b=captures;names=[p.name for p in a.iterdir() if p.suffix=='.jsonl' or p.name in ['terrain-final.bin','brick-final.bin','object-map-final.bin','inputs.txt']]
        for filename in names:
            if (a/filename).read_bytes()!=(b/filename).read_bytes():raise AssertionError((theme,id,filename))
        states=[json.loads(s) for s in (b/'full.jsonl').read_text().splitlines()]
        deaths=[json.loads(s) for s in (b/'hazards.jsonl').read_text().splitlines()]
        final=deaths[-1];expected='drowned' if effect==5 else 'vaporized'
        if final[expected]!=1 or final['trapped']!=0 or states[-1]['exit']['saved']!=0:raise AssertionError((name,final))
        cases.append(dict(theme=theme,id=id,count=count,effect=effect,directory=directory.name,files=sorted(files),states=states,death=final,identical_files=len(names)))
    (output/'cases.json').write_text(json.dumps(cases,separators=(',',':'))+'\n')
    report=dict(status='PASS',profiles=len(cases),states=sum(len(r['states']) for r in cases),drowned=sum(r['death']['drowned'] for r in cases),vaporized=sum(r['death']['vaporized'] for r in cases),mode_state_inputs_masks_equal=True,scope='invented courses with original effect5/6 metadata; actual SDL pairs, not original-level solutions or performance')
    (output/'summary.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,default=ROOT/'target/release/lemmings-desktop');p.add_argument('--pack',type=Path,default=ROOT/'assets/local/hd-runtime/atlas');p.add_argument('--output',type=Path,required=True);a=p.parse_args();check(a.binary,a.pack,a.output)
