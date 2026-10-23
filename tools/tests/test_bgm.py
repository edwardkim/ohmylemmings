import struct,unittest
from tools.check_bgm import note_trace,driver_actions,require_sustained_notes

class BgmTests(unittest.TestCase):
    def test_note_wire_keeps_adjacent_off_on_at_every_note_boundary(self):
        data=note_trace(22050);rate,frames,count=struct.unpack_from('<III',data)
        self.assertEqual((rate,frames,count),(22050,110250,30))
        rows=[struct.unpack_from('<IBB',data,12+n*6) for n in range(count)]
        for note in range(10):self.assertEqual(rows[10+note*2:12+note*2],[(note*11025,0xb0,0x11),(note*11025,0xb0,0x31)])
    def test_driver_requests_use_rational_boundaries_across_action_changes(self):
        actions,frames=driver_actions('m5:441000,e0:110250,e17:110250,n:661500',22050)
        self.assertEqual(frames,1323000)
        self.assertEqual(sum(int(a.split(':')[1]) for a in actions.split(',')),4206)
    def test_nonzero_dc_or_decaying_note_is_not_sustained_music(self):
        with self.assertRaises(ValueError):require_sustained_notes(struct.pack('<h',3000)*40000,8000)
        samples=[3000 if n%2 else -3000 for n in range(40000)]
        require_sustained_notes(struct.pack('<40000h',*samples),8000)
        samples[4000:]=[1]*36000
        with self.assertRaises(ValueError):require_sustained_notes(struct.pack('<40000h',*samples),8000)
