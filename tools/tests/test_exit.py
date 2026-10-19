import copy
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_exit import validate_exit_history


class ExitTests(unittest.TestCase):
    def states(self):
        actor = dict(id=0, action='Exiting', frame=0, end_of_animation=False, removed=None)
        done = dict(actor, frame=7, end_of_animation=True, removed='Rescued')
        return [dict(released=1, alive=1, removed=0, actors=[actor]),
                dict(released=1, alive=0, removed=1, actors=[done])]

    def test_rescue_is_a_removal_and_is_counted_once_across_later_boundaries(self):
        states = self.states()
        states.append(copy.deepcopy(states[-1]))
        rows = [dict(saved=0, exiting=[0]), dict(saved=1, exiting=[]), dict(saved=1, exiting=[])]
        validate_exit_history(states, rows)
        self.assertEqual([s['exit']['saved'] for s in states], [0, 1, 1])

    def test_missing_or_duplicate_rescue_counts_and_ids_are_rejected(self):
        rows = [dict(saved=0, exiting=[0]), dict(saved=1, exiting=[])]
        for bad in [[], rows[:1], rows + rows[-1:],
                    [dict(saved=0, exiting=[0, 0]), rows[1]],
                    [rows[0], dict(saved=2, exiting=[])],
                    [rows[0], dict(saved=1, exiting=[0])]]:
            with self.assertRaises(ValueError):
                validate_exit_history(self.states(), bad)

    def test_rescue_requires_final_exit_animation_and_consistent_population(self):
        for patch in [dict(action='Walking'), dict(frame=6), dict(end_of_animation=False)]:
            states = self.states()
            states[1]['actors'][0].update(patch)
            with self.assertRaises(ValueError):
                validate_exit_history(states, [dict(saved=0, exiting=[0]), dict(saved=1, exiting=[])])
        states = self.states()
        states[1]['alive'] = 1
        with self.assertRaises(ValueError):
            validate_exit_history(states, [dict(saved=0, exiting=[0]), dict(saved=1, exiting=[])])

    def test_counter_boolean_wrong_schema_and_reversed_history_are_rejected(self):
        for patch in [dict(saved=True), dict(saved=-1), dict(extra=0), dict(exiting=None)]:
            rows = [dict(saved=0, exiting=[0]), dict(saved=1, exiting=[])]
            rows[1].update(patch)
            with self.assertRaises(ValueError):
                validate_exit_history(self.states(), rows)
        with self.assertRaises(ValueError):
            validate_exit_history(list(reversed(self.states())), [dict(saved=1, exiting=[]), dict(saved=0, exiting=[0])])
