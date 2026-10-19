#!/usr/bin/env python3
"""Run pinned finish/result methods with a documented DOS boundary adapter."""
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
    if not args.output.resolve().is_relative_to(ROOT/'assets/local'):
        raise ValueError('local evidence required')
    source = (ROOT/'assets/local/reference-sources/lemmix__src__Game.pas').read_bytes()
    if sha(source) != SOURCE_SHA:
        raise ValueError('pinned source differs')
    text = source.decode('cp1252').replace('\r\n', '\n')
    entries = list(re.finditer(r'^(?:class )?(?:function|procedure) TLemmingGame\.(\w+)\b', text, re.M))
    names = ['CheckForGameFinished', 'SetGameResult']
    bodies = {m[1]: text[m.start():entries[i+1].start()].rstrip()+'\n'
              for i,m in enumerate(entries[:-1]) if m[1] in names}
    if set(bodies) != set(names): raise ValueError('missing completion methods')
    args.output.mkdir()
    base = args.output/'hazards-base'
    subprocess.run([sys.executable, str(Path(__file__).with_name('build_hazards_reference.py')), str(base),
                    '--compiler', str(args.compiler), '--units', str(args.units)], check=True)
    host = (base/'harness.pas').read_text()
    def change(old,new):
        nonlocal host
        if host.count(old) != 1: raise ValueError('hazards host drift: '+old[:90])
        host = host.replace(old,new)
    change('    procedure IncrementIteration;', '''    procedure IncrementIteration;
    procedure CheckForGameFinished;
    procedure SetGameResult;
    procedure Finish;
    procedure CheckCompletion(secondsBefore:Integer);''')
    change('    fEntranceAnimationCompleted:Boolean;', '''    fIsFinished,fIsCheated:Boolean;
    EndReason:String;
    Level:record Info:record LemmingsCount,RescueCount:Integer end end;
    GameResultRec:record Cheated,TimeIsUp,Success:Boolean;
      LemmingCount,ToRescue,Rescued,Target,Done:Integer end;
    fEntranceAnimationCompleted:Boolean;''')
    change('procedure TToolbar.SetInfoMinutes(v:Integer); begin end;', '''procedure TToolbar.SetInfoMinutes(v:Integer); begin end;
procedure TLemmingGame.Finish;
begin SetGameResult; fIsFinished:=True end;
procedure TLemmingGame.CheckCompletion(secondsBefore:Integer);
var m,s,delay:Integer;
begin
  m:=Minutes;s:=Seconds;delay:=fParticleFinishTimer;
  Minutes:=secondsBefore div 60;Seconds:=secondsBefore mod 60;
  EndReason:='TimeExpired';
  if (delay=0) and ((LemmingsRemoved>=MaxNumLemmings) or (fIsNukedByUser and (LemmingsOut=0))) then
    EndReason:='PopulationExhausted';
  { DOS checks timeout despite particles. Original methods below stay unchanged. }
  if secondsBefore=0 then fParticleFinishTimer:=0;
  CheckForGameFinished;
  Minutes:=m;Seconds:=s;fParticleFinishTimer:=delay;
end;''')
    change('    if ParamCount<>7 then', '    if ParamCount<>8 then')
    change('  TrapFile:TFileStream;', '  CompletionFile:Text; secondsBefore:Integer;\n  TrapFile:TFileStream;')
    change('    G.currReleaseRate:=initialRate;', '    G.Level.Info.LemmingsCount:=G.MaxNumLemmings;\n    AssignFile(CompletionFile,ParamStr(8));Reset(CompletionFile);ReadLn(CompletionFile,G.Level.Info.RescueCount);CloseFile(CompletionFile);\n    GameOptions.ShowParticles:=True;\n    G.currReleaseRate:=initialRate;')
    change("      G.Events:='';", "      G.Events:=''; secondsBefore:=G.Minutes*60+G.Seconds;")
    change('      Snapshot(G,step+1);', '      G.CheckCompletion(secondsBefore);\n      Snapshot(G,step+1);\n      if G.fIsFinished then Break;')
    change('procedure Snapshot(G:TLemmingGame; step:Integer);', '''procedure CompletionSnapshot(G:TLemmingGame);
var done,target:Integer;phase,reason:String;
begin
  target:=0;done:=0;
  if G.Level.Info.LemmingsCount<>0 then target:=G.Level.Info.RescueCount*100 div G.Level.Info.LemmingsCount;
  if G.MaxNumLemmings<>0 then done:=G.LemmingsSaved*100 div G.MaxNumLemmings;
  phase:='Playing';reason:='null';
  if G.fIsFinished then begin
    if G.GameResultRec.Success then phase:='Won' else phase:='Lost';
    reason:='"'+G.EndReason+'"';
    done:=G.GameResultRec.Done;target:=G.GameResultRec.Target;
  end;
  Write(',"completion":{"phase":"',phase,'","reason":',reason,',"original_total":',G.Level.Info.LemmingsCount,
    ',"lemming_count":',G.MaxNumLemmings,',"required":',G.Level.Info.RescueCount,',"rescued":',G.LemmingsSaved,
    ',"target_percent":',target,',"rescued_percent":',done,'}');
end;
procedure Snapshot(G:TLemmingGame; step:Integer);''')
    change("  WriteLn('],\"drowned\":',drowned,',\"vaporized\":',vaporized,',\"trapped\":',trapped,'}}');",
           "  Write('],\"drowned\":',drowned,',\"vaporized\":',vaporized,',\"trapped\":',trapped,'}');CompletionSnapshot(G);WriteLn('}');")
    (args.output/'harness.pas').write_text(host)
    extracted = (base/'reference.inc').read_text()+'\n'+'\n'.join(bodies[n] for n in names)
    (args.output/'reference.inc').write_text(extracted)
    subprocess.run([str(args.compiler.resolve()), '-Mdelphi', '-O1', '-Fu'+str(args.units.resolve()),
                    '-o'+str((args.output/'reference').resolve()), str((args.output/'harness.pas').resolve())], check=True)
    provenance = json.loads((base/'reference.provenance.json').read_text())
    provenance['methods'].update({name:sha(body.encode()) for name,body in bodies.items()})
    provenance.update(completion_builder_sha256=sha(Path(__file__).read_bytes()), generated_sha256=sha(host.encode()),
        extracted_sha256=sha(extracted.encode()), executable_sha256=sha((args.output/'reference').read_bytes()),
        scope='56 original methods/table; own DOS post-actor/pre-clock completion boundary, time bypasses particle delay; not original InternalUpdate or whole DOS',
        completion_adapter='original CheckForGameFinished/SetGameResult unchanged; stub Finish freezes result; caller supplies pre-tick clock and clears finish delay only for timeout; original target/Nuke denominator; enabled particles')
    (args.output/'reference.provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')


if __name__=='__main__': main()
