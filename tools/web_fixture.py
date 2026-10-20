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


def bank(win=False, special=False):
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
    files={**{f'LEVEL{n:03}.DAT':packed(level)*8 for n in range(10)},'GROUND0O.DAT':ground,'VGAGR0.DAT':graphics,'MAIN.DAT':main,'ODDTABLE.DAT':odd}
    if special:
        # The four independent catalog routes from dos_compatibility.json.
        routes=[(1,3,4),(0,2,2),(0,1,1),(0,4,3)]
        file_records=[[packed(level)]*8 for _ in range(10)]
        for file,part,selector in routes:
            record=bytearray(level);record[29]=selector
            record[32:36]=bytes([1,64,0,40])  # entrance world304,40
            record[24:26]=(272).to_bytes(2,'big')
            if win: record[40:44]=bytes([1,64,0,4])
            file_records[file][part]=packed(record)
        for file,records in enumerate(file_records): files[f'LEVEL{file:03}.DAT']=b''.join(records)
        for index in range(4):
            picture=bytearray(40)
            colors=[(4,5,6),(12+index*3,30,45),(45,18,8),(0,0,0),(25,40,12),(50,40,20),(40,20,50),(60,60,60)]
            picture[:24]=bytes(v for rgb in colors for v in rgb)
            for strip in range(4):
                planes=bytearray(14400)
                for y in range(40):
                    world_y=strip*40+y
                    for x in range(960):
                        color=((x//80+world_y//20+index)%7)+1 if world_y>=100 or (world_y>=50 and abs((x%160)-80)+abs(world_y-75)<35) else 0
                        for plane in range(3):
                            if color&(1<<plane):planes[plane*4800+y*120+x//8]|=128>>(x%8)
                for begin in range(0,len(planes),128):
                    chunk=planes[begin:begin+128];picture.append(len(chunk)-1);picture.extend(chunk)
                picture.append(128)
            files[f'VGASPEC{index}.DAT']=packed(picture)
    return files
