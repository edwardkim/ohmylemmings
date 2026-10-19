from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_digger import collect
from observe_builder_dos import decode_builder, make_runtime_seed
from compare_builder_dos import repeat_builder_verdict


class BuilderTests(unittest.TestCase):
    def test_dos_verdict_requires_two_successes_with_the_same_execution_protocol(self):
        good = dict(status='MATCH_VISUAL_ANCHORS', protocol_sha256='same')
        self.assertEqual(repeat_builder_verdict([good, good]), 'MATCH_FULL_PLAYFIELD')
        for rows in [[], [good], [good, good | dict(status='UNVERIFIED_VISUAL_ANCHORS')],
                     [good, good | dict(protocol_sha256='different')],
                     [good, dict(status='MATCH_VISUAL_ANCHORS')]]:
            self.assertEqual(repeat_builder_verdict(rows), 'UNVERIFIED')
    def test_fresh_runtime_preserves_every_byte_except_the_documented_counter(self):
        from hashlib import sha256
        original = bytes([9])+bytes(range(255))
        seeded = make_runtime_seed(original)
        self.assertEqual([i for i, (a,b) in enumerate(zip(original,seeded)) if a!=b], [0])
        self.assertEqual(seeded[0], 0)
        self.assertEqual(make_runtime_seed(original, sha256(original).hexdigest()), seeded)
        for value, expected in [(original, '0'*64), (original[:-1], None), (seeded, None)]:
            with self.assertRaisesRegex(ValueError, 'generated runtime'):
                make_runtime_seed(value, expected)
    def test_dos_builder_fields_use_inventory_and_brick_offsets_from_original_assignment(self):
        data = bytearray(0x2200)
        data[0x76], data[0x79], data[0x7c], data[0x7f] = 20, 3, 2, 1
        data[0x90], data[0xa9] = 128, 12
        sample = decode_builder(data)
        self.assertEqual(sample['remaining_builders'], 20)
        self.assertEqual(sample['actor0_bricks'], 12)
        self.assertEqual(sample['actor0_action_bits'], 128)
        self.assertEqual([sample[k] for k in ['remaining_bashers', 'remaining_miners', 'remaining_diggers']], [3, 2, 1])
    def fixture(self, root, filled='0'):
        (root/'trace.jsonl').write_text('{"control_step":0}\n{"control_step":1}\n')
        (root/'digger.jsonl').write_text('{"cleared":[]}\n{"cleared":[1]}\n')
        (root/'builder.jsonl').write_text('{"filled":[]}\n{"filled":['+filled+']}\n')
        (root/'terrain-initial.bin').write_bytes(bytes([0, 1]))
        (root/'terrain-final.bin').write_bytes(bytes([1, 0]))

    def test_additions_and_deletions_reconstruct_the_complete_final_terrain(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.fixture(root)
            self.assertEqual(len(collect(root)), 2)

    def test_duplicate_outside_boolean_and_nonempty_additions_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for filled in ['0,0', '-1', '2', 'false', '1']:
                self.fixture(root, filled)
                with self.assertRaisesRegex(ValueError, 'invalid or repeated terrain addition'):
                    collect(root)

    def test_missing_addition_boundary_cannot_hide_terrain_changes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.fixture(root)
            (root/'builder.jsonl').write_text('{"filled":[]}\n')
            with self.assertRaisesRegex(ValueError, 'Builder'):
                collect(root)


if __name__ == '__main__':
    unittest.main()
