#!/usr/bin/env python3
"""Verify original Fun01 Digger against executed reference methods and SDL pixels."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

from check_digger import collect, reference_run, replay_text, run_native
from check_desktop import verify_pixels
from state_replay import ROOT, digest, verified_assets
from verify_native_view import png_from_ppm


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--binary', type=Path, default=ROOT/'target/release/lemmings-desktop')
    parser.add_argument('--reference', type=Path, required=True)
    parser.add_argument('--assets', type=Path, default=ROOT/'assets/local/dos-original')
    parser.add_argument('--xdotool', type=Path)
    args = parser.parse_args()
    args.output = args.output.resolve()
    if not args.output.is_relative_to(ROOT/'assets/local'):
        raise ValueError('original evidence must stay in assets/local')
    verified_assets(args.assets)
    args.output.mkdir()
    inputs = [(60, 0, 'AssignDigger', 0)]
    replay = args.output/'F01.inputs.txt'
    replay.write_text(replay_text(inputs))
    config = dict(width=1584, height=160, minutes=5, total=10, rate=50,
                  entrances=[[704,36]], pause_glitch=True, faller_three=True, steps=340)
    (args.output/'steel.bin').write_bytes(bytes(1584*160))
    rows = []
    for name, render, fast, pacing in [('F01-single',1,True,'dos'), ('F01-sparse',7,True,'dos'), ('F01-realtime',1,False,'dos')]:
        start = time.monotonic()
        states, captures = run_native(args.binary, args.output/name, replay, render,
                                     assets=args.assets, fast=fast, pacing=pacing)
        expected = reference_run(args.reference, config, args.output/name/'terrain-initial.bin', args.output/'steel.bin', inputs)
        if states != expected:
            (args.output/(name+'.reference.json')).write_text(json.dumps(expected))
            for step, (a,b) in enumerate(zip(states,expected)):
                if a != b: raise AssertionError(f'original reference first difference {step}: {a} != {b}')
            raise AssertionError('missing states')
        if states[61]['actors'][0]['action'] != 'Digging' or states[61]['digger']['remaining'] != 9:
            raise AssertionError('F01 assignment not accepted at measured boundary')
        for ppm in (args.output/name).glob('*.ppm'): png_from_ppm(ppm)
        rows.append(dict(name=name, snapshots=len(states), sha256=digest(states),
                         pixel_exact_captures=captures, elapsed_seconds=round(time.monotonic()-start,3)))
        print(json.dumps(rows[-1]),flush=True)
    if args.xdotool:
        directory = args.output/'F01-mouse'
        command = [str(args.binary.resolve()), '--assets', str(args.assets), '--steps', '180', '--capture', str(directory)]
        process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        def xdo(*parts):
            return subprocess.check_output([str(args.xdotool),*parts],text=True).strip()
        try:
            end = time.monotonic()+12
            clicked = False
            while time.monotonic()<end and process.poll() is None:
                path = directory/'trace.jsonl'
                if path.exists():
                    lines = path.read_text().splitlines()
                    state = json.loads(lines[-1]) if lines else None
                    if state and state['control_step']>=60 and state['actors'][0]['action']=='Walking':
                        windows = xdo('search','--onlyvisible','--name','Oh My Lemmings').splitlines()
                        if len(windows)!=1: raise ValueError('one dedicated SDL window required')
                        xdo('windowfocus','--sync',windows[0])
                        a = state['actors'][0]
                        xdo('mousemove','--window',windows[0],str((a['x']-624)*3),str((a['y']-5)*3),'click','1')
                        clicked = True
                        break
                time.sleep(0.002)
            stdout, stderr = process.communicate(timeout=20)
            if process.returncode or not clicked: raise ValueError(stderr+'; live mouse did not execute')
        finally:
            if process.poll() is None: process.terminate(); process.wait(timeout=5)
        recorded = []
        for line in (directory/'inputs.txt').read_text().splitlines():
            s,q,c,v = line.split(); recorded.append((int(s),int(q),c,int(v)))
        states = collect(directory)
        expected = reference_run(args.reference, config | dict(steps=180), directory/'terrain-initial.bin', args.output/'steel.bin', recorded)
        if len(recorded)!=1 or states!=expected or states[-1]['digger']['remaining']!=9:
            raise AssertionError('actual mouse boundary replay differs')
        captures = verify_pixels(directory)
        rows.append(dict(name='F01-mouse', inputs=recorded, snapshots=len(states),sha256=digest(states),pixel_exact_captures=captures))
        print(json.dumps(rows[-1]),flush=True)
    verified_assets(args.assets)
    result = dict(status='MATCH_REFERENCE_SUPPORTED', runs=rows, binary_sha256=hashlib.sha256(args.binary.read_bytes()).hexdigest(),
                  reference_sha256=hashlib.sha256(args.reference.read_bytes()).hexdigest(),
                  unverified=['whole DOS state/input trace','other jobs and cursor priority','rescue/exit/finish','sound'])
    (args.output/'evidence.json').write_text(json.dumps(result,indent=2)+'\n')


if __name__ == '__main__':
    main()
