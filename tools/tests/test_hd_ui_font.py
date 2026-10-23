import json,unittest
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]

class HdUiFontTests(unittest.TestCase):
    def test_pinned_font_covers_printable_ascii_with_smooth_alpha_and_safe_gutters(self):
        from tools.hd_ui_font import verify
        r=verify(ROOT)
        self.assertEqual(r['glyphs'],95)
        self.assertEqual(r['nonempty_glyphs'],94)
        self.assertTrue(r['partial_alpha'])
        self.assertLessEqual(r['runtime_rgba_bytes'],5*1024*1024)
