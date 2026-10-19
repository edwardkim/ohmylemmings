#!/usr/bin/env python3
"""Execute pinned Builder methods over the unchanged Miner reference host."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys

from build_reference import ROOT, SOURCE_SHA
from build_digger_reference import sha


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--units', type=Path, required=True)
    args = parser.parse_args()
    if not args.output.resolve().is_relative_to(ROOT/'assets/local'):
        raise ValueError('reference evidence must stay in assets/local')
    source = (ROOT/'assets/local/reference-sources/lemmix__src__Game.pas').read_bytes()
    if sha(source) != SOURCE_SHA:
        raise ValueError('pinned Game.pas differs')
    styles = (ROOT/'assets/local/issue-16-sources/Styles.Base.pas').read_bytes()
    if sha(styles) != '2c68d83660cf0c2d141e7449d20b9d747fe5c3496d31d0551ff9c547656cf2a8':
        raise ValueError('pinned animation metadata differs')
    for pattern in [r"Lem\(\$1090, 'Building'\s*,\s*16, 16, 13,\s*3,\s*8,\s*13,\s*TLemmingAnimationType.Loop\)",
                    r"Lem\(\$4BF0, 'Shrugging'\s*,\s*8, 16, 10,\s*2,\s*8,\s*10,\s*TLemmingAnimationType.Once\)"]:
        if not re.search(pattern, styles.decode()):
            raise ValueError('Builder/Shrugging animation metadata differs')
    text = source.decode('cp1252').replace('\r\n', '\n')
    entries = list(re.finditer(r'^(?:class )?(?:function|procedure) TLemmingGame\.(\w+)\b', text, re.M))
    names = ['AssignBuilder', 'LayBrick', 'HandleBuilding', 'HandleShrugging']
    bodies = {m[1]: text[m.start():entries[i+1].start()].rstrip()+'\n'
              for i, m in enumerate(entries[:-1]) if m[1] in names}
    if set(bodies) != set(names):
        raise ValueError('missing Builder methods')
    args.output.mkdir()
    base = args.output/'miner-base'
    subprocess.run([sys.executable, str(Path(__file__).with_name('build_miner_reference.py')),
                    str(base), '--compiler', str(args.compiler), '--units', str(args.units)], check=True)
    host = (base/'harness.pas').read_text()

    def change(old, new):
        nonlocal host
        if host.count(old) != 1:
            raise ValueError('Miner host drift: '+old[:90])
        host = host.replace(old, new)

    change('type\n  TRect', 'type\n  TColor32 = LongWord;\n  TRect')
    change('TSkillPanelButton=(Digger,Basher,Miner);', 'TSkillPanelButton=(Digger,Basher,Miner,Builder);')
    change('    property Pixel[X,Y:Integer]:Integer read GetPixel; default;', '''    function GetPixelS(X,Y:Integer):Integer;
    procedure PutPixelS(X,Y,V:Integer);
    property PixelS[X,Y:Integer]:Integer read GetPixelS write PutPixelS;
    property Pixel[X,Y:Integer]:Integer read GetPixel; default;''')
    change('CurrDiggerCount,CurrBasherCount,CurrMinerCount:Integer;', 'CurrDiggerCount,CurrBasherCount,CurrMinerCount,CurrBuilderCount:Integer;')
    change('    function AssignMiner(Lemming1,Lemming2:TLemming):TLemming;', '''    class var BrickPixelColors:array[0..11] of TColor32;
    function AssignBuilder(Lemming1,Lemming2:TLemming):TLemming;
    procedure LayBrick(L:TLemming);
    function HandleBuilding(L:TLemming):Boolean;
    function HandleShrugging(L:TLemming):Boolean;
    function AssignMiner(Lemming1,Lemming2:TLemming):TLemming;''')
    change('SFX_HITS_STEEL,SFX_BASHER,SFX_MINER:Integer', 'SFX_HITS_STEEL,SFX_BASHER,SFX_MINER,SFX_BUILDER_WARNING:Integer')
    change(' ((Action=TLemmingAction.Mining) and (flags and ACTION_BIT_MINING<>0)) end;', ''' ((Action=TLemmingAction.Mining) and (flags and ACTION_BIT_MINING<>0)) or
 ((Action=TLemmingAction.Building) and (flags and ACTION_BIT_BUILDING<>0)) or
 ((Action=TLemmingAction.Shrugging) and (flags and ACTION_BIT_SHRUGGING<>0)) end;''')
    change('function TEntranceEnumerator.MoveNext:Boolean;', '''function TWorld.GetPixelS(X,Y:Integer):Integer;
begin Result:=0; if (X>=0) and (Y>=0) and (X<Width) and (Y<Height) then Result:=Cells[Y*Width+X] end;
procedure TWorld.PutPixelS(X,Y,V:Integer);
begin if (X>=0) and (Y>=0) and (X<Width) and (Y<Height) then Cells[Y*Width+X]:=Ord(V<>0) end;
function TEntranceEnumerator.MoveNext:Boolean;''')
    change("TLemmingAction.Mining:Write('Mining');", "TLemmingAction.Mining:Write('Mining'); TLemmingAction.Building:Write('Building'); TLemmingAction.Shrugging:Write('Shrugging');")
    change('if G.Previous[i]<>G.fWorld.Cells[i] then begin', 'if (G.Previous[i]=1) and (G.fWorld.Cells[i]=0) then begin')
    change('  G.Previous:=Copy(G.fWorld.Cells);\n', '')
    change("  WriteLn(']},\"basher\":{\"remaining\":',G.CurrBasherCount,'},\"miner\":{\"remaining\":',G.CurrMinerCount,'}}');", '''  Write(']},"basher":{"remaining":',G.CurrBasherCount,'},"miner":{"remaining":',G.CurrMinerCount,'},"builder":{"remaining":',G.CurrBuilderCount,',"bricks":[');
  First:=True;
  for i:=0 to Length(G.fLemmingList.Items)-1 do begin
    L:=G.fLemmingList.Items[i];
    if L.Action=TLemmingAction.Building then begin
      if not First then Write(','); First:=False; Write('[',L.ListIndex,',',L.NumberOfBricksLeft,']') end end;
  Write('],"filled":['); First:=True;
  if step<>0 then for i:=0 to Length(G.fWorld.Cells)-1 do if (G.Previous[i]=0) and (G.fWorld.Cells[i]=1) then begin
    if not First then Write(','); First:=False; Write(i) end;
  G.Previous:=Copy(G.fWorld.Cells);
  WriteLn(']}}');''')
    change('ReadLn(G.CurrDiggerCount,G.CurrBasherCount,G.CurrMinerCount,bug);', 'ReadLn(G.CurrDiggerCount,G.CurrBasherCount,G.CurrMinerCount,G.CurrBuilderCount,bug);')
    change('          TLemmingAction.Bashing:begin FrameCount:=32;', '''          TLemmingAction.Building:begin FrameCount:=16; FootY:=13; AnimationType:=TLemmingAnimationType.Loop end;
          TLemmingAction.Shrugging:FrameCount:=8;
          TLemmingAction.Bashing:begin FrameCount:=32;''')
    change('    G.LemmingMethods[TLemmingAction.Mining]:=G.HandleMining;', '''    G.LemmingMethods[TLemmingAction.Mining]:=G.HandleMining;
    G.LemmingMethods[TLemmingAction.Building]:=G.HandleBuilding;
    G.LemmingMethods[TLemmingAction.Shrugging]:=G.HandleShrugging;''')
    change("          else raise Exception.Create('unsupported input');", '''          6:begin accepted:=False;
            if (rate>=0) and (rate<Length(G.fLemmingList.Items)) and not G.fLemmingList.Items[rate].IsRemoved then
              accepted:=G.AssignBuilder(G.fLemmingList.Items[rate],nil)<>nil end;
          else raise Exception.Create('unsupported input');''')
    (args.output/'harness.pas').write_text(host)
    extracted = (base/'reference.inc').read_text()+'\n'+'\n'.join(bodies[k] for k in names)
    (args.output/'reference.inc').write_text(extracted)
    subprocess.run([str(args.compiler.resolve()), '-Mdelphi', '-O1', '-Fu'+str(args.units.resolve()),
                    '-o'+str((args.output/'reference').resolve()), str((args.output/'harness.pas').resolve())], check=True)
    provenance = json.loads((base/'reference.provenance.json').read_text())
    provenance['methods'].update({k: sha(v.encode()) for k, v in bodies.items()})
    provenance.update(builder_sha256=sha(Path(__file__).read_bytes()), generated_sha256=sha(host.encode()),
        extracted_sha256=sha(extracted.encode()), executable_sha256=sha((args.output/'reference').read_bytes()),
        scope='28 original functions; four jobs, Shrugging and mutable terrain; not full DOS',
        builder_adapter='Safe PixelS clips reads/writes; zero RGB brick colors and ALPHA_TERRAIN=1 retain collision occupancy; graphics/sound are stubbed')
    (args.output/'reference.provenance.json').write_text(json.dumps(provenance, indent=2)+'\n')


if __name__ == '__main__':
    main()
