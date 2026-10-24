"""Lossless, content-addressed Web transport; desktop packs remain unchanged."""
import gzip
import hashlib
import io
import json
import struct
import zlib

MAX_BYTES=128*1024*1024

def require(ok,message):
    if not ok:raise ValueError(message)

def parse(data):
    require(64<=len(data)<=MAX_BYTES and data[:8]==b'LHDPAK01','Invalid Web HD page')
    atlases,count=struct.unpack_from('<HH',data,8)
    require(atlases==1 and 1<=count<=4096,'Invalid Web HD page count')
    width,height,size,checksum,cutoff=struct.unpack_from('<IIIII',data,12)
    require(1<=width<=4096 and 1<=height<=4096 and size==width*height*4 and cutoff<=254
            and 32+size+count*32==len(data),'Invalid Web HD page bounds')
    pixels=data[32:32+size]
    require(zlib.crc32(pixels)==checksum,'Invalid Web HD pixels')
    rows=[]
    for at in range(32+size,len(data),32):
        row=struct.unpack_from('<BBHHHIIIIHHhh',data,at)
        kind,slot,frame,atlas,reserved,x,y,w,h,dw,dh,ax,ay=row
        require(kind<=4 and atlas==reserved==0 and w>0 and h>0 and x+w<=width and y+h<=height
                and 1<=dw<=4096 and 1<=dh<=4096,'Invalid Web HD frame')
        rows.append(row)
    return width,height,cutoff,pixels,rows

def split_materials(data):
    width,height,cutoff,pixels,rows=parse(data)
    if len(rows)==1 or any(row[0]!=1 for row in rows):return [data]
    parts=[]
    for row in rows:
        x,y,w,h=row[5:9]
        left,top,right,bottom=max(0,x-1),max(0,y-1),min(width,x+w+1),min(height,y+h+1)
        pw,ph=right-left,bottom-top
        rgba=b''.join(pixels[(py*width+left)*4:(py*width+right)*4] for py in range(top,bottom))
        frame=list(row);frame[5:7]=[x-left,y-top]
        parts.append(b'LHDPAK01'+struct.pack('<HHIIIII',1,1,pw,ph,len(rgba),zlib.crc32(rgba),cutoff)
                     +rgba+struct.pack('<BBHHHIIIIHHhh',*frame))
    return parts

def delivery(members):
    require('catalog.hdi' in members,'Missing canonical HD catalog')
    files={};pages=[];seen=set()
    for name,data in sorted(members.items()):
        if name=='catalog.hdi':continue
        for raw in split_materials(data):
            keys=[list(row[:3]) for row in parse(raw)[4]]
            for key in keys:
                require(tuple(key) not in seen,'Duplicate Web HD key');seen.add(tuple(key))
            compressed=io.BytesIO()
            with gzip.GzipFile(filename='',mode='wb',fileobj=compressed,compresslevel=9,mtime=0) as stream:stream.write(raw)
            wire=compressed.getvalue();digest=hashlib.sha256(wire).hexdigest()
            path=f'web-hd/{digest}.hdp.gz';files[path]=wire
            pages.append(dict(path=path,download_bytes=len(wire),sha256=digest,wire_crc32=zlib.crc32(wire),bytes=len(raw),crc32=zlib.crc32(raw),keys=keys))
    require(1<=len(pages)<=128 and sum(p['bytes'] for p in pages)<=512*1024*1024,'Web HD transport bounds')
    info=dict(schema_version=1,codec='gzip',catalog_sha256=hashlib.sha256(members['catalog.hdi']).hexdigest(),
              logical_keys=len(seen),actor_frames=sum(k[0]==0 for k in seen),glyphs=95,pages=pages)
    info['transport_sha256']=hashlib.sha256(json.dumps(info,sort_keys=True,separators=(',',':')).encode()).hexdigest()
    files['HD-WEB.json']=(json.dumps(info,sort_keys=True,indent=2)+'\n').encode()
    require(len(files['HD-WEB.json'])<=128*1024,'Web HD metadata size limit')
    return files


def write_delivery(files, output):
    """Remove only obsolete generated hash pages after the new delivery is written."""
    import re
    for name, content in files.items():
        path=output/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(content)
    folder=output/'web-hd'
    if folder.exists():
        for path in folder.iterdir():
            if re.fullmatch(r'[a-f0-9]{64}\.hdp\.gz',path.name) and 'web-hd/'+path.name not in files and path.is_file() and not path.is_symlink():
                path.unlink()
