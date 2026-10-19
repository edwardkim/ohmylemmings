#!/usr/bin/env python3
"""Observe original DOS Fun04 Climber through keyboard input and read-only RAM."""
import argparse
import json
from pathlib import Path
import subprocess
import threading
import time

import capture_dos
from capture_dos import MAPPER, ROOT, identity, record, save_json, sha
from observe_digger_dos import Controller
from observe_blocker_dos import decode_blocker
from observe_dos import Observer
from reference_assets import verify_assets

from observe_blocker_dos import GENERATED_RUNTIME_SHA, make_runtime_seed


def decode_climber(data):
    if len(data) != 0x2200:
        raise ValueError('incomplete live DOS sample')
    result = decode_blocker(data)
    # Original EXE33cf decrements DS:6a;33d3 writes1 at actor+24h (DS:ac).
    result.update(remaining_climbers=data[0x6a],actor0_ability_flags=data[0xac])
    return result


class ClimberObserver(Observer):
    def __init__(self, process, exe, output):
        self.last_complete = None
        self.terminal_short_read = None

        def decode(data):
            if len(data) == 0x2200:
                self.last_complete = decode_climber(data)
                return self.last_complete
            end = time.monotonic()+0.5
            while process.poll() is None and time.monotonic() < end:
                time.sleep(0.005)
            if data or process.poll() is None or self.last_complete is None:
                raise ValueError('incomplete live DOS sample')
            self.terminal_short_read = dict(bytes=len(data), child_returncode=process.returncode)
            return self.last_complete

        super().__init__(process, exe, output, decoder=decode)

    def close(self):
        super().close()
        result = json.loads(self.output.read_text())
        result['terminal_short_read'] = self.terminal_short_read
        save_json(self.output, result)


class ClimberController(Controller):
    def __init__(self, process, exe, output, xdotool):
        self.observer = ClimberObserver(process, exe, output/'memory-observation.json')
        self.process, self.output, self.xdotool = process, output, xdotool
        self.events, self.error = [], None
        self.start = time.monotonic_ns()
        self.thread = threading.Thread(target=self.control, daemon=True)
        self.thread.start()

    def control(self):
        try:
            # Grant compositor focus to our visible Xwayland child before the
            # menu protocol starts. X11 windowfocus alone does not grant it.
            end = time.monotonic()+1.8
            while time.monotonic() < end:
                result = subprocess.run([str(self.xdotool), 'search', '--onlyvisible',
                    '--pid', str(self.process.pid), '--name', 'DOSBox'],
                    capture_output=True, text=True, timeout=1)
                windows = result.stdout.splitlines()
                if len(windows) == 1:
                    # SDL can replace its startup X11 window between search
                    # and focus. Retry only this vanished-window status while
                    # still searching exclusively our own child process.
                    try:
                        self.command('mousemove', '--window', windows[0], '10', '10')
                        self.command('click', '1')
                    except subprocess.CalledProcessError as error:
                        if error.returncode != 1: raise
                        time.sleep(.02)
                        continue
                    break
                time.sleep(.02)
            else:
                raise ValueError('own visible DOSBox window did not appear')
            self.wait(lambda s:5<=s['startup_counter_u8']<20 and s['scroll_x']==576 and s['remaining_climbers']==10,timeout=65)
            self.move('cursor_x',695-576+8,'o','p')
            self.move('cursor_y',77-14,'q','a')
            self.wait(lambda s:35<=s['startup_counter_u8']<54 and s['entrances_triggered_u8']==255)
            self.command('keydown','F3')
            try:self.wait(lambda s:s['selected_skill']==2,timeout=2)
            finally:self.command('keyup','F3')
            self.wait(lambda s:s['released_u8']>=1 and s['actor0_action_bits']==0 and s['actor0_x']>=695,timeout=20)
            self.command('keydown','space');time.sleep(.075);self.command('keyup','space')
            self.wait(lambda s:s['remaining_climbers']==9 and s['actor0_ability_flags']&1)
            self.command('keydown','a','keydown','p');time.sleep(1.5);self.command('keyup','a','keyup','p')
        except Exception as error:
            self.error = str(error)

    def close(self):
        self.thread.join(timeout=5)
        self.observer.close()
        save_json(self.output/'control-events.json', dict(events=self.events, error=self.error))
        if self.thread.is_alive() or self.error:
            raise ValueError('DOS Climber control failed: '+str(self.error))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--dosbox', type=Path, required=True)
    parser.add_argument('--xdotool', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--seed', type=Path, help='verified previously generated RUSSELL.DAT')
    parser.add_argument('--seed-sha256', default=GENERATED_RUNTIME_SHA,
                        help='expected hash when reusing a generated runtime')
    parser.add_argument('--code', default='jldlcinecr', help='candidate code; validated by original level state')
    args = parser.parse_args()
    args.output = args.output.resolve()
    if not args.output.is_relative_to(ROOT/'assets/local'):
        raise ValueError('DOS evidence must stay in assets/local')
    if len(args.code) != 10 or not args.code.isascii() or not args.code.isalpha():
        raise ValueError('ten ASCII letters required for candidate code')
    args.code = args.code.lower()
    manifest = json.loads((ROOT/'assets/manifests/dos_original.json').read_text())
    verify_assets(manifest, ROOT/manifest['local_directory'])
    args.output.mkdir()
    letters = set(args.code+'oqp')
    capture_dos.MAPPER = MAPPER+'\nkey_f2 "key 283"\nkey_f3 "key 284"\n'
    capture_dos.MAPPER += ''.join(f'key_{key} "key {ord(key)}"\n' for key in sorted(letters) if key != 'a')
    original = args.seed.read_bytes() if args.seed else record(args, args.output/'bootstrap', manifest)
    # A freshly generated file is tied to this verified bootstrap. Other bytes
    # can differ from the historical host; preserve them and pin reused seeds.
    runtime = make_runtime_seed(original, args.seed_sha256 if args.seed else None)
    save_json(args.output/'runtime-seed.json', dict(generated_sha256=sha(original), seeded_sha256=sha(runtime),
              changed_offsets=[0], before=[9], after=[0], exe_or_original_dat_modified=False))
    procedure = [('2', 500), ('Return', 7000), ('F4', 5000), ('3', 3000), ('F2', 1500)]
    procedure += [(key, 100) for key in args.code]
    procedure += [('Return', 3000), ('s', 200), ('F1', 5000), ('s', 200), ('v', 300),
                  ('space', 2000), ('wait', 32000), ('s', 200), ('v', 200), ('x', 0)]
    directory = args.output/'run'
    record(args, directory, manifest, runtime, procedure=procedure,
           observer=lambda process: ClimberController(process,
               (ROOT/manifest['local_directory']/'VGALEMMI.EXE').read_bytes(), directory, args.xdotool))
    verify_assets(manifest, ROOT/manifest['local_directory'])
    samples = json.loads((directory/'memory-observation.json').read_text())['samples']
    if not any(s['remaining_climbers']==9 and s['actor0_ability_flags']&1 and s['actor0_action_bits']==32 for s in samples):
        raise ValueError('Climber consumption was not observed')
    save_json(directory/'provenance.json', dict(controller_sha256=sha(Path(__file__).read_bytes()),level_candidate=args.code,dosbox=identity(args.dosbox), original_files=manifest['files'],
              runtime_seed_sha256=sha(runtime), config=identity(directory/'dosbox.conf'),
              mapper=identity(directory/'mapper.map'), procedure=procedure,
              limitation='asynchronous numeric read-only polls and keyboard inputs; not atomic/full DOS state'))
    print(json.dumps(dict(status='OBSERVED_CLIMBER', samples=len(samples))))


if __name__ == '__main__':
    main()
