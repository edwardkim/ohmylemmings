#!/usr/bin/env python3
"""Execute pinned Floater methods and its original motion table over Climber."""
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
    if not re.search(r"'Umbrella'\s*,\s*8, 16, 16,\s*3,\s*8,\s*16,\s*TLemmingAnimationType\.Loop",styles.decode()):raise ValueError('umbrella metadata differs')
    text=source.decode('cp1252').replace('\r\n','\n');entries=list(re.finditer(r'^(?:class )?(?:function|procedure) TLemmingGame\.(\w+)\b',text,re.M))
    names=['AssignFloater','HandleFloating']
    bodies={m[1]:text[m.start():entries[i+1].start()].rstrip()+'\n' for i,m in enumerate(entries[:-1]) if m[1] in names}
    if set(bodies)!=set(names):raise ValueError('missing methods')
    table=re.search(r'FloatParametersTable: array\[0\.\.15\] of TFloatParameterRec = \([\s\S]*?\n  \);',text)[0]
    values=[(int(a),int(b)) for a,b in re.findall(r'\(Dy:\s*(-?\d+); AnimationFrameIndex:\s*(\d+)\)',table)]
    if values!=[(3,1),(3,2),(3,3),(3,5),(-1,5),(0,5),(1,5),(1,5),(2,5),(2,6),(2,7),(2,7),(2,6),(2,5),(2,4),(2,4)]:raise ValueError('float table differs')
    args.output.mkdir();base=args.output/'climber-base'
    subprocess.run([sys.executable,str(Path(__file__).with_name('build_climber_reference.py')),str(base),'--compiler',str(args.compiler),'--units',str(args.units)],check=True)
    host=(base/'harness.pas').read_text()
    def change(old,new):
        nonlocal host
        if host.count(old)!=1:raise ValueError('Climber host drift: '+old[:100])
        host=host.replace(old,new)
    change('type\n  TColor32','type\n  TFloatParameterRec=record Dy,AnimationFrameIndex:Integer end;\nconst\n  '+table+'\ntype\n  TColor32')
    change('COMBINE_FLAG_CLIMBER=2;', 'COMBINE_FLAG_FLOATER=4; COMBINE_FLAG_CLIMBER=2;')
    change('Blocker,Explode,Climber);','Blocker,Explode,Climber,Umbrella);')
    change('CurrBlockerCount,CurrBomberCount,CurrClimberCount:Integer;','CurrBlockerCount,CurrBomberCount,CurrClimberCount,CurrFloaterCount:Integer;')
    change('    function HandleClimbing(L:TLemming):Boolean;', '''    function AssignFloater(Lemming1,Lemming2:TLemming):TLemming;
    function HandleFloating(L:TLemming):Boolean;
    function HandleClimbing(L:TLemming):Boolean;''')
    change('SFX_DROWNING,SFX_VAPORIZING:Integer','SFX_DROWNING,SFX_VAPORIZING,SFX_OPENUMBRELLA:Integer')
    change("TLemmingAction.Climbing:Write('Climbing');", "TLemmingAction.Floating:Write('Floating'); TLemmingAction.Climbing:Write('Climbing');")
    change("  WriteLn(']}}');", '''  Write(']},"floater":{"remaining":',G.CurrFloaterCount,',"actors":[');
  for i:=0 to Length(G.fLemmingList.Items)-1 do begin
    if i>0 then Write(','); L:=G.fLemmingList.Items[i];
    Write('[',L.ListIndex,',',LowerCase(BoolToStr(L.IsFloater,True)),',',L.FloatParametersTableIndex,']') end;
  WriteLn(']}}');''')
    change('G.CurrClimberCount,bug);','G.CurrClimberCount,G.CurrFloaterCount,bug);')
    change('          TLemmingAction.Shrugging:FrameCount:=8;', '''          TLemmingAction.Floating:begin FrameCount:=8;FootY:=16;AnimationType:=TLemmingAnimationType.Loop end;
          TLemmingAction.Shrugging:FrameCount:=8;''')
    change('    G.LemmingMethods[TLemmingAction.Climbing]:=G.HandleClimbing;', '''    G.LemmingMethods[TLemmingAction.Floating]:=G.HandleFloating;
    G.LemmingMethods[TLemmingAction.Climbing]:=G.HandleClimbing;''')
    change("          else raise Exception.Create('unsupported input');", '''          10:begin accepted:=False;
            if (rate>=0) and (rate<Length(G.fLemmingList.Items)) and not G.fLemmingList.Items[rate].IsRemoved then
              accepted:=G.AssignFloater(G.fLemmingList.Items[rate],nil)<>nil end;
          else raise Exception.Create('unsupported input');''')
    (args.output/'harness.pas').write_text(host)
    extracted=(base/'reference.inc').read_text()+'\n'+'\n'.join(bodies[k] for k in names)
    (args.output/'reference.inc').write_text(extracted)
    subprocess.run([str(args.compiler.resolve()),'-Mdelphi','-O1','-Fu'+str(args.units.resolve()),'-o'+str((args.output/'reference').resolve()),str((args.output/'harness.pas').resolve())],check=True)
    provenance=json.loads((base/'reference.provenance.json').read_text());provenance['methods'].update({k:sha(v.encode()) for k,v in bodies.items()})
    provenance.update(floater_builder_sha256=sha(Path(__file__).read_bytes()),float_table_sha256=sha(table.encode()),float_table=values,generated_sha256=sha(host.encode()),extracted_sha256=sha(extracted.encode()),executable_sha256=sha((args.output/'reference').read_bytes()),scope='48 original functions plus original FloatParametersTable; eight jobs and both permanent abilities; not full DOS')
    (args.output/'reference.provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')

if __name__=='__main__':main()
