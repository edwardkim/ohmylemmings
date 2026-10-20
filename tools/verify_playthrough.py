#!/usr/bin/env python3
"""Drive one own X11 game through win, loss, retry, win and next level; replay each attempt."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time

from check_completion import collect,reference_run
from check_desktop import verify_pixels
from check_results import attempts
from check_skill_interactions import compare_states
from state_replay import ROOT,digest,verified_assets
from verify_native_view import png_from_ppm


def validate_journey(histories,configs,transitions):
    if len(histories)!=4 or len(configs)!=4 or len(transitions)!=3:
        raise ValueError('four attempts and three transitions required')
    if [c.get('level') for c in configs]!=[1,1,1,2] or configs[0]!=configs[1] or configs[0]!=configs[2]:
        raise ValueError('restart configuration changed')
    for i,history in enumerate(histories):
        if not history or history[0].get('control_step')!=0 or history[0].get('iteration')!=0 or history[0].get('actors')!=[]:
            raise ValueError('attempt did not start fresh')
        if history[0].get('paused') is not False or history[0].get('released')!=0 or history[0]['nuke']['is_nuked']:
            raise ValueError('held input or previous population leaked')
        if i<3 and history[0]!=histories[0][0]:raise ValueError('restart initial state differs')
        born=next((s['actors'] for s in history if s['actors']),[])
        if not born or born[0].get('id')!=0:raise ValueError('new attempt did not restart actor IDs')
    phases=[h[-1]['completion']['phase'] for h in histories]
    if phases!=['Won','Lost','Won','Playing']:raise ValueError('wrong journey outcomes')
    for i,(action,target) in enumerate([('Restart',1),('Continue',1),('Continue',2)]):
        expected=dict(attempt=i,level=1,step=histories[i][-1]['control_step'],action=action,target=target)
        if transitions[i]!=expected:raise ValueError('transition differs from frozen result')
    for i in [0,2]:
        if histories[i][-1]['completion']['rescued_percent']!=100 or histories[i][-1]['nuke']['is_nuked']:
            raise ValueError('complete rescue without Nuke required')
    loss=histories[1][-1]
    if not loss['nuke']['is_nuked'] or loss['nuke']['max_lemmings']>=configs[1]['total']:
        raise ValueError('early Nuke failure not exercised')
    return [True,False,True]


def completed(path):
    if not path.exists():return []
    text=path.read_text();return [json.loads(s) for s in text[:text.rfind('\n')+1].splitlines()]


class Driver:
    def __init__(self,binary,root,assets):
        self.root=root;self.assets=assets;self.events=[];self.window=None
        flags=['--assets',str(assets)] if assets else ['--synthetic-outcome','win']
        self.process=subprocess.Popen([str(binary.resolve()),*flags,'--capture',str(root)],
            env=dict(os.environ,SDL_VIDEODRIVER='x11'),cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        self.scale=3;self.offset=(0,0);self.folder=root
    def command(self,*words):
        latest=completed(self.folder/'trace.jsonl')
        self.events.append(dict(attempt=self.folder.name,control_step=latest[-1]['control_step'] if latest else None,command=list(words)))
        return subprocess.check_output(['xdotool',*map(str,words)],text=True,timeout=10).strip()
    def wait(self,predicate=lambda s,p:True,timeout=90):
        end=time.monotonic()+timeout
        while time.monotonic()<end and self.process.poll() is None:
            states=completed(self.folder/'trace.jsonl');points=completed(self.folder/'pointer.jsonl')
            if states and points and states[-1]['control_step']==points[-1]['control_step'] and predicate(states[-1],points[-1]):
                return states[-1],points[-1]
            time.sleep(.035)
        if self.process.poll() is not None:
            out,err=self.process.communicate(timeout=5);raise AssertionError(f'SDL exited early: {out} {err}')
        raise AssertionError('journey control boundary not reached: '+str(self.folder))
    def key(self,key):self.command('key',key)
    def move(self,x,y):self.command('mousemove','--window',self.window,self.offset[0]+x*self.scale,self.offset[1]+y*self.scale)
    def click(self,x,y):
        step=self.wait()[0]['control_step'];self.move(x,y);self.command('click','1')
        return self.wait(lambda s,p:s['control_step']>step and p['point']==[x,y])
    def result(self):
        def ready(s,p):
            rows=completed(self.folder/'completion.jsonl')
            if not rows or rows[-1]['phase']=='Playing':return False
            frame=self.folder/f'frame-{s["control_step"]:06}.rgb'
            return frame.exists() and frame.stat().st_size==320*200*3 and frame.read_bytes()[:3]==bytes([12,18,32])
        result=self.wait(ready,timeout=130);time.sleep(.1);return result
    def change(self,index,action):
        if action=='click':self.move(160,130);self.command('click','1')
        else:self.key(action)
        self.folder=self.root/f'attempt-{index:04}'
        return self.wait(lambda s,p:len(s['actors'])==0)
    def win(self,toolbar):
        s,p=self.wait(lambda s,p:bool(s['actors']) and s['actors'][0]['action']=='Walking' and s['iteration']>=(60 if self.assets else 75))
        s,p=self.click(148 if self.assets else 36,190)
        a=s['actors'][0];self.click(a['x']-p['camera'],a['y']-5)
        skill='Digging' if self.assets else None
        self.wait(lambda s,p:s['actors'][0]['action']==skill if skill else p['primary']==0 and p['caption'].startswith('CLIMBER'))
        if toolbar:
            s,p=self.click(164,190);self.wait(lambda s,p:s['paused'])
            self.move(20,190);self.command('mousedown','1');time.sleep(.3);self.command('mouseup','1')
            s,p=self.wait();camera=p['camera']
            self.move(310,190);self.command('mousedown','1');self.wait(lambda s,p:p['camera']>0)
            self.move(208,190);self.wait(lambda s,p:p['camera']==0)
            back=208+(camera+176)//16
            self.move(back,190);self.wait(lambda s,p:p['camera']==camera);self.command('mouseup','1')
            self.command('windowsize',self.window,1000,700);time.sleep(.15);self.scale,self.offset=3,(20,50)
            # Physical letterbox coordinates must not assign or select a button.
            self.command('mousemove','--window',self.window,1,1);self.command('click','1')
            self.wait(lambda s,p:p['point'] is not None and p['point'][0]<0)
            self.click(164,190);self.wait(lambda s,p:not s['paused'])
        return self.result()
    def run(self):
        try:
            self.wait(lambda s,p:s['control_step']>=5)
            windows=self.command('search','--onlyvisible','--pid',self.process.pid,'--name','Oh My Lemmings').splitlines()
            if len(windows)!=1:raise AssertionError('one own game window required')
            self.window=windows[0];self.command('windowfocus','--sync',self.window)
            self.win(True);print('live attempt 0 won',flush=True)
            self.change(1,'r')
            if self.assets:self.wait(lambda s,p:s['iteration']>=120)
            else:
                s,p=self.wait(lambda s,p:bool(s['actors']) and s['actors'][0]['action']=='Walking')
                s,p=self.click(148,190);a=s['actors'][0];self.click(a['x']-p['camera'],a['y']-5)
                self.wait(lambda s,p:s['actors'][0]['action']=='Digging')
            single=self.click(180,190)[0]['control_step']
            single=self.wait(lambda s,p:s['control_step']>=single+2)[0]['control_step']
            time.sleep(.6);self.move(180,190);self.command('click','--repeat','2','--delay','120','1')
            self.result();print('live attempt 1 lost',flush=True)
            self.change(2,'click');self.win(False);print('live attempt 2 won',flush=True)
            self.change(3,'Return');self.wait(lambda s,p:s['control_step']>=80)
            self.key('Escape');stdout,stderr=self.process.communicate(timeout=10)
            if self.process.returncode:raise AssertionError(stderr)
            return json.loads(stdout),self.events,single
        finally:
            if self.process.poll() is None:self.process.terminate();self.process.communicate(timeout=10)


def check(args):
    args.output.mkdir();live=args.output/'live'
    if args.assets:verified_assets(args.assets)
    (args.output/'controller-at-run.py').write_bytes(Path(__file__).read_bytes())
    summary,events,single=Driver(args.binary,live,args.assets).run()
    (args.output/'live-run.json').write_text(json.dumps(dict(summary=summary,events=events,single_nuke_end=single),indent=2)+'\n')
    folders=attempts(live);histories=[collect(f) for f in folders]
    configs=[json.loads((f/'config.json').read_text()) for f in folders]
    transitions=completed(live/'transitions.jsonl');validate_journey(histories,configs,transitions)
    if [t['target'] for t in transitions]!=[1,1,2]:raise AssertionError('wrong target level')
    reports=[]
    for index,(folder,states,config) in enumerate(zip(folders,histories,configs)):
        inputs=(folder/'inputs.txt').read_text();rows=inputs.splitlines()
        if index==1:
            nuke=[int(row.split()[0]) for row in rows if row.split()[2]=='Nuke']
            if len(nuke)!=1 or nuke[0]<=single:raise AssertionError('single/double Nuke boundary differs')
        if index==0 and not all(word in inputs for word in ['Pause','Resume','SetReleaseRate','Assign']):
            raise AssertionError('combined toolbar inputs absent')
        if index==3 and inputs:raise AssertionError('game input leaked into next level')
        if index<3 and (folder/'terrain-initial.bin').read_bytes()!=(folders[0]/'terrain-initial.bin').read_bytes():
            raise AssertionError('terrain did not reset')
        first_pointer=completed(folder/'pointer.jsonl')[0]
        if first_pointer['point'] is not None or first_pointer['right']:raise AssertionError('pointer memory leaked')
        frames=verify_pixels(folder);replays=[]
        for render,fast in [(1,True),(7,True),(13,index!=0)]:
            dest=args.output/f'attempt-{index:04}-replay-{render}'
            flags=['--assets',str(args.assets),'--level',str(config['level'])] if args.assets else ['--synthetic-outcome','win' if index<3 else 'loss']
            command=[str(args.binary.resolve()),*flags,'--steps',str(states[-1]['control_step']),'--render-every',str(render),
                     '--capture',str(dest),'--replay',str(folder/'inputs.txt')]
            if fast:command+=['--fast']
            result=subprocess.run(command,env=dict(os.environ,SDL_VIDEODRIVER='dummy'),cwd=ROOT,check=True,capture_output=True,text=True,timeout=150)
            compare_states(collect(dest),states,f'attempt {index} render {render}')
            replays.append(dict(render=render,fast=fast,states=len(states),captures=verify_pixels(dest),summary=json.loads(result.stdout)))
        reference=False
        if args.reference:
            parsed=[]
            for row in rows:
                w=row.split();parsed.append((int(w[0]),int(w[1]),w[2],int(w[3]) if len(w)==4 else 0))
            expected=reference_run(args.reference,dict(config,steps=states[-1]['control_step']),folder,parsed)
            (folder/'reference.json').write_text(json.dumps(expected)+'\n')
            compare_states(states,expected,f'attempt {index} reference');reference=True
        frame=folder/f'frame-{states[-1]["control_step"]:06}.ppm'
        if not frame.exists():frame=max(folder.glob('frame-*.ppm'))
        if not args.assets:png_from_ppm(frame)
        reports.append(dict(attempt=index,level=config['level'],states=len(states),sha256=digest(states),
            final=states[-1]['completion'],inputs=inputs,live_captures=frames,replays=replays,reference_executed=reference))
        print(json.dumps(reports[-1]),flush=True)
    report=dict(status='PASS',original=bool(args.assets),summary=summary,attempts=reports,events=events,transitions=transitions,
        scope='native actual SDL journey and three independent ID replays; optional unchanged reference with DOS completion boundary adapter; not whole DOS')
    (args.output/'summary.json').write_text(json.dumps(report,indent=2)+'\n')
    if args.assets:verified_assets(args.assets)
    return report


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('output',type=Path);p.add_argument('--assets',type=Path)
    p.add_argument('--reference',type=Path)
    p.add_argument('--binary',type=Path,default=ROOT/'target/release/lemmings-desktop')
    a=p.parse_args()
    if not a.output.resolve().is_relative_to(ROOT/'assets/local'):raise ValueError('local evidence required')
    check(a)


if __name__=='__main__':main()
