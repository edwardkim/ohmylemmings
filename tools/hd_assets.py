#!/usr/bin/env python3
"""Validate generated HD PNGs and build a deterministic, pixel-preserving RGBA pack."""
import argparse
from functools import lru_cache
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import struct
import tempfile
import zlib

ROOT = Path(__file__).resolve().parents[1]
MAX_BYTES = 128 * 1024 * 1024
MAX_ATLASES = 32
MAX_PNG = 32 * 1024 * 1024
MAX_DIM = 4096
ICON_NAMES = ('climber', 'floater', 'bomber', 'blocker', 'builder', 'basher',
              'miner', 'digger', 'slower', 'faster', 'pause', 'nuke', 'restart')


class AssetError(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise AssetError(message)


def integer(value, low, high):
    require(type(value) is int and low <= value <= high, 'invalid integer/range')
    return value


def numbers(value, count, low, high):
    require(isinstance(value, list) and len(value) == count, 'invalid coordinate array')
    return [integer(n, low, high) for n in value]


def decode_png(data):
    """Decode noninterlaced RGB/RGBA8; preserve encoded RGB and straight alpha."""
    require(len(data) <= MAX_PNG and data[:8] == b'\x89PNG\r\n\x1a\n', 'PNG signature/size')
    pos = 8; header = None; compressed = bytearray(); ended = False; idat_ended = False; chunks = 0
    seen_idat = False; seen_palette = False
    while pos < len(data):
        require(pos + 12 <= len(data), 'truncated PNG chunk')
        size = struct.unpack_from('>I', data, pos)[0]
        require(size <= MAX_PNG and pos + 12 + size <= len(data), 'PNG chunk size')
        kind = data[pos+4:pos+8]; body = data[pos+8:pos+8+size]
        crc = struct.unpack_from('>I', data, pos+8+size)[0]
        require(zlib.crc32(kind + body) == crc, 'PNG CRC')
        require(all(65 <= b <= 90 or 97 <= b <= 122 for b in kind) and not kind[2] & 32, 'PNG chunk name')
        chunks += 1
        require(chunks <= 4096, 'too many PNG chunks')
        if header is None:
            require(kind == b'IHDR' and size == 13, 'PNG must start with IHDR')
            header = struct.unpack('>IIBBBBB', body)
            w,h,depth,color,compression,filter_method,interlace = header
            require(0 < w <= MAX_DIM and 0 < h <= MAX_DIM and depth == 8 and color in [2,6]
                    and (compression,filter_method,interlace) == (0,0,0), 'unsupported PNG format/dimensions')
        elif kind == b'IDAT':
            require(not idat_ended, 'noncontiguous IDAT')
            seen_idat = True
            compressed.extend(body)
        elif kind == b'IEND':
            require(size == 0 and bool(compressed), 'invalid IEND')
            pos += size + 12; ended = True; break
        else:
            require(kind != b'IHDR' and (kind[0] & 32 or kind == b'PLTE'), 'unknown/duplicate critical PNG chunk')
            if kind == b'PLTE':
                require(not seen_palette and not seen_idat and 0 < size <= 768 and size % 3 == 0, 'invalid PLTE')
                seen_palette = True
            if seen_idat: idat_ended = True
        pos += size + 12
    require(ended and pos == len(data), 'missing IEND/trailing PNG bytes')
    channels = 4 if color == 6 else 3
    stride = w * channels; expected = (stride + 1) * h
    try:
        decoder = zlib.decompressobj()
        raw = decoder.decompress(compressed, expected + 1)
    except zlib.error as error:
        raise AssetError('PNG zlib') from error
    require(len(raw) == expected and decoder.eof and not decoder.unused_data and not decoder.unconsumed_tail,
            'PNG inflated size/stream')
    rgba = bytearray(); previous = bytearray(stride)
    for y in range(h):
        offset = y * (stride + 1); mode = raw[offset]
        require(mode <= 4, 'PNG filter')
        row = bytearray(raw[offset+1:offset+1+stride])
        if mode:
            for i in range(stride):
                left = row[i-channels] if i >= channels else 0
                up = previous[i]; upper_left = previous[i-channels] if i >= channels else 0
                if mode == 1: prediction = left
                elif mode == 2: prediction = up
                elif mode == 3: prediction = (left + up) // 2
                else:
                    p = left + up - upper_left
                    a,b,c = abs(p-left),abs(p-up),abs(p-upper_left)
                    prediction = left if a <= b and a <= c else up if b <= c else upper_left
                row[i] = (row[i] + prediction) & 255
        if channels == 4: rgba.extend(row)
        else:
            for i in range(0,stride,3): rgba.extend(row[i:i+3]); rgba.append(255)
        previous = row
    return w,h,bytes(rgba)


def read_pin(pin, root):
    require(isinstance(pin,dict), 'asset record')
    name = pin.get('path')
    require(isinstance(name,str) and bool(name) and '\\' not in name and ':' not in name, 'asset path')
    relative = PurePosixPath(name)
    require(not relative.is_absolute() and '..' not in relative.parts, 'asset path escape')
    integer(pin.get('bytes'),1,MAX_PNG)
    try:
        root = root.resolve(); path = (root / name).resolve()
        require(path.is_relative_to(root), 'asset symlink escape')
        size = path.stat().st_size
        require(size <= MAX_PNG and size == pin.get('bytes'), 'asset byte size')
        with path.open('rb') as stream: data = stream.read(MAX_PNG+1)
        require(len(data) == size, 'asset size changed')
    except (OSError,RuntimeError) as error:
        raise AssetError(f'missing/unreadable asset: {name}') from error
    require(hashlib.sha256(data).hexdigest() == pin.get('sha256'), 'asset SHA-256')
    return data


@lru_cache(maxsize=1)
def source_counts():
    metadata = json.loads((ROOT/'assets/manifests/lemming_sprites.json').read_text())['animations']
    require(len(metadata) == 28 and [r['slot'] for r in metadata] == list(range(28)), 'source animation metadata')
    return [row['frame_count'] for row in metadata]


EXIT_EFFECT_PROFILES=((0,7),(1,6),(2,6),(3,6),(4,8))
# GROUND0O..4O effect5/6 disk frames; exact records pinned in hd_hazards.json.
HAZARD_PROFILES={(0,5):8,(1,5):8,(1,7):8,(1,8):10,(1,10):10,
                 (2,5):8,(2,9):16,(3,5):8,(4,6):8}

TRAP_PROFILES={(0,6):15,(0,8):17,(0,10):12,(2,8):15,(3,8):37,
               (3,9):7,(3,10):7,(4,7):25,(4,9):8,(4,10):16}

# Remaining original GROUND objects: green/blue pennants and one-way arrows.
DECOR_PROFILES={(s,i):14 for s,ids in [(0,[2,9]),(1,[2,9]),(2,[2,7]),(3,[2,7]),(4,[2,3])] for i in ids}
DECOR_PROFILES.update({(s,i):8 if s==1 else 7 for s in range(5) for i in ([4,5] if s==4 else [3,4])})

def key_fields(key):
    require(isinstance(key,dict), 'frame key')
    if set(key) == {'actor'}:
        slot,frame = numbers(key['actor'],2,0,65535)
        require(slot < len(source_counts()) and frame < source_counts()[slot], 'source slot/frame')
        return 0,slot,frame
    if set(key) == {'object'}:
        theme,id,frame = numbers(key['object'],3,0,65535)
        count = DECOR_PROFILES.get((theme,id),TRAP_PROFILES.get((theme,id)))
        if count is None:count = HAZARD_PROFILES.get((theme,id), 6 if (theme,id) in EXIT_EFFECT_PROFILES else {0:1, 1:10}.get(id) if theme < 5 else None)
        require(count is not None and frame < count, 'unsupported object/frame')
        return 2,theme*16+id,frame
    if set(key) == {'icon'}:
        require(isinstance(key['icon'],str) and key['icon'] in ICON_NAMES, 'unknown icon')
        return 3,ICON_NAMES.index(key['icon']),0
    if set(key) == {'special'}:
        require(type(key['special']) is int and 0 <= key['special'] < 4, 'unknown special')
        return 4,key['special'],0
    require(set(key) == {'material'} and type(key['material']) is int and 0 <= key['material'] < 5, 'unknown material/key')
    return 1,key['material'],0


def sprite_bounds(rgba, width, rect, cutoff=0):
    x,y,w,h = rect
    bounds = [w,h,-1,-1]
    for row in range(h):
        for col in range(w):
            alpha = rgba[((y+row)*width+x+col)*4+3]
            if alpha > cutoff:
                require(row not in [0,h-1] and col not in [0,w-1], 'sprite touches atlas cell edge')
                bounds = [min(bounds[0],col),min(bounds[1],row),max(bounds[2],col),max(bounds[3],row)]
    require(bounds[2] >= 0, 'empty sprite frame')
    return bounds


def actor_coverage(frames):
    """Count original logical actor keys, independent of atlas packing/layout."""
    actors=set()
    for entry in frames:
        require(isinstance(entry,dict), 'frame record')
        kind,slot,frame=key_fields(entry.get('key'))
        if kind==0:actors.add((slot,frame))
    counts=source_counts()
    missing=[[slot,frame] for slot,count in enumerate(counts) for frame in range(count) if (slot,frame) not in actors]
    return dict(animations=sum(all((slot,frame) in actors for frame in range(count)) for slot,count in enumerate(counts)),frames=len(actors),missing=missing)


def portal_coverage(frames):
    """Original five GROUND sets have one exit and ten entrance disk frames."""
    found=set()
    for entry in frames:
        require(isinstance(entry,dict), 'frame record')
        kind,slot,frame=key_fields(entry.get('key'))
        if kind==2 and slot%16<=1:found.add((slot//16,slot%16,frame))
    missing=[[theme,id,frame] for theme in range(5) for id,count in [(0,1),(1,10)]
             for frame in range(count) if (theme,id,frame) not in found]
    complete=sum(not any(row[0]==theme for row in missing) for theme in range(5))
    return dict(sets=complete,frames=len(found),missing=missing)


def exit_effect_coverage(frames):
    """Pure decorative exit flames, separate from hazard coverage."""
    found=set()
    for entry in frames:
        require(isinstance(entry,dict), 'frame record')
        kind,slot,frame=key_fields(entry.get('key'))
        if kind==2 and (slot//16,slot%16) in EXIT_EFFECT_PROFILES:
            found.add((slot//16,slot%16,frame))
    missing=[[theme,id,frame] for theme,id in EXIT_EFFECT_PROFILES
             for frame in range(6) if (theme,id,frame) not in found]
    return dict(frames=len(found),missing=missing)


def hazard_coverage(frames):
    """Effect5 water and effect6 lethal effects, retaining original disk counts."""
    found=set()
    for entry in frames:
        require(isinstance(entry,dict), 'frame record')
        kind,slot,frame=key_fields(entry.get('key'))
        if kind==2 and (slot//16,slot%16) in HAZARD_PROFILES:
            found.add((slot//16,slot%16,frame))
    missing=[[theme,id,frame] for (theme,id),count in HAZARD_PROFILES.items()
             for frame in range(count) if (theme,id,frame) not in found]
    return dict(frames=len(found),missing=missing)


def trap_coverage(frames):
    """Triggered effect4 frames, separate from continuously looping hazards."""
    found=set()
    for entry in frames:
        require(isinstance(entry,dict), 'frame record')
        kind,slot,frame=key_fields(entry.get('key'))
        if kind==2 and (slot//16,slot%16) in TRAP_PROFILES:
            found.add((slot//16,slot%16,frame))
    missing=[[theme,id,frame] for (theme,id),count in TRAP_PROFILES.items()
             for frame in range(count) if (theme,id,frame) not in found]
    return dict(frames=len(found),missing=missing)


def decor_coverage(frames):
    """All140 flag and72 arrow disk keys, including the8-frame fire arrows."""
    found=set()
    for entry in frames:
        require(isinstance(entry,dict), 'frame record')
        kind,slot,frame=key_fields(entry.get('key'))
        if kind==2 and (slot//16,slot%16) in DECOR_PROFILES:
            found.add((slot//16,slot%16,frame))
    missing=[[s,i,f] for (s,i),count in DECOR_PROFILES.items()
             for f in range(count) if (s,i,f) not in found]
    return dict(frames=len(found),missing=missing)


def pack_rectangles(width, height, rgba, rectangles):
    """Copy referenced rectangles exactly, with one sample-extruded gutter.

    Each source atlas retains its index/cutoff. Only runtime coordinates change;
    source PNGs, dimensions, anchors and every referenced RGBA sample stay intact.
    Keep the original atlas whenever shelves would use more space.
    """
    original={rect:rect for rect in rectangles}
    if not rectangles or any(w+2 > MAX_DIM or h+2 > MAX_DIM for x,y,w,h in rectangles):
        return width,height,rgba,original
    ordered=sorted(rectangles,key=lambda r:(-r[3],-r[2],r[1],r[0]))
    minimum=max(r[2]+2 for r in ordered)
    candidates=sorted({minimum,width,MAX_DIM,*range(((minimum+63)//64)*64,MAX_DIM+1,64)})
    best=None
    for limit in candidates:
        if limit < minimum:continue
        shelves=[];positions={};used_width=0;used_height=0
        for rect in ordered:
            x,y,w,h=rect;pw,ph=w+2,h+2
            fits=[s for s in shelves if ph <= s[2] and s[0]+pw <= limit]
            if fits:
                shelf=min(fits,key=lambda s:(s[2]-ph,limit-s[0]-pw,s[1]))
            else:
                shelf=[0,used_height,ph];shelves.append(shelf);used_height+=ph
            positions[rect]=(shelf[0]+1,shelf[1]+1,w,h)
            shelf[0]+=pw;used_width=max(used_width,shelf[0])
        if used_height > MAX_DIM:continue
        score=(used_width*used_height,used_width,used_height)
        if best is None or score < best[0]:best=(score,positions)
    if best is None or best[0][0] >= width*height:return width,height,rgba,original
    (_,packed_width,packed_height),positions=best
    pixels=bytearray(packed_width*packed_height*4)
    for (sx,sy,w,h),(dx,dy,_,_) in positions.items():
        for row in range(-1,h+1):
            start=((sy+min(max(row,0),h-1))*width+sx)*4
            samples=rgba[start:start+w*4]
            target=((dy+row)*packed_width+dx-1)*4
            pixels[target:target+(w+2)*4]=samples[:4]+samples+samples[-4:]
    return packed_width,packed_height,bytes(pixels),positions


def compact_layout(rectangles, margins=None):
    """Keep complete frame extents; share only protected transparent borders."""
    require(bool(rectangles), 'empty compact page')
    margins=margins or {}
    ordered=sorted(rectangles,key=lambda key:(-rectangles[key][1],-rectangles[key][0],key))
    require(all(0<w<=MAX_DIM and 0<h<=MAX_DIM for w,h in rectangles.values()), 'compact rectangle dimensions')
    large=[key for key in ordered if max(rectangles[key])+2>MAX_DIM]
    ordered=[key for key in ordered if key not in large]
    fixed=[(rectangles[k][0],rectangles[k][1],{k:(0,0,*rectangles[k])}) for k in large]
    if not ordered:return fixed
    minimum=max(rectangles[key][0]+2 for key in ordered)
    best=None
    for limit in sorted({minimum,MAX_DIM,*range(((minimum+63)//64)*64,MAX_DIM+1,64)}):
        pages=[]
        for key in ordered:
            w,h=rectangles[key];pw,ph=w+2,h+2;left,top,right,bottom=margins.get(key,(0,0,0,0))
            fits=[(s['height']-ph,limit-s['cursor']-pw+min(s['right'],left),i,j) for i,p in enumerate(pages) for j,s in enumerate(p['shelves']) if ph<=s['height'] and s['cursor']+pw-min(s['right'],left)<=limit]
            if fits:
                _,_,i,j=min(fits);page=pages[i];shelf=page['shelves'][j]
            else:
                page=next((p for p in pages if p['height']+ph<=MAX_DIM),None)
                if page is None:
                    page=dict(width=0,height=0,shelves=[],positions={});pages.append(page)
                shelf=dict(cursor=0,y=page['height'],height=ph,right=0,top=MAX_DIM,bottom=MAX_DIM,keys=[])
                page['shelves'].append(shelf);page['height']+=ph
            overlap=min(shelf['right'],left);x=shelf['cursor']+1-overlap
            page['positions'][key]=(x,shelf['y']+1,w,h)
            shelf['keys'].append(key);shelf['cursor']=x+w+1;shelf['right']=right
            shelf['top']=min(shelf['top'],top);shelf['bottom']=min(shelf['bottom'],shelf['height']-ph+bottom)
            page['width']=max(page['width'],shelf['cursor'])
        result=list(fixed)
        for page in pages:
            height=0;previous=None
            for shelf in page['shelves']:
                if previous is not None:height-=min(previous['bottom'],shelf['top'])
                shift=shelf['y']-height
                for key in shelf['keys']:
                    x,y,w,h=page['positions'][key];page['positions'][key]=(x,y-shift,w,h)
                height+=shelf['height'];previous=shelf
            result.append((page['width'],height,page['positions']))
        score=(sum(w*h for w,h,_ in result),len(result),limit)
        if best is None or score<best[0]:best=(score,result)
    return best[1]


def compile_compact(atlases, frames, records, dimensions, root):
    regions={};sources=[];transforms=[None]*len(records);bounds=[];scratch=0;fixed=[];locked_sources=[]
    for index,atlas in enumerate(atlases):
        width,height,rgba=decode_png(read_pin(atlas,root))
        require((width,height)==dimensions[index], 'atlas dimensions')
        cutoff=atlas.get('alpha_cutoff',0);alpha=rgba[3::4]
        if atlas['alpha']=='opaque':require(all(a==255 for a in alpha), 'opaque atlas has transparency')
        else:require(min(alpha)<=cutoff<max(alpha), 'sprite atlas must have transparency and visible ink')
        sources.append(dict(width=width,height=height,alpha_cutoff=cutoff,rgba_sha256=hashlib.sha256(rgba).hexdigest()))
        require(type(atlas.get('sampling_locked',False)) is bool, 'sampling lock policy')
        locked=atlas.get('sampling_locked',False)
        source_rects={record[2] for record in records if record[1]==index}
        if locked and source_rects:
            fw,fh,fp,fm=pack_rectangles(width,height,rgba,source_rects)
            fixed_page=len(fixed);fixed.append((fw,fh,cutoff,None,fp));locked_sources.append(index);scratch+=len(fp)
            require(scratch<=MAX_BYTES, 'compact region size limit')
        cached={}
        for number,(_,source,rect,display,anchor) in enumerate(records):
            if source!=index:continue
            rect2,display2,anchor2=rect,display,anchor;protected=[0,0,0,0]
            if atlas['alpha']=='sprite':
                if rect not in cached:cached[rect]=sprite_bounds(rgba,width,rect,cutoff)
                box=cached[rect];bounds.append(dict(key=frames[number]['key'],bbox=box))
                protected=[max(0,box[0]-2),max(0,box[1]-2),max(0,rect[2]-box[2]-3),max(0,rect[3]-box[3]-3)]
            if locked:
                transforms[number]=dict(key=frames[number]['key'],source_atlas=index,source_rect=list(rect),source_display=display,source_anchor=anchor,shared_margins=[0,0,0,0],source_region=list(rect),display=display,anchor=anchor,fixed_page=fixed_page,rect=list(fm[rect]))
                continue
            token=(index,rect2)
            if token not in regions:
                x,y,w,h=rect2;scratch+=w*h*4
                require(scratch<=MAX_BYTES, 'compact region size limit')
                pixels=b''.join(rgba[((y+row)*width+x)*4:((y+row)*width+x+w)*4] for row in range(h))
                regions[token]=dict(id=len(regions),width=w,height=h,rgba=pixels,cutoff=cutoff,margins=protected)
            transforms[number]=dict(key=frames[number]['key'],source_atlas=index,source_rect=list(rect),source_display=display,source_anchor=anchor,shared_margins=protected,source_region=list(rect2),display=display2,anchor=anchor2,region=regions[token]['id'])
    pages=list(fixed);destinations={};by_id={r['id']:r for r in regions.values()}
    for cutoff in sorted({r['cutoff'] for r in regions.values()}):
        same={r['id']:(r['width'],r['height']) for r in regions.values() if r['cutoff']==cutoff}
        for width,height,positions in compact_layout(same,{key:by_id[key]['margins'] for key in same}):
            pages.append((width,height,cutoff,positions,None))
    require(1<=len(pages)<=MAX_ATLASES, 'runtime atlas count')
    size=12+len(pages)*20+len(records)*32+sum(w*h*4 for w,h,_,_,_ in pages)
    require(size<=MAX_BYTES, 'pack size limit')
    output=bytearray(b'LHDPAK01'+struct.pack('<HH',len(pages),len(records)));atlas_report=[]
    shared=0
    for index,(width,height,cutoff,positions,original) in enumerate(pages):
        if original is not None:
            output.extend(struct.pack('<IIIII',width,height,len(original),zlib.crc32(original),cutoff));output.extend(original)
            atlas_report.append(dict(width=width,height=height,alpha_cutoff=cutoff,rgba_sha256=hashlib.sha256(original).hexdigest(),sampling_locked=True))
            continue
        pixels=bytearray(width*height*4);written=bytearray(width*height)
        for key,(x,y,w,h) in positions.items():
            region=by_id[key];data=region['rgba']
            for row in range(-int(y>0),h+int(y+h<height)):
                source=min(max(row,0),h-1)*w*4;samples=data[source:source+w*4]
                left=int(x>0);right=int(x+w<width);target=((y+row)*width+x-left)*4
                line=(samples[:4] if left else b'')+samples+(samples[-4:] if right else b'')
                first=target//4;count=w+left+right;used=written[first:first+count]
                if any(used):
                    for col,flag in enumerate(used):
                        if flag:
                            require(pixels[target+col*4+3]<=cutoff and line[col*4+3]<=cutoff, 'visible atlas overlap')
                            shared+=flag==1
                        written[first+col]=2 if flag else 1
                else:written[first:first+count]=b'\1'*count
                pixels[target:target+len(line)]=line
            destinations[key]=(index,(x,y,w,h))
        output.extend(struct.pack('<IIIII',width,height,len(pixels),zlib.crc32(pixels),cutoff));output.extend(pixels)
        atlas_report.append(dict(width=width,height=height,alpha_cutoff=cutoff,rgba_sha256=hashlib.sha256(pixels).hexdigest()))
    for (fields,_,_,_,_),transform in zip(records,transforms):
        if 'fixed_page' in transform:index,rect=transform.pop('fixed_page'),transform['rect']
        else:index,rect=destinations[transform.pop('region')]
        transform.update(atlas=index,rect=list(rect))
        output.extend(struct.pack('<BBHHHIIIIHHhh',*fields,index,0,*rect,*transform['display'],*transform['anchor']))
    compaction=dict(sampling_locked_sources=locked_sources,shared_invisible_samples=shared,retained_region_bytes=scratch,frame_transforms=transforms,
                    policy='Source PNGs unchanged. Full source-frame dimensions, display sizes and anchors unchanged. Share only cutoff-invisible border samples, retaining a two-source-pixel guard around visible ink and one sample-extruded page gutter. Equal cutoffs only; sampling-locked sources retain their entire canonical rectangles-v1 page to preserve host filtering phase. Shared invisible filler RGB may differ; visible and protected boundary RGBA stay exact.')
    return bytes(output),dict(atlases=atlas_report,source_atlases=len(atlases),source_images=sources,sprite_bounds=bounds,compaction=compaction)


def compile_pack(manifest, *, root=ROOT):
    require(isinstance(manifest,dict) and type(manifest.get('schema_version')) is int and manifest['schema_version'] == 1, 'manifest schema')
    atlases = manifest.get('atlases'); frames = manifest.get('frames')
    packing=manifest.get('packing','source-v1')
    require(isinstance(atlases,list) and 1 <= len(atlases) <= (128 if packing=='compact-v1' else MAX_ATLASES), 'atlas count')
    require(isinstance(frames,list) and 1 <= len(frames) <= 4096, 'frame count')
    packing=manifest.get('packing','source-v1')
    require(packing in ('source-v1','rectangles-v1','compact-v1'), 'unknown packing policy')
    coverage=actor_coverage(frames)
    require(type(manifest.get('complete_actors',False)) is bool, 'complete actors policy')
    if manifest.get('complete_actors',False):require(not coverage['missing'], 'incomplete actor coverage')
    portals=portal_coverage(frames)
    require(type(manifest.get('complete_portals',False)) is bool, 'complete portals policy')
    if manifest.get('complete_portals',False):require(not portals['missing'], 'incomplete portal coverage')
    effects=exit_effect_coverage(frames)
    require(type(manifest.get('complete_exit_effects',False)) is bool, 'complete exit effects policy')
    if manifest.get('complete_exit_effects',False):require(not effects['missing'], 'incomplete exit effect coverage')
    hazards=hazard_coverage(frames)
    require(type(manifest.get('complete_hazards',False)) is bool, 'complete hazards policy')
    if manifest.get('complete_hazards',False):require(not hazards['missing'], 'incomplete hazard coverage')
    traps=trap_coverage(frames)
    require(type(manifest.get('complete_traps',False)) is bool, 'complete traps policy')
    if manifest.get('complete_traps',False):require(not traps['missing'], 'incomplete trap coverage')
    decor=decor_coverage(frames)
    require(type(manifest.get('complete_decor',False)) is bool, 'complete decor policy')
    if manifest.get('complete_decor',False):require(not decor['missing'], 'incomplete decor coverage')
    provenance = manifest.get('provenance',[])
    require(isinstance(provenance,list) and len(provenance) <= 128, 'provenance list')
    for pin in provenance: read_pin(pin,root)
    dimensions = []
    for atlas in atlases:
        require(isinstance(atlas,dict), 'atlas record')
        dimensions.append((integer(atlas.get('width'),1,MAX_DIM),integer(atlas.get('height'),1,MAX_DIM)))
        integer(atlas.get('alpha_cutoff',0),0,254)
        require(atlas.get('alpha') in ['sprite','opaque'], 'alpha policy')
    seen = set(); bounds = []; records=[]; rectangles=[set() for _ in atlases]
    for entry in frames:
        require(isinstance(entry,dict), 'frame record')
        fields = key_fields(entry.get('key'))
        require(fields not in seen, 'duplicate frame key'); seen.add(fields)
        index = integer(entry.get('atlas'),0,len(atlases)-1)
        x,y,w,h = numbers(entry.get('rect'),4,0,MAX_DIM)
        width,height = dimensions[index]
        require(w > 0 and h > 0 and x+w <= width and y+h <= height, 'frame rectangle')
        display = numbers(entry.get('display'),2,1,MAX_DIM)
        anchor = numbers(entry.get('anchor'),2,-MAX_DIM,MAX_DIM)
        rectangles[index].add((x,y,w,h))
        records.append((fields,index,(x,y,w,h),display,anchor))
    if 'required' in manifest:
        require(isinstance(manifest['required'],list) and len(manifest['required']) <= 4096, 'required keys')
        required = [key_fields(key) for key in manifest['required']]
        require(len(set(required)) == len(required) and set(required) == seen, 'required coverage differs')
    if packing=='compact-v1':
        pack,report=compile_compact(atlases,frames,records,dimensions,root)
        report.update(status='PASS',frames=len(frames),actor_coverage=coverage,portal_coverage=portals,exit_effect_coverage=effects,hazard_coverage=hazards,trap_coverage=traps,decor_coverage=decor,bytes=len(pack),sha256=hashlib.sha256(pack).hexdigest(),packing=packing,pixel_policy=report['compaction']['policy'])
        return pack,report
    output = bytearray(b'LHDPAK01'+struct.pack('<HH',len(atlases),len(frames)))
    atlas_report=[];mappings=[]
    # Decode one source at a time rather than retaining up to32 full PNGs.
    for index,atlas in enumerate(atlases):
        data=read_pin(atlas,root);width,height,rgba=decode_png(data)
        require((width,height)==dimensions[index], 'atlas dimensions')
        alpha=rgba[3::4];cutoff=atlas.get('alpha_cutoff',0)
        if atlas['alpha']=='opaque':require(all(a==255 for a in alpha), 'opaque atlas has transparency')
        else:
            require(min(alpha)<=cutoff<max(alpha), 'sprite atlas must have transparency and visible ink')
            for entry in frames:
                if entry['atlas']==index:bounds.append(dict(key=entry['key'],bbox=sprite_bounds(rgba,width,entry['rect'],cutoff)))
        if packing=='rectangles-v1':w,h,pixels,mapping=pack_rectangles(width,height,rgba,rectangles[index])
        else:w,h,pixels,mapping=width,height,rgba,{r:r for r in rectangles[index]}
        require(len(output) + 20 + len(pixels) + len(frames)*32 <= MAX_BYTES, 'pack size limit')
        output.extend(struct.pack('<IIIII',w,h,len(pixels),zlib.crc32(pixels),cutoff));output.extend(pixels)
        mappings.append(mapping)
        atlas_report.append(dict(width=w,height=h,source_width=width,source_height=height,alpha_cutoff=cutoff,
                                 rgba_sha256=hashlib.sha256(pixels).hexdigest(),source_rgba_sha256=hashlib.sha256(rgba).hexdigest()))
    for fields,index,rect,display,anchor in records:
        output.extend(struct.pack('<BBHHHIIIIHHhh',*fields,index,0,*mappings[index][rect],*display,*anchor))
    pack = bytes(output)
    return pack,dict(status='PASS',atlases=atlas_report,frames=len(frames),actor_coverage=coverage,portal_coverage=portals,exit_effect_coverage=effects,hazard_coverage=hazards,trap_coverage=traps,decor_coverage=decor,sprite_bounds=bounds,
                     bytes=len(pack),sha256=hashlib.sha256(pack).hexdigest(),packing=packing,
                     pixel_policy='unchanged referenced RGB/straight alpha RGBA8; sRGB interpretation; optional lossless rectangle relocation')


def runtime_pack(*,root=ROOT):
    """Pixel-preserving runtime cache; verify sources even on cache hits."""
    manifest_bytes=(root/'assets/manifests/remaster_pack.json').read_bytes()
    manifest=json.loads(manifest_bytes)
    for pin in manifest['atlases']+manifest.get('provenance',[]):read_pin(pin,root)
    identity=hashlib.sha256(manifest_bytes+Path(__file__).read_bytes()).hexdigest()
    directory=root/'assets/local/hd-runtime';path=directory/'assets.hdp';meta=directory/'validation.json'
    try:
        cached=json.loads(meta.read_text())
        if cached['source']==identity and path.stat().st_size==cached['bytes'] and path.stat().st_size<=MAX_BYTES:
            data=path.read_bytes()
            if hashlib.sha256(data).hexdigest()==cached['sha256']:return data
    except (OSError,ValueError,KeyError,TypeError):pass
    data,report=compile_pack(manifest,root=root);directory.mkdir(parents=True,exist_ok=True)
    # Each writer owns its temporary file; concurrent builds produce identical bytes.
    with tempfile.NamedTemporaryFile(dir=directory,delete=False) as stream:
        stream.write(data);temporary=Path(stream.name)
    temporary.replace(path)
    meta.write_text(json.dumps(dict(source=identity,bytes=len(data),sha256=report['sha256']))+'\n')
    return data


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command',choices=['check','build','runtime'])
    parser.add_argument('--manifest',type=Path,default=ROOT/'assets/manifests/remaster_pack.json')
    parser.add_argument('--output',type=Path)
    args = parser.parse_args()
    if args.command=='runtime':
        data=runtime_pack()
        try:from .hd_catalog import runtime_catalog,write_files
        except ImportError:from hd_catalog import runtime_catalog,write_files
        write_files(runtime_catalog(ROOT,data),ROOT/'assets/local/hd-runtime/atlas')
        print(json.dumps(dict(status='PASS',path=str(ROOT/'assets/local/hd-runtime/assets.hdp'),bytes=len(data),sha256=hashlib.sha256(data).hexdigest())))
        return
    manifest = json.loads(args.manifest.read_text())
    pack,report = compile_pack(manifest)
    if args.command == 'build':
        if not args.output: parser.error('build requires --output (new directory)')
        args.output.mkdir(parents=True,exist_ok=False)
        (args.output/'assets.hdp').write_bytes(pack)
        preview = dict(manifest)
        preview['atlases'] = [dict(row,path=f'atlas-{i}.png') for i,row in enumerate(manifest['atlases'])]
        for i,row in enumerate(manifest['atlases']): shutil.copyfile(ROOT/row['path'],args.output/f'atlas-{i}.png')
        (args.output/'manifest.json').write_text(json.dumps(preview,ensure_ascii=False,indent=2)+'\n')
        shutil.copyfile(ROOT/'tools/hd_assets_preview.html',args.output/'index.html')
        (args.output/'validation.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report))


if __name__ == '__main__': main()
