#!/usr/bin/env python3
"""Add an own assignment probe to the unchanged 50-function Nuke reference."""
import argparse
import json
from pathlib import Path
import subprocess
import sys

from build_reference import ROOT
from build_digger_reference import sha


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--compiler', type=Path, required=True)
    parser.add_argument('--units', type=Path, required=True)
    args = parser.parse_args()
    if not args.output.resolve().is_relative_to(ROOT / 'assets/local'):
        raise ValueError('local evidence required')
    args.output.mkdir()
    base = args.output / 'nuke-base'
    subprocess.run([sys.executable, str(Path(__file__).with_name('build_nuke_reference.py')),
                    str(base), '--compiler', str(args.compiler), '--units', str(args.units)], check=True)
    host = (base / 'harness.pas').read_text()
    probe = Path(__file__).with_name('assignment_probe.pas').read_text()

    def change(old, new):
        nonlocal host
        if host.count(old) != 1:
            raise ValueError('Nuke host drift: ' + old[:80])
        host = host.replace(old, new)

    change('uses SysUtils, Classes, Types;', 'uses SysUtils, Classes, Types, TypInfo;')
    change('var\n  G:TLemmingGame;', probe + '\nvar\n  G:TLemmingGame;')
    change('if ParamCount<>6 then', 'if not (ParamCount in [6,7]) then')
    change('    Snapshot(G,0); j:=0;', """    if ParamCount=7 then begin
      if ParamStr(7)<>'--assignments' then raise Exception.Create('unknown probe');
      AssignmentProbe(G); Halt(0)
    end;
    Snapshot(G,0); j:=0;""")
    (args.output / 'harness.pas').write_text(host)
    extracted = (base / 'reference.inc').read_bytes()
    (args.output / 'reference.inc').write_bytes(extracted)
    subprocess.run([str(args.compiler.resolve()), '-Mdelphi', '-O1',
                    '-Fu' + str(args.units.resolve()),
                    '-o' + str((args.output / 'reference').resolve()),
                    str((args.output / 'harness.pas').resolve())], check=True)
    provenance = json.loads((base / 'reference.provenance.json').read_text())
    provenance.update(interactions_builder_sha256=sha(Path(__file__).read_bytes()),
                      assignment_probe_sha256=sha(probe.encode()), generated_sha256=sha(host.encode()),
                      extracted_sha256=sha(extracted), executable_sha256=sha((args.output / 'reference').read_bytes()),
                      scope='50 unchanged original methods/table; direct single-ID assignment matrix plus mixed replays; not full DOS')
    (args.output / 'reference.provenance.json').write_text(json.dumps(provenance, indent=2) + '\n')


if __name__ == '__main__':
    main()
