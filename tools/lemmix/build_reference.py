#!/usr/bin/env python3
"""Extract and compile pinned Lemmix methods; copies remain under assets/local."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[2]
SOURCE_SHA = 'c1ba9ffda9e8e277f5c9b171df2bb611210d3326b9623b28fcd35426f4d6a448'
METHODS = ['HasPixelAt', 'HasPixelAt_ClipY', 'Transition', 'TurnAround', 'HandleLemming',
           'HandleWalking', 'HandleJumping', 'HandleFalling', 'HandleSplatting',
           'CheckForLevelTopBoundary', 'RemoveLemming', 'IncrementIteration',
           'CheckSpawnLemming', 'BtnPause', 'BtnPauseStop']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--compiler', default='fpc')
    parser.add_argument('--units', type=Path)
    args = parser.parse_args()
    if not args.output.resolve().is_relative_to(ROOT/'assets/local'):
        raise ValueError('extracted source and executables must stay under assets/local')
    original = (ROOT/'assets/local/reference-sources/lemmix__src__Game.pas').read_bytes()
    if hashlib.sha256(original).hexdigest() != SOURCE_SHA:
        raise ValueError('pinned Game.pas hash mismatch')
    text = original.decode('cp1252').replace('\r\n', '\n')
    for name, expected in {'LEMMING_MIN_X': 0, 'LEMMING_MAX_X': 1647, 'LEMMING_MAX_Y': 163,
                           'HEAD_MIN_Y': -5, 'MAX_FALLDISTANCE': 60}.items():
        match = re.search(r'\b' + name + r'\s*=\s*(-?\d+)\s*;', text)
        if not match or int(match[1]) != expected:
            raise ValueError(f'reference boundary constant differs: {name}')
    entries = list(re.finditer(r'^(?:class )?(?:function|procedure) TLemmingGame\.(\w+)\b', text, re.M))
    methods = {}
    for index, match in enumerate(entries):
        if match[1] in METHODS:
            methods[match[1]] = text[match.start():entries[index+1].start()].rstrip() + '\n'
    if set(methods) != set(METHODS):
        raise ValueError('missing reference methods')
    # FPC 3.2.2 requires local variables before begin. Only these two syntax
    # changes are allowed; no reference expression/control flow is translated.
    patches = {
        'IncrementIteration': [
            ('procedure TLemmingGame.IncrementIteration;\nbegin',
             'procedure TLemmingGame.IncrementIteration;\nvar entrance: TInteractiveObjectInfo;\nbegin'),
            ('for var entrance: TInteractiveObjectInfo in Entrances do', 'for entrance in Entrances do')],
        'BtnPause': [
            ('procedure TLemmingGame.BtnPause(mode: TPauseCommandMode = TPauseCommandMode.None);\nbegin',
             'procedure TLemmingGame.BtnPause(mode: TPauseCommandMode = TPauseCommandMode.None);\nvar iteration: Integer;\nbegin'),
            ('var iteration: Integer := 34 - fRecorder.fRecordedGlitchPauseIterations;',
             'iteration := 34 - fRecorder.fRecordedGlitchPauseIterations;')],
    }
    provenance = dict(engine='lemmix-extracted-methods', commit='40e9bc34451f0e9127fd53b290a3877e700d143d',
                      source_sha256=SOURCE_SHA, compiler=subprocess.check_output([args.compiler, '-iV'], text=True).strip(),
                      scope='G05 startup and four unskilled motion actions; not full Lemmix/DOS',
                      methods={k: hashlib.sha256(v.encode()).hexdigest() for k, v in methods.items()},
                      syntax_patches=patches,
                      stubs='graphics, toolbar, recorder and sound; objects/game finish/skills not executed',
                      driver='normalized boundary input; IncrementIteration -> CheckSpawnLemming -> HandleLemming in ID order')
    metadata = ROOT/'assets/local/issue-16-sources/Styles.Base.pas'
    data = metadata.read_bytes()
    if hashlib.sha256(data).hexdigest() != '2c68d83660cf0c2d141e7449d20b9d747fe5c3496d31d0551ff9c547656cf2a8':
        raise ValueError('pinned animation metadata hash mismatch')
    rows = re.findall(r"Lem\(\$[0-9A-Fa-f]+,\s*'([^']+)'\s*,\s*(\d+),\s*16,\s*10,\s*2,\s*8,\s*10,\s*TLemmingAnimationType\.(\w+)\)", data.decode())
    expected = {'Walking': ('8', 'Loop'), 'Walking (rtl)': ('8', 'Loop'),
                'Jumping': ('1', 'Once'), 'Jumping (rtl)': ('1', 'Once'),
                'Falling': ('4', 'Loop'), 'Falling (rtl)': ('4', 'Loop'), 'Splatting': ('16', 'Once')}
    if any({name: (count, kind) for name, count, kind in rows}.get(k) != v for k, v in expected.items()):
        raise ValueError('animation metadata differs from adapter table')
    provenance['metadata_sha256'] = hashlib.sha256(data).hexdigest()
    provenance['compiler_sha256'] = hashlib.sha256(Path(shutil.which(args.compiler) or args.compiler).read_bytes()).hexdigest()
    provenance['builder_sha256'] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    provenance['compiler_flags'] = ['-Mdelphi', '-O1']
    for name, replacements in patches.items():
        for old, new in replacements:
            if methods[name].count(old) != 1:
                raise ValueError(f'syntax patch drift: {name}')
            methods[name] = methods[name].replace(old, new)
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output/'reference.inc').write_text('\n'.join(methods[m] for m in METHODS))
    template = Path(__file__).with_name('harness.pas')
    (args.output/'harness.pas').write_bytes(template.read_bytes())
    provenance['adapter_sha256'] = hashlib.sha256(template.read_bytes()).hexdigest()
    provenance['extracted_sha256'] = hashlib.sha256((args.output/'reference.inc').read_bytes()).hexdigest()
    command = [str(Path(args.compiler).resolve()) if '/' in args.compiler else args.compiler, '-Mdelphi', '-O1',
               '-o' + str((args.output/'reference').resolve())]
    if args.units:
        command.append('-Fu' + str(args.units.resolve()))
    command.append(str((args.output/'harness.pas').resolve()))
    subprocess.run(command, check=True)
    (args.output/'reference.provenance.json').write_text(json.dumps(provenance, indent=2) + '\n')


if __name__ == '__main__':
    main()
