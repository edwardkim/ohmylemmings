import gzip
import json
import struct
import unittest
import zlib
from tools.web_hd import delivery, split_materials


def mixed():
    # Two different materials in a shared atlas, with surrounding samples.
    pixels = bytes(range(4*4*4))
    data = b'LHDPAK01'+struct.pack('<HHIIIII',1,2,4,4,len(pixels),zlib.crc32(pixels),0)+pixels
    for slot,x in [(0,0),(1,2)]:
        data += struct.pack('<BBHHHIIIIHHhh',1,slot,0,0,0,x,1,2,2,12,12,1,2)
    return data


class WebHdTests(unittest.TestCase):
    def test_shared_material_page_is_split_without_changing_its_samples_or_geometry(self):
        original=mixed();parts=split_materials(original)
        self.assertEqual(len(parts),2)
        for part in parts:
            self.assertEqual(part[:8],b'LHDPAK01')
            self.assertEqual(struct.unpack_from('<HH',part,8),(1,1))
            row=struct.unpack_from('<BBHHHIIIIHHhh',part,len(part)-32)
            slot,x,y,w,h=row[1],row[5],row[6],row[7],row[8]
            width=struct.unpack_from('<I',part,12)[0]
            for dy in range(h):
                got=part[32+((y+dy)*width+x)*4:32+((y+dy)*width+x+w)*4]
                expected=original[32+((1+dy)*4+slot*2)*4:32+((1+dy)*4+slot*2+w)*4]
                self.assertEqual(got,expected)
            self.assertEqual(row[9:],(12,12,1,2))

    def test_transport_is_lossless_deterministic_and_content_addressed(self):
        members={'catalog.hdi':b'catalog identity','atlas-000.hdp':mixed()}
        files=delivery(members)
        self.assertEqual(files,delivery(members))
        info=json.loads(files['HD-WEB.json'])
        self.assertEqual(info['schema_version'],1)
        self.assertEqual(len(info['pages']),2)
        self.assertEqual({tuple(k) for r in info['pages'] for k in r['keys']},{(1,0,0),(1,1,0)})
        import hashlib
        for row in info['pages']:
            compressed=files[row['path']]
            self.assertIn(hashlib.sha256(compressed).hexdigest(),row['path'])
            raw=gzip.decompress(compressed)
            self.assertEqual(len(raw),row['bytes'])
            self.assertEqual(zlib.crc32(raw),row['crc32'])
            self.assertEqual(len(compressed),row['download_bytes'])
        self.assertFalse(any(n.endswith('.hdp') for n in files))

    def test_malformed_atlas_or_duplicate_key_is_rejected(self):
        for raw in [b'broken',mixed()[:-1]]:
            with self.assertRaises(ValueError):delivery({'catalog.hdi':b'c','atlas-000.hdp':raw})
        with self.assertRaises(ValueError):delivery({'catalog.hdi':b'c','atlas-000.hdp':mixed(),'atlas-001.hdp':mixed()})

    def test_write_removes_only_stale_content_addressed_generated_pages(self):
        import tempfile
        from pathlib import Path
        from tools.web_hd import write_delivery
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);folder=root/'web-hd';folder.mkdir()
            stale=folder/('a'*64+'.hdp.gz');stale.write_bytes(b'old')
            user=folder/'notes.txt';user.write_text('keep')
            files={'HD-WEB.json':b'{}','web-hd/'+'b'*64+'.hdp.gz':b'new'}
            write_delivery(files,root)
            self.assertFalse(stale.exists());self.assertEqual(user.read_text(),'keep')
            self.assertEqual((root/('web-hd/'+'b'*64+'.hdp.gz')).read_bytes(),b'new')
