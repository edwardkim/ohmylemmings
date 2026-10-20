#!/usr/bin/env python3
"""Optional original x86 sound-dispatch proof over synthetic guest state."""
import argparse
import hashlib
import json
from pathlib import Path
from state_replay import ROOT, verified_assets

EXE_SHA = '4f21e94d53b7ea814ade802b4a37d42481da3876e31fb5a23090e19c9600f21c'


def verify(directory):
    import unicorn
    from unicorn import Uc, UC_ARCH_X86, UC_MODE_16, UC_HOOK_CODE, UC_HOOK_INTR
    from unicorn.x86_const import (UC_X86_REG_CS, UC_X86_REG_DS, UC_X86_REG_ES,
        UC_X86_REG_SS, UC_X86_REG_SP, UC_X86_REG_SI, UC_X86_REG_DI,
        UC_X86_REG_BP, UC_X86_REG_AX, UC_X86_REG_IP)
    if unicorn.__version__ != '2.1.4':
        raise ValueError('requires pinned Unicorn 2.1.4')
    before = verified_assets(directory)
    raw = (directory / 'VGALEMMI.EXE').read_bytes()
    if hashlib.sha256(raw).hexdigest() != EXE_SHA:
        raise ValueError('original executable differs')
    code = raw[0x120:]
    rows = []

    def execute(name, begin, stops, writes=(), registers=()):
        cpu = Uc(UC_ARCH_X86, UC_MODE_16)
        cpu.mem_map(0, 0x100000)
        cpu.mem_write(0x20000, code)
        for reg, value in [(UC_X86_REG_CS, 0x2000), (UC_X86_REG_DS, 0x6000),
                (UC_X86_REG_ES, 0x7000), (UC_X86_REG_SS, 0x8000),
                (UC_X86_REG_SP, 0xfff0), (UC_X86_REG_SI, 0x1000),
                (UC_X86_REG_DI, 0x4000), (UC_X86_REG_BP, 116), *registers]:
            cpu.reg_write(reg, value)
        # Distinct writable data/stack; original code bytes are never patched.
        for offset, data in [(0x2010, b'\xff'), (0x25a7, b'\x03'),
                (0x1000, (40).to_bytes(2, 'little') + (116).to_bytes(2, 'little')),
                (0x1006, b'\xf6\xff'), (0x1027, b'\x01'), *writes]:
            cpu.mem_write(0x60000 + offset, data)
        calls = []
        cpu.hook_add(UC_HOOK_INTR, lambda u, n, _: calls.append(
            [n, u.reg_read(UC_X86_REG_AX)]))
        reached = []
        def stop(u, address, _size, _data):
            if address - 0x20000 in stops:
                reached.append(address - 0x20000)
                u.emu_stop()
        cpu.hook_add(UC_HOOK_CODE, stop)
        cpu.emu_start(0x20000 + begin, 0x2ffff, count=10000)
        if len(reached) != 1:
            raise ValueError('original slice did not reach bounded stop: ' + name)
        row = dict(name=name, begin=begin, stop=reached[0], calls=calls,
            state_flags=int.from_bytes(cpu.mem_read(0x61008, 2), 'little'),
            frame=cpu.mem_read(0x61028, 1)[0],
            timer=cpu.mem_read(0x61020, 1)[0])
        rows.append(row)
        return cpu, row

    for fallen in [60, 61]:
        _, row = execute(f'landing-{fallen}', 0x224d, {0x1fe8},
            [(0x1023, bytes([fallen]))])
        if row['calls'] != ([[0x61, 0x408]] if fallen > 60 else []):
            raise ValueError('landing sound differs')
    for bricks in range(1, 13):
        _, row = execute(f'builder-{bricks}', 0x2693, {0x26d7},
            [(0x1028, b'\x09'), (0x1021, bytes([bricks]))])
        if row['calls'] != ([[0x61, 0x412]] if bricks <= 3 else []):
            raise ValueError('builder warning differs')
    for nuked in [0, 255]:
        for flags in [0, 4, 0x400, 0x1000, 0x4000]:
            _, row = execute(f'expiry-{flags}-{nuked}', 0x2e04, {0x2e62, 0x2ebf},
                [(0x1020, b'\x01'), (0x1008, flags.to_bytes(2, 'little')),
                 (0x4b, bytes([nuked]))])
            want = 0x40c if flags & 0x5404 else 0x405
            if row['calls'] != [[0x61, want]]:
                raise ValueError('countdown sound differs')
    _, row = execute('ohno-end', 0x2d0f, {0x1feb}, [(0x1028, b'\x0f')])
    if row['calls'] != [[0x61, 0x40c]]:
        raise ValueError('explosion transition differs')
    for name, begin, stop, want in [('exit',0x2f3b,0x2fa1,16),
            ('drowning',0x2ff8,0x304e,17),('vaporizing',0x304f,0x30a5,13),
            ('digger-steel',0x24bd,0x24d0,10),('basher-steel',0x2a57,0x2a6a,10),
            ('miner-steel',0x2cfa,0x2d0d,10),('skill-select',0x3f69,0x3f7c,1)]:
        _, row = execute(name,begin,{stop})
        if row['calls'] != [[0x61,0x400+want]]:
            raise ValueError('call-site effect differs: '+name)
    for job, begin in [('basher',0x2a2f),('miner',0x2c99)]:
        for direction in [-1,1]:
            for code_value in [7,8,9]:
                _, row = execute(f'oneway-{job}-{direction}-{code_value}',begin,{0x1fe8},
                    [(0x1027,bytes([direction&255]))],[(UC_X86_REG_AX,code_value)])
                if row['calls'] != ([[0x61,0x40a]] if code_value==9 else []):
                    raise ValueError('one-way/steel cue distinction differs')
    skills = [('climber',0x329b,0x6a),('floater',0x3557,0x6d),
        ('bomber',0x31c4,0x70),('blocker',0x3336,0x73),('builder',0x32bd,0x76),
        ('basher',0x34ce,0x79),('miner',0x3598,0x7c),('digger',0x320d,0x7f)]
    for name, begin, stock in skills:
        cpu, row = execute('assign-'+name,begin,{0x320b,0x320c,0x329a},[(stock,b'\x02')])
        if row['calls'] != [[0x61,0x404]] or cpu.mem_read(0x60000+stock,1)[0] != 1:
            raise ValueError('successful assignment differs: '+name)
        _, row = execute('empty-'+name,begin,{0x320b,0x320c,0x329a})
        if row['calls']:
            raise ValueError('empty assignment emitted sound')
    for name, begin, stock in skills:
        if name not in ['basher','miner','digger']:
            continue
        for below, front in [(0,9),(9,9),(0,7)]:
            cpu, row = execute(f'blocked-{name}-{below}-{front}',begin,{0x320b,0x320c,0x329a},
                [(stock,b'\x02'),(0x102b,bytes([front])),(0x102c,bytes([below]))])
            expected = [[0x61,0x412]] if name in ['basher','miner'] and front==9 and (name=='basher' or below!=9) else []
            # Digger ignores front restrictions and may succeed on an empty foot.
            if name=='digger' and below!=9:
                expected=[[0x61,0x404]]
            if row['calls'] != expected:
                raise ValueError('blocked assignment differs: '+row['name'])
    for raw_id in range(19):
        _, row = execute('trap-'+str(raw_id),0x2fd5,{0x2ff7,0x2f3a},
            [(0x401b,bytes([raw_id]))])
        if row['calls'] != ([[0x61,0x400+raw_id]] if raw_id else []):
            raise ValueError('raw trap request differs')
    for counter in [14,15,16,34,35,36,54,55,56]:
        _, row = execute('startup-'+str(counter),0x31e,{0x37f,0x393,0x3ac,0x3c0},
            [(0x4e,b'\x04'),(0x52,bytes([counter])),(0x54,b'\x06')])
        expected={15:[[0x61,0x403]],35:[[0x61,0x402]],55:[[0x61,0x306],[0x61,0]]}.get(counter,[])
        if row['calls'] != expected:
            raise ValueError('startup dispatch differs')
    music=[]
    for special in range(5):
        for advance in [0,255]:
            for cursor in range(17):
                cpu, _ = execute(f'music-{special}-{advance}-{cursor}',0x482f,{0x4865,0x4876},
                    [(0xfe8,bytes([special])),(0x1f6d,bytes([advance])),(0x1f6c,bytes([cursor]))])
                music.append(dict(special=special,advance=bool(advance),cursor=cursor,
                    next_cursor=cpu.mem_read(0x61f6c,1)[0],raw_music=cpu.mem_read(0x60054,1)[0]))
    if verified_assets(directory)!=before:
        raise ValueError('original files changed')
    return dict(status='PASS',exe_sha256=EXE_SHA,original_files=len(before),unicorn=unicorn.__version__,
        scope='Unmodified original x86 slices with synthetic guest actors/flags/maps; software INT requests recorded, handlers not executed. No complete DOS/device/listening claim.',
        effect_cases=[r for r in rows if not r['name'].startswith('music-')],music_cases=music,
        normal_music=list(code[0x4877:0x4888]),special_music=list(code[0x4888:0x488c]),
        startup_order='CS019d calls startup dispatch before CS01a3 increments DS0052; offsets relative to MZ load image')


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--original-assets',type=Path,required=True)
    p.add_argument('--evidence',type=Path,required=True)
    args=p.parse_args()
    if not args.evidence.resolve().is_relative_to((ROOT/'assets/local').resolve()):
        p.error('original evidence must remain ignored')
    result=verify(args.original_assets.resolve())
    args.evidence.parent.mkdir(parents=True,exist_ok=True)
    with args.evidence.open('x') as output:
        output.write(json.dumps(result,indent=2)+'\n')
    print(json.dumps({k:v for k,v in result.items() if k not in ['effect_cases','music_cases']}))


if __name__=='__main__':
    main()
