#!/usr/bin/env python3
"""Compare initial host output to pinned static hashes and independent Python drawing.

Reuses verified DAT decompression only; placement, object planes, trigger maps,
palette choice and startup object frame selection are computed here independently.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import time
try:
    from .initial_replay import ROOT, file_identity, verified_assets
except ImportError:
    from initial_replay import ROOT, file_identity, verified_assets
sha=lambda b:hashlib.sha256(b).hexdigest()
le=lambda b,at:int.from_bytes(b[at:at+2],'little')
be=lambda b,at:int.from_bytes(b[at:at+2],'big')


def placements(level):
    result=[]
    for slot in range(32):
        b=level[32+slot*8:40+slot*8]
        if any(b):result.append([slot,be(b,0)-16,be(b,2),b[5]&15,b[7]==143,bool(b[6]&128),bool(b[6]&64)])
    return result


def draw_scene(terrain,palette,terrain_palette,objects,ground,data,first,width=1584,height=160):
    solid=bytearray(c<128 for c in terrain);occupied=solid.copy()
    lookup={128:bytes(4),**{c:terrain_palette[c*3:c*3+3]+b'\xff' for c in range(8)}}
    rgba=bytearray(b''.join(lookup[c] for c in terrain))
    for only in [True,False]:
        for slot,x,y,object_id,flip,no_overwrite,on_terrain in objects:
            if on_terrain!=only:continue
            m=ground[object_id*28:object_id*28+28];w,h=m[4:6];count=m[3]
            frame=m[2] if first or object_id==1 or le(m,0)&1 else (m[2]+2)%count
            offset=le(m,21)+frame*le(m,6);plane=w*h//8;mask=offset+le(m,8)
            for dy in range(max(0,y),min(height,y+h)):
                sy=h-1-(dy-y) if flip else dy-y
                for dx in range(max(0,x),min(width,x+w)):
                    pixel=sy*w+dx-x;byte=pixel//8;bit=128>>(pixel%8)
                    if not data[mask+byte]&bit:continue
                    at=dy*width+dx
                    if on_terrain and not solid[at]:continue
                    if not on_terrain and no_overwrite and occupied[at]:continue
                    color=sum((1<<p) for p in range(4) if data[offset+p*plane+byte]&bit)
                    rgba[at*4:at*4+4]=palette[color*3:color*3+3]+b'\xff';occupied[at]=1
    return bytes(rgba)


def trigger_maps(level,objects,ground):
    materials=bytearray(1584*160);raw=bytearray([128])*(415*47)
    def cells(left,top,w,h,value):
        for gy in range(max(0,top//4+4),min(47,(top+h+3)//4+4)):
            for gx in range(max(0,left//4+4),min(415,(left+w+3)//4+4)):raw[gy*415+gx]=value
    def paint(left,top,w,h,value):
        for y in range(max(0,top),min(160,top+h)):
            for x in range(max(0,left),min(1584,left+w)):materials[y*1584+x]=value
    for at in range(1888,2016,4):
        a,b,c,d=level[at:at+4]
        if a or b or c or d:
            x=((a<<1)|(b>>7))*4-16;y=(b&127)*4;w=(c>>4)*4+4;h=(c&15)*4+4
            paint(x,y,w,h,137);cells(x,y,w,h,137)
    compact=0
    for slot,x,y,oid,*_ in objects:
        m=ground[oid*28:oid*28+28];left=(x//4)*4+le(m,14)*4;top=(y//4)*4+le(m,16)*4-4;w=m[18]*4;h=m[19]*4;effect=m[20]
        if slot<16:paint(left,top,w,h,{9:137,7:135,8:136}.get(effect,0))
        if oid!=1:
            if compact<16:cells(left,top,w,h,compact if effect==4 else effect+128)
            compact+=1
    return bytes(materials),bytes(raw)


def check(assets,directory,native):
    assets=assets.resolve();directory=directory.resolve();native=native.resolve()
    if not directory.is_relative_to(ROOT/'assets/local') or not native.is_relative_to(ROOT/'assets/local'):raise ValueError('ignored evidence required')
    directory.mkdir(parents=True,exist_ok=False);originals=verified_assets(assets)
    subprocess.run(['cargo','build','--release','--locked','--offline','-p','lemmings-resources','--examples'],cwd=ROOT,check=True)
    examples=ROOT/'target/release/examples'
    for name in ['inspect_terrain','inspect_objects']:
        command=[str(examples/name),str(assets),str(directory/name)]
        if name=='inspect_objects':command.append('initial')
        with (directory/(name+'.jsonl')).open('w') as output:subprocess.run(command,stdout=output,check=True)
    for name in [*[f'LEVEL{i:03}.DAT' for i in range(10)],*[f'VGAGR{i}.DAT' for i in range(5)]]:
        subprocess.run([str(examples/'unpack_dat'),str(assets/name),str(directory/name)],stdout=subprocess.DEVNULL,check=True)
    maps=json.loads((ROOT/'assets/manifests/level_terrain.json').read_text())['maps'];scenes=json.loads((ROOT/'assets/manifests/level_objects.json').read_text())['maps'];catalog=json.loads((ROOT/'assets/manifests/dos_compatibility.json').read_text())['levels'];graphics=json.loads((ROOT/'assets/manifests/vgagr_images.json').read_text())['sets']
    for graphics_set in graphics:
        for part in graphics_set['sections']:
            raw=(directory/f'VGAGR{graphics_set["set"]}.DAT'/f'part-{part["part"]:03}.bin').read_bytes();assert len(raw)==part['size'] and sha(raw)==part['sha256']
    terrain_rows=[json.loads(s) for s in (directory/'inspect_terrain.jsonl').read_text().splitlines()];scene_rows=[json.loads(s) for s in (directory/'inspect_objects.jsonl').read_text().splitlines()];reports=[]
    for index,(entry,t,s) in enumerate(zip(catalog,terrain_rows,scene_rows)):
        key=(entry['file'],entry['part']);target=next(x for x in maps if (x['file'],x['part'])==key);static=next(x for x in scenes if (x['file'],x['part'],x['mode'])==(*key,'preview'))
        terrain=(directory/'inspect_terrain'/t['pixel_file']).read_bytes();scene=(directory/'inspect_objects'/s['pixel_file']).read_bytes()
        assert sha(terrain)==target['pixel_sha256'] and sha(scene)==static['pixel_sha256']
        assert t['palette_hex']==target['palette_hex'] and s['palette_hex']==static['palette_hex'] and s['terrain_palette_hex']==static['terrain_palette_hex']
        level=(directory/entry['file']/f'part-{entry["part"]:03}.bin').read_bytes();assert sha(level)==entry['base_record_sha256']
        objects=placements(level);ground=(assets/f'GROUND{entry["graphics"]}O.DAT').read_bytes();data=(directory/f'VGAGR{entry["graphics"]}.DAT'/'part-001.bin').read_bytes();palette=bytes.fromhex(s['palette_hex']);terrain_palette=bytes.fromhex(s['terrain_palette_hex'])
        first=draw_scene(terrain,palette,terrain_palette,objects,ground,data,True)
        assert sha(first)==static['rgba_sha256'],(entry['id'],'independent first-frame RGBA')
        initial=draw_scene(terrain,palette,terrain_palette,objects,ground,data,False);materials,raw=trigger_maps(level,objects,ground)
        expected_path=native/(entry['id']+'.json');deadline=time.monotonic()+1800
        while not expected_path.exists():
            if time.monotonic()>deadline:raise TimeoutError('native case missing: '+entry['id'])
            time.sleep(1)
        expected=json.loads(expected_path.read_text());info=expected['initial']['info'];camera=min(1264,max(0,entry['screen_x']))
        for field in ['minutes','total','required','skills','screen_x']:
            assert info[field]==entry[field],(entry['id'],field)
        assert info['rate']==entry['release_rate'] and info['title']==entry['title'].strip().upper() and info['music_special']==entry['special']
        assert info['palette']==[list(palette[i:i+3]) for i in range(0,48,3)]
        assert info['entrances']==[[o[1],o[2]] for o in objects if o[3]==1]
        assert sha(materials)==expected['initial']['materials'] and sha(raw)==expected['rows'][0][3]
        assert sha(bytes(c<128 for c in terrain))==expected['rows'][0][1]
        viewport=b''.join(initial[(y*1584+x)*4:(y*1584+x)*4+3] for y in range(160) for x in range(camera,camera+320))
        actual=(native/(entry['id']+'-initial.rgb')).read_bytes();assert actual[:320*160*3]==viewport,(entry['id'],'initial world viewport')
        report=dict(id=entry['id'],static_first_rgba_sha256=sha(first),startup_world_rgba_sha256=sha(initial),startup_viewport_rgb_sha256=sha(viewport),terrain_sha256=expected['rows'][0][1],materials_sha256=sha(materials),object_map_sha256=sha(raw),initial_frame_rgb_sha256=sha(actual),camera=camera,palette_hex=s['palette_hex'],frame_policy='entrance first; continuous first+2 modulo count; other first')
        reports.append(report);print('REFERENCE',entry['id'],'PASS',flush=True)
        (directory/'progress.json').write_text(json.dumps(reports,indent=2)+'\n')
    assert len(reports)==120 and verified_assets(assets)==originals
    result=dict(status='PASS',levels=120,scope='Headers/inventories and immutable base bytes; terrain and static first-frame hashes from independent pinned #15/#19; independent Python startup object planes/composition and steel/one-way/raw object-map geometry; initial320x160 world viewport. Entire HUD compared between SDL/WASM, not a new independent fullHUD or DOS observation oracle.',reports=reports,inputs=[file_identity(ROOT/'assets/manifests'/f'{n}.json') for n in ['dos_original','dos_compatibility','level_terrain','level_objects','vgagr_images']])
    (directory/'summary.json').write_text(json.dumps(result,indent=2)+'\n');return result


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--assets',type=Path,required=True);p.add_argument('--evidence',type=Path,required=True);p.add_argument('--native',type=Path,required=True);a=p.parse_args();check(a.assets,a.evidence,a.native)

if __name__=='__main__':main()
