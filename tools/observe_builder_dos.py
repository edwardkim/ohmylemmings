#!/usr/bin/env python3
"""Observe original DOS Fun07 Builder through keyboard input and read-only RAM."""
import argparse
import json
from pathlib import Path
import threading
import time

import capture_dos
from capture_dos import MAPPER, ROOT, identity, record, save_json, sha
from observe_digger_dos import Controller
from observe_miner_dos import decode_miner
from observe_dos import Observer
from reference_assets import verify_assets

GENERATED_RUNTIME_SHA = '82abd5f27dfa78c0329f7f0ff67755725f28a92e5d8c96f642a3e7df77e333bf'


def make_runtime_seed(original, expected_sha=None):
    if (len(original) != 256 or original[0] != 9
            or (expected_sha is not None and sha(original) != expected_sha)):
        raise ValueError('documented generated runtime differs')
    return b'\0'+original[1:]


def decode_builder(data):
    if len(data) != 0x2200:
        raise ValueError('incomplete live DOS sample')
    result = decode_miner(data)
    # VGALEMMI.EXE offsets33dd/33f6 read/decrement DS:76. Offset33ff
    # writes action0080 and3412 writes [actor+21]=12, hence DS:a9.
    result.update(remaining_builders=data[0x76], actor0_bricks=data[0xa9])
    return result


class BuilderObserver(Observer):
    def __init__(self, process, exe, output):
        self.last_complete = None
        self.terminal_short_read = None

        def decode(data):
            if len(data) == 0x2200:
                self.last_complete = decode_builder(data)
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


class BuilderController(Controller):
    def __init__(self, process, exe, output, xdotool):
        self.observer = BuilderObserver(process, exe, output/'memory-observation.json')
        self.process, self.output, self.xdotool = process, output, xdotool
        self.events, self.error = [], None
        self.start = time.monotonic_ns()
        self.thread = threading.Thread(target=self.control, daemon=True)
        self.thread.start()

    def control(self):
        try:
            self.wait(lambda s: 5 <= s['startup_counter_u8'] < 20 and s['scroll_x'] == 464
                      and s['remaining_builders'] == 20, timeout=65)
            self.move('cursor_x', 567-464+8, 'o', 'p')
            self.move('cursor_y', 144-14, 'q', 'a')
            self.wait(lambda s: 35 <= s['startup_counter_u8'] < 54 and s['entrances_triggered_u8'] == 255)
            self.key('F7')
            self.wait(lambda s: s['selected_skill'] == 6)
            self.wait(lambda s: s['released_u8'] >= 1 and s['actor0_action_bits'] == 0
                      and s['actor0_x'] >= 567, timeout=20)
            self.command('keydown', 'space')
            time.sleep(0.075)
            self.command('keyup', 'space')
            self.wait(lambda s: s['remaining_builders'] == 19)
            self.command('keydown', 'a', 'keydown', 'p')
            time.sleep(1.5)
            self.command('keyup', 'a', 'keyup', 'p')
        except Exception as error:
            self.error = str(error)

    def close(self):
        self.thread.join(timeout=5)
        self.observer.close()
        save_json(self.output/'control-events.json', dict(events=self.events, error=self.error))
        if self.thread.is_alive() or self.error:
            raise ValueError('DOS Builder control failed: '+str(self.error))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--dosbox', type=Path, required=True)
    parser.add_argument('--xdotool', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--seed', type=Path, help='verified previously generated RUSSELL.DAT')
    parser.add_argument('--seed-sha256', default=GENERATED_RUNTIME_SHA,
                        help='expected hash when reusing a generated runtime')
    parser.add_argument('--code', default='lcanlldhco', help='candidate code; validated by original level state')
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
    capture_dos.MAPPER = MAPPER+'\nkey_f2 "key 283"\nkey_f7 "key 288"\n'
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
           observer=lambda process: BuilderController(process,
               (ROOT/manifest['local_directory']/'VGALEMMI.EXE').read_bytes(), directory, args.xdotool))
    verify_assets(manifest, ROOT/manifest['local_directory'])
    samples = json.loads((directory/'memory-observation.json').read_text())['samples']
    if not any(s['remaining_builders'] == 19 and s['actor0_action_bits'] & 128 for s in samples):
        raise ValueError('Builder consumption was not observed')
    save_json(directory/'provenance.json', dict(dosbox=identity(args.dosbox), original_files=manifest['files'],
              runtime_seed_sha256=sha(runtime), config=identity(directory/'dosbox.conf'),
              mapper=identity(directory/'mapper.map'), procedure=procedure,
              limitation='asynchronous numeric read-only polls and keyboard inputs; not atomic/full DOS state'))
    print(json.dumps(dict(status='OBSERVED_BUILDER', samples=len(samples))))


if __name__ == '__main__':
    main()
