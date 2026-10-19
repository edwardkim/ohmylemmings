#!/usr/bin/env python3
"""Offline SDL toolbar regression: inventories, live terrain minimap and replay."""
import argparse
from contextlib import nullcontext
import json
import os
from pathlib import Path
import subprocess
import tempfile

from check_hazards import collect
from check_desktop import verify_pixels
from state_replay import ROOT, digest


def rows(path):
    return [json.loads(line) for line in path.read_text().splitlines()]


def verify_map(folder):
    config=json.loads((folder/'config.json').read_text())
    width,height=config['width'],config['height']
    terrain=bytearray((folder/'terrain-initial.bin').read_bytes())
    states=rows(folder/'trace.jsonl')
    erased=rows(folder/'digger.jsonl');added=rows(folder/'builder.jsonl');bombers=rows(folder/'bomber.jsonl')
    signatures=set();checked=0
    for state,holes,bricks,bomber in zip(states,erased,added,bombers):
        for i in holes['cleared']:terrain[i]=0
        for i in bricks['filled']:terrain[i]=1
        path=folder/f'frame-{state["control_step"]:06}.rgb'
        if not path.exists():continue
        frame=path.read_bytes()
        # Prefix sums independently implement the DOS >8 horizontal-bit test.
        expected=bytearray(104*20*3)
        for my in range(1,20):
            y=(my+1)*8
            if y>=height:continue
            prefix=[0]
            for value in terrain[y*width:(y+1)*width]:prefix.append(prefix[-1]+value)
            for mx in range(1,104):
                start=(mx-1)*16;end=min(start+16,width)
                if start<width and prefix[end]-prefix[start]>8:
                    expected[(my*104+mx)*3:(my*104+mx)*3+3]=bytes([220,140,70])
        signatures.add(bytes(expected))
        particle_timers={a[0]:a[2] for a in bomber['actors']}
        for a in state['actors']:
            mx=a['x']//16+1;my=a['y']//8-1
            if (a['removed'] is None or particle_timers.get(a['id'],0)>0) and 0<=mx<104 and 0<=my<20:
                expected[(my*104+mx)*3:(my*104+mx)*3+3]=bytes([0,80,240])
        # Fixture camera starts at zero; border is x1..24 in the map.
        for my in range(20):
            for mx in range(1,25):
                if my in [0,19] or mx in [1,24]:
                    expected[(my*104+mx)*3:(my*104+mx)*3+3]=bytes([240,240,240])
        actual=b''.join(frame[(y*320+208)*3:(y*320+312)*3] for y in range(178,198))
        if actual!=bytes(expected):raise AssertionError(f'{path.name}: minimap differs from current terrain/actors')
        checked+=1
    if bytes(terrain)!=(folder/'terrain-final.bin').read_bytes():raise AssertionError('terrain reconstruction differs')
    return checked,len(signatures)


def check(binary,root):
    report=[]
    for name,flags,script in [
        ('digger',['--synthetic'],'80 0 AssignDigger 0\n100 0 SetReleaseRate 99\n'),
        ('builder_bomber',['--synthetic-builder'],'80 0 AssignBuilder 0\n150 0 AssignBomber 0\n250 0 Pause\n275 0 Resume\n')]:
        replay=root/f'{name}.txt';replay.write_text(script)
        previous=None
        for every in [1,7]:
            folder=root/f'{name}-{every}'
            subprocess.run([str(binary.resolve()),*flags,'--steps','340','--fast','--render-every',str(every),
                            '--capture',str(folder),'--replay',str(replay)],check=True,capture_output=True,text=True,
                           env=dict(os.environ,SDL_VIDEODRIVER='dummy'),timeout=30)
            states=collect(folder)
            if previous is not None and states!=previous:raise AssertionError('toolbar rendering changed logical state')
            previous=states
            frames,changes=verify_map(folder)
            report.append(dict(case=name,render_every=every,states=len(states),sha256=digest(states),
                               minimap_frames=frames,terrain_images=changes,scaled_captures=verify_pixels(folder)))
    if not any(row['terrain_images']>1 for row in report):raise AssertionError('no visible minimap terrain edit was exercised')
    return dict(status='PASS',runs=report)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary',type=Path,default=ROOT/'target/release/lemmings-desktop')
    parser.add_argument('--evidence',type=Path)
    args=parser.parse_args()
    if args.evidence:
        args.evidence.mkdir()
        context=nullcontext(str(args.evidence))
    else:context=tempfile.TemporaryDirectory(prefix='lemmings-panel-')
    with context as folder:
        report=check(args.binary,Path(folder))
        if args.evidence:(args.evidence/'summary.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps(report))


if __name__=='__main__':main()
