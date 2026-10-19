"""Guard pixel normalization and explicit scope of original toolbar comparisons."""
from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))

from compare_panel_dos import normalize, quantize, region, REGIONS


class PanelComparisonTests(unittest.TestCase):
    def test_nearest_double_checks_the_bottom_hud_rows_too(self):
        logical=bytes([8,12,24])*(320*200)
        doubled=bytes([8,12,24])*(640*400)
        self.assertEqual(normalize(320,200,logical),logical)
        self.assertEqual(normalize(640,400,doubled),logical)
        corrupt=bytearray(doubled)
        corrupt[-1]=25
        with self.assertRaises(ValueError):normalize(640,400,bytes(corrupt))
        with self.assertRaises(ValueError):normalize(640,400,doubled[:-1])

    def test_rgb565_normalization_does_not_accept_a_whole_component_code_difference(self):
        self.assertEqual(quantize(bytes([8,12,24])),quantize(bytes([15,15,31])))
        self.assertNotEqual(quantize(bytes([8,12,24])),quantize(bytes([16,12,24])))

    def test_regions_cover_info_and_map_and_explicitly_exclude_only_three_slots(self):
        covered=set()
        for item in REGIONS:
            x0,y0,x1,y1=item['rect']
            self.assertTrue(0<=x0<x1<=320 and 160<=y0<y1<=200)
            points={(x,y) for y in range(y0,y1) for x in range(x0,x1)}
            self.assertFalse(points & covered)
            covered |= points
        excluded={(x,y) for y in range(176,200) for x in range(320) if x//16 in [2,8,9]}
        self.assertEqual(len(covered),11648)
        self.assertEqual(covered | excluded,{(x,y) for y in range(160,200) for x in range(320)})
        self.assertEqual(len(region(bytes(320*200*3),[208,178,312,198])),104*20*3)
