import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_basher import collect, replay_text, synthetic_masks
from observe_basher_dos import decode_basher


class BasherTests(unittest.TestCase):
    def test_synthetic_masks_are_eight_disjoint_directional_slices(self):
        masks = synthetic_masks()
        self.assertEqual(len(masks), 160)
        for direction in range(2):
            for frame in range(4):
                for y in range(10):
                    row = masks[(direction*4+frame)*20+y*2:(direction*4+frame)*20+y*2+2]
                    self.assertEqual(int.from_bytes(row, 'big'),
                                     0xc0 >> (frame*2) if direction == 0 else 0x300 << (frame*2))

    def test_replay_keeps_same_boundary_sequence_and_job_changes(self):
        inputs = [(80, 0, 'Pause', 0), (80, 1, 'AssignBasher', 3), (80, 2, 'AssignDigger', 3), (90, 0, 'Resume', 0)]
        self.assertEqual(replay_text(inputs), '80 0 Pause\n80 1 AssignBasher 3\n80 2 AssignDigger 3\n90 0 Resume\n')

    def test_missing_inventory_boundary_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root/'trace.jsonl').write_text('{"control_step":0}\n')
            (root/'digger.jsonl').write_text('{"cleared":[]}\n')
            (root/'basher.jsonl').write_text('')
            (root/'terrain-initial.bin').write_bytes(b'\x01')
            (root/'terrain-final.bin').write_bytes(b'\x01')
            with self.assertRaisesRegex(ValueError, 'Basher inventory'):
                collect(root)
            (root/'basher.jsonl').write_text('{"remaining":50}\n')
            self.assertEqual(collect(root)[0]['basher'], dict(remaining=50))

    def test_read_only_decoder_distinguishes_basher_and_digger_inventory(self):
        data = bytearray(0x2200)
        data[0x79], data[0x7f] = 49, 7
        decoded = decode_basher(data)
        self.assertEqual(decoded['remaining_bashers'], 49)
        self.assertEqual(decoded['remaining_diggers'], 7)


if __name__ == '__main__':
    unittest.main()
