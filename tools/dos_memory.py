"""Read-only Linux child-process observation of this repository's pinned DOS EXE.

Offsets refer to VGALEMMI.EXE's MZ image/data segment, identified from original
instructions, not an emulator-specific host address. No guest/host bytes change.
"""
import struct


def locate_ram(region, host_start, exe, anchor_offset=0x2a6):
    if len(exe) < anchor_offset+16 or exe[:2] != b'MZ':
        raise ValueError('complete MZ executable required')
    header = struct.unpack_from('<H', exe, 8)[0]*16
    if header+3 > len(exe) or exe[header] != 0xba or anchor_offset < header:
        raise ValueError('expected pinned MZ entry loading data segment into DX')
    original_ds = struct.unpack_from('<H', exe, header+1)[0]
    needle = exe[anchor_offset:anchor_offset+16]
    offset, candidates = 0, []
    while True:
        index = region.find(needle, offset)
        if index < 0:
            break
        offset = index+1
        code = index-(anchor_offset-header)
        if code < 0 or region[code] != 0xba:
            continue
        relocated_ds = struct.unpack_from('<H', region, code+1)[0]
        load_segment = relocated_ds-original_ds
        base = code-load_segment*16
        data = base+relocated_ds*16
        if load_segment >= 0x10 and base >= 0 and base+0x110000 <= len(region) and data+0x2200 <= base+0xa0000:
            candidates.append((host_start+base, relocated_ds*16))
    if len(candidates) > 1:
        raise ValueError('ambiguous emulated RAM location')
    return candidates[0] if candidates else None


def decode_sample(data):
    if len(data) != 0x2200:
        raise ValueError('complete documented data-segment window required')
    return dict(startup_counter_u8=data[0x52], released_u8=data[0x43],
                entrances_triggered_u8=data[0x55], spawn_countdown_u8=data[0x48],
                pit_divisor_u16=struct.unpack_from('<H', data, 0x1fc9)[0],
                irq_counter_u8=data[0x1fcd],
                pc_flags=list(data[0x200a:0x200f]),
                object_frames_u8=[data[0x17cb+slot*28+2] for slot in range(32)])
