import unittest
from tools.initial_reference import draw_scene, placements, trigger_maps


class InitialReferenceTests(unittest.TestCase):
    def fixture(self):
        meta=bytearray(1056);meta[2:6]=bytes([0,1,8,1]);meta[6:10]=bytes([5,0,4,0])
        # Opaque color0 atx0; transparent color1 atx1.
        return meta,bytes([0x40,0,0,0,0x80]),[[0,0,0,0,False,False,False]]

    def test_independent_compositor_preserves_opaque_black_and_masked_color(self):
        meta,data,objects=self.fixture();palette=bytes([0]*48);terrain=bytes([128]*8)
        rgba=draw_scene(terrain,palette,bytes(24),objects,meta,data,True,width=8,height=1)
        self.assertEqual(rgba[:4],bytes([0,0,0,255]));self.assertEqual(rgba[4:],bytes(28))

    def test_only_on_terrain_precedes_no_overwrite_and_clips(self):
        meta,data,objects=self.fixture();objects[0][5:]=[True,True]
        rgba=draw_scene(bytes([1]+[128]*7),bytes(48),bytes([30]*24),objects,meta,data,True,width=8,height=1)
        self.assertEqual(rgba[:4],bytes([0,0,0,255]))
        objects[0][1]=-1
        self.assertEqual(draw_scene(bytes([128]*8),bytes(48),bytes(24),objects,meta,data,True,width=8,height=1),bytes(32))

    def test_object_record_uses_unsigned_words_and_keeps_holes(self):
        level=bytearray(2048);level[40:48]=bytes([0,0,255,255,0,17,192,143])
        self.assertEqual(placements(level),[[1,-16,65535,1,True,True,True]])

    def test_steel_override_and_non_entrance_compact_object_limit_differ(self):
        level=bytearray(2048);level[1888:1892]=bytes([2,0,0,0]) # steel at0,0 size4x4
        ground=bytearray(1056)
        for oid,effect in [(1,7),(2,4)]:
            at=oid*28;ground[at+16:at+21]=bytes([1,0,1,1,effect])
        objects=[[0,0,0,1,False,False,False]]+[[i+1,i*4,0,2,False,False,False] for i in range(17)]
        material,raw=trigger_maps(level,objects,ground)
        self.assertEqual(material[0],0) # slot1 nonsteel trigger clears material again
        self.assertEqual(raw[4*415+4],0) # entrance excluded; first compact trap0
        self.assertEqual(raw[4*415+19],15)
        self.assertEqual(raw[4*415+20],128) # seventeenth nonentrance disabled
