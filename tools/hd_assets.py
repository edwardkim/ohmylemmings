#!/usr/bin/env python3
"""Validate generated HD PNGs and build a deterministic, pixel-preserving RGBA pack."""
import argparse
from functools import lru_cache
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import struct
import zlib

ROOT = Path(__file__).resolve().parents[1]
MAX_BYTES = 64 * 1024 * 1024
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


def key_fields(key):
    require(isinstance(key,dict), 'frame key')
    if set(key) == {'actor'}:
        slot,frame = numbers(key['actor'],2,0,65535)
        require(slot < len(source_counts()) and frame < source_counts()[slot], 'source slot/frame')
        return 0,slot,frame
    if set(key) == {'object'}:
        theme,id,frame = numbers(key['object'],3,0,65535)
        count = {(0,0):1, (0,1):10}.get((theme,id))
        require(count is not None and frame < count, 'unsupported object/frame')
        return 2,theme*16+id,frame
    if set(key) == {'icon'}:
        require(isinstance(key['icon'],str) and key['icon'] in ICON_NAMES, 'unknown icon')
        return 3,ICON_NAMES.index(key['icon']),0
    require(set(key) == {'material'} and type(key['material']) is int and key['material'] == 0, 'unknown material/key')
    return 1,0,0


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


def compile_pack(manifest, *, root=ROOT):
    require(isinstance(manifest,dict) and type(manifest.get('schema_version')) is int and manifest['schema_version'] == 1, 'manifest schema')
    atlases = manifest.get('atlases'); frames = manifest.get('frames')
    require(isinstance(atlases,list) and 1 <= len(atlases) <= 16, 'atlas count')
    require(isinstance(frames,list) and 1 <= len(frames) <= 4096, 'frame count')
    provenance = manifest.get('provenance',[])
    require(isinstance(provenance,list) and len(provenance) <= 128, 'provenance list')
    for pin in provenance: read_pin(pin,root)
    output = bytearray(b'LHDPAK01'+struct.pack('<HH',len(atlases),len(frames)))
    images = []; atlas_report = []
    for atlas in atlases:
        data = read_pin(atlas,root); w,h,rgba = decode_png(data)
        require([w,h] == [integer(atlas.get('width'),1,MAX_DIM),integer(atlas.get('height'),1,MAX_DIM)], 'atlas dimensions')
        alpha = rgba[3::4]
        cutoff = integer(atlas.get('alpha_cutoff',0),0,254)
        require(atlas.get('alpha') in ['sprite','opaque'], 'alpha policy')
        if atlas['alpha'] == 'opaque': require(all(a == 255 for a in alpha), 'opaque atlas has transparency')
        else: require(min(alpha) <= cutoff < max(alpha), 'sprite atlas must have transparency and visible ink')
        require(len(output) + 20 + len(rgba) + len(frames)*32 <= MAX_BYTES, 'pack size limit')
        output.extend(struct.pack('<IIIII',w,h,len(rgba),zlib.crc32(rgba),cutoff)); output.extend(rgba)
        images.append((w,h,rgba)); atlas_report.append(dict(width=w,height=h,alpha_cutoff=cutoff,rgba_sha256=hashlib.sha256(rgba).hexdigest()))
    seen = set(); bounds = []
    for entry in frames:
        require(isinstance(entry,dict), 'frame record')
        fields = key_fields(entry.get('key'))
        require(fields not in seen, 'duplicate frame key'); seen.add(fields)
        index = integer(entry.get('atlas'),0,len(atlases)-1)
        x,y,w,h = numbers(entry.get('rect'),4,0,MAX_DIM)
        width,height,rgba = images[index]
        require(w > 0 and h > 0 and x+w <= width and y+h <= height, 'frame rectangle')
        display = numbers(entry.get('display'),2,1,MAX_DIM)
        anchor = numbers(entry.get('anchor'),2,-MAX_DIM,MAX_DIM)
        if atlases[index]['alpha'] == 'sprite': bounds.append(dict(key=entry['key'],bbox=sprite_bounds(rgba,width,[x,y,w,h],atlases[index].get('alpha_cutoff',0))))
        output.extend(struct.pack('<BBHHHIIIIHHhh',*fields,index,0,x,y,w,h,*display,*anchor))
    if 'required' in manifest:
        require(isinstance(manifest['required'],list) and len(manifest['required']) <= 4096, 'required keys')
        required = [key_fields(key) for key in manifest['required']]
        require(len(set(required)) == len(required) and set(required) == seen, 'required coverage differs')
    pack = bytes(output)
    return pack,dict(status='PASS',atlases=atlas_report,frames=len(frames),sprite_bounds=bounds,
                     bytes=len(pack),sha256=hashlib.sha256(pack).hexdigest(),pixel_policy='unchanged RGB, straight alpha RGBA8; sRGB interpretation')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command',choices=['check','build'])
    parser.add_argument('--manifest',type=Path,default=ROOT/'assets/manifests/remaster_pack.json')
    parser.add_argument('--output',type=Path)
    args = parser.parse_args()
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
