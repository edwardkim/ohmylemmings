"""Invented five-theme portal courses; no original graphics or level bytes.

Only documented portal dimensions/first-frame/trigger constants are reused.
The flat floor catches the spawned actor; one actor walks right into the exit.
"""
import struct
from .web_fixture import packed

HEIGHTS=(17,28,16,17,15)
LEFT=(6,5,6,6,5)
TOP=(4,7,4,4,3)

def bank(theme):
    if type(theme) is not int or not 0<=theme<5:raise ValueError('theme must be0..4')
    level=bytearray(2048)
    for i,value in enumerate([50,1,1,1,0,0,0,0,0,0,0,0]):struct.pack_into('>H',level,i*2,value)
    struct.pack_into('>H',level,26,theme)
    level[32:40]=struct.pack('>HHBBBB',64,40,0,1,0,0) # entrance world48,40
    level[40:48]=struct.pack('>HHBBBB',176,103-HEIGHTS[theme],0,0,0,0)
    decoration=[7,6,6,6,8][theme];band_height=24 if theme==1 else 8
    level[48:56]=struct.pack('>HHBBBB',176,103-HEIGHTS[theme]-band_height,0,decoration,0,0)
    level[288:1888]=b'\xff'*1600
    for i in range(32):level[288+i*4:292+i*4]=struct.pack('>HBB',16+i*8,53,0) # floor world0..255,y102
    level[2016:]=b'PORTAL FIXTURE'.ljust(32,b' ')
    ground=bytearray(1056);objects=bytearray()
    for id,width,height,count,first,flags in [(0,48,HEIGHTS[theme],1,0,0),(1,48,25,10,1,3),(decoration,48,band_height,6,0,2)]:
        n=width*height//8;offset=id*28
        struct.pack_into('<HBBBBHH',ground,offset,flags,first,count,width,height,n*5,n*4)
        struct.pack_into('<H',ground,offset+21,len(objects))
        if id==0:
            struct.pack_into('<HH',ground,offset+14,LEFT[theme],TOP[theme]);ground[offset+18:offset+21]=bytes([1,2,1])
        elif id==1:ground[offset+18:offset+20]=bytes([1,1])
        objects.extend((b'\xff'*n+b'\0'*(n*3)+b'\xff'*n)*count)
    ground[448:454]=bytes([8,1,0,0,3,0]);ground[984:987]=bytes([24,30,34])
    main=b''.join(packed(bytes(size)) for size in [21104,388,8384,1,1,1,1])
    record=packed(level)
    return {**{f'LEVEL{i:03}.DAT':record*8 for i in range(10)},f'GROUND{theme}O.DAT':bytes(ground),
            f'VGAGR{theme}.DAT':packed(bytes([255,0,0,255]))+packed(objects),'MAIN.DAT':main,
            'ODDTABLE.DAT':bytes(level[:24]+level[2016:])*80}
