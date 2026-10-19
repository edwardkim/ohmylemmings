import sys
from pathlib import Path
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from check_blocker import validate_map_history, map_signature
from observe_blocker_dos import decode_blocker

class BlockerTests(unittest.TestCase):
    def test_original_blocker_inventory_action_and_saved_map_offsets(self):
        data=bytearray(0x2200);data[0x73]=10;data[0x76]=20;data[0x91]=1
        data[0x9e:0xa7]=bytes(range(9))
        sample=decode_blocker(data)
        self.assertEqual(sample['remaining_blockers'],10)
        self.assertEqual(sample['actor0_action_bits'],256)
        self.assertEqual(sample['actor0_saved_map'],list(range(9)))
    def test_raw_object_changes_restore_codes_and_match_each_boundary(self):
        rows=[dict(changed=[],object_map_fnv1a64=map_signature(bytes([137,128]))),
              dict(changed=[[0,130],[1,138]],object_map_fnv1a64=map_signature(bytes([130,138]))),
              dict(changed=[[0,137],[1,128]],object_map_fnv1a64=map_signature(bytes([137,128])))]
        validate_map_history(rows,bytes([137,128]),bytes([137,128]))
        for changes in [[[0,130],[0,138]],[[-1,130]],[[2,130]],[[False,130]],[[0,256]],[[0,137]]]:
            with self.assertRaises(ValueError):validate_map_history([rows[0],rows[1]|dict(changed=changes)],bytes([137,128]),bytes([130,138]))
    def test_missing_history_wrong_hash_and_changed_initial_boundary_are_rejected(self):
        row=dict(changed=[],object_map_fnv1a64=map_signature(bytes([128])))
        for rows,final in [([],bytes([128])),([row],bytes([137])),([row|dict(changed=[[0,130]])],bytes([130])),([row|dict(object_map_fnv1a64='0'*16)],bytes([128]))]:
            with self.assertRaises(ValueError):validate_map_history(rows,bytes([128]),final)
if __name__=='__main__':unittest.main()
