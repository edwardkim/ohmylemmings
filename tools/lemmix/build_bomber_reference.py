#!/usr/bin/env python3
"""Execute pinned Bomber methods and CheckLemmings over the Blocker host."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys
from build_reference import ROOT, SOURCE_SHA
from build_digger_reference import sha


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output',type=Path)
    parser.add_argument('--compiler',type=Path,required=True)
    parser.add_argument('--units',type=Path,required=True)
    args=parser.parse_args()
    if not args.output.resolve().is_relative_to(ROOT/'assets/local'):raise ValueError('local evidence required')
    source=(ROOT/'assets/local/reference-sources/lemmix__src__Game.pas').read_bytes()
    if sha(source)!=SOURCE_SHA:raise ValueError('pinned source differs')
    styles=(ROOT/'assets/local/issue-16-sources/Styles.Base.pas').read_bytes()
    if sha(styles)!='2c68d83660cf0c2d141e7449d20b9d747fe5c3496d31d0551ff9c547656cf2a8':raise ValueError('metadata differs')
    for pattern in [r"Lem\(\$4E70, 'Oh-No-ing'\s*,\s*16, 16, 10,\s*2,\s*8,\s*10,\s*TLemmingAnimationType.Once\)",r"Lem\(\$50F0, 'Exploding'\s*,\s*1, 32, 32,\s*3,\s*16,\s*25,\s*TLemmingAnimationType.Once\)"]:
        if not re.search(pattern,styles.decode()):raise ValueError('Bomber animation metadata differs')
    text=source.decode('cp1252').replace('\r\n','\n')
    entries=list(re.finditer(r'^(?:class )?(?:function|procedure) TLemmingGame\.(\w+)\b',text,re.M))
    names=['AssignBomber','UpdateExplosionTimer','HandleOhNoing','HandleExploding','ApplyExplosionMask','CheckLemmings']
    bodies={m[1]:text[m.start():entries[i+1].start()].rstrip()+'\n' for i,m in enumerate(entries[:-1]) if m[1] in names}
    if set(bodies)!=set(names):raise ValueError('missing Bomber methods')
    args.output.mkdir();base=args.output/'blocker-base'
    subprocess.run([sys.executable,str(Path(__file__).with_name('build_blocker_reference.py')),str(base),'--compiler',str(args.compiler),'--units',str(args.units)],check=True)
    host=(base/'harness.pas').read_text()
    def change(old,new):
        nonlocal host
        if host.count(old)!=1:raise ValueError('Blocker host drift: '+old[:100])
        host=host.replace(old,new)
    change('type\n  TColor32', '''  ACTION_BIT_FALLING=1 shl 18; ACTION_BIT_OHNOING=1 shl 19;
  ACTION_BIT_EXPLODING=1 shl 20; ACTION_BIT_VAPORIZING=1 shl 21;
  ACTION_BIT_DROWNING=1 shl 22; ACTION_BIT_SPLATTING=1 shl 23; PARTICLE_FRAMECOUNT=52;
type
  TColor32''')
    change('TSkillPanelButton=(Digger,Basher,Miner,Builder,Blocker);','TSkillPanelButton=(Digger,Basher,Miner,Builder,Blocker,Explode);')
    change('    NumberOfBricksLeft,ExplosionTimer,','    ParticleTimer,ParticleFrame:Integer; IsExploded:Boolean;\n    NumberOfBricksLeft,ExplosionTimer,')
    change('    procedure Put(X,Y,V:Integer);','    procedure Put(X,Y,V:Integer);\n    procedure FillRectS(a,b,c,d,v:Integer);')
    change('    procedure DrawTo(Target:TWorld; const D,S:TRect);','    procedure DrawTo(Target:TWorld; const D,S:TRect); overload;\n    procedure DrawTo(Target:TWorld; X,Y:Integer); overload;')
    change('  TLemmingList = class','''  TActorEnumerator=class
    Items:array of TLemming; Index:Integer;
    function MoveNext:Boolean; function GetCurrent:TLemming;
    property Current:TLemming read GetCurrent;
  end;
  TLemmingList = class''')
    change('    function Add(L:TLemming):Integer;','    function Add(L:TLemming):Integer;\n    function IsEmpty:Boolean;\n    function GetEnumerator:TActorEnumerator;')
    change('CurrBlockerCount:Integer;','CurrBlockerCount,CurrBomberCount:Integer;')
    change('    function HandleBlocking(L:TLemming):Boolean;','''    function HandleBlocking(L:TLemming):Boolean;
    function AssignBomber(Lemming1,Lemming2:TLemming):TLemming;
    function UpdateExplosionTimer(L:TLemming):Boolean;
    function HandleOhNoing(L:TLemming):Boolean;
    function HandleExploding(L:TLemming):Boolean;
    procedure ApplyExplosionMask(L:TLemming);
    procedure CheckLemmings;
''')
    change('fTargetBitmap:TWorld;', 'fTargetBitmap:TWorld; ExplodeMaskBmp:TBitmap32;')
    change('GameOptions:record HighlightedPauseButton:Boolean','GameOptions:record HighlightedPauseButton,ShowParticles:Boolean')
    change(' ((Action=TLemmingAction.Shrugging) and (flags and ACTION_BIT_SHRUGGING<>0)) end;', ''' ((Action=TLemmingAction.Shrugging) and (flags and ACTION_BIT_SHRUGGING<>0)) or
 ((Action=TLemmingAction.Falling) and (flags and ACTION_BIT_FALLING<>0)) or
 ((Action=TLemmingAction.Splatting) and (flags and ACTION_BIT_SPLATTING<>0)) or
 ((Action=TLemmingAction.Ohnoing) and (flags and ACTION_BIT_OHNOING<>0)) or
 ((Action=TLemmingAction.Exploding) and (flags and ACTION_BIT_EXPLODING<>0)) or
 ((Action=TLemmingAction.Drowning) and (flags and ACTION_BIT_DROWNING<>0)) or
 ((Action=TLemmingAction.Vaporizing) and (flags and ACTION_BIT_VAPORIZING<>0)) end;''')
    change('{$I reference.inc}', '''procedure TMini.FillRectS(a,b,c,d,v:Integer); begin end;
procedure TBitmap32.DrawTo(Target:TWorld; X,Y:Integer);
begin DrawTo(Target,Rect(X,Y,X+16,Y+22),Rect(0,0,16,22)) end;
function TActorEnumerator.MoveNext:Boolean; begin Inc(Index); Result:=Index<Length(Items) end;
function TActorEnumerator.GetCurrent:TLemming; begin Result:=Items[Index] end;
function TLemmingList.IsEmpty:Boolean; begin Result:=Length(Items)=0 end;
function TLemmingList.GetEnumerator:TActorEnumerator;
begin Result:=TActorEnumerator.Create;Result.Items:=Items;Result.Index:=-1 end;
{$I reference.inc}''')
    change("else removal:='\"BottomBoundary\"';", "else if L.IsExploded then removal:='\"Exploded\"' else removal:='\"BottomBoundary\"';")
    change("TLemmingAction.Blocking:Write('Blocking');", "TLemmingAction.Blocking:Write('Blocking'); TLemmingAction.Ohnoing:Write('OhNo'); TLemmingAction.Exploding:Write('Exploding');")
    change("G.MapPrevious:=Copy(G.fObjectMap.Cells); WriteLn(']}}');", '''G.MapPrevious:=Copy(G.fObjectMap.Cells);
  Write(']},"bomber":{"remaining":',G.CurrBomberCount,',"actors":[');
  for i:=0 to Length(G.fLemmingList.Items)-1 do begin
    if i>0 then Write(','); L:=G.fLemmingList.Items[i];
    Write('[',L.ListIndex,',',L.ExplosionTimer,',',L.ParticleTimer,',',L.ParticleFrame,']') end;
  WriteLn(']}}');''')
    change('if ParamCount<>5 then','if ParamCount<>6 then')
    change('G.CurrBuilderCount,G.CurrBlockerCount,bug);','G.CurrBuilderCount,G.CurrBlockerCount,G.CurrBomberCount,bug);')
    change('    G.fMiniMap:=TMini.Create; G.HyperSpeed:=True;', '''    G.fMiniMap:=TMini.Create; G.HyperSpeed:=True;
    G.ExplodeMaskBmp:=TBitmap32.Create; G.ExplodeMaskBmp.FrameHeight:=22;
    F:=TFileStream.Create(ParamStr(6),fmOpenRead);if F.Size<>44 then raise Exception.Create('explosion mask size');
    F.ReadBuffer(G.ExplodeMaskBmp.Masks[0],44);F.Free;''')
    change('          TLemmingAction.Splatting:FrameCount:=16;', '''          TLemmingAction.Splatting:FrameCount:=16;
          TLemmingAction.Ohnoing:FrameCount:=16;
          TLemmingAction.Exploding:begin FrameCount:=1;FootX:=16;FootY:=25 end;''')
    change('    G.LemmingMethods[TLemmingAction.Blocking]:=G.HandleBlocking;', '''    G.LemmingMethods[TLemmingAction.Blocking]:=G.HandleBlocking;
    G.LemmingMethods[TLemmingAction.Ohnoing]:=G.HandleOhNoing;
    G.LemmingMethods[TLemmingAction.Exploding]:=G.HandleExploding;''')
    change("          else raise Exception.Create('unsupported input');", '''          8:begin accepted:=False;
            if (rate>=0) and (rate<Length(G.fLemmingList.Items)) and not G.fLemmingList.Items[rate].IsRemoved then
              accepted:=G.AssignBomber(G.fLemmingList.Items[rate],nil)<>nil end;
          else raise Exception.Create('unsupported input');''')
    change('''        for i:=0 to Length(G.fLemmingList.Items)-1 do
          if not G.fLemmingList.Items[i].IsRemoved then if G.HandleLemming(G.fLemmingList.Items[i]) then G.CheckForInteractiveObjects(G.fLemmingList.Items[i]);''','        G.CheckLemmings;')
    (args.output/'harness.pas').write_text(host)
    extracted=(base/'reference.inc').read_text()+'\n'+'\n'.join(bodies[k] for k in names)
    (args.output/'reference.inc').write_text(extracted)
    subprocess.run([str(args.compiler.resolve()),'-Mdelphi','-O1','-Fu'+str(args.units.resolve()),'-o'+str((args.output/'reference').resolve()),str((args.output/'harness.pas').resolve())],check=True)
    provenance=json.loads((base/'reference.provenance.json').read_text());provenance['methods'].update({k:sha(v.encode()) for k,v in bodies.items()})
    provenance.update(bomber_builder_sha256=sha(Path(__file__).read_bytes()),generated_sha256=sha(host.encode()),extracted_sha256=sha(extracted.encode()),executable_sha256=sha((args.output/'reference').read_bytes()),scope='43 original functions; six jobs, particles counters, countdown/mask/blocker restore; not full DOS',bomber_adapter='Clipped16x22 MSB-first DrawTo; original CheckLemmings over caller-owned actors; graphics/particle drawing and unrelated object effects excluded')
    (args.output/'reference.provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')

if __name__=='__main__':main()
