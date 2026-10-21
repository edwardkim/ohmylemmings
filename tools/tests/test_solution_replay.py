import copy
import unittest
from tools.solution_replay import parse_inputs, outcome, validate_restart


class SolutionReplayTests(unittest.TestCase):
    def test_inputs_require_ordered_unique_boundaries_and_real_commands(self):
        self.assertEqual(parse_inputs('60 0 AssignDigger 0\n60 1 Pause\n61 0 Resume\n',100),
                         [[60,0,'AssignDigger',0],[60,1,'Pause',0],[61,0,'Resume',0]])
        for text in ['60 0 AssignDigger 0\n60 0 Nuke\n','61 0 Pause\n60 0 Resume\n',
                     '-1 0 Nuke\n','100 0 Nuke\n','0 0 SetRequired 0\n','0 0 Nuke 1\n',
                     '0 0 AssignBuilder -1\n','0 0 SetReleaseRate 100\n','0 0 Pause']:
            with self.subTest(text=text),self.assertRaises(ValueError):parse_inputs(text,100)

    def rows(self,phase='Won',rescued=2):
        initial=dict(total=10,required=2)
        return initial,[dict(session=dict(control_step=0),completion=dict(phase='Playing',original_total=10,required=2,rescued=0)),
                        dict(session=dict(control_step=4),completion=dict(phase=phase,original_total=10,required=2,rescued=rescued,reason='PopulationExhausted'))]

    def test_win_requires_actual_result_and_original_goal_without_counter_replacement(self):
        initial,rows=self.rows();self.assertEqual(outcome(initial,rows,'Won')['rescued'],2)
        for key,value in [('phase','Playing'),('rescued',1),('required',1),('original_total',1)]:
            broken=copy.deepcopy(rows);broken[-1]['completion'][key]=value
            with self.assertRaises(ValueError):outcome(initial,broken,'Won')
        broken=copy.deepcopy(rows);broken[0]['completion']['required']=0
        with self.assertRaises(ValueError):outcome(initial,broken,'Won')

    def test_loss_must_finish_below_original_goal_with_explicit_reason(self):
        initial,rows=self.rows('Lost',0);outcome(initial,rows,'Lost')
        for phase,rescued,reason in [('Playing',0,None),('Lost',2,'PopulationExhausted'),('Lost',0,None)]:
            broken=copy.deepcopy(rows);broken[-1]['completion'].update(phase=phase,rescued=rescued,reason=reason)
            with self.assertRaises(ValueError):outcome(initial,broken,'Lost')

    def test_restart_compares_every_initial_field_and_each_map(self):
        initial=dict(state={'session':{'control_step':0,'actors':[]},'inventory':10,'inputs':'[]'},
                     terrain='a',bricks='b',objects='c')
        validate_restart(initial,copy.deepcopy(initial))
        for key in ['terrain','bricks','objects']:
            changed=copy.deepcopy(initial);changed[key]='changed'
            with self.assertRaises(ValueError):validate_restart(initial,changed)
        changed=copy.deepcopy(initial);changed['state']['inventory']=9
        with self.assertRaises(ValueError):validate_restart(initial,changed)
        changed=copy.deepcopy(initial);changed['state']['inputs']='old command'
        with self.assertRaises(ValueError):validate_restart(initial,changed)
