import copy
import json
from pathlib import Path
import tempfile
import unittest
from tools import check_adlib_stream as stream

class AdlibStreamToolTests(unittest.TestCase):
    def setUp(self):
        self.state=dict(step=0,channels=[[0]*20 for _ in range(9)],globals=[16384,0,9,255,1],flags=[0]*4,registers=[0]*256,writes=[[176,0],[160,35],[176,41]])
    def test_whole_state_and_ordered_writes_match(self):
        stream.validate_record(self.state,stream.record(self.state))
    def test_write_order_is_significant(self):
        expected=stream.record(self.state);actual=copy.deepcopy(self.state);actual['writes'].reverse()
        with self.assertRaises(ValueError):stream.validate_record(actual,expected)
    def test_every_channel_and_shadow_register_is_significant(self):
        expected=stream.record(self.state)
        for key,index in [('channels',8),('registers',255),('globals',4),('flags',3)]:
            actual=copy.deepcopy(self.state)
            if key=='channels':actual[key][index][19]=1
            else:actual[key][index]+=1
            with self.subTest(key=key),self.assertRaises(ValueError):stream.validate_record(actual,expected)
    def test_boolean_is_not_an_integer_trace_field(self):
        self.state['step']=False
        with self.assertRaises(ValueError):stream.validate_record(self.state,stream.record(self.state))
    def test_synthetic_inputs_and_step_counts_are_pinned(self):
        fixture=json.loads(stream.FIXTURE.read_text());scenarios=stream.scenarios()
        self.assertEqual(len(scenarios),10);self.assertEqual(sum(len(v['records']) for v in fixture['scenarios'].values()),877)
        self.assertEqual(set(fixture['scenarios']),{name for name,_,_ in scenarios})
        for name,data,sequence in scenarios:
            expected=fixture['scenarios'][name]
            self.assertEqual(len(data),22125);self.assertEqual(expected['image_sha256'],stream.hashlib.sha256(data).hexdigest());self.assertEqual(expected['actions'],sequence)
            self.assertEqual(len(expected['records']),sum(count for _,count in stream.actions(sequence)))
    def test_failed_rust_trace_is_not_an_empty_success(self):
        with tempfile.TemporaryDirectory() as p:
            path=Path(p)/'bad.bin';path.write_bytes(b'bad')
            # Use a real failed command without building anything in Python-only CI.
            from unittest.mock import patch
            with patch.object(stream,'BINARY',Path('/usr/bin/false')):
                with self.assertRaises(ValueError):list(stream.trace(path,'m0:1'))
    def test_fixture_hash_is_pinned_in_provenance(self):
        manifest=json.loads((stream.ROOT/'assets/manifests/adlib_stream.json').read_text())
        self.assertEqual(manifest['synthetic_fixture']['sha256'],stream.hashlib.sha256(stream.FIXTURE.read_bytes()).hexdigest())

if __name__=='__main__':unittest.main()
