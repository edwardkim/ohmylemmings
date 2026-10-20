#!/usr/bin/env python3
"""Optional original IRQ dispatch proof; requires isolated Unicorn 2.1.4."""
import argparse
import hashlib
import json
from pathlib import Path

from state_replay import ROOT, verified_assets

EXE_SHA256 = '4f21e94d53b7ea814ade802b4a37d42481da3876e31fb5a23090e19c9600f21c'


def verify(directory):
    # Keep emulator and original code outside distributable dependencies.
    import unicorn
    from unicorn import Uc, UC_ARCH_X86, UC_MODE_16, UC_HOOK_INTR
    from unicorn.x86_const import UC_X86_REG_CS, UC_X86_REG_DS, UC_X86_REG_AX, UC_X86_REG_IP
    if unicorn.__version__ != '2.1.4':
        raise ValueError('requires pinned Unicorn 2.1.4')
    original = verified_assets(directory)
    data = (directory / 'VGALEMMI.EXE').read_bytes()
    if hashlib.sha256(data).hexdigest() != EXE_SHA256:
        raise ValueError('original executable differs')
    emulator = Uc(UC_ARCH_X86, UC_MODE_16)
    emulator.mem_map(0, 0x100000)
    emulator.mem_write(0x20000, data[0x120:])
    emulator.reg_write(UC_X86_REG_CS, 0x2000)
    emulator.reg_write(UC_X86_REG_DS, 0x6000)
    calls = []
    emulator.hook_add(UC_HOOK_INTR, lambda cpu, number, _: calls.append(
        (number, cpu.reg_read(UC_X86_REG_AX))))
    rows = []
    for name, selected, mask, expected_interrupt in [
        ('adlib-mask-1', 255, 1, 0x61),
        ('adlib-mask-2', 255, 2, 0x61),
        ('adlib-mask-3', 255, 3, 0x61),
        ('speaker', 0, 1, 0x60),
        ('muted', 255, 0, None),
    ]:
        emulator.mem_write(0x6001c, b'\0')
        emulator.mem_write(0x62010, bytes([selected]))
        emulator.mem_write(0x625a7, bytes([mask]))
        calls.clear()
        pattern = []
        for _ in range(512):  # Two complete u8 wraps, including odd/even parity.
            before = len(calls)
            emulator.emu_start(0x21843, 0x21874, count=100)
            if emulator.reg_read(UC_X86_REG_IP) != 0x1874:
                raise ValueError('IRQ slice did not reach stop address')
            pattern.append(len(calls) - before)
        expected = ([1] * 512 if expected_interrupt == 0x61 else
                    [1, 0] * 256 if expected_interrupt == 0x60 else [0] * 512)
        if pattern != expected or (calls and set(calls) != {(expected_interrupt, 0)}):
            raise ValueError('original IRQ dispatch differs')
        rows.append(dict(name=name, irqs=512, calls=len(calls),
                         interrupt=expected_interrupt, ax=0 if calls else None,
                         pattern_sha256=hashlib.sha256(bytes(pattern)).hexdigest()))
    if verified_assets(directory) != original:
        raise ValueError('original assets changed')
    return dict(status='PASS', original_files=len(original), exe_sha256=EXE_SHA256,
                unicorn=unicorn.__version__, rows=rows,
                scope='Unmodified IRQ dispatch slice 0x1843..0x1874 with injected enable flags; INT payloads recorded without executing handlers. No full DOS, PIT, PCM or device emulation.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--original-assets', type=Path, required=True)
    parser.add_argument('--evidence', type=Path, required=True)
    args = parser.parse_args()
    if not args.evidence.resolve().is_relative_to((ROOT / 'assets/local').resolve()):
        parser.error('original evidence must remain ignored')
    result = verify(args.original_assets.resolve())
    args.evidence.parent.mkdir(parents=True, exist_ok=True)
    with args.evidence.open('x') as output:
        output.write(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result))


if __name__ == '__main__':
    main()
