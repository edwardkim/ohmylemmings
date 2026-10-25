"""Invented pennant/one-way courses, using only original metadata constants.

No original image or level bytes. Both arrows are clipped onto destructible
walls; green/blue flags are free-standing. All20 profiles are exercised.
"""
import struct
from .web_fixture import packed
from .portal_fixture import bank as portal_bank

PROFILES=tuple((s,i,14,16,16,0) for s,ids in [(0,[2,9]),(1,[2,9]),(2,[2,7]),(3,[2,7]),(4,[2,3])] for i in ids)
PROFILES+=tuple((s,i,8 if s==1 else 7,32,21 if s==1 else 32,e) for s in range(5) for i,e in ([(4,7),(5,8)] if s==4 else [(3,7),(4,8)]))

def bank(theme, *, scenery=False):
    if type(theme) is not int or not 0<=theme<5:raise ValueError('theme must be0..4')
    files=portal_bank(theme);level=bytearray(2048)
    for n,v in enumerate([50,2,2,1,0,0,0,0,2,2,2,2]):struct.pack_into('>H',level,n*2,v)
    struct.pack_into('>H',level,26,theme)
    profiles=[p for p in PROFILES if p[0]==theme]
    level[32:40]=struct.pack('>HHBBBB',112,40,0,1,0,0)
    if scenery:level[32:40]=struct.pack('>HHBBBB',64,40,0,1,0,0)
    for slot,(_,i,n,w,h,e) in enumerate(profiles,1):
        x,y=(112 if slot%2 else 176), (72 if e==0 else 88)
        if scenery:
            x,y=(160,64 if e==7 else 96) if e else (56,72) if slot==1 else (256,104)
        level[32+slot*8:40+slot*8]=struct.pack('>HHBBBB',x,y,0,i,64 if e else 0,0)
    level[288:1888]=b'\xff'*1600
    tiles=[(16+i*8,120) for i in range(32)]
    tiles += [(x+16+i*8,y) for x in [96,160] for y in range(88,120) for i in range(4)]
    if scenery:
        tiles=[(x+16,y) for y in [120,128] for x in range(0,320,8)]
        tiles += [(x+16,y) for y in range(64,120,8) for x in range(144,208,8)]
        tiles += [(x+16,y) for y in range(88,120,8) for x in range(24,104,8)]
        # Two adjoining original-format rectangles form one80x32 steel slab.
        level[1888:1896]=bytes([5,22,0xf7,0,13,22,0x37,0])
    for slot,(x,y) in enumerate(tiles):level[288+slot*4:292+slot*4]=struct.pack('>HBB',x,(y+4)//2,((y+4)%2)<<7)
    level[2016:]=(b'SCENERY QUALITY FIXTURE' if scenery else b'DECOR FIXTURE').ljust(32,b' ')
    ground=bytearray(1056);objects=bytearray()
    for i,n,w,h,e in [(1,10,48,25,0)]+[(p[1],p[2],p[3],p[4],p[5]) for p in profiles]:
        at=i*28;size=w*h//8
        struct.pack_into('<HBBBBHH',ground,at,3 if i==1 else 2,1 if i==1 else 0,n,w,h,size*5,size*4)
        struct.pack_into('<H',ground,at+21,len(objects))
        ground[at+18:at+21]=bytes([w//4 if e else 1,6 if theme==1 and e else h//4 if e else 1,e])
        objects.extend((b'\xff'*size+b'\0'*(size*3)+b'\xff'*size)*n)
    ground[448:454]=bytes([8,1,0,0,3,0]);ground[984:987]=bytes([24,30,34])
    terrain=bytes([255,0,0,255])
    if scenery:
        ground[448:454]=bytes([8,8,0,0,24,0])
        terrain=b'\xff'*8+b'\0'*16+b'\xff'*8
    files[f'GROUND{theme}O.DAT']=bytes(ground);files[f'VGAGR{theme}.DAT']=packed(terrain)+packed(objects)
    record=packed(level);files.update({f'LEVEL{i:03}.DAT':record*8 for i in range(10)});files['ODDTABLE.DAT']=bytes(level[:24]+level[2016:])*80
    return files
