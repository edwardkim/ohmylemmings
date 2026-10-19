#!/usr/bin/env python3
"""Execute unchanged pinned Climber bodies over the six-job Bomber host."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys
from build_reference import ROOT,SOURCE_SHA
from build_digger_reference import sha


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('output',type=Path);p.add_argument('--compiler',type=Path,required=True);p.add_argument('--units',type=Path,required=True)
    args=p.parse_args()
    if not args.output.resolve().is_relative_to(ROOT/'assets/local'):raise ValueError('local evidence required')
    source=(ROOT/'assets/local/reference-sources/lemmix__src__Game.pas').read_bytes()
    if sha(source)!=SOURCE_SHA:raise ValueError('pinned source differs')
    styles=(ROOT/'assets/local/issue-16-sources/Styles.Base.pas').read_bytes()
    if sha(styles)!='2c68d83660cf0c2d141e7449d20b9d747fe5c3496d31d0551ff9c547656cf2a8':raise ValueError('metadata differs')
    for name,kind in [('Climbing','Loop'),('Hoisting','Once')]:
        if not re.search("'"+name+"'\\s*,\\s*8, 16, 12,\\s*2,\\s*8,\\s*12,\\s*TLemmingAnimationType\\."+kind,styles.decode()):raise ValueError('Climber metadata differs')
    text=source.decode('cp1252').replace('\r\n','\n');entries=list(re.finditer(r'^(?:class )?(?:function|procedure) TLemmingGame\.(\w+)\b',text,re.M))
    names=['AssignClimber','HandleClimbing','HandleHoisting']
    bodies={m[1]:text[m.start():entries[i+1].start()].rstrip()+'\n' for i,m in enumerate(entries[:-1]) if m[1] in names}
    if set(bodies)!=set(names):raise ValueError('missing methods')
    args.output.mkdir();base=args.output/'bomber-base'
    subprocess.run([sys.executable,str(Path(__file__).with_name('build_bomber_reference.py')),str(base),'--compiler',str(args.compiler),'--units',str(args.units)],check=True)
    host=(base/'harness.pas').read_text()
    def change(old,new):
        nonlocal host
        if host.count(old)!=1:raise ValueError('Bomber host drift: '+old[:100])
        host=host.replace(old,new)
    change('COMBINE_FLAG_BUILDER = 1;', 'ACTION_BIT_BLOCKING=1 shl 24; COMBINE_FLAG_CLIMBER=2; COMBINE_FLAG_BUILDER = 1;')
    change(' ((Action=TLemmingAction.Falling) and (flags and ACTION_BIT_FALLING<>0)) or', ' ((Action=TLemmingAction.Blocking) and (flags and ACTION_BIT_BLOCKING<>0)) or\n ((Action=TLemmingAction.Falling) and (flags and ACTION_BIT_FALLING<>0)) or')
    change('Blocker,Explode);','Blocker,Explode,Climber);')
    change('Shrugging, Drowning, Vaporizing);','Shrugging, Drowning, Vaporizing, Hoisting);')
    change('TriggeredTrapLemmixBugSolved);','TriggeredTrapLemmixBugSolved, AssignClimberShruggerActionBug);')
    change('CurrBlockerCount,CurrBomberCount:Integer;','CurrBlockerCount,CurrBomberCount,CurrClimberCount:Integer;')
    change('    function HandleOhNoing(L:TLemming):Boolean;', '''    function AssignClimber(Lemming1,Lemming2:TLemming):TLemming;
    function HandleClimbing(L:TLemming):Boolean;
    function HandleHoisting(L:TLemming):Boolean;
    function HandleOhNoing(L:TLemming):Boolean;''')
    change("TLemmingAction.Ohnoing:Write('OhNo');", "TLemmingAction.Climbing:Write('Climbing'); TLemmingAction.Hoisting:Write('Hoisting'); TLemmingAction.Ohnoing:Write('OhNo');")
    change("  WriteLn(']}}');", '''  Write(']},"climber":{"remaining":',G.CurrClimberCount,',"actors":[');
  for i:=0 to Length(G.fLemmingList.Items)-1 do begin
    if i>0 then Write(','); L:=G.fLemmingList.Items[i];
    Write('[',L.ListIndex,',',LowerCase(BoolToStr(L.IsClimber,True)),',',LowerCase(BoolToStr((L.Action=TLemmingAction.Walking) and (L.AnimationType=TLemmingAnimationType.Once),True)),']') end;
  WriteLn(']}}');''')
    change('G.CurrBlockerCount,G.CurrBomberCount,bug);','G.CurrBlockerCount,G.CurrBomberCount,G.CurrClimberCount,bug); Include(G.fMechanics,TMechanic.AssignClimberShruggerActionBug);')
    change('          TLemmingAction.Shrugging:FrameCount:=8;', '''          TLemmingAction.Climbing:begin FrameCount:=8;FootY:=12;AnimationType:=TLemmingAnimationType.Loop end;
          TLemmingAction.Hoisting:begin FrameCount:=8;FootY:=12 end;
          TLemmingAction.Shrugging:FrameCount:=8;''')
    change('    G.LemmingMethods[TLemmingAction.Ohnoing]:=G.HandleOhNoing;', '''    G.LemmingMethods[TLemmingAction.Climbing]:=G.HandleClimbing;
    G.LemmingMethods[TLemmingAction.Hoisting]:=G.HandleHoisting;
    G.LemmingMethods[TLemmingAction.Ohnoing]:=G.HandleOhNoing;''')
    change("          else raise Exception.Create('unsupported input');", '''          9:begin accepted:=False;
            if (rate>=0) and (rate<Length(G.fLemmingList.Items)) and not G.fLemmingList.Items[rate].IsRemoved then
              accepted:=G.AssignClimber(G.fLemmingList.Items[rate],nil)<>nil end;
          else raise Exception.Create('unsupported input');''')
    (args.output/'harness.pas').write_text(host)
    extracted=(base/'reference.inc').read_text()+'\n'+'\n'.join(bodies[k] for k in names)
    (args.output/'reference.inc').write_text(extracted)
    subprocess.run([str(args.compiler.resolve()),'-Mdelphi','-O1','-Fu'+str(args.units.resolve()),'-o'+str((args.output/'reference').resolve()),str((args.output/'harness.pas').resolve())],check=True)
    provenance=json.loads((base/'reference.provenance.json').read_text());provenance['methods'].update({k:sha(v.encode()) for k,v in bodies.items()})
    provenance.update(climber_builder_sha256=sha(Path(__file__).read_bytes()),generated_sha256=sha(host.encode()),extracted_sha256=sha(extracted.encode()),executable_sha256=sha((args.output/'reference').read_bytes()),scope='46 original functions; seven jobs, permanent climber, climbing/hoisting, original shrugger bug; not full DOS')
    (args.output/'reference.provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')

if __name__=='__main__':main()
