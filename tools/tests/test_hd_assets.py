import hashlib
import json
from pathlib import Path
import struct
import tempfile
import unittest
import zlib

from tools.hd_assets import AssetError, compile_pack, decode_png


def chunk(kind, data):
    return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data))


def png(width=2, height=2, color=6, raw=None):
    channels = 4 if color == 6 else 3
    if raw is None:
        raw = (b'\0' + bytes([20, 30, 40, 255] if color == 6 else [20,30,40]) * width) * height
    return (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, color, 0, 0, 0))
            + chunk(b'IDAT', zlib.compress(raw)) + chunk(b'IEND', b''))


class HdAssetsTests(unittest.TestCase):
    def test_rgba_and_rgb_decode_without_changing_color_or_alpha(self):
        self.assertEqual(decode_png(png()), (2,2,bytes([20,30,40,255])*4))
        self.assertEqual(decode_png(png(color=2)), decode_png(png()))

    def test_all_five_png_filters_decode_correctly(self):
        # Two identical pixels: first scanline byte pattern, then each PNG filter.
        rgba=bytes([20,30,40,255])*2
        for kind in range(5):
            encoded=bytearray()
            for i,value in enumerate(rgba):
                left=rgba[i-4] if i>=4 else 0
                up=rgba[i]
                ul=rgba[i-4] if i>=4 else 0
                if kind==0: predictor=0
                elif kind==1: predictor=left
                elif kind==2: predictor=up
                elif kind==3: predictor=(left+up)//2
                else:
                    p=left+up-ul; ds=[abs(p-left),abs(p-up),abs(p-ul)]
                    predictor=[left,up,ul][ds.index(min(ds))]
                encoded.append((value-predictor)&255)
            data=png(raw=b'\0'+rgba+bytes([kind])+encoded)
            self.assertEqual(decode_png(data)[2],rgba*2)

    def test_truncation_crc_extra_data_and_compression_size_are_rejected(self):
        b=png()
        for end in range(len(b)):
            with self.assertRaises(AssetError): decode_png(b[:end])
        bad=bytearray(b);bad[45]^=1
        for data in [bytes(bad),b+b'x',png(raw=b'x'*100),png(raw=b'\5'+b'\0'*8+b'\0'+b'\0'*8),png(width=0),png(width=4097)]:
            with self.assertRaises(AssetError): decode_png(data)

    def fixture(self, root):
        path=root/'art.png';path.write_bytes(png())
        return dict(schema_version=1,atlases=[dict(path='art.png',bytes=path.stat().st_size,
            sha256=hashlib.sha256(path.read_bytes()).hexdigest(),width=2,height=2,alpha='opaque')],
            frames=[dict(key=dict(actor=[0,0]),atlas=0,rect=[0,0,2,2],display=[96,60],anchor=[48,60])])

    def test_pack_is_deterministic_and_contains_exact_rgba(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m=self.fixture(root)
            a,report=compile_pack(m,root=root)
            self.assertEqual(a,compile_pack(m,root=root)[0])
            self.assertEqual(a[:12],b'LHDPAK01\1\0\1\0')
            self.assertEqual(a[32:48],bytes([20,30,40,255])*4)
            self.assertEqual(report['frames'],1)

    def test_missing_tampered_escaped_and_invalid_assets_are_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m=self.fixture(root)
            for key,value in [('path','../art.png'),('path','/tmp/art.png'),('path','missing.png'),
                ('bytes',0),('sha256','0'*64),('width',3),('alpha','sprite')]:
                changed=json.loads(json.dumps(m));changed['atlases'][0][key]=value
                with self.assertRaises(AssetError):compile_pack(changed,root=root)

    def test_frame_source_mapping_bounds_and_duplicates_are_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m=self.fixture(root)
            for key,value in [('key',dict(actor=[0,8])),('key',dict(actor=[28,0])),
                ('key',dict(material=5)),('atlas',1),('rect',[1,0,2,2]),('rect',[0,0,0,2]),
                ('display',[0,60]),('anchor',[5000,60])]:
                changed=json.loads(json.dumps(m));changed['frames'][0][key]=value
                with self.assertRaises(AssetError):compile_pack(changed,root=root)
            m['frames']*=2
            with self.assertRaises(AssetError):compile_pack(m,root=root)

    def test_sprite_frames_require_transparent_gutters_and_visible_pixels(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m=self.fixture(root);m['atlases'][0]['alpha']='sprite'
            rgba=bytearray(4*4*4);rgba[20:24]=bytes([20,30,40,255])
            p=root/'art.png';p.write_bytes(png(4,4,raw=b''.join(b'\0'+rgba[i:i+16] for i in range(0,64,16))))
            m['atlases'][0].update(bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest(),width=4,height=4)
            m['frames'][0]['rect']=[0,0,4,4]
            self.assertEqual(compile_pack(m,root=root)[1]['frames'],1)
            m['frames'][0]['rect']=[0,0,1,1]
            with self.assertRaises(AssetError):compile_pack(m,root=root)

    def test_schema_and_numeric_fields_reject_boolean_and_wrong_types(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m=self.fixture(root)
            for key,value in [('schema_version',True),('atlases',[False]),('frames',[None]),('provenance',False)]:
                changed=json.loads(json.dumps(m));changed[key]=value
                with self.assertRaises(AssetError):compile_pack(changed,root=root)
            m['frames'][0]['key']={'material':False}
            with self.assertRaises(AssetError):compile_pack(m,root=root)

    def test_required_coverage_and_provenance_are_checked(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m=self.fixture(root)
            m['required']=[{'actor':[0,0]},{'actor':[2,0]}]
            with self.assertRaises(AssetError):compile_pack(m,root=root)
            m['required']=[{'actor':[0,0]}];m['provenance']=[dict(m['atlases'][0],sha256='0'*64)]
            with self.assertRaises(AssetError):compile_pack(m,root=root)

    def test_png_chunk_order_formats_and_multiple_streams_are_rejected(self):
        header=chunk(b'IHDR',struct.pack('>IIBBBBB',2,2,8,6,0,0,0))
        prefix=b'\x89PNG\r\n\x1a\n'+header
        stream=zlib.compress((b'\0'+b'\0'*8)*2)
        bad=[prefix+header+chunk(b'IDAT',stream)+chunk(b'IEND',b''),
             prefix+chunk(b'ABCD',b'')+chunk(b'IDAT',stream)+chunk(b'IEND',b''),
             prefix+chunk(b'PLTE',b'x')+chunk(b'IDAT',stream)+chunk(b'IEND',b''),
             prefix+chunk(b'IDAT',b'')+chunk(b'tEXt',b'x')+chunk(b'IDAT',stream)+chunk(b'IEND',b''),
             prefix+chunk(b'IDAT',stream+stream)+chunk(b'IEND',b'')]
        for b in bad:
            with self.assertRaises(AssetError):decode_png(b)

    def test_symlink_cannot_escape_asset_root(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m=self.fixture(root);outside=root/'outside';outside.mkdir();inside=root/'inside';inside.mkdir()
            (outside/'art.png').write_bytes((root/'art.png').read_bytes())
            try:(inside/'art.png').symlink_to(outside/'art.png')
            except OSError:self.skipTest('symlinks unavailable')
            with self.assertRaises(AssetError):compile_pack(m,root=inside)

    def test_alpha_cutoff_is_explicit_and_never_edits_packed_pixel_alpha(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m=self.fixture(root)
            rgba=bytearray([10,20,30,1]*16);rgba[20:24]=bytes([20,30,40,255])
            p=root/'art.png';p.write_bytes(png(4,4,raw=b''.join(b'\0'+rgba[i:i+16] for i in range(0,64,16))))
            m['atlases'][0].update(bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest(),width=4,height=4,alpha='sprite')
            m['frames'][0]['rect']=[0,0,4,4]
            with self.assertRaises(AssetError):compile_pack(m,root=root)
            m['atlases'][0]['alpha_cutoff']=1
            packed,_=compile_pack(m,root=root)
            self.assertEqual(packed[28:32],b'\1\0\0\0')
            self.assertEqual(packed[32:96],rgba)
            m['atlases'][0]['alpha_cutoff']=255
            with self.assertRaises(AssetError):compile_pack(m,root=root)

    def test_facility_and_icon_keys_are_distinct_bounded_and_keep_legacy_encoding(self):
        from tools.hd_assets import key_fields, ICON_NAMES
        self.assertEqual(key_fields({'actor':[0,0]}),(0,0,0))
        self.assertEqual(key_fields({'material':0}),(1,0,0))
        for theme in range(5):
            for id,count in [(0,1),(1,10)]:
                for frame in range(count):
                    self.assertEqual(key_fields({'object':[theme,id,frame]}),(2,theme*16+id,frame))
        for id,name in enumerate(ICON_NAMES):
            self.assertEqual(key_fields({'icon':name}),(3,id,0))
        for key in [{'object':[0,0,1]},{'object':[0,1,10]},{'object':[5,0,0]},
                    {'object':[0,5,0]},{'object':[False,0,0]},{'object':[0,1]},
                    {'icon':'minimap'},{'icon':13},{'icon':False},{'icon':[]}]:
            with self.assertRaises(AssetError): key_fields(key)

    def test_partially_opaque_sprite_pixels_are_preserved_without_requiring_alpha255(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m=self.fixture(root)
            rgba=bytearray(4*4*4);rgba[20:24]=bytes([20,30,40,253])
            p=root/'art.png';p.write_bytes(png(4,4,raw=b''.join(b'\0'+rgba[i:i+16] for i in range(0,64,16))))
            m['atlases'][0].update(bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest(),width=4,height=4,alpha='sprite')
            m['frames'][0]['rect']=[0,0,4,4]
            packed,_=compile_pack(m,root=root)
            self.assertEqual(packed[32:96],rgba)
            m['atlases'][0]['alpha_cutoff']=253
            with self.assertRaises(AssetError):compile_pack(m,root=root)

    def test_runtime_cache_validates_sources_and_repairs_damaged_compiled_bytes(self):
        from tools.hd_assets import runtime_pack
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m=self.fixture(root)
            manifest=root/'assets/manifests/remaster_pack.json';manifest.parent.mkdir(parents=True);manifest.write_text(json.dumps(m))
            data=runtime_pack(root=root)
            self.assertEqual(data,compile_pack(m,root=root)[0])
            cached=root/'assets/local/hd-runtime/assets.hdp';cached.write_bytes(b'broken')
            self.assertEqual(runtime_pack(root=root),data)
            (root/'art.png').write_bytes(b'corrupted source')
            with self.assertRaises(AssetError):runtime_pack(root=root)

    def test_complete_actor_contract_requires_all_337_original_frame_keys(self):
        from tools.hd_assets import actor_coverage, source_counts
        frames=[dict(key=dict(actor=[slot,frame])) for slot,count in enumerate(source_counts()) for frame in range(count)]
        self.assertEqual(actor_coverage(frames),dict(animations=28,frames=337,missing=[]))
        partial=actor_coverage(frames[:-1]);self.assertEqual(partial['missing'],[[27,0]])
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m=self.fixture(root);m['complete_actors']=True
            with self.assertRaisesRegex(AssetError,'actor coverage'):compile_pack(m,root=root)
            m['complete_actors']='yes'
            with self.assertRaises(AssetError):compile_pack(m,root=root)

    def test_expanded_atlas_count_is_bounded_and_legacy_fixture_still_builds(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m=self.fixture(root)
            m['atlases']*=32
            self.assertEqual(len(compile_pack(m,root=root)[1]['atlases']),32)
            m['atlases'].append(m['atlases'][0])
            with self.assertRaises(AssetError):compile_pack(m,root=root)

    def test_default_action_art_uses_strikes_at_original_terrain_cut_frames(self):
        # Visual cells selected during #154 review: downward pickaxe and fully
        # extended fists. This guards the data binding, not simulation timing.
        from tools.hd_assets import ROOT
        pack=json.loads((ROOT/'assets/manifests/remaster_pack.json').read_text())
        art=json.loads((ROOT/'assets/manifests/hd_actors.json').read_text())
        for slot,frame,name,cell in [(14,1,'mine',5),(14,2,'mine',6),
                (15,1,'mine',29),(15,2,'mine',30),(12,5,'bash',12),
                (12,21,'bash',52),(13,5,'bash',36),(13,21,'bashleft2',4)]:
            group=next(g for g in art['layout'] if g['name']==name)
            row=next(r for r in group['rows'] if r['slot']==slot and r['frame']==frame)
            self.assertEqual(row['cell'],cell,(slot,frame))
            bound=next(r for r in pack['frames'] if r['key'].get('actor')==[slot,frame])
            self.assertEqual((bound['atlas'],bound['rect']),(group['atlas'],row['rect']))

    def test_five_material_ids_are_bounded_and_keep_the_legacy_rock_key(self):
        from tools.hd_assets import key_fields
        for theme in range(5):
            self.assertEqual(key_fields({'material':theme}),(1,theme,0))
        for invalid in [-1,5,True,'1']:
            with self.assertRaises(AssetError):key_fields({'material':invalid})

    def test_rectangle_packing_preserves_every_sample_and_reuses_duplicate_rectangles(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m=self.fixture(root)
            rgba=bytes(v for y in range(16) for x in range(16) for v in [x,y,70,255])
            p=root/'art.png';p.write_bytes(png(16,16,raw=b''.join(b'\0'+rgba[y*64:(y+1)*64] for y in range(16))))
            m['atlases'][0].update(bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest(),width=16,height=16)
            m['frames']=[dict(key={'actor':[0,i]},atlas=0,rect=rect,display=[96,60],anchor=[48,60])
                         for i,rect in enumerate([[2,3,2,3],[10,9,3,2],[2,3,2,3]])]
            m['packing']='rectangles-v1'
            data,report=compile_pack(m,root=root)
            self.assertEqual(data,compile_pack(m,root=root)[0])
            w,h,length,checksum,cutoff=struct.unpack_from('<IIIII',data,12)
            self.assertLess(w*h,16*16)
            pixels=data[32:32+length];self.assertEqual(zlib.crc32(pixels),checksum)
            packed=[]
            for i,f in enumerate(m['frames']):
                record=struct.unpack_from('<BBHHHIIIIHHhh',data,32+length+i*32)
                x,y,fw,fh=record[5:9];packed.append([x,y,fw,fh])
                sx,sy,sw,sh=f['rect'];self.assertEqual((fw,fh),(sw,sh))
                for row in range(sh):
                    self.assertEqual(pixels[((y+row)*w+x)*4:((y+row)*w+x+fw)*4],
                                     rgba[((sy+row)*16+sx)*4:((sy+row)*16+sx+sw)*4])
                self.assertEqual(record[9:],(96,60,48,60))
            self.assertEqual(packed[0],packed[2])
            self.assertEqual(report['packing'],'rectangles-v1')
            self.assertEqual(p.read_bytes(),png(16,16,raw=b''.join(b'\0'+rgba[y*64:(y+1)*64] for y in range(16))))

    def test_unknown_packing_policy_is_rejected_and_full_image_stays_unchanged(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m=self.fixture(root);legacy=compile_pack(m,root=root)[0]
            m['packing']='rectangles-v1';self.assertEqual(compile_pack(m,root=root)[0],legacy)
            for value in [True,'lossy',{}]:
                m['packing']=value
                with self.assertRaises(AssetError):compile_pack(m,root=root)

    def test_relocated_gutters_copy_straight_alpha_and_colored_transparent_samples(self):
        from tools.hd_assets import pack_rectangles
        rgba=bytes(v for y in range(16) for x in range(16) for v in [x,y,80,(x+y)*8])
        w,h,pixels,rects=pack_rectangles(16,16,rgba,{(2,3,2,3),(10,9,3,2)})
        self.assertLess(w*h,256)
        for (sx,sy,sw,sh),(dx,dy,_,_) in rects.items():
            for y in range(-1,sh+1):
                for x in range(-1,sw+1):
                    source=((sy+min(max(y,0),sh-1))*16+sx+min(max(x,0),sw-1))*4
                    dest=((dy+y)*w+dx+x)*4
                    self.assertEqual(pixels[dest:dest+4],rgba[source:source+4])


    def test_complete_portal_policy_requires_all_five_sets_and_55_frames(self):
        from tools.hd_assets import portal_coverage
        keys=[dict(object=[theme,id,frame]) for theme in range(5) for id,count in [(0,1),(1,10)] for frame in range(count)]
        self.assertEqual(portal_coverage([dict(key=k) for k in keys]),dict(sets=5,frames=55,missing=[]))
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m=self.fixture(root);m['complete_portals']=True
            with self.assertRaisesRegex(AssetError,'portal coverage'):compile_pack(m,root=root)
            f=m['frames'][0];m['frames']=[dict(f,key=k) for k in keys]
            self.assertEqual(compile_pack(m,root=root)[1]['portal_coverage'],dict(sets=5,frames=55,missing=[]))
            m['frames'].pop()
            self.assertEqual(portal_coverage(m['frames'])['missing'],[[4,1,9]])
            with self.assertRaisesRegex(AssetError,'portal coverage'):compile_pack(m,root=root)
            m['complete_portals']='yes'
            with self.assertRaises(AssetError):compile_pack(m,root=root)


    def test_exit_flame_frames_keep_pure_decoration_profiles_and_complete_coverage(self):
        from tools.hd_assets import exit_effect_coverage, key_fields
        keys=[{'object':[theme,id,frame]} for theme,id in [(0,7),(1,6),(2,6),(3,6),(4,8)] for frame in range(6)]
        self.assertEqual(exit_effect_coverage([{'key':key} for key in keys]),dict(frames=30,missing=[]))
        for key in keys:
            theme,id,frame=key['object'];self.assertEqual(key_fields(key),(2,theme*16+id,frame))
        for key in [{'object':[0,7,6]},{'object':[0,2,0]},{'object':[3,3,0]},{'object':[4,4,0]},{'object':[5,2,0]}]:
            with self.assertRaises(AssetError):key_fields(key)
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m=self.fixture(root);m['complete_exit_effects']=True
            with self.assertRaisesRegex(AssetError,'exit effect coverage'):compile_pack(m,root=root)
            f=m['frames'][0];m['frames']=[dict(f,key=key) for key in keys]
            report=compile_pack(m,root=root)[1]
            self.assertEqual(report['exit_effect_coverage'],dict(frames=30,missing=[]))
            self.assertEqual(report['portal_coverage']['frames'],0)

if __name__=='__main__':unittest.main()
