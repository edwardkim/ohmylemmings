#!/usr/bin/env python3
"""Verify real SDL toolbar clicks, resize coordinates and replay on isolated X11."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time

from check_hazards import collect
from check_desktop import verify_pixels
from state_replay import ROOT, digest
from verify_native_view import png_from_ppm


def check(binary, xdotool, root, assets=None):
    root.mkdir()
    capture = root/'live'
    flags = ['--assets', str(assets)] if assets else ['--synthetic']
    process = subprocess.Popen([str(binary), *flags, '--steps', '1000', '--capture', str(capture)],
                               text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               env=dict(os.environ, SDL_VIDEODRIVER='x11'), cwd=ROOT)
    events = []
    def command(*words):
        events.append(list(words))
        return subprocess.check_output([str(xdotool), *map(str, words)], text=True, timeout=10).strip()
    def latest():
        frames = sorted(capture.glob('frame-*.rgb'))
        ready = [f for f in frames if f.stat().st_size == 320*200*3 and f.with_suffix('.ppm').exists()]
        return ready[-1] if ready else None
    def wait(after=-1):
        deadline = time.monotonic()+15
        while process.poll() is None and time.monotonic() < deadline:
            frame = latest()
            if frame and int(frame.stem.split('-')[1]) >= after+1:
                return frame
            time.sleep(.03)
        if process.poll() is not None:
            out,err=process.communicate(timeout=5)
            raise AssertionError(f'SDL exited {process.returncode}: {err}')
        raise AssertionError('SDL capture boundary not reached')
    def step(frame):
        return int(frame.stem.split('-')[1])
    def pointer(x,y):
        command('mousemove', '--window', window, offset[0]+x*scale, offset[1]+y*scale)
    def click(x,y):
        prior=step(wait())
        pointer(x,y)
        command('click','1')
        time.sleep(.12)
        return wait(prior)
    try:
        wait(78)
        windows=command('search','--onlyvisible','--pid',process.pid,'--name','Oh My Lemmings').splitlines()
        if len(windows)!=1:
            raise AssertionError('one own SDL window required')
        window=windows[0]
        command('windowfocus','--sync',window)
        scale,offset=3,(0,0)
        click(164,190)  # pause
        white=bytes([240,208,208] if assets else [240,240,240])
        selections=[]
        for i in range(8):
            frame=click(36+i*16,190)
            at=(176*320+32+i*16)*3
            if frame.read_bytes()[at:at+3]!=white:
                raise AssertionError(f'skill button {i} not selected')
            selections.append(frame.name)
        # Digger selected via panel; use the observed actor position, not a guessed time.
        trace=[json.loads(line) for line in (capture/'trace.jsonl').read_text().splitlines() if line.endswith('}')]
        actor=next(a for a in trace[-1]['actors'] if a['removed'] is None)
        camera=624 if assets else 0
        click(actor['x']-camera,actor['y']-5)
        # Hold rate across logical boundaries while paused; then release.
        pointer(20,190);command('mousedown','1');time.sleep(.4);command('mouseup','1')
        wait(step(latest()))
        # Resize to a letterboxed 1000×700 surface (scale3, offset20/50).
        command('windowsize',window,1000,700)
        time.sleep(.2)
        scale,offset=3,(20,50)
        frame=click(36,190)
        if frame.read_bytes()[(176*320+32)*3:(176*320+32)*3+3]!=white:
            raise AssertionError('resized/letterboxed logical button coordinates differ')
        click(310,190)
        click(208,190)
        click(164,190)  # resume
        time.sleep(.6)
        single_nuke_end_step=step(click(180,190))  # single Nuke must not trigger
        time.sleep(.6)
        pointer(180,190);command('click','--repeat','2','--delay','120','1')
        frame=wait(step(latest()))
        pointer(200,170)
        command('key','Escape')
        stdout,stderr=process.communicate(timeout=10)
        if process.returncode:
            raise AssertionError(stderr)
    finally:
        if process.poll() is None:
            process.terminate();process.communicate(timeout=10)
    summary=json.loads(stdout)
    inputs=(capture/'inputs.txt').read_text()
    if inputs.count('Nuke')!=1 or inputs.count('AssignDigger')!=1 or 'SetReleaseRate' not in inputs:
        raise AssertionError('panel did not produce the expected shared simulation commands')
    nuke_step=next(int(line.split()[0]) for line in inputs.splitlines() if line.split()[2]=='Nuke')
    if nuke_step<=single_nuke_end_step:raise AssertionError('single click triggered Nuke')
    if inputs.count('Pause')!=1 or inputs.count('Resume')!=1 or summary['camera']!=0:
        raise AssertionError('pause or minimap inputs differ')
    live=collect(capture)
    captures=verify_pixels(capture)
    replays=[]
    for every in [1,7]:
        folder=root/f'replay-{every}'
        subprocess.run([str(binary),*flags,'--fast','--steps',str(summary['control_step']),
                        '--render-every',str(every),'--replay',str(capture/'inputs.txt'),'--capture',str(folder)],
                       env=dict(os.environ,SDL_VIDEODRIVER='dummy'),cwd=ROOT,check=True,capture_output=True,text=True,timeout=90)
        replay=collect(folder)
        if live!=replay:
            raise AssertionError('real panel input differs from independent replay')
        captures+=verify_pixels(folder)
        replays.append(dict(render_every=every,states=len(replay),sha256=digest(replay)))
    for name in [selections[0],selections[-1],frame.name]:
        ppm=(capture/name).with_suffix('.ppm')
        if not ppm.with_suffix('.png').exists():png_from_ppm(ppm)
    report=dict(status='PASS',original=bool(assets),runtime=summary,selected_frames=selections,
                inputs=inputs,events=events,single_nuke_end_step=single_nuke_end_step,nuke_step=nuke_step,live_states=len(live),replays=replays,pixel_exact_captures=captures)
    (root/'summary.json').write_text(json.dumps(report,indent=2)+'\n')
    return report


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary',type=Path,default=ROOT/'target/release/lemmings-desktop')
    parser.add_argument('--xdotool',type=Path,required=True)
    parser.add_argument('--evidence',type=Path,required=True)
    parser.add_argument('--assets',type=Path)
    args=parser.parse_args()
    root=args.evidence.resolve()
    if not root.is_relative_to(ROOT/'assets/local'):
        raise ValueError('local evidence required')
    report=check(args.binary.resolve(),args.xdotool.resolve(),root,args.assets.resolve() if args.assets else None)
    print(json.dumps({k:report[k] for k in ['status','original','live_states','replays','pixel_exact_captures']}))


if __name__=='__main__':
    main()
