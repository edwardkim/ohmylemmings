#!/usr/bin/env python3
"""Execute unchanged pinned pointer methods on the existing eight-job host."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
from build_reference import ROOT, SOURCE_SHA


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('output',type=Path)
    p.add_argument('--compiler',type=Path,required=True)
    p.add_argument('--units',type=Path,required=True)
    a=p.parse_args()
    if not a.output.resolve().is_relative_to(ROOT/'assets/local'):
        raise ValueError('reference output must remain local')
    a.output.mkdir()
    base=a.output/'base'
    subprocess.run([sys.executable,str(Path(__file__).with_name('build_interactions_reference.py')),str(base),
                    '--compiler',str(a.compiler),'--units',str(a.units)],check=True)
    raw=(ROOT/'assets/local/reference-sources/lemmix__src__Game.pas').read_bytes()
    sha=lambda b:hashlib.sha256(b).hexdigest()
    if sha(raw)!=SOURCE_SHA:raise ValueError('source hash differs')
    styles=(ROOT/'assets/local/issue-16-sources/Styles.Base.pas').read_bytes()
    if sha(styles)!='2c68d83660cf0c2d141e7449d20b9d747fe5c3496d31d0551ff9c547656cf2a8':
        raise ValueError('metadata hash differs')
    feet={m[1]:(int(m[2]),int(m[3])) for m in re.finditer(
        r"Lem\(\$[0-9A-Fa-f]+,\s*'([^']+)'\s*,\s*\d+,\s*\d+,\s*\d+,\s*\d+,\s*(\d+),\s*(\d+),",styles.decode())}
    expected={'Walking':10,'Jumping':10,'Falling':10,'Splatting':10,'Digging':12,'Bashing':10,
              'Mining':13,'Building':13,'Shrugging':10,'Blocking':10,'Oh-No-ing':10,'Exploding':25,
              'Climbing':12,'Hoisting':12,'Umbrella':16,'Exiting':13,'Drowning':10,'Vaporizing':14}
    if any(feet.get(k)!=(16 if k=='Exploding' else 8,v) for k,v in expected.items()):
        raise ValueError('selection foot metadata differs')
    source=raw.decode('cp1252').replace('\r\n','\n')
    entries=list(re.finditer(r'^(?:class )?(?:function|procedure) TLemmingGame\.(\w+)\b',source,re.M))
    names=['PrioritizedHitTest','ProcessSkillAssignment','AssignSkill']
    bodies={m[1]:source[m.start():entries[i+1].start()].rstrip()+'\n' for i,m in enumerate(entries[:-1]) if m[1] in names}
    if set(bodies)!=set(names):raise ValueError('missing methods')
    host=(base/'harness.pas').read_text()
    def change(old,new):
        nonlocal host
        if host.count(old)!=1:raise ValueError('host drift: '+old[:90])
        host=host.replace(old,new)
    change('AssignClimberShruggerActionBug,NukeGlitch);','AssignClimberShruggerActionBug,NukeGlitch,RightClickGlitch);')
    change('  TLemmingList = class', '''  TLemmingEnumerator=class
    Items:array of TLemming; Index:Integer;
    function MoveNext:Boolean;
    function GetCurrent:TLemming;
    property Current:TLemming read GetCurrent;
  end;
  TLemmingList = class
    function GetEnumerator:TLemmingEnumerator;
  public''')
    change('  TLemmingGame = class','''  TSkillMethod=function(Lemming1,Lemming2:TLemming):TLemming of object;
  TLemmingGame = class
    CursorPoint:TPoint;RightMouseButtonHeldDown:Boolean;
    fLastNonPrioritizedLemming:TLemming;fSelectedSkill:TSkillPanelButton;
    SkillMethods:array[TLemmingAction] of TSkillMethod;
    function PrioritizedHitTest(out Lemming1,Lemming2:TLemming;const CP:TPoint;CheckRightMouseButton:Boolean):Integer;
    function ProcessSkillAssignment(checkRegainControl:Boolean):TLemming;
    function AssignSkill(Lemming1,Lemming2:TLemming;aSkill:TLemmingAction):TLemming;
  public''')
    change('{$I reference.inc}', '''var SkillPanelButtonToAction:array[TSkillPanelButton] of TLemmingAction;
function TLemmingEnumerator.MoveNext:Boolean;begin Inc(Index);Result:=Index<Length(Items) end;
function TLemmingEnumerator.GetCurrent:TLemming;begin Result:=Items[Index] end;
function TLemmingList.GetEnumerator:TLemmingEnumerator;
begin Result:=TLemmingEnumerator.Create;Result.Items:=Items;Result.Index:=-1 end;
{$I reference.inc}''')
    change('SFX_NUKE:Integer','SFX_NUKE,SFX_ASSIGN_SKILL:Integer')
    change("procedure TLemmingGame.UpdatePixelCombine(L:TLemming); begin raise Exception.Create('pixel combining unsupported') end;",
           'procedure TLemmingGame.UpdatePixelCombine(L:TLemming); begin end;')
    probe=Path(__file__).with_name('pointer_probe.pas').read_text()
    change('var\n  G:TLemmingGame;',probe+'\nvar\n  G:TLemmingGame;')
    change('    if ParamCount=7 then begin','''    if ParamCount=7 then begin
      if ParamStr(7)='--pointer' then begin PointerProbe(G);Halt(0) end;''')
    (a.output/'harness.pas').write_text(host)
    extracted=(base/'reference.inc').read_text()+'\n'+'\n'.join(bodies[n] for n in names)
    (a.output/'reference.inc').write_text(extracted)
    subprocess.run([str(a.compiler.resolve()),'-Mdelphi','-O1','-Fu'+str(a.units.resolve()),
                    '-o'+str((a.output/'reference').resolve()),str((a.output/'harness.pas').resolve())],check=True)
    provenance=json.loads((base/'reference.provenance.json').read_text())
    provenance['methods'].update({k:sha(v.encode()) for k,v in bodies.items()})
    provenance.update(pointer_builder_sha256=sha(Path(__file__).read_bytes()),pointer_probe_sha256=sha(probe.encode()),
                      generated_sha256=sha(host.encode()),extracted_sha256=sha(extracted.encode()),
                      executable_sha256=sha((a.output/'reference').read_bytes()),
                      scope='53 unchanged original functions/table; pointer hotspot grid, overlap arbitration and live/trap remote assignment; not full DOS',
                      pointer_adapter='Lemmix hotspot adjustment -3,+2; stable ordered list; no renderer, actual AssignSkill/Transition/AssignBlocker')
    (a.output/'reference.provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')


if __name__=='__main__':main()
