#!/usr/bin/env python3
"""Observe Fun01 exit animation/rescues using keyboard input and read-only RAM."""
import argparse
import json
from pathlib import Path
import threading
import time

import capture_dos
from capture_dos import MAPPER, PROCEDURE, ROOT, identity, record, save_json, sha
from observe_digger_dos import Controller
from observe_nuke_dos import decode_nuke
from observe_dos import Observer
from reference_assets import verify_assets


def decode_exit(data):
    result = decode_nuke(data)
    # Original EXE 20f8 dispatches 0x2000 to 2446. 2455 increments DS:46;
    # 2459 marks actor +26 removed after the eight-frame counter wraps to zero.
    # DS:42 is a temporary iteration counter, NOT the live population.
    result['saved_u8'] = data[0x46]
    return result


class ExitObserver(Observer):
    def __init__(self, process, exe, output):
        self.last_complete = None
        def decode(data):
            if len(data) == 0x2200:
                self.last_complete = decode_exit(data)
                return self.last_complete
            end = time.monotonic() + .5
            while process.poll() is None and time.monotonic() < end:
                time.sleep(.005)
            if data or process.poll() is None or self.last_complete is None:
                raise ValueError('incomplete live DOS exit sample')
            return self.last_complete
        super().__init__(process, exe, output, decoder=decode)


class ExitController(Controller):
    def __init__(self, process, exe, output, xdotool):
        self.observer = ExitObserver(process, exe, output / 'memory-observation.json')
        self.process, self.output, self.xdotool = process, output, xdotool
        self.events, self.error = [], None
        self.start = time.monotonic_ns()
        self.thread = threading.Thread(target=self.control, daemon=True)
        self.thread.start()

    def control(self):
        # Identical first-Walking Digger protocol to #31 on a dedicated X11
        # display with one DOSBox window. No memory writes or EXE patches.
        Controller.control(self)
        if self.error:
            return
        try:
            self.wait(lambda s: s['saved_u8'] >= 1, timeout=55)
        except Exception as error:
            self.error = str(error)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--dosbox', type=Path, required=True)
    p.add_argument('--xdotool', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    args = p.parse_args()
    args.output = args.output.resolve()
    if not args.output.is_relative_to(ROOT / 'assets/local'):
        raise ValueError('DOS evidence must stay in assets/local')
    manifest = json.loads((ROOT / 'assets/manifests/dos_original.json').read_text())
    verify_assets(manifest, ROOT / manifest['local_directory'])
    seed = (ROOT / 'assets/local/issue-28-dos-proof-2/bootstrap/drive/RUSSELL.DAT').read_bytes()
    if sha(seed) != '82abd5f27dfa78c0329f7f0ff67755725f28a92e5d8c96f642a3e7df77e333bf':
        raise ValueError('documented runtime seed differs')
    runtime = b'\0' + seed[1:]
    capture_dos.MAPPER = MAPPER + '\nkey_f10 "key 291"\nkey_o "key 111"\nkey_q "key 113"\n'
    procedure = [(name, 40000 if name == 'wait' else delay) for name, delay in PROCEDURE
                 if name not in ['cursor_down_right', 'x']]
    record(args, args.output, manifest, runtime, procedure=procedure, direct_window_keys=True,
           terminate_after_procedure=True, observer=lambda process: ExitController(process,
               (ROOT / manifest['local_directory'] / 'VGALEMMI.EXE').read_bytes(), args.output, args.xdotool))
    verify_assets(manifest, ROOT / manifest['local_directory'])
    samples = json.loads((args.output / 'memory-observation.json').read_text())['samples']
    saved = [s for s in samples if s['saved_u8'] > 0 and s['remaining_diggers'] == 9]
    if not saved:
        raise ValueError('no original rescue observed')
    exiting = [s for s in samples if any(a['action_bits'] == 0x2000 and not a['removed'] for a in s['actors'])]
    if not exiting:
        raise ValueError('no original exit animation observed')
    first_id = next(a['id'] for a in exiting[0]['actors'] if a['action_bits'] == 0x2000 and not a['removed'])
    frames = sorted({a['frame'] for s in samples for a in s['actors'] if a['id'] == first_id
                     and a['action_bits'] == 0x2000 and not a['removed']})
    if frames != list(range(8)):
        raise ValueError('incomplete observed exit frame sequence')
    if not any(a['id'] == first_id and a['action_bits'] == 0x2000 and a['removed'] == 255 and a['frame'] == 0
               for s in saved for a in s['actors']):
        raise ValueError('exit completion/removal not observed')
    save_json(args.output / 'provenance.json', dict(controller=identity(Path(__file__)), dosbox=identity(args.dosbox),
        original_files=manifest['files'], runtime_seed_sha256=sha(runtime), config=identity(args.output / 'dosbox.conf'),
        mapper=identity(args.output / 'mapper.map'), procedure=procedure,
        limitation='asynchronous read-only polls and keyboard inputs; no atomic/full DOS state or result assertion'))
    summary = dict(status='OBSERVED_EXIT_RESCUE', samples=len(samples), first_exit_id=first_id, frames=frames,
                   max_saved=max(s['saved_u8'] for s in saved), raw_removed_frame=0, normalized_removed_frame=7)
    save_json(args.output / 'summary.json', summary)
    print(json.dumps(summary))


if __name__ == '__main__':
    main()
