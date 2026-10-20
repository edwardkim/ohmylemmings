import json
from pathlib import Path
import struct
import tempfile
import unittest
from unittest.mock import patch

from tools import check_opl_reference as reference


class OplReferenceTests(unittest.TestCase):
    def test_timeline_preserves_write_order_at_absolute_sample_boundaries(self):
        raw = reference.timeline([
            dict(step=0, writes=[[0x20, 1], [0x20, 2]]),
            dict(step=1, writes=[[0x40, 3]]),
        ], 8000, 200)
        self.assertEqual(struct.unpack('<III', raw[:12]), (8000, 200, 3))
        self.assertEqual([struct.unpack('<IBB', raw[i:i+6])
                          for i in range(12, len(raw), 6)],
                         [(0, 0x20, 1), (0, 0x20, 2), (114, 0x40, 3)])

    def test_truncated_skipped_or_invalid_trace_is_rejected(self):
        for rows in [[], [dict(step=0, writes=[])],
                     [dict(step=1, writes=[])],
                     [dict(step=False, writes=[])],
                     [dict(step=0, writes=[[256, 0]])],
                     [dict(step=0, writes=[[0, True]])]]:
            with self.subTest(rows=rows), self.assertRaises(ValueError):
                reference.timeline(rows, 8000, 200)

    def test_source_hash_and_size_are_both_checked_before_compilation(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            source = root / 'core.c'
            source.write_bytes(b'original')
            manifest = root / 'pins.json'
            import hashlib
            manifest.write_text(json.dumps(dict(files=[dict(
                path='core.c', size=8,
                sha256=hashlib.sha256(b'original').hexdigest())])))
            with patch.object(reference, 'ROOT', root), patch.object(reference, 'MANIFEST', manifest):
                reference.verify_sources()
                for data in [b'changed!', b'original!']:
                    source.write_bytes(data)
                    with self.assertRaises(ValueError):
                        reference.verify_sources()
