#!/usr/bin/env python3
"""Split immutable v1 atlas pages for bounded, level-scoped host loading."""
import hashlib,json,struct,tempfile,zlib
from pathlib import Path
MAX_BYTES=128*1024*1024

def require(ok,message):
    if not ok:raise ValueError(message)

def valid_scope(scope):
    return type(scope) is int and scope&1 and scope&~1023==0 and (scope&62).bit_count()==1 and (scope&960).bit_count()<=1

def split_pack(data):
    require(data[:8]==b'LHDPAK01' and len(data)<=MAX_BYTES,'v1 source required')
    count,frames=struct.unpack_from('<HH',data,8);require(1<=count<=32 and 1<=frames<=4096,'source limits');at=12;pages=[]
    for _ in range(count):
        start=at;w,h,size,crc,cutoff=struct.unpack_from('<IIIII',data,at);at+=20
        require(0<w<=4096 and 0<h<=4096 and size==w*h*4 and at+size<=len(data),'atlas bounds')
        require(zlib.crc32(data[at:at+size])==crc,'atlas checksum');at+=size;pages.append(data[start:at])
    require(at+frames*32==len(data),'frame bounds');groups=[[] for _ in pages];masks=[0]*count
    for offset in range(at,len(data),32):
        row=bytearray(data[offset:offset+32]);kind,slot,frame,index=struct.unpack_from('<BBHH',row)
        require(index<count,'atlas reference')
        if kind in (0,3):scope=1
        elif kind in (1,2):
            theme=slot if kind==1 else slot//16;require(theme<5,'theme');scope=1<<(theme+1)
        elif kind==4:
            require(slot<4 and frame==0,'special');scope=1<<(6+slot)
        else:raise ValueError('unsupported asset kind')
        masks[index]|=scope;struct.pack_into('<H',row,4,0);groups[index].append(row)
    files={};catalog=bytearray(b'LHDIDX01'+struct.pack('<HH',count,0))
    for index,(page,rows,mask) in enumerate(zip(pages,groups,masks)):
        require(rows,'unreferenced atlas');part=b'LHDPAK01'+struct.pack('<HH',1,len(rows))+page+b''.join(rows)
        files[f'atlas-{index:03}.hdp']=part;catalog+=struct.pack('<HHII',mask,0,len(part),zlib.crc32(part))
    files['catalog.hdi']=bytes(catalog);return files

def select_parts(files,scope):
    require(valid_scope(scope),'invalid level scope');data=files.get('catalog.hdi',b'');require(data[:8]==b'LHDIDX01' and len(data)>=12,'catalog magic')
    count,reserved=struct.unpack_from('<HH',data,8);require(1<=count<=128 and reserved==0 and len(data)==12+count*12,'catalog bounds')
    result=[];total=12;catalog_total=0
    for index in range(count):
        mask,reserved,size,crc=struct.unpack_from('<HHII',data,12+index*12);require(0<mask<=1023 and reserved==0 and 64<=size<=MAX_BYTES,'catalog entry');catalog_total+=size
        if not mask&scope:continue
        part=files.get(f'atlas-{index:03}.hdp',b'');require(len(part)==size and zlib.crc32(part)==crc,'missing or corrupt selected atlas');result.append(part);total+=4+size
    require(catalog_total<=512*1024*1024 and 1<=len(result)<=32 and total<=MAX_BYTES,'selection limits');return result

def split_packs(packs):
    files={};entries=[];keys=set();total=0
    for data in packs:
        batch=split_pack(data);catalog=batch['catalog.hdi']
        for offset in range(12,len(catalog),12):
            local=(offset-12)//12;part=batch[f'atlas-{local:03}.hdp'];frame_count=struct.unpack_from('<H',part,10)[0]
            for at in range(len(part)-frame_count*32,len(part),32):
                key=bytes(part[at:at+4]);require(key not in keys,'duplicate catalog key');keys.add(key)
            index=len(entries);require(index<128,'catalog atlas count');total+=len(part);require(total<=512*1024*1024,'catalog size limit')
            files[f'atlas-{index:03}.hdp']=part;entries.append(catalog[offset:offset+12])
    require(entries,'empty catalog');files['catalog.hdi']=b'LHDIDX01'+struct.pack('<HH',len(entries),0)+b''.join(entries)
    for theme in range(5):
        for special in [0,64,128,256,512]:select_parts(files,1|(1<<(theme+1))|special)
    return files

def runtime_catalog(root,base):
    try:from .hd_assets import read_pin,compile_pack
    except ImportError:from hd_assets import read_pin,compile_pack
    manifest_bytes=(root/'assets/manifests/remaster_pack.json').read_bytes()
    manifest=json.loads(manifest_bytes)
    extensions=manifest.get('catalog_extensions',[])
    require(isinstance(extensions,list) and len(extensions)<=15,'catalog extensions')
    identity=hashlib.sha256(base+manifest_bytes+Path(__file__).read_bytes()+Path(__file__).with_name("hd_assets.py").read_bytes())
    sources=[]
    for pin in extensions:
        raw=read_pin(pin,root);identity.update(raw);source=json.loads(raw)
        # Source verification remains mandatory on a cache hit.
        for asset in source['atlases']+source.get('provenance',[]):read_pin(asset,root)
        sources.append(source)
    directory=root/'assets/local/hd-catalog';meta=directory/'validation.json'
    try:
        report=json.loads(meta.read_text());require(report['source']==identity.hexdigest(),'stale catalog cache')
        files={}
        require(len(report['files'])<=129,'cached catalog limit')
        for pin in report['files']:
            name=pin['path'];require(name=='catalog.hdi' or (len(name)==13 and name.startswith('atlas-') and name[6:9].isdigit() and name.endswith('.hdp')),'cached catalog path')
            path=directory/name;require(path.stat().st_size==pin['bytes']<=MAX_BYTES,'cached member size')
            data=path.read_bytes();require(hashlib.sha256(data).hexdigest()==pin['sha256'],'cached checksum');files[name]=data
        for theme in range(5):
            for special in [0,64,128,256,512]:select_parts(files,1|(1<<(theme+1))|special)
        require(len(files)==1+struct.unpack_from('<H',files['catalog.hdi'],8)[0],'cached inventory')
        return files
    except (OSError,ValueError,KeyError,TypeError,struct.error):pass
    files=split_packs([base]+[compile_pack(source,root=root)[0] for source in sources])
    directory.mkdir(parents=True,exist_ok=True)
    for name,data in files.items():
        with tempfile.NamedTemporaryFile(dir=directory,delete=False) as stream:
            stream.write(data);temporary=Path(stream.name)
        temporary.replace(directory/name)
    report=dict(source=identity.hexdigest(),files=[dict(path=n,bytes=len(b),sha256=hashlib.sha256(b).hexdigest()) for n,b in sorted(files.items())])
    with tempfile.NamedTemporaryFile(dir=directory,delete=False) as stream:
        stream.write((json.dumps(report)+'\n').encode());temporary=Path(stream.name)
    temporary.replace(meta)
    return files

def write_files(files,output):
    output.mkdir(parents=True,exist_ok=True)
    for name,part in files.items():(output/name).write_bytes(part)
    # Delete only stale generated atlas members, never arbitrary user files.
    for path in output.glob('atlas-[0-9][0-9][0-9].hdp'):
        if path.name not in files:path.unlink()
    return files

def write_catalog(data,output):
    return write_files(split_packs([data]),output)
