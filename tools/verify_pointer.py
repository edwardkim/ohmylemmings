#!/usr/bin/env python3
"""Drive own SDL window on isolated X11; prove raw right-click and ID replay."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time
from check_hazards import collect
from check_desktop import verify_pixels
from state_replay import ROOT,digest
from verify_native_view import png_from_ppm


def completed(path):
    if not path.exists():return []
    text=path.read_text();return [json.loads(s) for s in text[:text.rfind('\n')+1].splitlines()]


def check(binary,root,trap=False,assets=None):
    root.mkdir();capture=root/'live';events=[]
    flags=['--assets',str(assets),'--level','3'] if assets else ['--synthetic-trap' if trap else '--synthetic-builder']
    process=subprocess.Popen([str(binary.resolve()),*flags,'--steps','1000','--capture',str(capture)],
        env=dict(os.environ,SDL_VIDEODRIVER='x11'),cwd=ROOT,text=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    def command(*args):
        events.append(list(args));return subprocess.check_output(['xdotool',*map(str,args)],text=True,timeout=10).strip()
    def wait(predicate=lambda s,p:True):
        deadline=time.monotonic()+15
        while time.monotonic()<deadline and process.poll() is None:
            states=completed(capture/'trace.jsonl');pointers=completed(capture/'pointer.jsonl')
            if states and pointers and states[-1]['control_step']==pointers[-1]['control_step']:
                if predicate(states[-1],pointers[-1]):return states[-1],pointers[-1]
            time.sleep(.025)
        if process.poll() is not None:
            out,err=process.communicate(timeout=5);raise AssertionError(f'SDL exited: {out} {err}')
        raise AssertionError('expected SDL pointer/state boundary not reached')
    def key(name):command('key',name)
    def move(x,y):command('mousemove','--window',window,offset[0]+x*scale,offset[1]+y*scale)
    def actor_pointer(actor,p):move(actor['x']-p['camera'],actor['y']-5)
    def pause():
        key('space');return wait(lambda s,p:s['paused'])
    try:
        wait(lambda s,p:s['control_step']>= (90 if trap else 100) and len(s['actors'])>=2)
        windows=command('search','--onlyvisible','--pid',process.pid,'--name','Oh My Lemmings').splitlines()
        if len(windows)!=1:raise AssertionError('one own window required')
        window=windows[0];command('windowfocus','--sync',window);scale,offset=3,(0,0)
        s,p=pause();worker_id=0;remote_id=1
        worker=s['actors'][worker_id];key('k' if assets else 'u');actor_pointer(worker,p);command('click','1')
        action='Blocking' if assets else 'Building'
        s,p=wait(lambda s,p:s['actors'][worker_id]['action']==action)
        if trap:
            key('space');wait(lambda s,p:s['iteration']>=110 and s['actors'][remote_id]['action']=='Walking')
            s,p=pause()
        actor_pointer(s['actors'][remote_id],p)
        wait(lambda s,p:p['primary']==remote_id)
        command('mousedown','3')
        if trap:
            key('space');wait(lambda s,p:s['actors'][remote_id]['removed']=='Trapped')
            s,p=pause()
        actor_pointer(s['actors'][worker_id],p)
        s,p=wait(lambda s,p:p['right'] and p['primary'] is None and p['count']>0)
        key('k' if trap or assets else 'e');command('click','1')
        if trap or assets:
            s,p=wait(lambda s,p:s['actors'][remote_id]['action']=='Blocking' and (not trap or s['actors'][remote_id]['removed']=='Trapped'))
        else:
            def armed(s,p):
                rows=completed(capture/'bomber.jsonl')
                return bool(rows) and any(a[0]==remote_id and a[1]>0 for a in rows[-1]['actors'])
            s,p=wait(armed)
        remote_step=s['control_step'];command('mouseup','3')
        s,p=wait(lambda s,p:not p['right'] and p['primary']==worker_id)
        # Resize and click the letterbox; it must not enqueue another assignment.
        command('windowsize',window,1000,700);time.sleep(.15);scale,offset=3,(20,50)
        command('mousemove','--window',window,1,1);command('click','1')
        wait(lambda s,p:p['point'] is not None and p['primary'] is None)
        actor_pointer(s['actors'][worker_id],p);wait(lambda s,p:p['primary']==worker_id)
        key('Escape');stdout,stderr=process.communicate(timeout=10)
        if process.returncode:raise AssertionError(stderr)
    finally:
        if process.poll() is None:process.terminate();process.communicate(timeout=10)
    inputs=(capture/'inputs.txt').read_text()
    expected=f'AssignPointer {"Blocker" if trap or assets else "Bomber"} {remote_id} - true'
    if inputs.count(expected)!=1 or len([s for s in inputs.splitlines() if 'Assign' in s])!=2:
        raise AssertionError('remote ID/flag or letterbox input differs: '+inputs)
    live=collect(capture);frames=verify_pixels(capture);replays=[]
    for every in [1,7]:
        folder=root/f'replay-{every}'
        subprocess.run([str(binary.resolve()),*flags,'--steps',str(len(live)-1),'--fast','--render-every',str(every),
            '--capture',str(folder),'--replay',str(capture/'inputs.txt')],env=dict(os.environ,SDL_VIDEODRIVER='dummy'),
            cwd=ROOT,check=True,text=True,capture_output=True,timeout=30)
        if collect(folder)!=live:raise AssertionError('live/raw pointer and stable-ID replay differ')
        replays.append(dict(render_every=every,captures=verify_pixels(folder)))
    if not assets:
        ppm=max(capture.glob('frame-*.ppm'));png_from_ppm(ppm)
    result=dict(status='PASS',trap=trap,original=bool(assets),states=len(live),sha256=digest(live),
        live_captures=frames,replays=replays,remote_step=remote_step,inputs=inputs,events=events,summary=json.loads(stdout))
    (root/'summary.json').write_text(json.dumps(result,indent=2)+'\n');return result


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('output',type=Path)
    p.add_argument('--binary',type=Path,default=ROOT/'target/release/lemmings-desktop')
    p.add_argument('--trap',action='store_true')
    p.add_argument('--assets',type=Path)
    a=p.parse_args()
    if a.trap and a.assets:raise ValueError('trap observation uses the own synthetic fixture')
    if not a.output.resolve().is_relative_to(ROOT/'assets/local'):raise ValueError('evidence must stay local')
    print(json.dumps(check(a.binary,a.output,a.trap,a.assets)))


if __name__=='__main__':main()
