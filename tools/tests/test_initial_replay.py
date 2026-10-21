import copy
import io
import struct
import unittest
from tools.initial_replay import BrowserActivity, catalog_ids, first_difference, frame_stream, validate_rows


class InitialReplayTests(unittest.TestCase):
    def test_checked_heartbeats_keep_long_runs_alive_without_extending_total_deadline(self):
        activity=BrowserActivity(0)
        for now in [100,200,300]:
            self.assertTrue(activity.note('/__initial/heartbeat',now))
            self.assertFalse(activity.timed_out(now+100,450))
        self.assertTrue(activity.note('/__initial/heartbeat',400))
        self.assertTrue(activity.timed_out(451,450))
        self.assertTrue(activity.timed_out(521,1000))

    def test_unknown_browser_requests_do_not_reset_stall_deadline(self):
        activity=BrowserActivity(0)
        self.assertFalse(activity.note('/unknown',110))
        self.assertTrue(activity.timed_out(121,1000))

    def catalog(self):
        return [{'id':f'{rank}-{n:02}'} for rank in ['fun','tricky','taxing','mayhem'] for n in range(1,31)]

    def test_catalog_requires_each_of_120_distinct_rank_ids(self):
        rows=self.catalog();self.assertEqual(len(catalog_ids(rows)),120)
        for broken in [rows[:-1],rows+[rows[0]],rows[:-1]+[dict(id='mayhem-31')]]:
            with self.assertRaisesRegex(ValueError,'catalog'):catalog_ids(broken)

    def test_first_difference_identifies_nested_actor_field_and_missing_boundary(self):
        a={'state':{'actors':[{'x':10}]},'terrain':'abc'};b=copy.deepcopy(a);b['state']['actors'][0]['x']=11
        self.assertEqual(first_difference(a,b),'$.state.actors[0].x')
        self.assertEqual(first_difference([a],[a,a]),'$.length')
        self.assertIsNone(first_difference(a,copy.deepcopy(a)))
        self.assertEqual(first_difference({'x':1},{'x':True}),'$.x')

    def test_stream_requires_fixed_rgb_size_order_and_explicit_terminator(self):
        def record(step,data):return struct.pack('<QI',step,len(data))+data
        end=struct.pack('<QI',2**64-1,0)
        raw=record(0,b'abc')+record(7,b'def')+end+b'{"frames":2}\n'
        stream=io.BytesIO(raw);self.assertEqual(list(frame_stream(stream,3)),[(0,b'abc'),(7,b'def')]);self.assertEqual(stream.read(),b'{"frames":2}\n')
        for broken in [raw[:8],record(0,b'ab')+end,record(1,b'abc')+record(0,b'def')+end,record(0,b'abc')]:
            with self.assertRaises(ValueError):list(frame_stream(io.BytesIO(broken),3))

    def test_rows_require_every_probe_and_frozen_state_after_result(self):
        def row(probe,step,phase='Playing'):
            return dict(probe=probe,state=dict(session=dict(control_step=step),completion=dict(phase=phase)),terrain='a',bricks='b',objects='c')
        rows=[row(0,0),row(1,1,'Lost'),row(2,1,'Lost')];validate_rows(rows,2)
        for broken in [rows[:-1],[rows[0],rows[2]], [rows[0],rows[1],row(2,2,'Lost')]]:
            with self.assertRaises(ValueError):validate_rows(broken,2)
        changed=copy.deepcopy(rows);changed[2]['terrain']='changed'
        with self.assertRaisesRegex(ValueError,'frozen'):validate_rows(changed,2)
