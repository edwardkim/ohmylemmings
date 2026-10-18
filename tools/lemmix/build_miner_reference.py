#!/usr/bin/env python3
"""Compile original Miner functions over the pinned Basher/Digger host."""
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
    args.output.mkdir()
    base = args.output/'basher-base'
    subprocess.run([sys.executable, str(Path(__file__).with_name('build_basher_reference.py')),
                    str(base), '--compiler', str(args.compiler), '--units', str(args.units)], check=True)
    source = (ROOT/'assets/local/reference-sources/lemmix__src__Game.pas').read_bytes()
    if sha(source) != SOURCE_SHA:
        raise ValueError('pinned source differs')
    text = source.decode('cp1252').replace('\r\n', '\n')
    entries = list(re.finditer(r'^(?:class )?(?:function|procedure) TLemmingGame\.(\w+)\b', text, re.M))
    names = ['AssignMiner', 'HandleMining', 'ApplyMinerMask']
    bodies = {m[1]: text[m.start():entries[i+1].start()].rstrip()+'\n'
              for i, m in enumerate(entries[:-1]) if m[1] in names}
    if set(bodies) != set(names):
        raise ValueError('missing Miner methods')
    styles = (ROOT/'assets/local/issue-16-sources/Styles.Base.pas').read_text()
    if not re.search(r"Lem\(\$2950, 'Mining'\s*,\s*24, 16, 13,\s*3,\s*8,\s*13,\s*TLemmingAnimationType.Loop\)", styles):
        raise ValueError('Miner metadata differs')
    host = (base/'harness.pas').read_text()

    def change(old, new):
        nonlocal host
        if host.count(old) != 1:
            raise ValueError('Basher host drift: '+old[:90])
        host = host.replace(old, new)

    change('TSkillPanelButton=(Digger,Basher);', '''TRect = record
    Left,Top,Right,Bottom:Integer;
    procedure Create(a,b,c,d:Integer);
    function GetWidth:Integer; function GetHeight:Integer;
    property Width:Integer read GetWidth; property Height:Integer read GetHeight;
  end;
  TSkillPanelButton=(Digger,Basher,Miner);''')
    # The older host did not execute Mining. Its placeholder flag collided
    # with Floating, which would incorrectly suppress normal animation ticks.
    change('ACTION_BIT_MINING=4096;', 'ACTION_BIT_MINING=65536;')
    change('PauseGlitch, FallerStartsWith3, EntranceX25)', 'PauseGlitch, FallerStartsWith3, EntranceX25, MinerOneWayRightBug)')
    change('Masks:array[0..79] of Byte;', 'Masks:array[0..79] of Byte; FrameHeight:Integer;')
    change('CurrDiggerCount,CurrBasherCount:Integer;', 'CurrDiggerCount,CurrBasherCount,CurrMinerCount:Integer;')
    change('BashMasks:array[Boolean] of TBitmap32;', 'BashMasks,MineMasks:array[Boolean] of TBitmap32;')
    change('    function AssignBasher(Lemming1,Lemming2:TLemming):TLemming;', '''    function AssignBasher(Lemming1,Lemming2:TLemming):TLemming;
    function AssignMiner(Lemming1,Lemming2:TLemming):TLemming;
    function HandleMining(L:TLemming):Boolean;
    procedure ApplyMinerMask(L:TLemming; MaskFrame,X,Y:Integer);''')
    change('SFX_HITS_STEEL,SFX_BASHER:Integer', 'SFX_HITS_STEEL,SFX_BASHER,SFX_MINER:Integer')
    change('constructor TLemming.Create;', '''procedure TRect.Create(a,b,c,d:Integer); begin Left:=a; Top:=b; Right:=c; Bottom:=d end;
function TRect.GetWidth:Integer; begin Result:=Right-Left end;
function TRect.GetHeight:Integer; begin Result:=Bottom-Top end;
function Rect(a,b,c,d:Integer):TRect; begin Result.Create(a,b,c,d) end;
constructor TLemming.Create;''')
    change(' ((Action=TLemmingAction.Bashing) and (flags and ACTION_BIT_BASHING<>0)) end;', ''' ((Action=TLemmingAction.Bashing) and (flags and ACTION_BIT_BASHING<>0)) or
 ((Action=TLemmingAction.Mining) and (flags and ACTION_BIT_MINING<>0)) end;''')
    change("begin if Count<>4 then raise Exception.Create('mask frame count'); Result:=Rect(0,Frame*10,16,(Frame+1)*10) end;", "begin if (Count<>4) and (Count<>2) then raise Exception.Create('mask frame count'); if FrameHeight=0 then FrameHeight:=10; Result:=Rect(0,Frame*FrameHeight,16,(Frame+1)*FrameHeight) end;")
    change('begin for y:=0 to 9 do for x:=0 to 15 do begin\n bitIndex:=(S.Top div 10)*160+y*16+x;', 'begin for y:=0 to S.Height-1 do for x:=0 to 15 do begin\n bitIndex:=S.Top*16+y*16+x;')
    change("TLemmingAction.Bashing:Write('Bashing');", "TLemmingAction.Bashing:Write('Bashing'); TLemmingAction.Mining:Write('Mining');")
    change("G.CurrBasherCount,'}}');", "G.CurrBasherCount,'},\"miner\":{\"remaining\":',G.CurrMinerCount,'}}');")
    change('oldReleased,oldIteration,initialRate:Integer;', 'oldReleased,oldIteration,initialRate,bug:Integer;')
    change('ReadLn(G.CurrDiggerCount,G.CurrBasherCount);', 'ReadLn(G.CurrDiggerCount,G.CurrBasherCount,G.CurrMinerCount,bug); if bug=1 then Include(G.fMechanics,TMechanic.MinerOneWayRightBug);')
    change('    if ParamCount<>3 then', '    if ParamCount<>4 then')
    change('    G.fMiniMap:=TMini.Create; G.HyperSpeed:=True;', '''    G.fMiniMap:=TMini.Create; G.HyperSpeed:=True;
    G.MineMasks[False]:=TBitmap32.Create; G.MineMasks[True]:=TBitmap32.Create;
    G.MineMasks[False].FrameHeight:=13; G.MineMasks[True].FrameHeight:=13;
    F:=TFileStream.Create(ParamStr(4),fmOpenRead); if F.Size<>104 then raise Exception.Create('mining mask size');
    F.ReadBuffer(G.MineMasks[False].Masks[0],52); F.ReadBuffer(G.MineMasks[True].Masks[0],52); F.Free;''')
    change('          TLemmingAction.Splatting:FrameCount:=16;', '''          TLemmingAction.Splatting:FrameCount:=16;
          TLemmingAction.Mining:begin FrameCount:=24; FootY:=13; AnimationType:=TLemmingAnimationType.Loop end;''')
    change('    G.LemmingMethods[TLemmingAction.Bashing]:=G.HandleBashing;', '''    G.LemmingMethods[TLemmingAction.Bashing]:=G.HandleBashing;
    G.LemmingMethods[TLemmingAction.Mining]:=G.HandleMining;''')
    change("          else raise Exception.Create('unsupported input');", '''          5:begin accepted:=False;
            if (rate>=0) and (rate<Length(G.fLemmingList.Items)) and not G.fLemmingList.Items[rate].IsRemoved then begin
              G.fLemmingList.Items[rate].ObjectInFront:=G.ReadObjectMap(G.fLemmingList.Items[rate].XPos+8*G.fLemmingList.Items[rate].xDelta,G.fLemmingList.Items[rate].YPos-8);
              G.fLemmingList.Items[rate].ObjectBelow:=G.ReadObjectMap(G.fLemmingList.Items[rate].XPos,G.fLemmingList.Items[rate].YPos);
              accepted:=G.AssignMiner(G.fLemmingList.Items[rate],nil)<>nil end end;
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
        scope='24 original functions; Walking/Digging/Bashing/Mining assignment and mutable terrain; not full DOS',
        stubs='toolbar/sound/minimap/recorder; caller object codes; distinct host action flags; stable ID without secondary selection',
        mask_adapter='MSB-first caller masks, clipped erase DrawTo, rectangle geometry helpers; no graphics framework')
    (args.output/'reference.provenance.json').write_text(json.dumps(provenance, indent=2)+'\n')


if __name__ == '__main__':
    main()
