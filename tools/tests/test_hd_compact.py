import hashlib,json,struct,tempfile,unittest
from unittest.mock import patch
from pathlib import Path
from tools.hd_assets import AssetError,compile_pack,source_counts
from tools.tests.test_hd_assets import png


def unpack(data):
    count,frames=struct.unpack_from('<HH',data,8);at=12;atlases=[]
    for _ in range(count):
        w,h,size,crc,cutoff=struct.unpack_from('<IIIII',data,at);at+=20
        atlases.append((w,h,data[at:at+size],cutoff));at+=size
    return atlases,[struct.unpack_from('<BBHHHIIIIHHhh',data,at+i*32) for i in range(frames)]


class HdCompactTests(unittest.TestCase):
    def manifest(self,root,cutoffs=(8,),display=(8,8),anchor=(-2,9)):
        w=h=16;rgba=bytes(v for y in range(h) for x in range(w) for v in [x*9,y*7,31,254 if 6<=x<=9 and 6<=y<=9 else 0])
        p=root/'art.png';p.write_bytes(png(w,h,raw=b''.join(b'\0'+rgba[y*w*4:(y+1)*w*4] for y in range(h))))
        pin=dict(path='art.png',width=w,height=h,bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest(),alpha='sprite')
        return dict(schema_version=1,packing='compact-v1',atlases=[dict(pin,alpha_cutoff=c) for c in cutoffs],frames=[dict(key=dict(actor=[0,i]),atlas=i,rect=[0,0,w,h],display=list(display),anchor=list(anchor)) for i in range(len(cutoffs))]),rgba

    def test_original_frame_geometry_and_negative_anchor_stay_exact(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m,rgba=self.manifest(root);data,r=compile_pack(m,root=root);atlases,frames=unpack(data);f=frames[0]
            self.assertEqual(f[7:],(16,16,8,8,-2,9))
            w,h,pixels,cutoff=atlases[f[3]];self.assertEqual(cutoff,8)
            for y in range(16):
                at=((f[6]+y)*w+f[5])*4
                self.assertEqual(pixels[at:at+64],rgba[y*64:(y+1)*64])
            self.assertEqual(data,compile_pack(m,root=root)[0])
            m['frames'][0].update(display=[7,7],anchor=[-4096,-4096]);_,frames=unpack(compile_pack(m,root=root)[0]);self.assertEqual(frames[0][7:],(16,16,7,7,-4096,-4096))

    def test_transparent_margins_share_storage_without_changing_visible_or_filter_samples(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m,rgba=self.manifest(root,cutoffs=(8,8));data,r=compile_pack(m,root=root);atlases,frames=unpack(data)
            self.assertGreater(r['compaction']['shared_invisible_samples'],0)
            self.assertLess(r['bytes'],12+20+2*32+2*18*18*4)
            for f in frames:
                w,h,pixels,_=atlases[f[3]]
                self.assertEqual(f[7:],(16,16,8,8,-2,9))
                for y in range(4,12):
                    at=((f[6]+y)*w+f[5]+4)*4
                    self.assertEqual(pixels[at:at+32],rgba[(y*16+4)*4:(y*16+12)*4])

    def test_actor_top_anchor_stays_fixed_for_the_existing_bomber_countdown_label(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m,_=self.manifest(root);_,frames=unpack(compile_pack(m,root=root)[0]);self.assertEqual(frames[0][12],9)

    def test_cutoff_groups_stay_separate_and_duplicate_source_regions_share_storage(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m,_=self.manifest(root,cutoffs=(1,8,8));m['frames'].append(dict(m['frames'][2],key=dict(actor=[0,3])))
            data,r=compile_pack(m,root=root);atlases,frames=unpack(data)
            self.assertEqual(sorted(a[3] for a in atlases),[1,8]);self.assertEqual(frames[2][3:9],frames[3][3:9]);self.assertEqual(r['source_atlases'],3)

    def test_compaction_accepts_more_source_images_without_relaxing_runtime_limits(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m,_=self.manifest(root);m['atlases']*=33
            keys=[dict(actor=[slot,frame]) for slot,count in enumerate(source_counts()) for frame in range(count)]
            f=m['frames'][0];m['frames']=[dict(f,key=keys[i],atlas=i) for i in range(33)]
            self.assertEqual(len(unpack(compile_pack(m,root=root)[0])[0]),1)
            m['packing']='rectangles-v1'
            with self.assertRaises(AssetError):compile_pack(m,root=root)
            m['packing']='compact-v1';m['atlases']*=4
            with self.assertRaises(AssetError):compile_pack(m,root=root)

    def test_large_rectangles_split_into_bounded_pages_without_allocating_pixels(self):
        from tools.hd_assets import compact_layout
        pages=compact_layout({0:(4094,2100),1:(4094,2100),2:(4094,2100)})
        self.assertEqual(len(pages),3)
        for w,h,positions in pages:
            self.assertLessEqual(w,4096);self.assertLessEqual(h,4096);self.assertEqual(len(positions),1)

    def test_opaque_materials_are_not_trimmed_and_nonzero_alpha_below_cutoff_is_allowed(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m,_=self.manifest(root);p=root/'art.png';p.write_bytes(png(16,16,color=2));m['atlases'][0].update(bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest(),alpha='opaque',alpha_cutoff=0)
            _,frames=unpack(compile_pack(m,root=root)[0]);self.assertEqual(frames[0][7:],(16,16,8,8,-2,9))
            m,rgba=self.manifest(root);rgba=bytearray(rgba)
            for i in range(3,len(rgba),4):
                if rgba[i]==0:rgba[i]=8
            p.write_bytes(png(16,16,raw=b''.join(b'\0'+rgba[y*64:(y+1)*64] for y in range(16))))
            m['atlases'][0].update(bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest())
            self.assertEqual(compile_pack(m,root=root)[1]['frames'],1)

    def test_runtime_page_and_payload_limits_still_fail_before_emitting_a_pack(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m,_=self.manifest(root,cutoffs=(1,8))
            with patch('tools.hd_assets.MAX_ATLASES',1):
                with self.assertRaisesRegex(AssetError,'runtime atlas count'):compile_pack(m,root=root)
            with patch('tools.hd_assets.MAX_BYTES',1500):
                with self.assertRaisesRegex(AssetError,'region size limit'):compile_pack(m,root=root)
            m,_=self.manifest(root)
            with patch('tools.hd_assets.MAX_BYTES',1300):
                with self.assertRaisesRegex(AssetError,'pack size limit'):compile_pack(m,root=root)

    def test_full_width_regions_keep_their_original_extent_without_an_extra_border(self):
        from tools.hd_assets import compact_layout
        self.assertEqual(compact_layout({0:(4096,3)}),[(4096,3,{0:(0,0,4096,3)})])

    def test_sampling_lock_preserves_the_entire_legacy_page_and_frame_coordinates(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m,_=self.manifest(root,cutoffs=(8,8))
            m['packing']='rectangles-v1';old_atlases,old_frames=unpack(compile_pack(m,root=root)[0])
            m['packing']='compact-v1';m['atlases'][1]['sampling_locked']=True
            data,report=compile_pack(m,root=root);atlases,frames=unpack(data)
            self.assertEqual(report['compaction']['sampling_locked_sources'],[1])
            self.assertEqual(atlases[frames[1][3]],old_atlases[1])
            self.assertEqual(frames[1][5:],old_frames[1][5:])
            self.assertNotEqual(frames[0][3],frames[1][3])
            m['atlases'][0]['sampling_locked']=True
            self.assertEqual(compile_pack(m,root=root)[0],compile_pack(dict(m,packing='rectangles-v1'),root=root)[0])
            with patch('tools.hd_assets.MAX_ATLASES',1):
                with self.assertRaisesRegex(AssetError,'runtime atlas count'):compile_pack(m,root=root)

    def test_sampling_lock_requires_a_boolean(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);m,_=self.manifest(root)
            for value in [1,'true',None,[]]:
                m['atlases'][0]['sampling_locked']=value
                with self.assertRaisesRegex(AssetError,'sampling lock policy'):compile_pack(m,root=root)
