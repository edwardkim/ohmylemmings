from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from observe_playthrough_dos import validate_result

class DosPlaythroughTests(unittest.TestCase):
    def samples(self):
        start=dict(playing_u8=255,finish_requested_u8=0,level_index_u8=0,startup_counter_u8=10,max_lemmings_u8=10,
            remaining_diggers=10,actor0_action_bits=0)
        dig=dict(start,startup_counter_u8=61,remaining_diggers=9,actor0_action_bits=16)
        transient=dict(dig,playing_u8=0,finish_requested_u8=255,result_success_u8=0,saved_u8=10,rescued_percent_u8=100,target_percent_u8=10)
        final=dict(transient,level_index_u8=1,result_success_u8=255)
        return [start,dig,transient,final]
    def test_success_advances_dos_index_before_results_are_dismissed(self):
        self.assertEqual(validate_result(self.samples(),'win')['level_index_u8'],1)
    def test_transient_result_cannot_override_later_completed_result(self):
        s=self.samples()
        with self.assertRaises(ValueError):validate_result(s[:-1],'win')
    def test_wrong_start_level_or_percentage_is_rejected(self):
        s=self.samples();s[0]['level_index_u8']=1;s[1]['level_index_u8']=1
        with self.assertRaises(ValueError):validate_result(s,'win')
        s=self.samples();s[-1]['target_percent_u8']=11
        with self.assertRaises(ValueError):validate_result(s,'win')
