import base64
import copy
import unittest
from tools.web_journey import validate_journey


def encoded(data): return base64.b64encode(bytes(data)).decode()


class WebJourneyTests(unittest.TestCase):
    def fixture(self):
        attempts=[]
        maps={name:encoded([0,0]) for name in ['terrain','bricks','objects']}
        for i,phase in enumerate(['Won','Lost','Won','Playing']):
            initial=dict(version=1,session=dict(control_step=0,iteration=0,paused=False,released=0,actors=[]),
                nuke=dict(is_nuked=False,assigning=False,next_index=0,max_lemmings=3),completion=dict(phase='Playing'),inputs='[]')
            final=copy.deepcopy(initial);final['session'].update(control_step=1,iteration=1,released=1,actors=[dict(id=0)])
            final['completion'].update(phase=phase,original_total=3,lemming_count=1 if phase=='Lost' else 3,required=2,rescued=3 if phase=='Won' else 0,rescued_percent=100 if phase=='Won' else 0)
            final['nuke'].update(is_nuked=phase=='Lost',max_lemmings=1 if phase=='Lost' else 3)
            view=dict(level=2 if i==3 else 1,camera=0,skill='Digger',right=False,dragging=False,pointer='None',held='None',error=False)
            row=lambda s:dict(state=copy.deepcopy(s),changes={name:[] for name in maps})
            attempts.append(dict(level=view['level'],reason=['synthetic','restart','continue','continue'][i],initial=dict(state=initial,view=view,maps=copy.deepcopy(maps)),
                rows=[row(initial),row(final)],events=[],frames=[],final=dict(state=copy.deepcopy(final),view=copy.deepcopy(view),maps=copy.deepcopy(maps),inputs='0 0 Nuke\n' if i==1 else '')))
        rgb=encoded(bytes(192000));rgba=encoded(bytes([0,0,0,255])*64000)
        for a in attempts:
            for i,row in enumerate(a['rows']):
                a['events'].append(dict(step=i,op='Frame',args=[i]))
                a['frames'].append(dict(index=i,step=i,state=copy.deepcopy(row['state']),view=copy.deepcopy(a['initial']['view']),rgb=rgb,canvas=rgba))
        return dict(version=1,original=False,attempts=attempts)

    def test_complete_journey_has_fresh_attempts_and_all_rescued(self):
        self.assertEqual(validate_journey(self.fixture())['states'],8)

    def test_rank_identity_cannot_be_lost_or_confused_with_fun_replay(self):
        for rank in ['tricky', 'invalid', 1]:
            data=self.fixture();data['attempts'][0]['difficulty']=rank
            with self.assertRaises(ValueError):validate_journey(data)
        data=self.fixture();data['attempts'][0]['initial']['view']['level']=30
        with self.assertRaises(ValueError):validate_journey(data)

    def test_missing_duplicate_out_of_order_boundaries_are_rejected(self):
        for mode in ['missing','duplicate','order']:
            data=self.fixture();rows=data['attempts'][0]['rows']
            if mode=='missing':rows.pop(0)
            elif mode=='duplicate':rows.insert(1,copy.deepcopy(rows[0]))
            else:rows.reverse()
            with self.assertRaises(ValueError):validate_journey(data)

    def test_partial_rescue_wrong_outcome_or_transition_cannot_pass(self):
        for field,value in [('phase','Lost'),('rescued_percent',99),('rescued',2)]:
            data=self.fixture();data['attempts'][0]['rows'][-1]['state']['completion'][field]=value;data['attempts'][0]['final']['state']['completion'][field]=value
            with self.assertRaises(ValueError):validate_journey(data)
        data=self.fixture();data['attempts'][2]['level']=2
        with self.assertRaises(ValueError):validate_journey(data)

    def test_map_changes_must_be_ordered_match_old_bytes_and_end_at_final(self):
        for changes in [[[0,1,0]],[[1,0,1],[0,0,1]],[[0,0,1],[0,0,1]],[[2,0,1]],[[0,0,1]]]:
            data=self.fixture();data['attempts'][0]['rows'][1]['changes']['terrain']=changes
            with self.assertRaises(ValueError):validate_journey(data)
        data=self.fixture();data['attempts'][0]['rows'][1]['changes']['terrain']=[[0,0,1]]
        data['attempts'][0]['final']['maps']['terrain']=encoded([1,0])
        # A successful restart must recover the pristine initial terrain.
        self.assertEqual(validate_journey(data)['states'],8)

    def test_old_population_clock_and_button_memory_cannot_leak(self):
        for target,key,value in [('session','iteration',1),('session','paused',True),('session','actors',[dict(id=0)]),('nuke','is_nuked',True)]:
            data=self.fixture();data['attempts'][1]['initial']['state'][target][key]=value
            with self.assertRaises(ValueError):validate_journey(data)
        for key,value in [('right',True),('held','Some(1)'),('pointer','Some((5, 5))')]:
            data=self.fixture();data['attempts'][1]['initial']['view'][key]=value
            with self.assertRaises(ValueError):validate_journey(data)

    def test_frame_markers_events_and_next_ids_are_required_to_match(self):
        data=self.fixture();data['attempts'][0]['events']=[dict(step=1,op='Frame',args=[0])]
        with self.assertRaises(ValueError):validate_journey(data)
        data=self.fixture();data['attempts'][3]['rows'][-1]['state']['session']['actors'][0]['id']=7
        with self.assertRaises(ValueError):validate_journey(data)

    def test_canvas_pixels_and_opacity_cannot_be_forged(self):
        for name,value in [('rgb',encoded(bytes([1])*192000)),('canvas',encoded(bytes(256000)))]:
            data=self.fixture();data['attempts'][0]['frames'][0][name]=value
            with self.assertRaises(ValueError):validate_journey(data)

    def test_original_media_cannot_be_written_outside_ignored_evidence(self):
        from tools.check_web_journey import compare_native
        from pathlib import Path
        data=self.fixture();data['original']=True
        with self.assertRaisesRegex(ValueError,'ignored evidence'):compare_native(data,Path('/tmp/not-original-evidence'),assets=Path('/tmp/no-assets'))

    def test_boolean_counters_are_not_integer_boundaries(self):
        data=self.fixture();data['attempts'][0]['rows'][0]['state']['session']['control_step']=False
        with self.assertRaises(ValueError):validate_journey(data)
        data=self.fixture();data['version']=True
        with self.assertRaises(ValueError):validate_journey(data)
