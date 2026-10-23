import unittest
from tools.hazard_fixture import bank,PROFILES

class HazardFixtureTests(unittest.TestCase):
    def test_every_original_water_fire_profile_has_distinct_frames_and_trigger_metadata(self):
        self.assertEqual(sum(p[2] for p in PROFILES),84)
        for theme,id,count,width,height,effect,trigger in PROFILES:
            files=bank(theme,id)
            ground=files[f'GROUND{theme}O.DAT'];at=id*28
            self.assertEqual(tuple(ground[at+2:at+6]),(0,count,width,height))
            self.assertEqual(ground[at+20],effect)
            self.assertEqual(tuple(ground[at+18:at+20]),(trigger[2]//4,trigger[3]//4))
            self.assertIn('LEVEL000.DAT',files)
        for theme,id in [(0,6),(1,6),(5,5)]:
            with self.assertRaises(ValueError):bank(theme,id)
