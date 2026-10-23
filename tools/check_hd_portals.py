#!/usr/bin/env python3
"""Run original/HD SDL portal courses; preserve fresh evidence for WASM replay."""
import argparse,json,os
from pathlib import Path
import subprocess
try:
    from .portal_fixture import bank
except ImportError:
    import sys
    sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
    from tools.portal_fixture import bank
ROOT=Path(__file__).resolve().parents[1]

def check(binary,pack,output,baseline=None):
    output=output.resolve()
    if not output.is_relative_to(ROOT/'assets/local') or output.exists():raise ValueError('use a new evidence directory under assets/local')
    output.mkdir(parents=True);cases=[]
    for theme in range(5):
        directory=output/f'theme-{theme}';directory.mkdir();files=bank(theme)
        for name,data in files.items():(directory/name).write_bytes(data)
        captures=[]
        for mode in ['original','hd']+(['baseline'] if baseline else []):
            capture=output/f'native-{theme}-{mode}';captures.append(capture)
            args=[str(binary.resolve()),'--assets',str(directory),'--level','1','--pacing','dos','--steps','300','--render-every','300','--fast','--capture',str(capture)]
            if mode!='original':args+=['--hd','--hd-pack',str((pack if mode=='hd' else baseline).resolve())]
            with (output/f'native-{theme}-{mode}.log').open('w') as log:subprocess.run(args,cwd=ROOT,env=dict(os.environ,SDL_VIDEODRIVER='dummy'),stdout=log,stderr=subprocess.STDOUT,check=True)
        a,b=captures[:2];names=[p.name for p in a.iterdir() if p.suffix=='.jsonl' or p.name in ['terrain-final.bin','brick-final.bin','object-map-final.bin','inputs.txt']]
        for name in names:
            if (a/name).read_bytes()!=(b/name).read_bytes():raise AssertionError((theme,name))
        if baseline:
            previous=captures[2]
            for path in b.iterdir():
                if path.suffix in ['.ppm','.rgb','.jsonl'] or path.name in names:
                    if path.read_bytes()!=(previous/path.name).read_bytes():raise AssertionError(('HD display changed',theme,path.name))
        states=[json.loads(s) for s in (b/'full.jsonl').read_text().splitlines()]
        final=states[-1]
        if final['completion']['phase']!='Won' or final['exit']['saved']!=1:raise AssertionError((theme,final))
        cases.append(dict(theme=theme,directory=directory.name,files=sorted(files),states=states,identical_files=len(names)))
    (output/'cases.json').write_text(json.dumps(cases,separators=(',',':'))+'\n')
    report=dict(status='PASS',themes=5,states=sum(len(r['states']) for r in cases),saved_per_theme=1,mode_state_inputs_masks_equal=True,baseline_hd_pixels_equal=bool(baseline),scope='invented courses with documented original portal metadata; actual SDL original/HD pairs, not original-level solutions or performance')
    (output/'summary.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,default=ROOT/'target/release/lemmings-desktop');p.add_argument('--pack',type=Path,default=ROOT/'assets/local/hd-runtime/assets.hdp');p.add_argument('--output',type=Path,required=True);p.add_argument('--baseline-pack',type=Path);a=p.parse_args();check(a.binary,a.pack,a.output,a.baseline_pack)
