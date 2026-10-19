#!/usr/bin/env python3
"""Execute pinned Blocker/object-map methods over the unchanged Builder host."""
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
    if not args.output.resolve().is_relative_to(ROOT/'assets/local'):
        raise ValueError('reference evidence must stay in assets/local')
    source=(ROOT/'assets/local/reference-sources/lemmix__src__Game.pas').read_bytes()
    if sha(source)!=SOURCE_SHA:raise ValueError('pinned Game.pas differs')
    styles=(ROOT/'assets/local/issue-16-sources/Styles.Base.pas').read_bytes()
    if sha(styles)!='2c68d83660cf0c2d141e7449d20b9d747fe5c3496d31d0551ff9c547656cf2a8':
        raise ValueError('pinned metadata differs')
    if not re.search(r"Lem\(\$4970, 'Blocking'\s*,\s*16, 16, 10,\s*2,\s*8,\s*10,\s*TLemmingAnimationType.Loop\)",styles.decode()):
        raise ValueError('Blocking metadata differs')
    text=source.decode('cp1252').replace('\r\n','\n')
    entries=list(re.finditer(r'^(?:class )?(?:function|procedure) TLemmingGame\.(\w+)\b',text,re.M))
    names=['ReadObjectMap','WriteObjectMap','SaveMap','RestoreMap','SetBlockerField',
           'CheckForOverlappingField','AssignBlocker','HandleBlocking','CheckForInteractiveObjects']
    bodies={m[1]:text[m.start():entries[i+1].start()].rstrip()+'\n' for i,m in enumerate(entries[:-1]) if m[1] in names}
    if set(bodies)!=set(names):raise ValueError('missing Blocker methods')
    args.output.mkdir()
    base=args.output/'builder-base'
    subprocess.run([sys.executable,str(Path(__file__).with_name('build_builder_reference.py')),str(base),
                    '--compiler',str(args.compiler),'--units',str(args.units)],check=True)
    host=(base/'harness.pas').read_text()
    def change(old,new):
        nonlocal host
        if host.count(old)!=1:raise ValueError('Builder host drift: '+old[:90])
        host=host.replace(old,new)
    change('DOM_NONE = 0;', 'DOM_NONE = 128; DOM_EXIT=129; DOM_FORCELEFT=130; DOM_FORCERIGHT=131; DOM_WATER=133; DOM_FIRE=134; DOM_BLOCKER=138; OBJMAPADD=4;')
    change('EntranceX25, MinerOneWayRightBug);','EntranceX25, MinerOneWayRightBug, TriggeredTrapLemmixBugSolved);')
    change('TSkillPanelButton=(Digger,Basher,Miner,Builder);','TSkillPanelButton=(Digger,Basher,Miner,Builder,Blocker);')
    change('Bashing, Shrugging);','Bashing, Shrugging, Drowning, Vaporizing);')
    host=host.replace('array[0..15]','array[0..17]')
    change('    NumberOfBricksLeft,ExplosionTimer,','    SavedMap:array[0..8] of Byte;\n    NumberOfBricksLeft,ExplosionTimer,')
    change('  TInteractiveObjectInfo = class','''  TByteArray=array[0..1000000] of Byte;
  TByteArrayPointer=^TByteArray;
  TByteMap=class
    Width,Height:Integer; Cells:array of Byte;
    function GetBits:TByteArrayPointer;
    property Bits:TByteArrayPointer read GetBits;
  end;
  TInteractiveObjectInfo = class
    MetaObj:record AnimationFrameCount,SoundEffect:Integer end;''')
    change('    CurrDiggerCount,CurrBasherCount,CurrMinerCount,CurrBuilderCount:Integer;',
           '    CurrDiggerCount,CurrBasherCount,CurrMinerCount,CurrBuilderCount,CurrBlockerCount:Integer;')
    change('    Steel,Previous:array of Byte;','    Steel,Previous,MapPrevious:array of Byte;\n    fObjectMap:TByteMap; ObjectInfos:array[0..127] of TInteractiveObjectInfo;')
    change('    function HandleShrugging(L:TLemming):Boolean;','''    function HandleShrugging(L:TLemming):Boolean;
    function HandleBlocking(L:TLemming):Boolean;
    function AssignBlocker(Lemming1,Lemming2:TLemming):TLemming;
    procedure WriteObjectMap(X,Y:Integer; aValue:Byte);
    procedure RestoreMap(L:TLemming);
    function CheckForOverlappingField(L:TLemming):Boolean;
    procedure CheckForInteractiveObjects(L:TLemming);
    function GetTrapSoundIndex(v:Integer):Integer;''')
    change('SFX_BUILDER_WARNING:Integer','SFX_BUILDER_WARNING,SFX_DROWNING,SFX_VAPORIZING:Integer')
    change("procedure TLemmingGame.SaveMap(L:TLemming); begin raise Exception.Create('blockers unsupported') end;",'')
    change("procedure TLemmingGame.SetBlockerField(L:TLemming); begin raise Exception.Create('blockers unsupported') end;",'')
    start=host.index('function TLemmingGame.ReadObjectMap(X,Y:Integer):Byte;')
    end=host.index('function TBitmap32.CalcFrameRect',start)
    host=host[:start]+host[end:]
    change('{$I reference.inc}', '''function TByteMap.GetBits:TByteArrayPointer; begin Result:=TByteArrayPointer(@Cells[0]) end;
function TLemmingGame.GetTrapSoundIndex(v:Integer):Integer; begin raise Exception.Create('traps unsupported') end;
{$I reference.inc}''')
    change('var i:Integer; L:TLemming; removal:String; H:QWord; First:Boolean;',
           'var i,j:Integer; L:TLemming; removal:String; H:QWord; First:Boolean;')
    change("TLemmingAction.Shrugging:Write('Shrugging');", "TLemmingAction.Shrugging:Write('Shrugging'); TLemmingAction.Blocking:Write('Blocking');")
    change("  WriteLn(']}}');", '''  Write(']},"blocker":{"remaining":',G.CurrBlockerCount,',"fields":['); First:=True;
  for i:=0 to Length(G.fLemmingList.Items)-1 do begin
    L:=G.fLemmingList.Items[i];
    if L.IsBlocking then begin
      if not First then Write(','); First:=False; Write('[',L.ListIndex,',[');
      for j:=0 to 8 do begin if j>0 then Write(','); Write(L.SavedMap[j]) end; Write(']]') end end;
  H:=$cbf29ce484222325;
  for i:=0 to Length(G.fObjectMap.Cells)-1 do H:=(H xor G.fObjectMap.Cells[i])*QWord($100000001b3);
  Write('],"objects":[');
  for i:=0 to Length(G.fLemmingList.Items)-1 do begin
    if i>0 then Write(','); L:=G.fLemmingList.Items[i]; Write('[',L.ListIndex,',',L.ObjectBelow,',',L.ObjectInFront,']') end;
  Write('],"object_map_fnv1a64":"',LowerCase(IntToHex(H,16)),'","changed":['); First:=True;
  if step<>0 then for i:=0 to Length(G.fObjectMap.Cells)-1 do if G.MapPrevious[i]<>G.fObjectMap.Cells[i] then begin
    if not First then Write(','); First:=False; Write('[',i,',',G.fObjectMap.Cells[i],']') end;
  G.MapPrevious:=Copy(G.fObjectMap.Cells); WriteLn(']}}');''')
    change('if ParamCount<>4 then','if ParamCount<>5 then')
    change('ReadLn(G.CurrDiggerCount,G.CurrBasherCount,G.CurrMinerCount,G.CurrBuilderCount,bug);',
           'ReadLn(G.CurrDiggerCount,G.CurrBasherCount,G.CurrMinerCount,G.CurrBuilderCount,G.CurrBlockerCount,bug);')
    change('    G.fLemmingList:=TLemmingList.Create; G.fRecorder:=TRecorder.Create;', '''    G.fLemmingList:=TLemmingList.Create; G.fRecorder:=TRecorder.Create;
    G.fObjectMap:=TByteMap.Create; G.fObjectMap.Width:=415; G.fObjectMap.Height:=47;
    SetLength(G.fObjectMap.Cells,415*47);
    F:=TFileStream.Create(ParamStr(5),fmOpenRead);
    if F.Size<>415*47 then raise Exception.Create('object map size');
    F.ReadBuffer(G.fObjectMap.Cells[0],415*47); F.Free;''')
    change('          TLemmingAction.Splatting:FrameCount:=16;', '''          TLemmingAction.Splatting:FrameCount:=16;
          TLemmingAction.Blocking:begin FrameCount:=16; AnimationType:=TLemmingAnimationType.Loop end;''')
    change('    G.LemmingMethods[TLemmingAction.Shrugging]:=G.HandleShrugging;', '''    G.LemmingMethods[TLemmingAction.Shrugging]:=G.HandleShrugging;
    G.LemmingMethods[TLemmingAction.Blocking]:=G.HandleBlocking;''')
    change("          else raise Exception.Create('unsupported input');", '''          7:begin accepted:=False;
            if (rate>=0) and (rate<Length(G.fLemmingList.Items)) and not G.fLemmingList.Items[rate].IsRemoved then
              accepted:=G.AssignBlocker(G.fLemmingList.Items[rate],nil)<>nil end;
          else raise Exception.Create('unsupported input');''')
    change('if not G.fLemmingList.Items[i].IsRemoved then G.HandleLemming(G.fLemmingList.Items[i]);',
           'if not G.fLemmingList.Items[i].IsRemoved then if G.HandleLemming(G.fLemmingList.Items[i]) then G.CheckForInteractiveObjects(G.fLemmingList.Items[i]);')
    # Blocker mutates the map: retain the original cached below/front values
    # instead of the legacy static-map assignment refresh adapter.
    host=re.sub(r'^.*G\.fLemmingList\.Items\[rate\]\.Object(?:Below|InFront):=G\.ReadObjectMap.*;\n','',host,flags=re.M)
    (args.output/'harness.pas').write_text(host)
    extracted=(base/'reference.inc').read_text()+'\n'+'\n'.join(bodies[k] for k in names)
    (args.output/'reference.inc').write_text(extracted)
    subprocess.run([str(args.compiler.resolve()),'-Mdelphi','-O1','-Fu'+str(args.units.resolve()),
                    '-o'+str((args.output/'reference').resolve()),str((args.output/'harness.pas').resolve())],check=True)
    provenance=json.loads((base/'reference.provenance.json').read_text())
    provenance['methods'].update({k:sha(v.encode()) for k,v in bodies.items()})
    provenance.update(blocker_builder_sha256=sha(Path(__file__).read_bytes()),generated_sha256=sha(host.encode()),
        extracted_sha256=sha(extracted.encode()),executable_sha256=sha((args.output/'reference').read_bytes()),
        scope='37 original functions; five jobs, raw four-pixel object map, force effects; not full DOS',
        blocker_adapter='Raw415x47 map at(-16,-16); original read/write/save/restore/overlap and interactive checks; traps/exit/water/fire are outside exercised scope')
    (args.output/'reference.provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')


if __name__=='__main__':main()
