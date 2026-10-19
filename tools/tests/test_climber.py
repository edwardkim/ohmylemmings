import copy
import sys
from pathlib import Path
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from check_climber import validate_climber_history
from observe_climber_dos import decode_climber

class ClimberTests(unittest.TestCase):
    def test_original_inventory_and_permanent_ability_use_distinct_offsets(self):
        data=bytearray(0x2200);data[0x6a]=9;data[0x73]=4;data[0xac]=1;data[0x90]=32
        row=decode_climber(data)
        self.assertEqual((row['remaining_climbers'],row['remaining_blockers'],row['actor0_ability_flags'],row['actor0_action_bits']),(9,4,1,32))
        with self.assertRaises(ValueError):decode_climber(data[:-1])
    def test_permanent_and_stale_once_flags_are_real_booleans(self):
        states=[dict(actors=[dict(id=0,action='Climbing')]),dict(actors=[dict(id=0,action='Walking')])]
        rows=[dict(remaining=0,actors=[[0,True,False]]),dict(remaining=0,actors=[[0,True,True]])]
        validate_climber_history(states,rows)
        self.assertEqual(states[1]['climber'],rows[1])
    def test_missing_bad_ids_and_impossible_ability_histories_are_rejected(self):
        base=[dict(actors=[dict(id=0,action='Climbing')])]
        for entry in [[0,False,False],[0,True,True],[0,1,False],[1,True,False],[0,True],[]]:
            with self.assertRaises(ValueError):validate_climber_history(copy.deepcopy(base),[dict(remaining=0,actors=[entry])])
        with self.assertRaises(ValueError):validate_climber_history(base,[])
