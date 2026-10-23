import struct,unittest
from tools.decor_fixture import bank,PROFILES
from tools.hd_assets import DECOR_PROFILES

class DecorFixtureTests(unittest.TestCase):
    def test_original_flag_and_arrow_metadata_with_both_direction_masks(self):
        self.assertEqual({p[:2]:p[2] for p in PROFILES},DECOR_PROFILES)
        self.assertEqual(sum(p[2] for p in PROFILES),212)
        for theme in range(5):
            files=bank(theme);ground=files[f'GROUND{theme}O.DAT']
            for s,i,n,w,h,effect in PROFILES:
                if s!=theme:continue
                at=i*28
                self.assertEqual(struct.unpack_from('<HBBBB',ground,at),(2,0,n,w,h))
                self.assertEqual(ground[at+20],effect)
                self.assertEqual(ground[at+27],0)
        for theme in [-1,5,True]:
            with self.assertRaises(ValueError):bank(theme)
