#!/usr/bin/env python3
"""Execute pinned drowning, vaporizing and interactive object updates over the exit host."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys
from build_reference import ROOT, SOURCE_SHA
from build_digger_reference import sha


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('output', type=Path)
    p.add_argument('--compiler', type=Path, required=True)
    p.add_argument('--units', type=Path, required=True)
    args = p.parse_args()
    if not args.output.resolve().is_relative_to(ROOT / 'assets/local'):
        raise ValueError('local evidence required')
    source = (ROOT / 'assets/local/reference-sources/lemmix__src__Game.pas').read_bytes()
    styles = (ROOT / 'assets/local/issue-16-sources/Styles.Base.pas').read_bytes()
    if sha(source) != SOURCE_SHA or sha(styles) != '2c68d83660cf0c2d141e7449d20b9d747fe5c3496d31d0551ff9c547656cf2a8':
        raise ValueError('pinned source/metadata differs')
    for pattern in [r"'Drowning'\s*,\s*16, 16, 10,\s*2,\s*8,\s*10,\s*TLemmingAnimationType.Once", r"'Vaporizing'\s*,\s*14, 16, 14,\s*4,\s*8,\s*14,\s*TLemmingAnimationType.Once"]:
        if not re.search(pattern, styles.decode()):
            raise ValueError('hazard animation metadata differs')
    source_text = source.decode('cp1252').replace('\r\n', '\n')
    entries = list(re.finditer(r'^(?:class )?(?:function|procedure) TLemmingGame\.(\w+)\b', source_text, re.M))
    names = ['HandleDrowning', 'HandleVaporizing', 'UpdateInteractiveObjects']
    bodies = {m[1]: source_text[m.start():entries[i+1].start()].rstrip()+'\n' for i,m in enumerate(entries[:-1]) if m[1] in names}
    if set(bodies) != set(names): raise ValueError('missing hazard methods')
    args.output.mkdir()
    base = args.output / 'exit-base'
    subprocess.run([sys.executable, str(Path(__file__).with_name('build_exit_reference.py')), str(base), '--compiler', str(args.compiler), '--units', str(args.units)], check=True)
    host = (base / 'harness.pas').read_text()
    def change(old,new):
        nonlocal host
        if host.count(old) != 1: raise ValueError('exit host drift: '+old[:80])
        host = host.replace(old,new)
    change('type\n  TColor32', '  oat_Continuous=0; oat_Triggered=1;\ntype\n  TColor32')
    change('  TInteractiveObjectInfo = class', '  TObjectAnimationType=Integer;\n  TInteractiveObjectInfo = class')
    change('MetaObj:record AnimationFrameCount,SoundEffect:Integer end;', 'MetaObj:record AnimationFrameCount,SoundEffect:Integer; AnimationType:TObjectAnimationType end;')
    change('ObjectInfos:array[0..127]', 'ObjectInfos:array[0..15]')
    change('    function HandleExiting(L:TLemming):Boolean;', '''    function HandleExiting(L:TLemming):Boolean;
    function HandleDrowning(L:TLemming):Boolean;
    function HandleVaporizing(L:TLemming):Boolean;
    procedure UpdateInteractiveObjects;''')
    change('    fMechanics:TMechanics;', '    fEntranceAnimationCompleted:Boolean; TrapEnabled:array[0..15] of Boolean;\n    fMechanics:TMechanics;')
    # Hoisting extended the enum in the earlier host; allocate its last slot too.
    host = host.replace('array[0..17] of TMeta', 'array[0..18] of TMeta').replace('LemmingBitmaps: array[0..17]', 'LemmingBitmaps: array[0..18]')
    change("function TLemmingGame.GetTrapSoundIndex(v:Integer):Integer; begin raise Exception.Create('traps unsupported') end;", 'function TLemmingGame.GetTrapSoundIndex(v:Integer):Integer; begin Result:=0 end;')
    change('    if ParamCount<>6 then', '    if ParamCount<>7 then')
    change('  Inputs:array of record', '  TrapFile:TFileStream; TrapData:array[0..47] of Byte;\n  Inputs:array of record')
    change('    G.currReleaseRate:=initialRate;', '''    Include(G.fMechanics,TMechanic.TriggeredTrapLemmixBugSolved);
    G.fEntranceAnimationCompleted:=True;
    TrapFile:=TFileStream.Create(ParamStr(7),fmOpenRead);
    if TrapFile.Size<>48 then raise Exception.Create('trap metadata size');
    TrapFile.ReadBuffer(TrapData[0],48);TrapFile.Free;
    for i:=0 to 15 do begin
      G.ObjectInfos[i]:=TInteractiveObjectInfo.Create;
      G.TrapEnabled[i]:=TrapData[i*3]>0;
      G.ObjectInfos[i].MetaObj.AnimationFrameCount:=TrapData[i*3];
      if TrapData[i*3]=0 then G.ObjectInfos[i].MetaObj.AnimationFrameCount:=1;
      G.ObjectInfos[i].MetaObj.AnimationType:=oat_Triggered;
      if TrapData[i*3+2]=1 then G.ObjectInfos[i].MetaObj.AnimationType:=oat_Continuous;
      G.ObjectInfos[i].CurrentFrame:=TrapData[i*3+1];
      if G.ObjectInfos[i].CurrentFrame>=G.ObjectInfos[i].MetaObj.AnimationFrameCount then raise Exception.Create('invalid first frame');
    end;
    for i:=0 to Length(G.fObjectMap.Cells)-1 do
      if G.fObjectMap.Cells[i]<128 then
        if (G.fObjectMap.Cells[i]>15) or not G.TrapEnabled[G.fObjectMap.Cells[i]] then raise Exception.Create('missing trap');
    G.currReleaseRate:=initialRate;''')
    change('          TLemmingAction.Exiting:begin FrameCount:=8;FootY:=13 end;', '''          TLemmingAction.Exiting:begin FrameCount:=8;FootY:=13 end;
          TLemmingAction.Drowning:FrameCount:=16;
          TLemmingAction.Vaporizing:begin FrameCount:=14;FootY:=14 end;''')
    change('    G.LemmingMethods[TLemmingAction.Exiting]:=G.HandleExiting;', '''    G.LemmingMethods[TLemmingAction.Exiting]:=G.HandleExiting;
    G.LemmingMethods[TLemmingAction.Drowning]:=G.HandleDrowning;
    G.LemmingMethods[TLemmingAction.Vaporizing]:=G.HandleVaporizing;''')
    change("TLemmingAction.Exiting:Write('Exiting');", "TLemmingAction.Exiting:Write('Exiting'); TLemmingAction.Drowning:Write('Drowning'); TLemmingAction.Vaporizing:Write('Vaporizing');")
    old = next(line for line in host.splitlines() if 'if L.IsRemoved then if L.Action=TLemmingAction.Exiting' in line)
    change(old, '    removal:=RemovalOf(L);')
    change('procedure Snapshot(G:TLemmingGame; step:Integer);', '''function RemovalOf(L:TLemming):String;
begin
  Result:='null'; if not L.IsRemoved then Exit;
  if L.Action=TLemmingAction.Exiting then Result:='"Rescued"'
  else if L.Action=TLemmingAction.Drowning then Result:='"Drowned"'
  else if L.Action=TLemmingAction.Vaporizing then Result:='"Vaporized"'
  else if L.IsExploded then Result:='"Exploded"'
  else if (L.Action=TLemmingAction.Splatting) and L.EndOfAnimation then Result:='"SplattingFinished"'
  else if L.YPos>163 then Result:='"BottomBoundary"'
  else if L.ObjectBelow<16 then Result:='"Trapped"'
  else raise Exception.Create('unclassified removal');
end;
procedure Snapshot(G:TLemmingGame; step:Integer);''')
    change(' H:QWord; First:Boolean;', ' H:QWord; First:Boolean; drowned,vaporized,trapped:Integer;')
    change("  WriteLn(']}}');", '''  Write(']},"hazards":{"traps":[');First:=True;
  for i:=0 to 15 do if G.TrapEnabled[i] then begin
    if not First then Write(',');First:=False;
    Write('[',i,',',G.ObjectInfos[i].CurrentFrame,',',B(G.ObjectInfos[i].Triggered),']') end;
  drowned:=0;vaporized:=0;trapped:=0;
  for i:=0 to Length(G.fLemmingList.Items)-1 do begin
    if RemovalOf(G.fLemmingList.Items[i])='"Drowned"' then Inc(drowned)
    else if RemovalOf(G.fLemmingList.Items[i])='"Vaporized"' then Inc(vaporized)
    else if RemovalOf(G.fLemmingList.Items[i])='"Trapped"' then Inc(trapped);
  end;
  WriteLn('],"drowned":',drowned,',"vaporized":',vaporized,',"trapped":',trapped,'}}');''')
    change('        G.CheckLemmings; G.CheckUpdateNuking;', '        G.CheckLemmings; G.CheckUpdateNuking; G.UpdateInteractiveObjects;')
    (args.output / 'harness.pas').write_text(host)
    extracted = (base / 'reference.inc').read_text()+'\n'+'\n'.join(bodies[name] for name in names)
    (args.output / 'reference.inc').write_text(extracted)
    subprocess.run([str(args.compiler.resolve()), '-Mdelphi', '-O1', '-Fu'+str(args.units.resolve()), '-o'+str((args.output/'reference').resolve()), str((args.output/'harness.pas').resolve())], check=True)
    provenance = json.loads((base/'reference.provenance.json').read_text())
    provenance['methods'].update({name:sha(body.encode()) for name,body in bodies.items()})
    provenance.update(hazards_builder_sha256=sha(Path(__file__).read_bytes()), generated_sha256=sha(host.encode()), extracted_sha256=sha(extracted.encode()), executable_sha256=sha((args.output/'reference').read_bytes()),
        scope='54 original methods/table; hazards/traps/action ordering and all previous jobs/exit; not whole DOS',
        hazards_adapter='original handlers/update unchanged; trap metadata/normalized removal and counter trace; sound stub; entrance update disabled in this host; corrected own Hoisting metadata array capacity')
    (args.output/'reference.provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')


if __name__ == '__main__': main()
