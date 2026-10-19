#!/usr/bin/env python3
"""Verify original Fun09 fire, Fun17 traps and Fun29 water with actual reference execution."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import time
from check_hazards import collect,reference_run,run_native
from check_skill_interactions import compare_states
from check_desktop import verify_pixels
from state_replay import ROOT,digest,verified_assets
from verify_native_view import png_from_ppm
from verify_nuke import latest_complete_state,parse_ui_inputs

PROFILES=[(9,'vaporized'),(17,'trapped'),(29,'drowned')]


def compare(directory,reference,inputs,steps,target):
    states=collect(directory)
    config=json.loads((directory/'config.json').read_text()) | dict(steps=steps)
    expected=reference_run(reference,config,directory,inputs)
    (directory/'reference.json').write_text(json.dumps(expected))
    compare_states(states,expected,'original '+str(config['level']))
    if states[-1]['hazards'][target]<=0:raise AssertionError('original hazard not reached')
    if target=='trapped':
        active=[s for s in states if any(t[2] for t in s['hazards']['traps'])]
        if not active or states[-1]['hazards'][target]<2:raise AssertionError('no original trap reuse')
    first=next(s['control_step'] for s in states if any(a['action'] in ['Drowning','Vaporizing'] or a['removed']=='Trapped' for a in s['actors']))
    for step in [0,first,first+7,first+14,first+16,steps]:
        ppm=directory/f'frame-{step:06}.ppm'
        if ppm.exists():png_from_ppm(ppm)
    return dict(snapshots=len(states),sha256=digest(states),pixel_exact_captures=verify_pixels(directory),inputs=inputs,first_hazard=first,final=states[-1]['hazards'])


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('output',type=Path);p.add_argument('--reference',type=Path,required=True)
    p.add_argument('--binary',type=Path,default=ROOT/'target/release/lemmings-desktop')
    p.add_argument('--assets',type=Path,default=ROOT/'assets/local/dos-original')
    p.add_argument('--xdotool',type=Path)
    args=p.parse_args();args.output=args.output.resolve()
    if not args.output.is_relative_to(ROOT/'assets/local'):raise ValueError('local original evidence required')
    verified_assets(args.assets);args.output.mkdir()
    signature=hashlib.sha256(args.binary.read_bytes()).hexdigest()
    replay=args.output/'empty.txt';replay.write_text('');rows=[];steps=340
    for level,target in PROFILES:
        hashes=[]
        for mode,render,fast in [('single',1,True),('sparse',7,True),('realtime',1,False)]:
            directory=args.output/f'F{level:02}-{mode}';start=time.monotonic()
            run_native(args.binary,directory,replay,render,steps=steps,assets=args.assets,level=level,fast=fast)
            row=dict(name=directory.name,**compare(directory,args.reference,[],steps,target),elapsed_seconds=round(time.monotonic()-start,3))
            hashes.append(row['sha256']);rows.append(row);print(json.dumps(row),flush=True)
        if len(set(hashes))!=1:raise AssertionError('original state depends on rendering/time')
    if args.xdotool:
        directory=args.output/'F17-keys'
        process=subprocess.Popen([str(args.binary.resolve()),'--assets',str(args.assets),'--level','17','--steps',str(steps),'--capture',str(directory)],text=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        def key():
            windows=subprocess.check_output([str(args.xdotool),'search','--onlyvisible','--pid',str(process.pid),'--name','Oh My Lemmings'],text=True).splitlines()
            if len(windows)!=1:raise ValueError('one own SDL window required')
            subprocess.run([str(args.xdotool),'windowfocus','--sync',windows[0],'key','space'],check=True,capture_output=True)
        try:
            paused=None;resumed=False;end=time.monotonic()+35
            while time.monotonic()<end and process.poll() is None:
                path=directory/'trace.jsonl';s=latest_complete_state(path) if path.exists() else None
                if s and paused is None and s['control_step']>=240:
                    key();paused=s['control_step']
                elif s and paused is not None and s['paused'] and s['control_step']>=paused+12:
                    key();resumed=True;break
                time.sleep(.002)
            stdout,stderr=process.communicate(timeout=25)
            if process.returncode or not resumed:raise ValueError('actual Pause/Resume failed: '+stdout+stderr)
        finally:
            if process.poll() is None:process.kill();process.wait()
        inputs=parse_ui_inputs((directory/'inputs.txt').read_text())
        if [r[2] for r in inputs]!=['Pause','Resume']:raise AssertionError('actual keys not recorded')
        row=dict(name=directory.name,**compare(directory,args.reference,inputs,steps,'trapped'))
        rows.append(row);print(json.dumps(row),flush=True)
    verified_assets(args.assets)
    if hashlib.sha256(args.binary.read_bytes()).hexdigest()!=signature:raise AssertionError('binary changed during verification')
    (args.output/'summary.json').write_text(json.dumps(dict(status='PASS',runs=rows,binary_sha256=signature,reference_sha256=hashlib.sha256(args.reference.read_bytes()).hexdigest(),scope='Fun09 fire/Fun17 traps/Fun29 water; no whole DOS/results certification'),indent=2)+'\n')


if __name__=='__main__':main()
