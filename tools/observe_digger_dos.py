#!/usr/bin/env python3
"""Observe DOS Digger using keyboard events; never write guest/host memory."""
import argparse
import json
from pathlib import Path
import struct
import subprocess
import threading
import time

from capture_dos import MAPPER, PROCEDURE, ROOT, identity, record, save_json, sha
from dos_memory import decode_sample
from observe_dos import Observer
from reference_assets import verify_assets


def decode_digger(data):
    result = decode_sample(data)
    word = lambda offset: struct.unpack_from('<h', data, offset)[0]
    result.update(cursor_x=word(0x10), cursor_y=word(0x12), scroll_x=word(0),
                  selected_skill=data[0x82], remaining_diggers=data[0x7f],
                  actor0_x=word(0x88), actor0_y=word(0x8a),
                  actor0_action_bits=word(0x90), actor0_frame=data[0xb0],
                  actor0_removed=data[0xae], pause_flags=data[0x4e])
    return result


class Controller:
    def __init__(self, process, exe, output, xdotool):
        self.observer = Observer(process, exe, output/'memory-observation.json', decoder=decode_digger)
        self.process, self.output, self.xdotool = process, output, xdotool
        self.events, self.error = [], None
        self.start = time.monotonic_ns()
        self.thread = threading.Thread(target=self.control, daemon=True)
        self.thread.start()

    def command(self, *parts):
        self.events.append(dict(host_elapsed_ns=time.monotonic_ns()-self.start,
                                command=list(parts), sample=self.observer.latest))
        save_json(self.output/'control-progress.json', dict(events=self.events, error=self.error))
        subprocess.run([str(self.xdotool), *parts], check=True, capture_output=True, timeout=5)

    def wait(self, condition, timeout=15):
        end = time.monotonic()+timeout
        while time.monotonic()<end and self.process.poll() is None:
            sample = self.observer.latest
            if self.observer.error:
                raise ValueError(self.observer.error)
            if sample is not None and condition(sample):
                return sample
            time.sleep(0.002)
        raise ValueError('DOS control condition timed out')

    def key(self, name):
        self.command('key', '--delay', '25', name)
        time.sleep(0.1)

    def move(self, axis, target, negative, positive):
        for _ in range(10):
            current = self.observer.latest[axis]
            if abs(current-target)<=3:
                return
            key = negative if current>target else positive
            self.command('keydown', key)
            try:
                self.wait(lambda s: (s[axis]<=target+2 if current>target else s[axis]>=target-2), timeout=3)
            finally:
                self.command('keyup', key)
            time.sleep(0.06)
        raise ValueError('keyboard cursor could not be positioned')

    def control(self):
        try:
            self.wait(lambda s: 5<=s['startup_counter_u8']<20 and s['scroll_x']==640 and s['remaining_diggers']==10, timeout=55)
            sample = self.observer.latest
            # DOS positions include the 16-pixel world border. The keyboard
            # cursor/selection uses DS:10 + scroll-16 and DS:12 + 8 (0x3e2a).
            self.move('cursor_x', 744-sample['scroll_x']+8, 'o', 'p')
            self.move('cursor_y', 84-14, 'q', 'a')
            self.wait(lambda s: 35<=s['startup_counter_u8']<54 and s['entrances_triggered_u8']==255)
            self.key('F10')
            self.wait(lambda s: s['selected_skill']==9)
            self.wait(lambda s: s['released_u8']==1 and s['actor0_action_bits']==0
                      and s['actor0_x']>700)
            self.command('keydown', 'space')
            time.sleep(0.075)
            self.command('keyup', 'space')
            self.wait(lambda s: s['remaining_diggers']==9)
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
            raise ValueError('DOS Digger control failed: '+str(self.error))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--dosbox', type=Path, required=True)
    parser.add_argument('--xdotool', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.output = args.output.resolve()
    if not args.output.is_relative_to(ROOT/'assets/local'):
        raise ValueError('DOS evidence must remain in assets/local')
    manifest = json.loads((ROOT/'assets/manifests/dos_original.json').read_text())
    verify_assets(manifest, ROOT/manifest['local_directory'])
    seed = (ROOT/'assets/local/issue-28-dos-proof-2/bootstrap/drive/RUSSELL.DAT').read_bytes()
    if sha(seed) != '82abd5f27dfa78c0329f7f0ff67755725f28a92e5d8c96f642a3e7df77e333bf':
        raise ValueError('documented generated runtime differs')
    # Extend a private mapper copy; all baseline mapper/config assets stay intact.
    import capture_dos
    capture_dos.MAPPER = MAPPER + '\nkey_f10 "key 291"\nkey_f11 "key 292"\nkey_o "key 111"\nkey_q "key 113"\n'
    procedure = [(name, 30000 if name=='wait' else delay) for name, delay in PROCEDURE if name!='cursor_down_right']
    runtime = b'\0'+seed[1:]
    record(args, args.output, manifest, runtime, procedure=procedure,
           observer=lambda process: Controller(process, (ROOT/manifest['local_directory']/'VGALEMMI.EXE').read_bytes(), args.output, args.xdotool))
    verify_assets(manifest, ROOT/manifest['local_directory'])
    observation = json.loads((args.output/'memory-observation.json').read_text())
    samples = observation['samples']
    if not any(s['remaining_diggers']==9 and s['actor0_action_bits'] & 16 for s in samples):
        raise ValueError('Digger assignment was not observed')
    save_json(args.output/'provenance.json', dict(dosbox=identity(args.dosbox), original_files=manifest['files'],
              runtime_seed_sha256=sha(runtime), config=identity(args.output/'dosbox.conf'),
              mapper=identity(args.output/'mapper.map'), procedure=procedure,
              limitation='asynchronous numeric read-only polls and host keyboard input; not atomic full DOS trace'))
    print(json.dumps(dict(status='OBSERVED_DIGGER', samples=len(samples))))


if __name__ == '__main__':
    main()
