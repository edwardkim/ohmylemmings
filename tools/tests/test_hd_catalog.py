import struct,tempfile,unittest,zlib
from pathlib import Path
from tools.hd_catalog import split_pack,select_parts
from tools.tests.test_hd_assets import png
from tools.hd_assets import compile_pack
import hashlib

class HdCatalogTests(unittest.TestCase):
    def pack(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);p=root/'p.png';p.write_bytes(png(2,2,color=2));pin=dict(path='p.png',bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest(),width=2,height=2,alpha='opaque')
            return compile_pack(dict(schema_version=1,atlases=[pin]*3,frames=[dict(key=k,atlas=i,rect=[0,0,2,2],display=[12,12],anchor=[0,0]) for i,k in enumerate([dict(actor=[0,0]),dict(material=0),dict(material=1)])]),root=root)[0]
    def test_split_keeps_full_atlas_bytes_and_only_selects_common_and_requested_theme(self):
        pack=self.pack();files=split_pack(pack);self.assertEqual(len(files),4)
        a=select_parts(files,3);b=select_parts(files,5)
        self.assertEqual(len(a),2);self.assertEqual(a[0],b[0]);self.assertNotEqual(a[1],b[1])
        self.assertEqual(a[0][12:48],pack[12:48])
    def test_missing_corrupt_or_unbounded_selected_parts_fail(self):
        files=split_pack(self.pack());bad=dict(files);del bad['atlas-001.hdp']
        with self.assertRaises(ValueError):select_parts(bad,3)
        bad=dict(files);bad['atlas-000.hdp']=bad['atlas-000.hdp'][:-1]
        with self.assertRaises(ValueError):select_parts(bad,3)
        for scope in [0,1,7,1024]:
            with self.assertRaises(ValueError):select_parts(files,scope)

    def test_independent_extensions_exceed_global_32_atlases_without_exceeding_active_theme_limit(self):
        from tools.hd_catalog import split_packs
        template=split_pack(self.pack())['atlas-001.hdp'];packs=[]
        for theme in range(5):
            for frame in range(10):
                part=bytearray(template);struct.pack_into('<BBH',part,len(part)-32,2,theme*16+1,frame);packs.append(bytes(part))
        files=split_packs(packs)
        self.assertEqual(len(files),51)
        for theme in range(5):self.assertEqual(len(select_parts(files,1|(1<<(theme+1)))),10)
        with self.assertRaisesRegex(ValueError,'duplicate catalog key'):split_packs([packs[0],packs[0]])

    def test_runtime_catalog_verifies_an_independently_pinned_extension_manifest(self):
        from tools.hd_catalog import runtime_catalog
        import json
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);(root/'assets/manifests').mkdir(parents=True);p=root/'p.png';p.write_bytes(png(2,2,color=2))
            pin=dict(path='p.png',bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest(),width=2,height=2,alpha='opaque')
            ext=dict(schema_version=1,atlases=[pin],frames=[dict(key=dict(material=4),atlas=0,rect=[0,0,2,2],display=[12,12],anchor=[0,0])])
            p=root/'extension.json';p.write_text(json.dumps(ext));ref=dict(path=p.name,bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest());m=root/'assets/manifests/remaster_pack.json';m.write_text(json.dumps(dict(catalog_extensions=[ref])))
            self.assertEqual(len(runtime_catalog(root,self.pack())),5)
            p.write_text(p.read_text()+' ')
            with self.assertRaises(ValueError):runtime_catalog(root,self.pack())
    def test_special_pages_only_load_for_the_matching_special_scope(self):
        from tools.hd_catalog import split_packs
        from tools.hd_assets import key_fields
        packs=[self.pack()]
        for index in range(4):
            self.assertEqual(key_fields(dict(special=index)),(4,index,0))
            part=bytearray(self.pack());part[-32]=4;part[-31]=index
            # Reuse a complete valid source with a special replacing material1.
            files=split_pack(bytes(part));packs.append(files['atlas-002.hdp'])
        files=split_packs(packs)
        ordinary=select_parts(files,3)
        for index in range(4):
            selected=select_parts(files,3|(1<<(6+index)))
            self.assertEqual(selected[:2],ordinary)
            self.assertEqual(len(selected),3)
            self.assertEqual(selected[-1][-32:-28],bytes([4,index,0,0]))
        for value in [-1,4,True]:
            with self.assertRaises(ValueError):key_fields(dict(special=value))
