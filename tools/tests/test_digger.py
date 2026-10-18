import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_digger import collect, replay_text
from observe_digger_dos import decode_digger


class DiggerTests(unittest.TestCase):
    def test_read_only_field_decoder_uses_documented_dos_offsets(self):
        raw = bytearray(0x2200)
        for offset, value in [(0x10,112), (0x12,70), (0,640), (0x88,744), (0x8a,84), (0x90,16)]:
            struct.pack_into('<h', raw, offset, value)
        raw[0x7f],raw[0x82],raw[0xb0] = 9,9,8
        before = bytes(raw)
        sample = decode_digger(raw)
        self.assertEqual((sample['actor0_x'],sample['actor0_y'],sample['actor0_frame']), (744,84,8))
        self.assertEqual((sample['cursor_x'],sample['cursor_y'],sample['remaining_diggers']), (112,70,9))
        self.assertEqual(raw, before)
        with self.assertRaises(ValueError): decode_digger(raw[:-1])

    def test_ordered_deletions_reconstruct_the_entire_final_mask(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root/'terrain-initial.bin').write_bytes(bytes([1,0,1]))
            (root/'terrain-final.bin').write_bytes(bytes([0,0,0]))
            (root/'trace.jsonl').write_text('{"control_step":0}\n{"control_step":1}\n')
            (root/'digger.jsonl').write_text('{"cleared":[]}\n{"cleared":[0,2]}\n')
            self.assertEqual(len(collect(root)),2)
            for invalid in [[2,0],[0,0],[1],[3]]:
                (root/'digger.jsonl').write_text('{"cleared":[]}\n'+json.dumps(dict(cleared=invalid))+'\n')
                with self.assertRaises(ValueError): collect(root)

    def test_digger_and_pause_inputs_preserve_boundary_sequence_and_id(self):
        self.assertEqual(replay_text([(60,0,'Pause',0),(60,1,'AssignDigger',7),(70,0,'Resume',0)]),
                         '60 0 Pause\n60 1 AssignDigger 7\n70 0 Resume\n')


if __name__ == '__main__':
    unittest.main()
