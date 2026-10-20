import copy
import unittest
from tools import check_audio_output as check

class AudioOutputTests(unittest.TestCase):
    def test_sdl_probe_requires_actual_consumption_nonzero_pcm_and_queue_bound(self):
        good=dict(status='PASS',driver='dummy',queued_bytes=2048,consumed_bytes=1024,cleared_bytes=0,queue_limit_frames=3072,nonzero_frames=30,submitted_frames=2048,mute_hide_generation=True)
        check.validate_sdl(good)
        for key,value in [('driver','other'),('consumed_bytes',0),('consumed_bytes',3000),('nonzero_frames',0),('queued_bytes',6146),('cleared_bytes',1),('mute_hide_generation',False),('submitted_frames',True)]:
            bad=dict(good);bad[key]=value
            with self.assertRaises(ValueError):check.validate_sdl(bad)
    def test_browser_requires_every_native_pcm_hash_and_real_context_evidence(self):
        expected=[dict(name='x',frames=1,pcm_sha256='a'*64)]
        good=dict(status='PASS',cases=[dict(name='x',frames=1,sha256='a'*64)],offline_context=True,source_delivery=dict(license=True,archive=True,wasm_pair=True),mocks=dict(bounded=True,blocked=True,activation_race=True,mute_hide_resume=True,volume=True,clock_jump=True,context_recreation=True),live=dict(state='running',nonzero=20,queued_nodes=3,queue_seconds=0.05),worklet_required=False,secure_context=False)
        check.validate_browser(good,expected,False)
        for key,value in [('cases',[]),('offline_context',False),('live',dict(good['live'],nonzero=0)),('live',dict(good['live'],queued_nodes=9)),('mocks',{}),('secure_context',True)]:
            bad=copy.deepcopy(good);bad[key]=value
            with self.assertRaises(ValueError):check.validate_browser(bad,expected,False)
    def test_original_evidence_cannot_be_exported_outside_ignored_directory(self):
        with self.assertRaises(ValueError):check.guard_original(check.ROOT/'tools/new-audio-evidence')
    def test_fixture_rejects_duplicates_missing_cases_and_boolean_frame_counts(self):
        import json
        data=json.loads(check.FIXTURE.read_text());check.validate_fixture(data)
        for cases in [data['cases'][:-1],data['cases']+[data['cases'][0]],[dict(data['cases'][0],frames=True),*data['cases'][1:]]]:
            with self.assertRaises(ValueError):check.validate_fixture(dict(data,cases=cases))
    def test_pcm_shape_and_silence_are_checked(self):
        with self.assertRaises(ValueError):check.pcm_info(b'\0')
        row=check.pcm_info(b'\0\0\xff\x7f\0\x80');self.assertEqual((row['frames'],row['peak'],row['nonzero']),(3,32768,2))

if __name__=='__main__':unittest.main()
