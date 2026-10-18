#!/usr/bin/env python3
"""Select DOS Fun04 through F2 and observe a Miner with read-only polling."""
import argparse
import json
from pathlib import Path
import threading
import time

import capture_dos
from capture_dos import MAPPER, ROOT, identity, record, save_json, sha
from observe_digger_dos import Controller, decode_digger
from observe_dos import Observer
from reference_assets import verify_assets


def decode_miner(data):
    result = decode_digger(data)
    result['remaining_miners'] = data[0x7c]
    result['remaining_bashers'] = data[0x79]
    return result


def decode_miner_sample(data,previous,exited):
    if len(data)==0x2200:
        return decode_miner(data)
    if len(data)!=0 or not exited or previous is None:
        raise ValueError('incomplete live DOS sample')
    return previous


class MinerObserver(Observer):
    def __init__(self,process,exe,output):
        self.last_complete=None
        self.terminal_short_read=None
        def decode(data):
            if len(data)!=0x2200:
                # /proc can return EOF while SDL is tearing down RAM, just
                # before waitpid observes exit. Never accept a short live read.
                end=time.monotonic()+0.5
                while process.poll() is None and time.monotonic()<end:
                    time.sleep(0.005)
                value=decode_miner_sample(data,self.last_complete,process.poll() is not None)
                self.terminal_short_read=dict(bytes=len(data),child_returncode=process.returncode)
                return value
            self.last_complete=decode_miner_sample(data,None,False)
            return self.last_complete
        super().__init__(process,exe,output,decoder=decode)

    def close(self):
        super().close()
        result=json.loads(self.output.read_text())
        result['terminal_short_read']=self.terminal_short_read
        save_json(self.output,result)


class MinerController(Controller):
    def __init__(self, process, exe, output, xdotool):
        self.observer = MinerObserver(process, exe, output/'memory-observation.json')
        self.process, self.output, self.xdotool = process, output, xdotool
        self.events, self.error = [], None
        self.start = time.monotonic_ns()
        self.thread = threading.Thread(target=self.control, daemon=True)
        self.thread.start()

    def control(self):
        try:
            self.wait(lambda s: 5 <= s['startup_counter_u8'] < 20 and s['scroll_x'] == 576
                      and s['remaining_miners'] == 1, timeout=65)
            self.move('cursor_x', 695-576+8, 'o', 'p')
            self.move('cursor_y', 77-14, 'q', 'a')
            self.wait(lambda s: 35 <= s['startup_counter_u8'] < 54 and s['entrances_triggered_u8'] == 255)
            self.key('F9')
            self.wait(lambda s: s['selected_skill'] == 8)
            self.wait(lambda s: s['released_u8'] >= 1 and s['actor0_action_bits'] == 0
                      and s['actor0_x'] >= 695, timeout=20)
            self.command('keydown', 'space')
            time.sleep(0.075)
            self.command('keyup', 'space')
            self.wait(lambda s: s['remaining_miners'] == 0)
            self.command('keydown', 'a', 'keydown', 'p')
            time.sleep(1.5)
            self.command('keyup', 'a', 'keyup', 'p')
        except Exception as error:
            self.error = str(error)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--dosbox', type=Path, required=True)
    parser.add_argument('--xdotool', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.output = args.output.resolve()
    if not args.output.is_relative_to(ROOT/'assets/local'):
        raise ValueError('DOS evidence must stay in assets/local')
    manifest = json.loads((ROOT/'assets/manifests/dos_original.json').read_text())
    verify_assets(manifest, ROOT/manifest['local_directory'])
    seed = (ROOT/'assets/local/issue-28-dos-proof-2/bootstrap/drive/RUSSELL.DAT').read_bytes()
    if sha(seed) != '82abd5f27dfa78c0329f7f0ff67755725f28a92e5d8c96f642a3e7df77e333bf':
        raise ValueError('documented generated runtime differs')
    letters = set('jldlcinecroq')
    capture_dos.MAPPER = MAPPER+'\nkey_f2 "key 283"\nkey_f9 "key 290"\n'
    capture_dos.MAPPER += ''.join(f'key_{key} "key {ord(key)}"\n' for key in sorted(letters) if key != 'a')
    # Built-in level access entry; the game validates this candidate code.
    procedure = [('2', 500), ('Return', 7000), ('F4', 5000), ('3', 3000), ('F2', 1500)]
    procedure += [(key, 100) for key in 'jldlcinecr']
    procedure += [('Return', 3000), ('s', 200), ('F1', 5000), ('s', 200), ('v', 300),
                  ('space', 2000), ('wait', 32000), ('s', 200), ('v', 200), ('x', 0)]
    runtime = b'\0'+seed[1:]
    record(args, args.output, manifest, runtime, procedure=procedure,
           observer=lambda process: MinerController(process,
               (ROOT/manifest['local_directory']/'VGALEMMI.EXE').read_bytes(), args.output, args.xdotool))
    verify_assets(manifest, ROOT/manifest['local_directory'])
    samples = json.loads((args.output/'memory-observation.json').read_text())['samples']
    if not any(s['remaining_miners'] == 0 and s['actor0_action_bits'] & 2048 for s in samples):
        raise ValueError('Miner consumption was not observed')
    save_json(args.output/'provenance.json', dict(dosbox=identity(args.dosbox), original_files=manifest['files'],
              runtime_seed_sha256=sha(runtime), config=identity(args.output/'dosbox.conf'),
              mapper=identity(args.output/'mapper.map'), procedure=procedure,
              limitation='asynchronous numeric read-only polls and keyboard inputs; not atomic/full DOS state'))
    print(json.dumps(dict(status='OBSERVED_MINER', samples=len(samples))))


if __name__ == '__main__':
    main()
