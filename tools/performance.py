#!/usr/bin/env python3
"""Reproducible G11-06 performance inputs, statistics and artifact checks."""
import hashlib
import json
import math
from pathlib import Path
import statistics

ROOT = Path(__file__).resolve().parents[1]


def summarize(values, operations):
    if type(operations) is not int or operations <= 0 or len(values) != 30:
        raise ValueError('exactly 30 samples and positive operation count required')
    if any(type(v) not in (int, float) or not math.isfinite(v) or v <= 0 for v in values):
        raise ValueError('sample times must be finite and positive')
    ordered = sorted(v / operations for v in values)
    median = statistics.median(ordered)
    return dict(samples=30, operations_per_sample=operations, median_ns_per_op=median,
                p95_ns_per_op=ordered[28], max_ns_per_op=ordered[-1],
                operations_per_second=1e9/median, raw_ns=values)


def validate_samples(rows, operations):
    if len(rows) != 33:
        raise ValueError('three warmups plus thirty measured samples required')
    if len({r['digest'] for r in rows}) != 1 or not rows[0]['digest']:
        raise ValueError('output changed between identical samples')
    if any(r['operations'] != operations or type(r['ns']) not in (float,int)
           or not math.isfinite(r['ns']) or r['ns'] <= 0 for r in rows):
        raise ValueError('work/time changed or invalid')
    return [r['ns'] for r in rows[3:]]


def budgets(refresh, rate, quantum):
    if any(not math.isfinite(v) or v <= 0 for v in [refresh,rate,quantum]):
        raise ValueError('positive finite rates required')
    return dict(dos_tick_ns=1e9*156250/2190197, reference_tick_ns=1e9/17,
                display_frame_ns=1e9/refresh, audio_quantum_ns=1e9*quantum/rate)


def select_cases():
    catalog=json.loads((ROOT/'assets/manifests/dos_compatibility.json').read_text())['levels']
    inputs={}
    for name in ['solution_replay','solution_replay_fun','solution_replay_tricky','solution_replay_taxing','solution_replay_mayhem']:
        for c in json.loads((ROOT/'assets/manifests'/f'{name}.json').read_text())['cases']:
            if c['phase']=='Won':inputs[c['id']]=c
    proof={c['id']:c for c in map(json.loads,(ROOT/'assets/manifests/compatibility_replays.jsonl').read_text().splitlines()) if c['phase']=='Won'}
    ids={'fun-01','fun-09','fun-12','fun-22','tricky-14','taxing-15','mayhem-22'}
    ids.update(next(r['id'] for r in catalog if r['graphics']==style and not r['special']) for style in range(5))
    ids.update(sorted(inputs,key=lambda k:len(inputs[k]['input_text'].splitlines()),reverse=True)[:2])
    result=[]
    for id in sorted(ids):
        c=inputs[id];final=proof[id]['control_step'];steps=final if id=='fun-01' else min(340,final-1)
        start=0
        if id in {'fun-05','fun-22','tricky-12','tricky-14','taxing-15','mayhem-22'} and final>1200:
            start=500;steps=700
        result.append(dict(id=id,phase='Won',start=start,steps=steps,final_control=final,
                           input_text=c['input_text'],input_sha256=c['input_sha256']))
    return result


def prepare(output):
    output=Path(output);output.mkdir(parents=True,exist_ok=True);cases=select_cases();lines=[]
    for c in cases:
        path=output/(c['id']+'.txt');path.write_text(c['input_text'])
        rank,number=c['id'].split('-');lines.append(f'{rank} {int(number)} {c["start"]} {c["steps"]} {path.resolve()}\n')
    (output/'cases.tsv').write_text(''.join(lines));(output/'cases.json').write_text(json.dumps(cases,indent=2)+'\n')
    return cases



def smoke():
    import os
    import subprocess
    import tempfile
    import time
    with tempfile.TemporaryDirectory(prefix='lemmings-perf-') as directory:
        root=Path(directory);plan=root/'cases.tsv';plan.write_text('synthetic 1 0 32 -\n')
        subprocess.run(['cargo','build','-p','lemmings-game','--example','performance_probe','--release','--locked','--offline'],cwd=ROOT,check=True)
        raw=subprocess.check_output([str(ROOT/'target/release/examples/performance_probe'),'synthetic',str(plan),str(root/'probe'),'smoke'],cwd=ROOT,text=True)
        rows=[json.loads(line) for line in raw.splitlines()]
        if len(rows)!=6:raise ValueError('missing smoke stages')
        for row in rows:
            values=[dict(ns=r[0],operations=r[1],digest=r[2]) for r in row['samples']]
            summarize(validate_samples(values,row['operations']),row['operations'])
        captures=[]
        for capture in [False,True]:
            command=[str(ROOT/'target/release/lemmings-desktop'),'--synthetic-builder','--steps','64','--render-every','7','--fast']
            folder=root/'capture'
            if capture:command+=['--capture',str(folder)]
            start=time.perf_counter_ns();result=subprocess.check_output(command,text=True,cwd=ROOT,env=dict(os.environ,SDL_VIDEODRIVER='dummy'));wall=time.perf_counter_ns()-start
            info=json.loads(result);assert info['control_step']==64 and info['timing']=='fixed-step'
            files=[p for p in folder.rglob('*') if p.is_file()] if capture else []
            captures.append(dict(capture=capture,wall_ns=wall,frames=info['frames'],files=len(files),bytes=sum(p.stat().st_size for p in files)))
        return dict(status='PASS',performance_smoke=True,rows=6,warmups=3,samples=30,capture_profile=captures,
                    scope='original-free harness checks; capture contrast includes extra renders/readback/serialization/I/O; not game FPS')


if __name__ == '__main__':
    import argparse
    parser=argparse.ArgumentParser(description=__doc__)
    group=parser.add_mutually_exclusive_group(required=True)
    group.add_argument('--prepare',type=Path);group.add_argument('--smoke',action='store_true')
    args=parser.parse_args()
    print(json.dumps(smoke() if args.smoke else dict(cases=len(prepare(args.prepare)))))
