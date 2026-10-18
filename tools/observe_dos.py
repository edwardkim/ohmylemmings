#!/usr/bin/env python3
"""Record one pinned DOS run with a read-only Linux /proc child-memory observer.

Only documented game counters/frames are exported; no guest or host memory is
written. The emulator executable and original EXE/DAT files remain unmodified.
"""
import argparse
import errno
import json
import os
from pathlib import Path
import threading
import time

from capture_dos import SETTINGS, PROCEDURE, identity, record, save_json, sha
from dos_memory import decode_sample, locate_ram
from reference_assets import ROOT, verify_assets


class Observer:
    def __init__(self, process, exe, output):
        self.process, self.exe, self.output = process, exe, output
        self.stop = threading.Event()
        self.error = None
        self.thread = threading.Thread(target=self.observe, daemon=True)
        self.thread.start()

    def observe(self):
        start, location = time.monotonic_ns(), None
        termination = 'process_exit'
        samples, previous = [], None
        try:
            with (Path('/proc')/str(self.process.pid)/'mem').open('rb', buffering=0) as memory:
                while not self.stop.is_set() and self.process.poll() is None:
                    if location is None:
                        maps = (Path('/proc')/str(self.process.pid)/'maps').read_text()
                        matches = []
                        for line in maps.splitlines():
                            fields = line.split(maxsplit=5)
                            lo, hi = (int(s, 16) for s in fields[0].split('-'))
                            if fields[1] != 'rw-p' or not 0x110000 <= hi-lo <= 64*1024*1024:
                                continue
                            if len(fields) == 6 and fields[5] != '[heap]':
                                continue
                            memory.seek(lo)
                            found = locate_ram(memory.read(hi-lo), lo, self.exe)
                            if found:
                                matches.append(found)
                        if len(matches) > 1:
                            raise ValueError('multiple emulated RAM mappings')
                        if matches:
                            location = matches[0]
                        else:
                            self.stop.wait(0.1)
                            continue
                    base, data = location
                    memory.seek(base+data)
                    try:
                        raw = memory.read(0x2200)
                    except OSError as error:
                        if error.errno != errno.EIO:
                            raise
                        # SDL teardown can unmap RAM before waitpid reports exit.
                        termination = 'ram_unmapped'
                        break
                    sample = decode_sample(raw)
                    key = tuple((k, json.dumps(v)) for k, v in sample.items() if k != 'irq_counter_u8')
                    if key != previous:
                        samples.append(dict(host_elapsed_ns=time.monotonic_ns()-start, **sample))
                        previous = key
                    self.stop.wait(0.002)
        except (ProcessLookupError, FileNotFoundError):
            if self.process.poll() is None:
                self.error = 'child process memory disappeared while emulator was running'
        except Exception as error:
            self.error = str(error)
        finally:
            save_json(self.output, dict(observation_kind='read-only child process memory',
                      poll_interval_ns=2000000, location=location, error=self.error,
                      termination=termination, samples=samples))

    def close(self):
        self.stop.set()
        self.thread.join(timeout=5)
        if self.thread.is_alive() or self.error:
            raise ValueError(f'DOS memory observation failed: {self.error}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--dosbox', type=Path, required=True)
    parser.add_argument('--xdotool', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--cycles', type=int, default=10000)
    parser.add_argument('--pc', type=int, choices=[0, 1, 2], default=1,
                        help='0: original #28 GUI procedure; 1/2: explicit -o/-x')
    args = parser.parse_args()
    if not 1000 <= args.cycles <= 100000:
        raise ValueError('cycles must be 1000..100000')
    args.output = args.output.resolve()
    if (ROOT/'assets/local').resolve() not in args.output.parents:
        raise ValueError('DOS recordings must stay under ignored assets/local/')
    args.dosbox, args.xdotool = args.dosbox.resolve(), args.xdotool.resolve()
    manifest = json.loads((ROOT/'assets/manifests/dos_original.json').read_text())
    verify_assets(manifest, ROOT/manifest['local_directory'])
    seed = ROOT/'assets/local/issue-28-dos-proof-2/bootstrap/drive/RUSSELL.DAT'
    original = seed.read_bytes()
    baseline = json.loads((ROOT/'assets/manifests/dos_observation.json').read_text())
    if sha(original) != baseline['runtime_seed']['generated_sha256']:
        raise ValueError('generate and verify the documented #29 runtime seed first')
    runtime = b'\0'+original[1:]
    settings = SETTINGS.replace('cycles=fixed 10000', f'cycles=fixed {args.cycles}')
    command = 'VGALEMMI /v'+({0: '', 1: ' -o', 2: ' -x'}[args.pc])
    settings = settings.replace('VGALEMMI /v', command)
    exe = (ROOT/manifest['local_directory']/'VGALEMMI.EXE').read_bytes()
    record(args, args.output, manifest, runtime, settings=settings,
           observer=lambda process: Observer(process, exe, args.output/'memory-observation.json'))
    save_json(args.output/'provenance.json', dict(command=command, pc=args.pc, cycles=args.cycles,
              dosbox=identity(args.dosbox), original_files=manifest['files'], procedure=PROCEDURE,
              runtime_seed_sha256=sha(runtime), config=identity(args.output/'dosbox.conf'),
              mapper=identity(args.output/'mapper.map'),
              tools=[identity(Path(__file__)), identity(ROOT/'tools/dos_memory.py')]))
    observation = json.loads((args.output/'memory-observation.json').read_text())
    if (not observation['samples'] or observation['location'] is None or
            max(s['released_u8'] for s in observation['samples']) != 10):
        raise ValueError('DOS memory could not be located; no observation verdict')
    print(json.dumps(dict(samples=len(observation['samples']), error=observation['error'])))


if __name__ == '__main__':
    main()
