import copy
from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_completion import validate_completion


class CompletionTests(unittest.TestCase):
    def history(self):
        row = dict(phase='Playing', reason=None, original_total=80, lemming_count=2,
                   required=40, rescued=1, target_percent=50, rescued_percent=50)
        state = dict(seconds_remaining=1, released=2, removed=1, alive=1,
                     exit=dict(saved=1), nuke=dict(max_lemmings=2,is_nuked=True), bomber=dict(actors=[[0,0,0,0],[1,0,0,0]]))
        final = copy.deepcopy(state);final.update(removed=2,alive=0)
        return [state,final],[row,dict(row,phase='Won',reason='PopulationExhausted')],dict(total=80,required=40)
    def test_nuke_uses_smaller_done_denominator_and_original_target(self):
        states,rows,config=self.history();validate_completion(states,rows,config)
        self.assertEqual(states[-1]['completion']['rescued_percent'],50)
    def test_bad_schema_boolean_counts_and_percentage_are_rejected(self):
        for patch in [dict(extra=0),dict(rescued=True),dict(target_percent=40),dict(rescued_percent=1),dict(phase='Won'),dict(lemming_count=80)]:
            states,rows,config=self.history();rows[0].update(patch)
            with self.assertRaises(ValueError):validate_completion(states,rows,config)
    def test_missing_and_premature_terminal_states_are_rejected(self):
        states,rows,config=self.history()
        with self.assertRaises(ValueError):validate_completion(states,rows[:1],config)
        states,rows,config=self.history();rows[-1].update(phase='Playing',reason=None)
        with self.assertRaises(ValueError):validate_completion(states,rows,config)
        states,rows,config=self.history();states[-1].update(removed=1,alive=1)
        with self.assertRaises(ValueError):validate_completion(states,rows,config)
    def test_timeout_bypasses_particles_and_zero_denominator_is_defined(self):
        states,rows,config=self.history();states[0]['seconds_remaining']=0
        states[1].update(removed=1,alive=1);states[1]['bomber']['actors'][0][2]=52
        rows[1].update(reason='TimeExpired');validate_completion(states,rows,config)
        states,rows,config=self.history()
        states=[dict(states[0],released=0,removed=0,alive=0,exit=dict(saved=0),nuke=dict(max_lemmings=0,is_nuked=True),bomber=dict(actors=[]))]
        rows=[dict(rows[0],phase='Lost',reason='PopulationExhausted',lemming_count=0,rescued=0,rescued_percent=0)]
        states.insert(0,dict(states[0],nuke=dict(max_lemmings=80,is_nuked=False)))
        rows.insert(0,dict(rows[0],phase='Playing',reason=None,lemming_count=80))
        validate_completion(states,rows,config)

    def test_frozen_result_cannot_advance_clock_or_change_counts(self):
        states,rows,config=self.history()
        states.append(copy.deepcopy(states[-1]));rows.append(copy.deepcopy(rows[-1]))
        states[-1]['seconds_remaining']=2
        with self.assertRaises(ValueError):validate_completion(states,rows,config)
