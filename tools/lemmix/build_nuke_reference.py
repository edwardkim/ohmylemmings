#!/usr/bin/env python3
"""Execute pinned BtnNuke/CheckUpdateNuking after the eight-job host."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys
from build_reference import ROOT, SOURCE_SHA
from build_digger_reference import sha


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('output',type=Path);p.add_argument('--compiler',type=Path,required=True);p.add_argument('--units',type=Path,required=True)
    args=p.parse_args()
    if not args.output.resolve().is_relative_to(ROOT/'assets/local'):raise ValueError('local evidence required')
    source=(ROOT/'assets/local/reference-sources/lemmix__src__Game.pas').read_bytes()
    if sha(source)!=SOURCE_SHA:raise ValueError('pinned source differs')
    text=source.decode('cp1252').replace('\r\n','\n');entries=list(re.finditer(r'^(?:class )?(?:function|procedure) TLemmingGame\.(\w+)\b',text,re.M))
    names=['BtnNuke','CheckUpdateNuking']
    bodies={m[1]:text[m.start():entries[i+1].start()].rstrip()+'\n' for i,m in enumerate(entries[:-1]) if m[1] in names}
    if set(bodies)!=set(names):raise ValueError('missing Nuke methods')
    args.output.mkdir();base=args.output/'floater-base'
    subprocess.run([sys.executable,str(Path(__file__).with_name('build_floater_reference.py')),str(base),'--compiler',str(args.compiler),'--units',str(args.units)],check=True)
    host=(base/'harness.pas').read_text()
    def change(old,new):
        nonlocal host
        if host.count(old)!=1:raise ValueError('Floater host drift: '+old[:100])
        host=host.replace(old,new)
    change('TriggeredTrapLemmixBugSolved, AssignClimberShruggerActionBug);','TriggeredTrapLemmixBugSolved, AssignClimberShruggerActionBug,NukeGlitch);')
    change('  TLemmingList = class\n    Items: array of TLemming;', '''  TLemmingList = class
    Items: array of TLemming;
    function GetItem(i:Integer):TLemming;
    property Item[i:Integer]:TLemming read GetItem; default;''')
    change('fIsNukedByUser,EntrancesOpened,','fIsExploderAssignInProgress:Boolean; fIndexOfLemmingToBeNuked:Integer;\n    fIsNukedByUser,EntrancesOpened,')
    change('    procedure CheckLemmings;', '    procedure CheckLemmings;\n    procedure BtnNuke;\n    procedure CheckUpdateNuking;\n    procedure RecordNuke;')
    change('SFX_OPENUMBRELLA:Integer','SFX_OPENUMBRELLA,SFX_NUKE:Integer')
    change('{$I reference.inc}', '''procedure TLemmingGame.RecordNuke; begin end;
function TLemmingList.GetItem(i:Integer):TLemming; begin Result:=Items[i] end;
{$I reference.inc}''')
    change("  WriteLn(']}}');", '''  WriteLn(']},"nuke":{"is_nuked":',LowerCase(BoolToStr(G.fIsNukedByUser,True)),
    ',"assigning":',LowerCase(BoolToStr(G.fIsExploderAssignInProgress,True)),
    ',"next_index":',G.fIndexOfLemmingToBeNuked,',"max_lemmings":',G.MaxNumLemmings,'}}');''')
    change('initialRate,bug:Integer;', 'initialRate,bug,ng:Integer;')
    change('G.CurrFloaterCount,bug);','G.CurrFloaterCount,bug,ng); if ng=1 then Include(G.fMechanics,TMechanic.NukeGlitch);')
    change("          else raise Exception.Create('unsupported input');", '''          11:begin accepted:=not G.IsPaused and not G.fIsNukedByUser;
            if accepted then G.BtnNuke end;
          else raise Exception.Create('unsupported input');''')
    change('        G.CheckLemmings;','        G.CheckLemmings; G.CheckUpdateNuking;')
    (args.output/'harness.pas').write_text(host)
    extracted=(base/'reference.inc').read_text()+'\n'+'\n'.join(bodies[k] for k in names)
    (args.output/'reference.inc').write_text(extracted)
    subprocess.run([str(args.compiler.resolve()),'-Mdelphi','-O1','-Fu'+str(args.units.resolve()),'-o'+str((args.output/'reference').resolve()),str((args.output/'harness.pas').resolve())],check=True)
    provenance=json.loads((base/'reference.provenance.json').read_text());provenance['methods'].update({k:sha(v.encode()) for k,v in bodies.items()})
    provenance.update(nuke_builder_sha256=sha(Path(__file__).read_bytes()),generated_sha256=sha(host.encode()),extracted_sha256=sha(extracted.encode()),executable_sha256=sha((args.output/'reference').read_bytes()),scope='50 original functions plus original float table; Nuke after eight-job motion, spawn suppression, scan/timers and optional NukeGlitch maximum; not whole DOS',nuke_adapter='normal unpaused/idempotent input policy; original BtnNuke and CheckUpdateNuking bodies; RecordNuke/sound stubbed, no game-result calculation')
    (args.output/'reference.provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')


if __name__=='__main__':main()
