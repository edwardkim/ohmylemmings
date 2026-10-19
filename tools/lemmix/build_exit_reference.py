#!/usr/bin/env python3
"""Execute the pinned exit handler with the existing 50-function job host."""
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
    if sha(source) != SOURCE_SHA:
        raise ValueError('pinned source differs')
    styles = (ROOT / 'assets/local/issue-16-sources/Styles.Base.pas').read_bytes()
    if sha(styles) != '2c68d83660cf0c2d141e7449d20b9d747fe5c3496d31d0551ff9c547656cf2a8':
        raise ValueError('metadata differs')
    if not re.search(r"'Exiting'\s*,\s*8, 16, 13,\s*2,\s*8,\s*13,\s*TLemmingAnimationType\.Once", styles.decode()):
        raise ValueError('exit metadata differs')
    text = source.decode('cp1252').replace('\r\n', '\n')
    entries = list(re.finditer(r'^(?:class )?(?:function|procedure) TLemmingGame\.(\w+)\b', text, re.M))
    bodies = [text[m.start():entries[i + 1].start()].rstrip() + '\n'
              for i, m in enumerate(entries[:-1]) if m[1] == 'HandleExiting']
    if len(bodies) != 1:
        raise ValueError('missing exit handler')
    args.output.mkdir()
    base = args.output / 'nuke-base'
    subprocess.run([sys.executable, str(Path(__file__).with_name('build_nuke_reference.py')), str(base),
                    '--compiler', str(args.compiler), '--units', str(args.units)], check=True)
    host = (base / 'harness.pas').read_text()

    def change(old, new):
        nonlocal host
        if host.count(old) != 1:
            raise ValueError('Nuke host drift: ' + old[:100])
        host = host.replace(old, new)

    change('LemmingsOut,LemmingsRemoved:Integer;', 'LemmingsOut,LemmingsRemoved,LemmingsSaved:Integer;')
    change('    function HandleSplatting(L:TLemming):Boolean;', '    function HandleSplatting(L:TLemming):Boolean;\n    function HandleExiting(L:TLemming):Boolean;')
    change('    procedure DrawSkillCount(b:TSkillPanelButton; v:Integer);', '''    procedure DrawSkillCount(b:TSkillPanelButton; v:Integer);
    procedure SetInfoLemmingsOut(v:Integer);
    procedure SetInfoLemmingsSaved(saved,total:Integer; absolute:Boolean);''')
    change('var\n  SoundData:record', 'var\n  MiscOptions:record LemmingsPercentages:Boolean end;\n  SoundData:record')
    change('procedure TToolbar.DrawSkillCount(b:TSkillPanelButton; v:Integer); begin end;', '''procedure TToolbar.DrawSkillCount(b:TSkillPanelButton; v:Integer); begin end;
procedure TToolbar.SetInfoLemmingsOut(v:Integer); begin end;
procedure TToolbar.SetInfoLemmingsSaved(saved,total:Integer; absolute:Boolean); begin end;''')
    change("if L.IsRemoved then if L.Action=TLemmingAction.Splatting then", "if L.IsRemoved then if L.Action=TLemmingAction.Exiting then removal:='\"Rescued\"' else if L.Action=TLemmingAction.Splatting then")
    change("TLemmingAction.Floating:Write('Floating');", "TLemmingAction.Exiting:Write('Exiting'); TLemmingAction.Floating:Write('Floating');")
    change('          TLemmingAction.Splatting:FrameCount:=16;', '          TLemmingAction.Splatting:FrameCount:=16;\n          TLemmingAction.Exiting:begin FrameCount:=8;FootY:=13 end;')
    change('    SoundData.SFX_LETSGO:=1;', '    G.LemmingMethods[TLemmingAction.Exiting]:=G.HandleExiting;\n    SoundData.SFX_LETSGO:=1;')
    change('  WriteLn(\']},"nuke":', '  Write(\']},"nuke":')
    change("G.MaxNumLemmings,'}}');", '''G.MaxNumLemmings,'},"exit":{"saved":',G.LemmingsSaved,',"exiting":[');
  First:=True;
  for i:=0 to Length(G.fLemmingList.Items)-1 do begin L:=G.fLemmingList.Items[i];
    if (L.Action=TLemmingAction.Exiting) and not L.IsRemoved then begin
      if not First then Write(',');First:=False;Write(L.ListIndex) end end;
  WriteLn(']}}');''')
    (args.output / 'harness.pas').write_text(host)
    extracted = (base / 'reference.inc').read_text() + '\n' + bodies[0]
    (args.output / 'reference.inc').write_text(extracted)
    subprocess.run([str(args.compiler.resolve()), '-Mdelphi', '-O1', '-Fu' + str(args.units.resolve()),
                    '-o' + str((args.output / 'reference').resolve()), str((args.output / 'harness.pas').resolve())], check=True)
    provenance = json.loads((base / 'reference.provenance.json').read_text())
    provenance['methods']['HandleExiting'] = sha(bodies[0].encode())
    provenance.update(exit_builder_sha256=sha(Path(__file__).read_bytes()), generated_sha256=sha(host.encode()),
                      extracted_sha256=sha(extracted.encode()), executable_sha256=sha((args.output / 'reference').read_bytes()),
                      scope='51 original methods/table; exit/action ordering, 8-frame rescue, jobs and Nuke; not whole DOS',
                      exit_adapter='original handler unchanged; own normalized saved/exiting trace and animation metadata; toolbar UI stubs')
    (args.output / 'reference.provenance.json').write_text(json.dumps(provenance, indent=2) + '\n')


if __name__ == '__main__':
    main()
