import copy
import unittest
from tools import check_audio

class AudioToolTests(unittest.TestCase):
    def test_pcm_hash_rejects_one_changed_sample(self):
        row=check_audio.pcm_info(b'\0\0\x01\0')
        with self.assertRaises(ValueError):check_audio.validate_pcm(b'\0\0\x02\0',row)
    def test_pcm_rejects_truncated_or_odd_byte_streams(self):
        with self.assertRaises(ValueError):check_audio.pcm_info(b'\0')
        row=check_audio.pcm_info(b'\0\0\x01\0')
        with self.assertRaises(ValueError):check_audio.validate_pcm(b'\0\0',row)
    def test_browser_result_requires_every_case_in_order(self):
        cases=[dict(name='a',expected=dict(frames=1,sha256='a'*64)),dict(name='b',expected=dict(frames=2,sha256='b'*64))]
        good=dict(status='PASS',cases=[dict(name=c['name'],**c['expected']) for c in cases])
        check_audio.validate_browser(good,cases)
        for rows in [good['cases'][:1],good['cases'][::-1],good['cases']+[good['cases'][0]]]:
            with self.assertRaises(ValueError):check_audio.validate_browser(dict(status='PASS',cases=rows),cases)
    def test_browser_numeric_fields_reject_booleans(self):
        cases=[dict(name='a',expected=dict(frames=1,sha256='a'*64))]
        with self.assertRaises(ValueError):check_audio.validate_browser(dict(status='PASS',cases=[dict(name='a',frames=True,sha256='a'*64)]),cases)
    def test_synthetic_inputs_cover_all_sample_rates_and_timing_paths(self):
        cases=check_audio.synthetic_cases();self.assertEqual(len({c['name'] for c in cases}),len(cases));self.assertEqual({c['rate'] for c in cases},{8000,44100,48000,49716,96000,192000});self.assertEqual({(c['num'],c['den']) for c in cases},{(2190197,31250),(596591,8192)})
    def test_original_output_must_remain_ignored(self):
        with self.assertRaises(ValueError):check_audio.guard_original(check_audio.ROOT/'tools/audio-invalid')
    def test_fixture_contains_only_hashes_and_synthetic_case_inputs(self):
        import json
        fixture=json.loads(check_audio.FIXTURE.read_text());self.assertEqual(set(fixture['cases']),{c['name'] for c in check_audio.synthetic_cases()});self.assertEqual(fixture['engine'],'Nuked-OPL3-1.8/rust-0.1.0-no_std')
        for row in fixture['cases'].values():self.assertEqual(set(row),{'frames','sha256','peak','nonzero'});self.assertEqual(len(row['sha256']),64)

if __name__=='__main__':unittest.main()
