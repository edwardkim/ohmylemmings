import copy
import sys
from pathlib import Path
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from check_bomber import validate_bomber_history
from observe_bomber_dos import decode_bomber

class BomberTests(unittest.TestCase):
    def test_original_bomber_inventory_timer_and_blocker_offsets_are_distinct(self):
        data=bytearray(0x2200);data[0x70]=4;data[0x73]=3;data[0xa8]=79;data[0x91]=1
        row=decode_bomber(data)
        self.assertEqual((row['remaining_bombers'],row['remaining_blockers'],row['actor0_explosion_timer'],row['actor0_action_bits']),(4,3,79,256))
        data[0x90:0x92]=bytes([2,129]);self.assertEqual(decode_bomber(data)['actor0_action_bits'] & 0xffff,0x8102)
        with self.assertRaises(ValueError):decode_bomber(data[:-1])
    def test_countdown_and_removed_particle_boundaries_validate(self):
        states=[dict(actors=[dict(id=0,action='Walking',removed=None)]),dict(actors=[dict(id=0,action='Exploding',removed='Exploded')])]
        rows=[dict(remaining=4,actors=[[0,78,0,0]]),dict(remaining=4,actors=[[0,0,52,0]])]
        validate_bomber_history(states,rows)
        self.assertEqual(states[1]['bomber'],rows[1])
    def test_missing_malformed_and_impossible_bomber_states_cannot_pass(self):
        base=[dict(actors=[dict(id=0,action='OhNo',removed=None)])]
        for entry in [[0,80,0,0],[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,51],[0,False,0,0],[0,0,0],[]]:
            with self.assertRaises(ValueError):validate_bomber_history(copy.deepcopy(base),[dict(remaining=4,actors=[entry])])
        with self.assertRaises(ValueError):validate_bomber_history(base,[])
