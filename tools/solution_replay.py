#!/usr/bin/env python3
"""Verify saved valid solutions and losses in actual SDL/WASM, with real restarts.

No solver, success injection or original data is included. Supply an ignored JSON
case list with id, steps, phase and relative inputs; --smoke invents a byte bank.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
try:
    from . import initial_replay as initial
    from .check_desktop import verify_pixels
    from .web_fixture import bank
except ImportError:
    import initial_replay as initial
    from check_desktop import verify_pixels
    from web_fixture import bank
ROOT=initial.ROOT
NO_ARGUMENT={'Pause','Resume','Nuke'}
SKILLS={'Assign'+s for s in ['Climber','Floater','Bomber','Blocker','Builder','Basher','Miner','Digger']}


def parse_inputs(text,steps):
    if type(steps) is not int or not 1<=steps<=20000:raise ValueError('invalid step bound')
    rows=[];previous=(-1,-1)
    for line in text.splitlines():
        words=line.split();name=words[2] if len(words)>=3 else ''
        if name not in NO_ARGUMENT|SKILLS|{'SetReleaseRate'}:raise ValueError('unknown game command')
        if len(words)!=(3 if name in NO_ARGUMENT else 4):raise ValueError('invalid command arity')
        numbers=words[:2]+words[3:]
        if any(not re.fullmatch(r'0|[1-9][0-9]*',n) for n in numbers):raise ValueError('invalid unsigned input')
        step,seq=map(int,words[:2]);value=int(words[3]) if len(words)==4 else 0
        wanted=previous[1]+1 if step==previous[0] else 0
        if not 0<=step<steps or (step,seq)<=previous or seq!=wanted or value>2**32-1:raise ValueError('invalid input order/boundary')
        if name=='SetReleaseRate' and value>99:raise ValueError('invalid release rate')
        rows.append([step,seq,name,value]);previous=(step,seq)
    canonical=''.join(f'{s} {q} {n}'+(f' {v}' if n not in NO_ARGUMENT else '')+'\n' for s,q,n,v in rows)
    if text!=canonical:raise ValueError('input text must use canonical LF records')
    return rows


def outcome(info,states,phase):
    if phase not in ['Won','Lost'] or not states:raise ValueError('missing expected terminal outcome')
    for row in states:
        c=row['completion']
        if c['original_total']!=info['total'] or c['required']!=info['required']:raise ValueError('original counters changed')
    final=states[-1]['completion']
    if final['phase']!=phase or not final.get('reason'):raise ValueError('level did not reach expected result')
    if (final['rescued']>=info['required'])!=(phase=='Won'):raise ValueError('original rescue goal not satisfied')
    return final


def validate_restart(expected,actual):
    difference=initial.first_difference(expected,actual)
    if difference:raise ValueError('restart initial state/maps differ: '+difference)


def restart_native(root,assets,case,replay,expected):
    """Exercise actual result action; compare reset and a fresh no-input prefix.

    The product deliberately discards the old replay on restart. This check
    verifies that contract rather than silently applying its commands again.
    """
    rank,number=case['id'].split('-');reports=[];prefix=None
    for render in [37,101,257]:
        dest=root/f'restart-r{render}'
        command=[str(ROOT/'target/release/lemmings-desktop'),'--assets',str(assets),'--difficulty',rank,'--level',str(int(number)),
                 '--steps',str(case['steps']),'--fast','--render-every',str(render),'--capture',str(dest),
                 '--replay',str(replay),'--result-actions','restart']
        result=subprocess.run(command,check=True,capture_output=True,text=True,env=dict(os.environ,SDL_VIDEODRIVER='dummy'),timeout=180)
        transitions=initial.jsonl(dest/'transitions.jsonl')
        if len(transitions)!=1 or any(transitions[0].get(k)!=v for k,v in dict(action='Restart',target=int(number),target_difficulty=rank,attempt=0).items()):raise ValueError('wrong SDL restart target/count')
        reset=dest/'attempt-0001'
        actual=[initial.sha((reset/'full.jsonl').read_text().splitlines()[0].encode()),initial.sha((reset/'terrain-initial.bin').read_bytes()),
                initial.sha(bytes(len((reset/'terrain-initial.bin').read_bytes()))),initial.sha((reset/'object-map-initial.bin').read_bytes())]
        # The v1 string field order is preserved by json.loads/dumps.
        validate_restart(expected['rows'][0],actual)
        for name in ['asset-info.json','config.json','objects.bin','traps.bin','terrain-initial.bin','object-map-initial.bin']:
            if (dest/name).read_bytes()!=(reset/name).read_bytes():raise ValueError('restart changed immutable asset/config: '+name)
        if (reset/'inputs.txt').read_text():raise ValueError('old replay leaked across SDL restart')
        first=initial.jsonl(dest/'full.jsonl');outcome(expected['initial']['info'],first,case['phase'])
        if (dest/'inputs.txt').read_text()!=replay.read_text():raise ValueError('SDL restart run did not consume full input')
        observed=[initial.sha(s.encode()) for s in (dest/'full.jsonl').read_text().splitlines()]
        validate_restart([r[0] for r in expected['rows'][:len(observed)]],observed)
        if any((reset/'brick-final.bin').read_bytes()):raise ValueError('fresh no-input restart retained bricks')
        # All three independent reset attempts must follow the same no-input path.
        rows=initial.jsonl(reset/'full.jsonl')
        current=rows[:min(len(rows),341)]
        if prefix is not None:validate_restart(prefix,current)
        prefix=current;frames=sum(verify_pixels(d) for d in [dest,reset])
        files=[initial.file_identity(p) for p in sorted(dest.rglob('*')) if p.is_file()]
        # Raw readbacks are verified then removed; retain their identities.
        for p in dest.rglob('frame-*'):
            if p.suffix in ['.rgb','.ppm']:p.unlink()
        reports.append(dict(render=render,transitions=transitions,reset_probes=len(rows),prefix_sha256=initial.sha(initial.canonical(current).encode()),
                            readbacks=frames,summary=json.loads(result.stdout),files=files))
        print(f'SDL {case["id"]} {case["phase"]} restart r{render}: PASS',flush=True)
    return reports


def case_run(directory,assets,case,inputs):
    key=case['id']+'-'+case['phase'].lower();root=directory/'native'/key;root.mkdir(parents=True)
    replay=root/'inputs.txt';replay.write_text(inputs);commands=parse_inputs(inputs,case['steps'])
    report=initial.native_case(root,assets,case,case['steps'],replay)
    path=root/(case['id']+'.json');expected=json.loads(path.read_text())
    final=outcome(expected['initial']['info'],initial.jsonl(root/(case['id']+'-r1')/'full.jsonl'),case['phase'])
    expected.update(commands=commands,restart=True,phase=case['phase'],final=final)
    (directory/(key+'.json')).write_text(json.dumps(expected,separators=(',',':'))+'\n')
    resets=restart_native(root,assets,case,replay,expected)
    return dict(key=key,id=case['id'],phase=case['phase'],steps=case['steps'],input=initial.file_identity(replay),final=final,
                native=report,restarts=resets,expected=initial.file_identity(directory/(key+'.json')))


def run(directory,assets=None,cases_path=None,workers=3):
    directory=directory.resolve()
    if not directory.is_relative_to(ROOT/'assets/local') or directory.exists():raise ValueError('new ignored evidence directory required')
    directory.mkdir(parents=True);(directory/'native').mkdir()
    originals=initial.verified_assets(assets) if assets else None
    if assets:
        if not cases_path or not cases_path.resolve().is_relative_to(ROOT/'assets/local'):raise ValueError('original case list must remain ignored')
        cases=json.loads(cases_path.read_text());inputs=[]
        for case in cases:
            path=(cases_path.parent/case['inputs']).resolve()
            if not path.is_relative_to(ROOT/'assets/local'):raise ValueError('ignored input required')
            inputs.append(path.read_bytes().decode())
    else:
        assets=directory/'fixture';assets.mkdir()
        for name,data in bank(win=True).items():(assets/name).write_bytes(data)
        cases=[dict(id='fun-01',phase='Won',steps=90),dict(id='fun-01',phase='Lost',steps=220)]
        inputs=['16 0 Pause\n20 0 Resume\n','16 0 Nuke\n']
    if not cases:raise ValueError('empty case list')
    keys=set()
    for case,text in zip(cases,inputs):
        if not re.fullmatch(r'(fun|tricky|taxing|mayhem)-(0[1-9]|[12][0-9]|30)',case['id']) or case['phase'] not in ['Won','Lost']:raise ValueError('invalid level/result case')
        key=(case['id'],case['phase'])
        if key in keys:raise ValueError('duplicate level/result case')
        keys.add(key);parse_inputs(text,case['steps'])
    ids={c['id'] for c in cases}
    if keys!={(i,p) for i in ids for p in ['Won','Lost']}:raise ValueError('each level requires success and failure inputs')
    subprocess.run(['cargo','build','-p','lemmings-desktop','--release','--locked','--offline'],cwd=ROOT,check=True)
    sources=[initial.file_identity(p) for base in ['apps/desktop/src','apps/web/src','crates/lemmings-game/src','crates/lemmings-session/src','crates/lemmings-core/src','crates/lemmings-resources/src'] for p in sorted((ROOT/base).rglob('*.rs'))]
    sources += [initial.file_identity(ROOT/p) for p in ['tools/solution_replay.py','tools/initial_replay.py','tools/web/initial_replay.js','apps/web/www/host.js','Cargo.lock','rust-toolchain.toml','Makefile']]
    provenance=dict(head=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),desktop=initial.file_identity(ROOT/'target/release/lemmings-desktop'),
                    sources=sources,python=initial.sys.version,os=initial.platform.platform(),rust=subprocess.check_output(['rustc','--version'],text=True).strip(),
                    cases=initial.file_identity(cases_path) if cases_path else None,
                    original_files=originals,variants=initial.VARIANTS,pacing='dos-vga-5',profile='release')
    (directory/'provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')
    reports=[]
    with ThreadPoolExecutor(max_workers=workers) as pool:
        futures=[pool.submit(case_run,directory,assets,c,t) for c,t in zip(cases,inputs)]
        for future in as_completed(futures):
            reports.append(future.result());(directory/'native-progress.json').write_text(json.dumps(reports,indent=2)+'\n')
    files=sorted(p.name for p in assets.iterdir() if re.fullmatch(r'(MAIN|ODDTABLE|ADLIB|LEVEL00[0-9]|GROUND[0-4]O|VGAGR[0-4]|VGASPEC[0-3])\.DAT|VGALEMMI\.EXE',p.name))
    (directory/'files').mkdir()
    for name in files:shutil.copyfile(assets/name,directory/'files'/name)
    web=initial.browser_check(directory,sorted(r['key'] for r in reports),files,0)
    if originals!= (initial.verified_assets(assets) if originals else None):raise ValueError('original files changed')
    if provenance['desktop']!=initial.file_identity(ROOT/'target/release/lemmings-desktop'):raise ValueError('desktop changed during run')
    provenance['wasm']=initial.file_identity(directory/'pkg/lemmings_web_bg.wasm')
    (directory/'provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')
    result=dict(status='PASS',original=bool(originals),levels=len(ids),cases=reports,web=web,
                scope='Saved inputs, actual goals/results, SDL/WASM repetition and actual restart; host equality is not a new DOS oracle. SDL discards replay on restart; WASM replays it after restart.')
    (directory/'summary.json').write_text(json.dumps(result,indent=2)+'\n')
    print('Solution replay PASS:',len(ids),'levels',flush=True);return result


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--assets',type=Path);p.add_argument('--cases',type=Path);p.add_argument('--evidence',type=Path);p.add_argument('--smoke',action='store_true');p.add_argument('--workers',type=int,default=3);a=p.parse_args()
    if not 1<=a.workers<=4:p.error('--workers must be1..4')
    if bool(a.assets)==bool(a.smoke) or bool(a.assets)!=bool(a.cases):p.error('choose --smoke or --assets with --cases')
    if a.assets and not a.evidence:p.error('original replay requires ignored --evidence')
    if a.evidence:run(a.evidence,a.assets.resolve() if a.assets else None,a.cases,a.workers)
    else:
        with tempfile.TemporaryDirectory(prefix='solution-smoke-',dir=ROOT/'assets/local') as d:run(Path(d)/'evidence',workers=a.workers)

if __name__=='__main__':main()
