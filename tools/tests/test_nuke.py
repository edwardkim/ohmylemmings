import copy
import sys
from pathlib import Path
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from check_nuke import validate_nuke_history
from check_bomber import validate_bomber_history
from observe_nuke_dos import decode_nuke


class NukeTests(unittest.TestCase):
    def test_original_nuke_scan_maximum_and_three_timer_offsets_are_distinct(self):
        data=bytearray(0x2200);data[0x43]=3;data[0x47]=3;data[0x4b]=255;data[0x4c]=2;data[0x70]=5
        for i,timer in enumerate([78,79,0]):data[0x88+i*45+32]=timer
        row=decode_nuke(data)
        self.assertEqual((row['max_lemmings_u8'],row['nuke_assigning_u8'],row['nuke_next_index_u8'],row['remaining_bombers']),(3,255,2,5))
        self.assertEqual([a['timer'] for a in row['actors']],[78,79,0])
        with self.assertRaises(ValueError):decode_nuke(data[:-1])

    def test_actual_nuke_log_keeps_id_free_inputs_and_pause_resume_order(self):
        from verify_nuke import parse_ui_inputs
        self.assertEqual(parse_ui_inputs('120 0 Nuke\n130 0 Pause\n150 0 Resume\n'),[(120,0,'Nuke',0),(130,0,'Pause',0),(150,0,'Resume',0)])
        for text in ['120 0 Nuke 0','120 0 Nuke\n119 0 Pause','-1 0 Nuke','120 0 Unknown']:
            with self.assertRaises(ValueError):parse_ui_inputs(text)

    def test_nuke_keeps_rearmed_ohno_timer_and_original_denominator_separate(self):
        states=[dict(released=1,actors=[dict(id=0,action='OhNo',removed=None)])]
        details=[dict(remaining=10,actors=[[0,79,0,0]])]
        with self.assertRaises(ValueError):validate_bomber_history(copy.deepcopy(states),details)
        validate_bomber_history(states,details,allow_nuke=True)
        rows=[dict(is_nuked=True,assigning=True,next_index=1,max_lemmings=1)]
        validate_nuke_history(states,rows)
        self.assertEqual(states[0]['nuke'],rows[0])

    def test_nuke_rejects_missing_malformed_and_impossible_scan_states(self):
        base=[dict(released=1,actors=[],bomber=dict(actors=[]))]
        valid=dict(is_nuked=True,assigning=True,next_index=1,max_lemmings=1)
        for change in [dict(is_nuked=1),dict(assigning=1),dict(next_index=True),dict(next_index=-1),dict(next_index=2),dict(max_lemmings=0),dict(max_lemmings=False),dict(is_nuked=False),dict(max_lemmings=2**32)]:
            with self.assertRaises(ValueError):validate_nuke_history(copy.deepcopy(base),[valid|change])
        with self.assertRaises(ValueError):validate_nuke_history(base,[])
