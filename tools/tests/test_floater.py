import copy
import sys
from pathlib import Path
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from check_floater import validate_floater_history
from observe_floater_dos import decode_floater

class FloaterTests(unittest.TestCase):
    def test_original_ability_bytes_and_table_cursor_do_not_alias_inventory(self):
        data=bytearray(0x2200);data[0x6d]=9;data[0xac]=1;data[0xad]=255;data[0xab]=18;data[0x91]=4;data[0x96:0x98]=(34).to_bytes(2,'little')
        r=decode_floater(data)
        self.assertEqual((r['remaining_floaters'],r['actor0_climber_byte'],r['actor0_floater_byte'],r['actor0_fall_counter_u8'],r['actor0_float_cursor_u16'],r['actor0_action_bits']),(9,1,255,18,34,1024))
        with self.assertRaises(ValueError):decode_floater(data[:-1])
    def test_table_cursor_is_retained_on_landing_and_both_booleans_are_explicit(self):
        states=[dict(actors=[dict(id=0,action='Floating')]),dict(actors=[dict(id=0,action='Walking')])]
        rows=[dict(remaining=9,actors=[[0,True,15]]),dict(remaining=9,actors=[[0,True,8]])]
        validate_floater_history(states,rows);self.assertEqual(states[1]['floater'],rows[1])
    def test_impossible_missing_and_malformed_floater_rows_cannot_pass(self):
        base=[dict(actors=[dict(id=0,action='Floating')])]
        for entry in [[0,False,0],[0,True,16],[0,True,-1],[0,1,0],[0,True,False],[1,True,0],[0,True],[]]:
            with self.assertRaises(ValueError):validate_floater_history(copy.deepcopy(base),[dict(remaining=9,actors=[entry])])
        with self.assertRaises(ValueError):validate_floater_history(base,[])
    def test_actual_ui_log_accepts_pause_resume_and_rate_without_losing_target_id(self):
        from verify_floater import parse_ui_inputs
        self.assertEqual(parse_ui_inputs('54 0 AssignFloater 0\n231 0 Pause\n277 0 Resume\n280 0 SetReleaseRate 51\n'),[(54,0,'AssignFloater',0),(231,0,'Pause',0),(277,0,'Resume',0),(280,0,'SetReleaseRate',51)])
        for text in ['54 0 AssignFloater','54 0 Pause 0','54 0 Resume 0','54 0 Unknown 0','-1 0 Pause','54 0 AssignFloater -1','54 0 Pause\n53 0 Resume']:
            with self.assertRaises(ValueError):parse_ui_inputs(text)
