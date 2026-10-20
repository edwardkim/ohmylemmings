import copy
from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from verify_playthrough import validate_journey

class PlaythroughTests(unittest.TestCase):
    def fixture(self):
        initial=dict(control_step=0,iteration=0,actors=[],released=0,paused=False,seconds_remaining=300,
            completion=dict(phase='Playing'),nuke=dict(is_nuked=False,max_lemmings=3))
        histories=[]
        for phase in ['Won','Lost','Won','Playing']:
            final=dict(initial,control_step=160,actors=[dict(id=0)],released=3,
                completion=dict(phase=phase,rescued_percent=100 if phase=='Won' else 0),
                nuke=dict(is_nuked=phase=='Lost',max_lemmings=1 if phase=='Lost' else 3))
            histories.append([copy.deepcopy(initial),final])
        configs=[dict(level=n,total=3) for n in [1,1,1,2]]
        transitions=[dict(attempt=i,level=1,step=160,action=a,target=t)
            for i,(a,t) in enumerate([('Restart',1),('Continue',1),('Continue',2)])]
        return histories,configs,transitions
    def test_win_loss_retry_win_and_next_level_is_complete(self):
        self.assertEqual(validate_journey(*self.fixture()),[True,False,True])
    def test_missing_attempt_and_wrong_transition_are_rejected(self):
        for field,value in [('attempt',2),('level',2),('target',2),('step',159),('action','Continue')]:
            h,c,t=self.fixture();t[0][field]=value
            with self.assertRaises(ValueError):validate_journey(h,c,t)
        h,c,t=self.fixture()
        with self.assertRaises(ValueError):validate_journey(h[:3],c,t)
    def test_wrong_outcome_or_partial_rescue_cannot_count_as_playthrough(self):
        for patch in [dict(phase='Playing'),dict(phase='Lost'),dict(rescued_percent=99)]:
            h,c,t=self.fixture();h[0][-1]['completion'].update(patch)
            with self.assertRaises(ValueError):validate_journey(h,c,t)
    def test_restarting_keeps_config_and_resets_clock_actors_and_nuke(self):
        for patch in [dict(actors=[dict(id=0)]),dict(paused=True),dict(iteration=1),dict(seconds_remaining=299)]:
            h,c,t=self.fixture();h[1][0].update(patch)
            with self.assertRaises(ValueError):validate_journey(h,c,t)
        h,c,t=self.fixture();c[2]['total']=1
        with self.assertRaises(ValueError):validate_journey(h,c,t)
        h,c,t=self.fixture();h[1][-1]['nuke']['is_nuked']=False
        with self.assertRaises(ValueError):validate_journey(h,c,t)
    def test_next_level_must_start_its_own_actor_ids(self):
        h,c,t=self.fixture();h[3][-1]['actors'][0]['id']=3
        with self.assertRaises(ValueError):validate_journey(h,c,t)
