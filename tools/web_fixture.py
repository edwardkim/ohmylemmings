"""Entirely invented DAT byte bank for browser file-input tests; no original bytes.

Layouts match crates/lemmings-game/tests/support and the public resource formats.
Literal-only encoding intentionally exercises the real decoder in native and WASM.
"""

def packed(data):
    chunks = [data[max(0, end-264):end] for end in range(len(data), 0, -264)]
    bits = ''
    for chunk in chunks:
        bits += f'00{len(chunk)-1:03b}' if len(chunk) <= 8 else f'111{len(chunk)-9:08b}'
        bits += ''.join(f'{value:08b}' for value in reversed(chunk))
    payload = bytes(sum((int(bit) << i) for i, bit in enumerate(bits[max(0,end-8):end])) for end in range(len(bits),0,-8))
    checksum = 0
    for value in payload: checksum ^= value
    return bytes([(len(bits)-1)%8+1, checksum, 0, 0])+len(data).to_bytes(2,'big')+b'\0\0'+(len(payload)+10).to_bytes(2,'big')+payload


def bank(win=False):
    level=bytearray(2048)
    for i,word in enumerate([50,1,1,1,1,2,3,4,5,6,7,8]): level[2*i:2*i+2]=word.to_bytes(2,'big')
    level[32:40]=bytes([0,64,0,40,0,1,0,0])
    level[288:1888]=b'\xff'*1600
    level[288:292]=bytes([0,64,52,0])
    if win:
        level[40:48]=bytes([0,16,0,4,0,0,0,0])
        level[288:292]=bytes([0,88,52,0])
    level[2016:]=b' '*32;level[2016:2025]=b'BYTE BANK'
    ground=bytearray(1056)
    ground[31:38]=bytes([1,8,1,5,0,4,0]);ground[448:454]=bytes([8,1,0,0,3,0]);ground[984:987]=bytes([63,10,2])
    if win:
        ground[:28]=ground[28:56]
        ground[16:21]=bytes([1,0,255,40,1])
    graphics=packed(bytes([255,0,0,255]))+packed(bytes([255,0,0,0,255]))
    main=b''.join(packed(bytes(size)) for size in [21104,388,8384,1,1,1,1])
    odd=(level[:24]+level[2016:])*80
    return {**{f'LEVEL{n:03}.DAT':packed(level)*8 for n in range(10)},'GROUND0O.DAT':ground,'VGAGR0.DAT':graphics,'MAIN.DAT':main,'ODDTABLE.DAT':odd}
