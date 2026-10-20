import copy
import json
import unittest
from tools import check_game_sound as check

class GameSoundTests(unittest.TestCase):
    def test_original_oracle_covers_every_case_and_music_state_without_guest_code(self):
        fixture=json.loads(check.FIXTURE.read_text())
        check.validate_fixture(fixture)
        self.assertEqual(len(fixture['effect_cases']),97)
        self.assertEqual(len(fixture['music_cases']),170)
        self.assertNotIn('code',fixture)
        bad=copy.deepcopy(fixture);bad['effect_cases'].append(bad['effect_cases'][0])
        with self.assertRaises(ValueError):check.validate_fixture(bad)
    def test_requests_match_exact_order_and_original_effect_numbers(self):
        case=dict(name='landing-61',calls=[[97,1032]])
        check.validate_requests(case,[[97,1032]])
        for bad in [[[97,1042]],[],[[97,1032],[97,1032]],[[True,1032]]]:
            with self.assertRaises(ValueError):check.validate_requests(dict(case,calls=bad),[[97,1032]])
    def test_pcm_schedule_preserves_same_boundary_requests_and_exact_rational_offsets(self):
        case=dict(start_control_step=10,steps=2,sounds=dict(events=[
            dict(source_unused=None,control_step=10,command=['effect',7]),
            dict(control_step=10,command=['effect',15]),
            dict(control_step=11,command=['music',5])]))
        actions=check.pcm_actions(case)
        self.assertEqual(actions,'e7:0,e15:3146,m5:9438')
    def test_schedule_rejects_early_late_unsorted_and_invalid_commands(self):
        base=dict(start_control_step=0,steps=2,sounds=dict(events=[dict(control_step=0,command=['effect',3])]))
        for event in [dict(control_step=-1,command=['effect',3]),dict(control_step=2,command=['effect',3]),
                      dict(control_step=True,command=['effect',3]),dict(control_step=0,command=['effect',18]),
                      dict(control_step=0,command=['music',21]),dict(control_step=0,command=['stop',0])]:
            with self.assertRaises(ValueError):check.pcm_actions(dict(base,sounds=dict(events=[event])))
    def test_browser_requires_actual_trace_and_every_pcm_hash(self):
        expected=dict(trace_sha256='a'*64,pcm=[dict(name='x',frames=1,sha256='b'*64)])
        good=dict(status='PASS',**expected);check.validate_browser(good,expected)
        for bad in [dict(good,status='FAIL'),dict(good,pcm=[]),dict(good,trace_sha256='c'*64)]:
            with self.assertRaises(ValueError):check.validate_browser(bad,expected)

if __name__=='__main__':unittest.main()
