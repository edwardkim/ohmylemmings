import struct,unittest
from tools.trap_fixture import bank,PROFILES
from tools.hd_assets import TRAP_PROFILES

class TrapFixtureTests(unittest.TestCase):
    def test_original_trap_metadata_survives_two_instances_and_a_preceding_nontrap(self):
        self.assertEqual(sum(p[2] for p in PROFILES),159)
        self.assertEqual({(p[0],p[1]):p[2] for p in PROFILES},TRAP_PROFILES)
        for theme,id,count,width,height,trigger,sound in PROFILES:
            files=bank(theme,id);ground=files[f'GROUND{theme}O.DAT'];at=id*28
            self.assertEqual(struct.unpack_from('<HBBBB',ground,at),(1,0,count,width,height))
            self.assertEqual(ground[at+20],4);self.assertEqual(ground[at+27],sound)
            self.assertEqual(struct.unpack_from('<HHBB',ground,at+14),(trigger[0]//4,(trigger[1]+4)//4,trigger[2]//4,trigger[3]//4))
            self.assertEqual(ground[2*28+20],0)
        for theme,id in [(0,5),(1,7),(4,8)]:
            with self.assertRaises(ValueError):bank(theme,id)
