#!/usr/bin/env python3
"""Verify both MAIN UI families, preserving 200 old images and checking 39 VGA images."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

from reference_assets import ROOT, verify_assets


def sha(data):return hashlib.sha256(data).hexdigest()


def verify(assets,output):
    original=json.loads((ROOT/'assets/manifests/dos_original.json').read_text())
    verify_assets(original,assets)
    previous=json.loads((ROOT/'assets/manifests/main_ui.json').read_text())
    output.mkdir()
    result=subprocess.run(['cargo','run','--quiet','--locked','--offline','-p','lemmings-resources',
                           '--example','inspect_ui','--',str(assets/'MAIN.DAT'),str(output/'ui')],
                          cwd=ROOT,check=True,capture_output=True,text=True)
    (output/'ui.jsonl').write_text(result.stdout)
    rows=[json.loads(s) for s in result.stdout.splitlines()]
    if len(rows)!=239:raise AssertionError('missing UI images')
    indexed={(r['kind'],r['index']):r for r in rows}
    for old in previous['images']:
        row=indexed[(old['kind'],old['index'])]
        if any(row[k]!=old[k] for k in row):raise AssertionError('previous UI metadata changed')
        data=(output/'ui'/row['pixel_file']).read_bytes()
        if sha(data)!=old['pixel_sha256']:raise AssertionError('previous UI pixels changed')
    section=(output/'ui/section-002.bin').read_bytes()
    frozen=next(r for r in previous['sections'] if r['part']==2)
    if len(section)!=8384 or sha(section)!=frozen['sha256']:raise AssertionError('section 2 changed')
    extra=[]
    for row in rows:
        if row['kind'] not in ['VgaSkillPanel','VgaPanelGlyphs']:continue
        size=row['width']*row['height']//8
        window=section[row['offset']:row['offset']+size*row['planes']]
        # Whole-plane binary strings are independent from Rust pixel addressing.
        planes=[''.join(f'{byte:08b}' for byte in window[p*size:(p+1)*size]) for p in range(row['planes'])]
        expected=bytes(sum(int(plane[i])<<p for p,plane in enumerate(planes)) for i in range(size*8))
        data=(output/'ui'/row['pixel_file']).read_bytes()
        if data!=expected:raise AssertionError('VGA planar pixels differ')
        extra.append(row|dict(pixel_sha256=sha(data)))
    if len(extra)!=39:raise AssertionError('missing VGA panel/font')
    exe=(assets/'VGALEMMI.EXE').read_bytes()
    high=exe[0xc1a0:0xc1b8]
    expected=bytes([0,42,0,21,63,21,21,21,21,42,0,0,42,21,0,0,42,42,63,21,63,42,0,42])
    if high!=expected:raise AssertionError('original UI DAC table differs')
    report=dict(status='PASS',original_files=len(original['files']),preserved_images=200,vga_images=extra,
                vga_pixels=sum(r['width']*r['height'] for r in extra),section_sha256=sha(section),
                exe_sha256=sha(exe),palette_file_offset=0xc1a0,palette_dac6=list(high),palette_sha256=sha(high))
    (output/'summary.json').write_text(json.dumps(report,indent=2)+'\n')
    return report


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--assets',type=Path,default=ROOT/'assets/local/dos-original')
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    if not args.output.resolve().is_relative_to(ROOT/'assets/local'):raise ValueError('local evidence required')
    report=verify(args.assets.resolve(),args.output.resolve())
    print(json.dumps({k:report[k] for k in ['status','original_files','preserved_images','vga_pixels']}))


if __name__=='__main__':main()
