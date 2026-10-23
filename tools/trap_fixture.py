"""Invented effect4 courses: two trap instances after one invisible non-trap.

Only documented GROUND dimensions/count/trigger/sound metadata is reused.
No original level or graphic bytes. Trap indices1/2 differ from the object ID.
"""
import struct
from .web_fixture import packed
from .portal_fixture import bank as portal_bank

# theme,id,count,width,height,(left,top,width,height),sound.
PROFILES=((0,6,15,16,21,(8,16,4,8),15),
          (0,8,17,16,38,(4,36,4,4),7),
          (0,10,12,32,42,(12,36,4,4),14),
          (2,8,15,32,34,(12,28,8,4),7),
          (3,8,37,16,41,(4,32,4,4),9),
          (3,9,7,16,26,(4,16,4,4),7),
          (3,10,7,16,26,(8,16,4,4),7),
          (4,7,25,32,19,(12,8,4,4),9),
          (4,9,8,16,71,(4,36,8,8),6),
          (4,10,16,32,15,(12,8,8,4),6))

def bank(theme,id):
    profile=next((p for p in PROFILES if p[:2]==(theme,id)),None)
    if profile is None:raise ValueError('unsupported mechanical trap')
    _,_,count,width,height,(left,top,tw,th),sound=profile
    files=portal_bank(theme);level=bytearray(2048)
    for i,value in enumerate([50,4,4,1,0,0,0,0,0,0,0,0]):struct.pack_into('>H',level,i*2,value)
    struct.pack_into('>H',level,26,theme)
    for slot,x,y,oid in [(0,64,40,1),(1,240,40,2),(2,112-left,100-top,id),(3,176-left,100-top,id)]:
        level[32+slot*8:40+slot*8]=struct.pack('>HHBBBB',x,y,0,oid,0,0)
    level[288:1888]=b'\xff'*1600
    for i in range(32):level[288+i*4:292+i*4]=struct.pack('>HBB',16+i*8,53,0)
    level[2016:]=b'TRAP FIXTURE'.ljust(32,b' ')
    ground=bytearray(1056);objects=bytearray()
    for oid,w,h,nframes,first,flags in [(1,48,25,10,1,3),(2,8,8,1,0,0),(id,width,height,count,0,1)]:
        n=w*h//8;offset=oid*28
        struct.pack_into('<HBBBBHH',ground,offset,flags,first,nframes,w,h,n*5,n*4)
        struct.pack_into('<H',ground,offset+21,len(objects))
        if oid==id:
            struct.pack_into('<HH',ground,offset+14,left//4,(top+4)//4)
            ground[offset+18:offset+21]=bytes([tw//4,th//4,4]);ground[offset+27]=sound
        elif oid==1:ground[offset+18:offset+20]=bytes([1,1])
        objects.extend((bytes(n*5) if oid==2 else b'\xff'*n+b'\0'*(n*3)+b'\xff'*n)*nframes)
    ground[448:454]=bytes([8,1,0,0,3,0]);ground[984:987]=bytes([24,30,34])
    files[f'GROUND{theme}O.DAT']=bytes(ground);files[f'VGAGR{theme}.DAT']=packed(bytes([255,0,0,255]))+packed(objects)
    record=packed(level);files.update({f'LEVEL{i:03}.DAT':record*8 for i in range(10)});files['ODDTABLE.DAT']=bytes(level[:24]+level[2016:])*80
    return files
