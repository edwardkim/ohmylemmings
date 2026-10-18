#!/usr/bin/env python3
"""Compile pinned original Digger methods using the existing minimal host."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

from build_reference import ROOT, SOURCE_SHA, METHODS


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--units', type=Path, required=True)
    args = parser.parse_args()
    if not args.output.resolve().is_relative_to(ROOT/'assets/local'):
        raise ValueError('reference outputs must stay in assets/local')
    args.output.mkdir()  # no overwritten evidence
    source = (ROOT/'assets/local/reference-sources/lemmix__src__Game.pas').read_bytes()
    if sha(source) != SOURCE_SHA:
        raise ValueError('pinned reference source differs')
    text = source.decode('cp1252').replace('\r\n', '\n')
    styles = (ROOT/'assets/local/issue-16-sources/Styles.Base.pas').read_bytes()
    if sha(styles) != '2c68d83660cf0c2d141e7449d20b9d747fe5c3496d31d0551ff9c547656cf2a8':
        raise ValueError('pinned animation metadata differs')
    if not re.search(r"Lem\(\$02D0, 'Digging'\s*,\s*16, 16, 14,\s*3,\s*8,\s*12,\s*TLemmingAnimationType.Loop\)", styles.decode()):
        raise ValueError('Digger animation metadata differs')
    entries = list(re.finditer(r'^(?:class )?(?:function|procedure) TLemmingGame\.(\w+)\b', text, re.M))
    names = METHODS + ['DigOneRow', 'HandleDigging', 'AssignDigger']
    bodies = {m[1]: text[m.start():entries[i+1].start()].rstrip()+'\n'
              for i, m in enumerate(entries[:-1]) if m[1] in names}
    if set(bodies) != set(names):
        raise ValueError('missing methods')
    method_hashes = {k: sha(v.encode()) for k, v in bodies.items()}
    bodies['IncrementIteration'] = bodies['IncrementIteration'].replace(
        'procedure TLemmingGame.IncrementIteration;\nbegin',
        'procedure TLemmingGame.IncrementIteration;\nvar entrance:TInteractiveObjectInfo;\nbegin').replace(
        'for var entrance: TInteractiveObjectInfo in Entrances do', 'for entrance in Entrances do')
    bodies['BtnPause'] = bodies['BtnPause'].replace(
        'procedure TLemmingGame.BtnPause(mode: TPauseCommandMode = TPauseCommandMode.None);\nbegin',
        'procedure TLemmingGame.BtnPause(mode: TPauseCommandMode = TPauseCommandMode.None);\nvar iteration:Integer;\nbegin').replace(
        'var iteration: Integer := 34 - fRecorder.fRecordedGlitchPauseIterations;',
        'iteration := 34 - fRecorder.fRecordedGlitchPauseIterations;')
    host_path = Path(__file__).with_name('harness.pas')
    host = host_path.read_text()
    def change(old, new):
        nonlocal host
        if host.count(old) != 1:
            raise ValueError('G05 host drift: '+old[:90])
        host = host.replace(old, new)
    change('DOM_NONE = 0; MUSIC_INDEX = 0;', '''DOM_NONE = 0; DOM_STEEL = 137; MUSIC_INDEX = 0;
  ACTION_BIT_WALKING=2; ACTION_BIT_BUILDING=256; ACTION_BIT_MINING=4096;
  ACTION_BIT_SHRUGGING=16384; ACTION_BIT_BASHING=32768;''')
    change('type\n  TLemmingAction', 'type\n  TSkillPanelButton=(Digger);\n  TLemmingAction')
    change('    procedure SetPauseHighlight(v:Boolean);', '    procedure SetPauseHighlight(v:Boolean);\n    procedure DrawSkillCount(b:TSkillPanelButton; v:Integer);')
    change('    Events:String;', '''    Events:String;
    CurrDiggerCount:Integer; HyperSpeed,fAssignmentIsRightClickGlitch:Boolean;
    Steel,Previous:array of Byte;
    fMiniMap:TMini;''')
    change('  TLemmingGame = class', '''  TMini = class
    procedure Put(X,Y,V:Integer);
    property PixelS[X,Y:Integer]:Integer write Put;
  end;
  TLemmingGame = class''')
    change('    function HandleWalking(L:TLemming):Boolean;', '''    function HandleWalking(L:TLemming):Boolean;
    function HandleDigging(L:TLemming):Boolean;
    function DigOneRow(L:TLemming; Y:Integer):Boolean;
    function AssignDigger(Lemming1,Lemming2:TLemming):TLemming;
    function ReadObjectMap(X,Y:Integer):Byte;
    procedure RemovePixelAt(X,Y:Integer);
    procedure RecordSkillAssignment(L:TLemming; A:TLemmingAction; Secondary,Glitch:Boolean);''')
    change('    SFX_SILENTDEATH,SFX_LETSGO,SFX_ENTRANCE:Integer end;', '    SFX_SILENTDEATH,SFX_LETSGO,SFX_ENTRANCE,SFX_DIGGER,SFX_HITS_STEEL:Integer end;')
    change(' ((Action=TLemmingAction.Digging) and (flags and ACTION_BIT_DIGGING<>0)) end;', ''' ((Action=TLemmingAction.Digging) and (flags and ACTION_BIT_DIGGING<>0)) or
 ((Action=TLemmingAction.Walking) and (flags and ACTION_BIT_WALKING<>0)) end;''')
    change('{$I reference.inc}', '''procedure TToolbar.DrawSkillCount(b:TSkillPanelButton; v:Integer); begin end;
procedure TMini.Put(X,Y,V:Integer); begin end;
procedure TLemmingGame.RecordSkillAssignment(L:TLemming; A:TLemmingAction; Secondary,Glitch:Boolean); begin end;
procedure TLemmingGame.RemovePixelAt(X,Y:Integer); begin fWorld.Cells[Y*fWorld.Width+X]:=0 end;
function TLemmingGame.ReadObjectMap(X,Y:Integer):Byte;
begin Result:=DOM_NONE; if (X>=0) and (Y>=0) and (X<fWorld.Width) and (Y<fWorld.Height) then
 if Steel[Y*fWorld.Width+X]<>0 then Result:=DOM_STEEL end;
{$I reference.inc}''')
    change('var i:Integer; L:TLemming; removal:String;', 'var i:Integer; L:TLemming; removal:String; H:QWord; First:Boolean;')
    change("TLemmingAction.Falling:Write('Falling'); TLemmingAction.Splatting:Write('Splatting');", "TLemmingAction.Falling:Write('Falling'); TLemmingAction.Splatting:Write('Splatting'); TLemmingAction.Digging:Write('Digging');")
    change('''  WriteLn('],"events":[',G.Events,']}');''', '''  Write('],"events":[',G.Events,'],"digger":{"remaining":',G.CurrDiggerCount,',"new_diggers":[');
  First:=True;
  for i:=0 to Length(G.fLemmingList.Items)-1 do if G.fLemmingList.Items[i].IsNewDigger and
      (G.fLemmingList.Items[i].Action=TLemmingAction.Digging) then begin
    if not First then Write(','); First:=False; Write(i) end;
  H:=$cbf29ce484222325;
  for i:=0 to Length(G.fWorld.Cells)-1 do H:=(H xor G.fWorld.Cells[i]) * QWord($100000001b3);
  Write('],"terrain_fnv1a64":"',LowerCase(IntToHex(H,16)),'","cleared":[');
  First:=True;
  if step<>0 then for i:=0 to Length(G.fWorld.Cells)-1 do if G.Previous[i]<>G.fWorld.Cells[i] then begin
    if not First then Write(','); First:=False; Write(i) end;
  G.Previous:=Copy(G.fWorld.Cells);
  WriteLn(']}}');''')
    change("    G.currReleaseRate:=initialRate; G.NextLemmingCountdown:=20;", "    G.currReleaseRate:=initialRate; G.NextLemmingCountdown:=20; ReadLn(G.CurrDiggerCount);")
    change("    G.fStyle:=TStyle.Create;", '''    SetLength(G.Steel,Length(G.fWorld.Cells));
    if ParamCount>=2 then begin
      F:=TFileStream.Create(ParamStr(2),fmOpenRead);
      if F.Size<>Length(G.Steel) then raise Exception.Create('steel size');
      F.ReadBuffer(G.Steel[0],F.Size); F.Free end;
    G.fMiniMap:=TMini.Create;
    G.fStyle:=TStyle.Create;''')
    change("    if ParamCount<>1 then", "    if (ParamCount<1) or (ParamCount>2) then")
    change('          TLemmingAction.Splatting:FrameCount:=16;', '''          TLemmingAction.Splatting:FrameCount:=16;
          TLemmingAction.Digging:begin FrameCount:=16; FootY:=12; AnimationType:=TLemmingAnimationType.Loop end;''')
    change('    G.LemmingMethods[TLemmingAction.Splatting]:=G.HandleSplatting;', '    G.LemmingMethods[TLemmingAction.Splatting]:=G.HandleSplatting;\n    G.LemmingMethods[TLemmingAction.Digging]:=G.HandleDigging;')
    change("          else raise Exception.Create('unsupported input');", '''          3:begin accepted:=False;
            if (rate>=0) and (rate<Length(G.fLemmingList.Items)) and not G.fLemmingList.Items[rate].IsRemoved then begin
              G.fLemmingList.Items[rate].ObjectBelow:=G.ReadObjectMap(G.fLemmingList.Items[rate].XPos,G.fLemmingList.Items[rate].YPos);
              accepted:=G.AssignDigger(G.fLemmingList.Items[rate],nil)<>nil end end;
          else raise Exception.Create('unsupported input');''')
    (args.output/'harness.pas').write_text(host)
    (args.output/'reference.inc').write_text('\n'.join(bodies[k] for k in names))
    subprocess.run([str(args.compiler.resolve()), '-Mdelphi', '-O1', '-Fu'+str(args.units.resolve()),
                    '-o'+str((args.output/'reference').resolve()), str((args.output/'harness.pas').resolve())], check=True)
    provenance = dict(metadata_sha256=sha(styles), source_sha256=SOURCE_SHA, methods=method_hashes, scope='walking-only Digger assignment, mutable terrain and existing G05; not full DOS/Lemmix',
                      host_sha256=sha(host_path.read_bytes()), builder_sha256=sha(Path(__file__).read_bytes()),
                      generated_sha256=sha(host.encode()), extracted_sha256=sha((args.output/'reference.inc').read_bytes()),
                      compiler_sha256=sha(args.compiler.read_bytes()), executable_sha256=sha((args.output/'reference').read_bytes()),
                      stubs='toolbar/sound/minimap/recorder; binary caller-provided steel; stable ID target, no secondary cursor candidate',
                      syntax_patches='two G05 inline-variable changes only')
    (args.output/'reference.provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')


if __name__ == '__main__':
    main()
