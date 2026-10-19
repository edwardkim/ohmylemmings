#!/usr/bin/env python3
"""Validate original Fun01 success, Nuke loss and time expiry through native replay."""
import argparse
import hashlib
import json
from pathlib import Path
import time
from check_completion import collect,reference_run,run_native
from check_nuke import replay_text
from check_skill_interactions import compare_states
from state_replay import ROOT,digest,verified_assets
from verify_native_view import png_from_ppm


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('output',type=Path);p.add_argument('--reference',type=Path,required=True)
    p.add_argument('--binary',type=Path,default=ROOT/'target/release/lemmings-desktop')
    p.add_argument('--assets',type=Path,default=ROOT/'assets/local/dos-original')
    args=p.parse_args();args.output=args.output.resolve()
    if not args.output.is_relative_to(ROOT/'assets/local'):raise ValueError('local original evidence required')
    verified_assets(args.assets);args.output.mkdir()
    signature=hashlib.sha256(args.binary.read_bytes()).hexdigest();rows=[]
    profiles=[('win',[(60,0,'AssignDigger',0)],2000,'Won','PopulationExhausted'),
              ('loss',[(120,0,'Nuke',0)],2000,'Lost','PopulationExhausted'),
              ('timeout',[],6000,'Lost','TimeExpired')]
    for name,inputs,steps,phase,reason in profiles:
        replay=args.output/(name+'.txt');replay.write_text(replay_text(inputs));hashes=[]
        modes=[('single',1,True),('sparse',7,True)]
        if name=='win':modes += [('realtime',1,False)]
        for mode,render,fast in modes:
            directory=args.output/(name+'-'+mode);start=time.monotonic()
            states,captures=run_native(args.binary,directory,replay,render,assets=args.assets,fast=fast,steps=steps)
            config=json.loads((directory/'config.json').read_text()) | dict(steps=steps)
            expected=reference_run(args.reference,config,directory,inputs)
            (directory/'reference.json').write_text(json.dumps(expected))
            compare_states(states,expected,name)
            final=states[-1]['completion']
            if (final['phase'],final['reason'])!=(phase,reason):raise AssertionError('original result not exercised')
            if name=='win' and final['rescued']!=10:raise AssertionError('original full rescue missing')
            if name=='loss' and final['lemming_count']!=3:raise AssertionError('early Nuke denominator missing')
            last=states[-1]['control_step'];ppm=directory/f'frame-{last:06}.ppm'
            png_from_ppm(ppm)
            row=dict(name=directory.name,snapshots=len(states),sha256=digest(states),pixel_exact_captures=captures,
                final_control=last,final_iteration=states[-1]['iteration'],final=final,inputs=inputs,elapsed_seconds=round(time.monotonic()-start,3))
            rows.append(row);hashes.append(row['sha256']);print(json.dumps(row),flush=True)
        if len(set(hashes))!=1:raise AssertionError('render or wall-time dependent result')
    verified_assets(args.assets)
    if hashlib.sha256(args.binary.read_bytes()).hexdigest()!=signature:raise AssertionError('binary changed during verification')
    summary=dict(status='PASS',runs=rows,binary_sha256=signature,reference_sha256=hashlib.sha256(args.reference.read_bytes()).hexdigest(),
        scope='original Fun01 full rescue, Nuke loss, timeout; not original result artwork or full DOS')
    (args.output/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')


if __name__=='__main__':main()
