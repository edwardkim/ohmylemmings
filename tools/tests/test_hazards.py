import copy
from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from check_hazards import validate_hazards


class HazardTests(unittest.TestCase):
    def states(self):
        return [dict(actors=[dict(id=0,action='Drowning',frame=0,end_of_animation=False,removed=None)]),
                dict(actors=[dict(id=0,action='Drowning',frame=15,end_of_animation=True,removed='Drowned')])]
    def rows(self):
        return [dict(traps=[[0,0,False]],drowned=0,vaporized=0,trapped=0),dict(traps=[[0,1,True]],drowned=1,vaporized=0,trapped=0)]
    def test_terminal_death_is_not_a_rescue_and_is_counted_once(self):
        states=self.states();rows=self.rows();states.append(copy.deepcopy(states[-1]));rows.append(copy.deepcopy(rows[-1]))
        validate_hazards(states,rows,bytes([3,0,0])+bytes(45));self.assertEqual(states[-1]['hazards']['drowned'],1)
    def test_missing_duplicate_trap_ids_and_wrong_counter_schema_are_rejected(self):
        rows=self.rows()
        for bad in [[],rows[:1],rows+[rows[-1]],[rows[0],dict(rows[1],traps=[[0,1,True],[0,1,True]])],[rows[0],dict(rows[1],drowned=2)],[rows[0],dict(rows[1],drowned=True)],[rows[0],dict(rows[1],extra=0)]]:
            with self.assertRaises(ValueError):validate_hazards(self.states(),bad,bytes([3,0,0])+bytes(45))
    def test_truncated_invalid_metadata_and_trap_frames_are_rejected(self):
        for data in [b'',bytes(47),bytes([3,3,0])+bytes(45),bytes([3,0,2])+bytes(45),bytes([0,1,0])+bytes(45)]:
            with self.assertRaises(ValueError):validate_hazards(self.states(),self.rows(),data)
        for trap in [[0,3,False],[0,-1,False],[1,0,False],[0,0,1],[False,0,False]]:
            rows=self.rows();rows[1]['traps']=[trap]
            with self.assertRaises(ValueError):validate_hazards(self.states(),rows,bytes([3,0,0])+bytes(45))
    def test_finished_death_requires_correct_action_frame_and_animation_end(self):
        for patch in [dict(action='Walking'),dict(frame=14),dict(end_of_animation=False)]:
            states=self.states();states[1]['actors'][0].update(patch)
            with self.assertRaises(ValueError):validate_hazards(states,self.rows(),bytes([3,0,0])+bytes(45))
        states=self.states();states[1]['actors'][0]['removed']='Trapped';rows=self.rows();rows[1].update(drowned=0,trapped=1)
        with self.assertRaises(ValueError):validate_hazards(states,rows,bytes([3,0,0])+bytes(45))
