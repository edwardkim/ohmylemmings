#!/usr/bin/env python3
"""Extend the pinned Digger host with original Basher functions and mask I/O."""
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
    base = args.output/'digger-base'
    subprocess.run([sys.executable, str(Path(__file__).with_name('build_digger_reference.py')),
                    str(base), '--compiler', str(args.compiler), '--units', str(args.units)], check=True)
    source = (ROOT/'assets/local/reference-sources/lemmix__src__Game.pas').read_bytes()
    if sha(source) != SOURCE_SHA:
        raise ValueError('pinned source differs')
    text = source.decode('cp1252').replace('\r\n', '\n')
    entries = list(re.finditer(r'^(?:class )?(?:function|procedure) TLemmingGame\.(\w+)\b', text, re.M))
    names = ['AssignBasher', 'HandleBashing', 'ApplyBashingMask']
    bodies = {m[1]: text[m.start():entries[i+1].start()].rstrip()+'\n'
              for i, m in enumerate(entries[:-1]) if m[1] in names}
    if set(bodies) != set(names):
        raise ValueError('missing Basher methods')
    styles = (ROOT/'assets/local/issue-16-sources/Styles.Base.pas').read_text()
    if not re.search(r"Lem\(\$1A50, 'Bashing'\s*,\s*32, 16, 10,\s*3,\s*8,\s*10,\s*TLemmingAnimationType.Loop\)", styles):
        raise ValueError('Basher metadata differs')
    host = (base/'harness.pas').read_text()

    def change(old, new):
        nonlocal host
        if host.count(old) != 1:
            raise ValueError('Digger host drift: '+old[:90])
        host = host.replace(old, new)

    change('uses SysUtils, Classes;', 'uses SysUtils, Classes, Types;')
    change('DOM_STEEL = 137;', 'DOM_STEEL = 137; DOM_ONEWAYLEFT=135; DOM_ONEWAYRIGHT=136;')
    change('TSkillPanelButton=(Digger);', 'TSkillPanelButton=(Digger,Basher);')
    change('Mining, Climbing);', 'Mining, Climbing, Bashing, Shrugging);')
    host = host.replace('array[0..13]', 'array[0..15]')
    change('  TInteractiveObjectInfo = class', '''  TBitmap32 = class
    Masks:array[0..79] of Byte;
    function CalcFrameRect(Count,Frame:Integer):TRect;
    procedure DrawTo(Target:TWorld; const D,S:TRect);
  end;
  TInteractiveObjectInfo = class''')
    change('    CurrDiggerCount:Integer;', '    CurrDiggerCount,CurrBasherCount:Integer;')
    change('    fMiniMap:TMini;', '    fMiniMap:TMini; BashMasks:array[Boolean] of TBitmap32; fTargetBitmap:TWorld;')
    change('    function AssignDigger(Lemming1,Lemming2:TLemming):TLemming;', '''    function AssignDigger(Lemming1,Lemming2:TLemming):TLemming;
    function AssignBasher(Lemming1,Lemming2:TLemming):TLemming;
    function HandleBashing(L:TLemming):Boolean;
    procedure ApplyBashingMask(L:TLemming; MaskFrame:Integer);''')
    change('SFX_HITS_STEEL:Integer', 'SFX_HITS_STEEL,SFX_BASHER:Integer')
    change(' ((Action=TLemmingAction.Walking) and (flags and ACTION_BIT_WALKING<>0)) end;', ''' ((Action=TLemmingAction.Walking) and (flags and ACTION_BIT_WALKING<>0)) or
 ((Action=TLemmingAction.Bashing) and (flags and ACTION_BIT_BASHING<>0)) end;''')
    change(' if Steel[Y*fWorld.Width+X]<>0 then Result:=DOM_STEEL end;', ''' case Steel[Y*fWorld.Width+X] of
 1,137:Result:=DOM_STEEL; 135:Result:=DOM_ONEWAYLEFT; 136:Result:=DOM_ONEWAYRIGHT end end;''')
    change('{$I reference.inc}', '''function TBitmap32.CalcFrameRect(Count,Frame:Integer):TRect;
begin if Count<>4 then raise Exception.Create('mask frame count'); Result:=Rect(0,Frame*10,16,(Frame+1)*10) end;
procedure TBitmap32.DrawTo(Target:TWorld; const D,S:TRect);
var x,y,dx,dy,bitIndex:Integer;
begin for y:=0 to 9 do for x:=0 to 15 do begin
 bitIndex:=(S.Top div 10)*160+y*16+x;
 if Masks[bitIndex div 8] and (128 shr (bitIndex mod 8))<>0 then begin
  dx:=D.Left+x; dy:=D.Top+y;
  if (dx>=0) and (dy>=0) and (dx<Target.Width) and (dy<Target.Height) then Target.Cells[dy*Target.Width+dx]:=0
 end end end;
{$I reference.inc}''')
    change("TLemmingAction.Digging:Write('Digging');", "TLemmingAction.Digging:Write('Digging'); TLemmingAction.Bashing:Write('Bashing');")
    change("  WriteLn(']}}');", "  WriteLn(']},\"basher\":{\"remaining\":',G.CurrBasherCount,'}}');")
    change('ReadLn(G.CurrDiggerCount);', 'ReadLn(G.CurrDiggerCount,G.CurrBasherCount);')
    change('    if (ParamCount<1) or (ParamCount>2) then', '    if ParamCount<>3 then')
    change('    G.fMiniMap:=TMini.Create;', '''    G.fMiniMap:=TMini.Create; G.HyperSpeed:=True;
    G.BashMasks[False]:=TBitmap32.Create; G.BashMasks[True]:=TBitmap32.Create;
    F:=TFileStream.Create(ParamStr(3),fmOpenRead);
    if F.Size<>160 then raise Exception.Create('mask size');
    F.ReadBuffer(G.BashMasks[False].Masks[0],80); F.ReadBuffer(G.BashMasks[True].Masks[0],80); F.Free;''')
    change('          TLemmingAction.Splatting:FrameCount:=16;', '''          TLemmingAction.Splatting:FrameCount:=16;
          TLemmingAction.Bashing:begin FrameCount:=32; AnimationType:=TLemmingAnimationType.Loop end;''')
    change('    G.LemmingMethods[TLemmingAction.Digging]:=G.HandleDigging;', '''    G.LemmingMethods[TLemmingAction.Digging]:=G.HandleDigging;
    G.LemmingMethods[TLemmingAction.Bashing]:=G.HandleBashing;''')
    change("          else raise Exception.Create('unsupported input');", '''          4:begin accepted:=False;
            if (rate>=0) and (rate<Length(G.fLemmingList.Items)) and not G.fLemmingList.Items[rate].IsRemoved then begin
              G.fLemmingList.Items[rate].ObjectInFront:=G.ReadObjectMap(G.fLemmingList.Items[rate].XPos+8*G.fLemmingList.Items[rate].xDelta,G.fLemmingList.Items[rate].YPos-8);
              accepted:=G.AssignBasher(G.fLemmingList.Items[rate],nil)<>nil end end;
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
                      scope='21 original functions; Walking/Digging/Bashing assignment, caller masks/object map; not full DOS',
                      mask_adapter='MSB-first packed frames, clipped erase DrawTo; no graphics framework, toolbar/sound/recorder stubs')
    (args.output/'reference.provenance.json').write_text(json.dumps(provenance, indent=2)+'\n')


if __name__ == '__main__':
    main()
