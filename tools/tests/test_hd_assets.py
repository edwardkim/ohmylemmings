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
                ('key',dict(material=1)),('atlas',1),('rect',[1,0,2,2]),('rect',[0,0,0,2]),
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
        for id,count in [(0,1),(1,10)]:
            for frame in range(count):
                self.assertEqual(key_fields({'object':[0,id,frame]}),(2,id,frame))
        for id,name in enumerate(ICON_NAMES):
            self.assertEqual(key_fields({'icon':name}),(3,id,0))
        for key in [{'object':[0,0,1]},{'object':[0,1,10]},{'object':[1,0,0]},
                    {'object':[0,2,0]},{'object':[False,0,0]},{'object':[0,1]},
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


if __name__=='__main__':unittest.main()
