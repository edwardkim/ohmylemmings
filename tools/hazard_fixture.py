"""Invented hazard courses with GROUND effect5/6 metadata pinned in hd_hazards.json.

No original image/level bytes. One actor walks from the entrance into a hazard;
the renderer uses actual theme/object keys while simulation retains disk triggers.
"""
import struct
from .web_fixture import packed
from .portal_fixture import bank as portal_bank

# theme,id,count,width,height,effect,(left,top,width,height) in local pixels.
PROFILES=((0,5,8,64,16,5,(0,4,64,8)),
          (1,5,8,64,20,5,(0,8,64,8)),
          (1,7,8,32,17,6,(4,8,24,8)),
          (1,8,10,80,20,6,(0,16,60,4)),
          (1,10,10,80,20,6,(20,16,56,4)),
          (2,5,8,64,25,5,(0,4,64,16)),
          (2,9,16,32,19,6,(4,12,24,12)),
          (3,5,8,64,16,5,(0,4,64,8)),
          (4,6,8,64,16,5,(0,4,64,8)))

def bank(theme,id):
    profile=next((p for p in PROFILES if p[:2]==(theme,id)),None)
    if profile is None:raise ValueError('unsupported hazard profile')
    _,_,count,width,height,effect,(left,top,tw,th)=profile
    files=portal_bank(theme)
    level=bytearray(2048)
    for i,value in enumerate([50,1,1,1,0,0,0,0,0,0,0,0]):struct.pack_into('>H',level,i*2,value)
    struct.pack_into('>H',level,26,theme)
    level[32:40]=struct.pack('>HHBBBB',64,40,0,1,0,0)
    level[40:48]=struct.pack('>HHBBBB',112-left,100-top,0,id,0,0)
    level[288:1888]=b'\xff'*1600
    for i in range(32):level[288+i*4:292+i*4]=struct.pack('>HBB',16+i*8,53,0)
    level[2016:]=b'HAZARD FIXTURE'.ljust(32,b' ')
    ground=bytearray(1056);objects=bytearray()
    for oid,w,h,nframes,first,flags in [(1,48,25,10,1,3),(id,width,height,count,0,2)]:
        n=w*h//8;offset=oid*28
        struct.pack_into('<HBBBBHH',ground,offset,flags,first,nframes,w,h,n*5,n*4)
        struct.pack_into('<H',ground,offset+21,len(objects))
        if oid==id:
            struct.pack_into('<HH',ground,offset+14,left//4,(top+4)//4)
            ground[offset+18:offset+21]=bytes([tw//4,th//4,effect])
        else:ground[offset+18:offset+20]=bytes([1,1])
        objects.extend((b'\xff'*n+b'\0'*(n*3)+b'\xff'*n)*nframes)
    ground[448:454]=bytes([8,1,0,0,3,0]);ground[984:987]=bytes([24,30,34])
    files[f'GROUND{theme}O.DAT']=bytes(ground)
    files[f'VGAGR{theme}.DAT']=packed(bytes([255,0,0,255]))+packed(objects)
    record=packed(level)
    files.update({f'LEVEL{i:03}.DAT':record*8 for i in range(10)})
    files['ODDTABLE.DAT']=bytes(level[:24]+level[2016:])*80
    return files
