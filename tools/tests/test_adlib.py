import copy
from pathlib import Path
import unittest
from tools.check_adlib import check_original,synthetic,validate

class AdlibCatalogTests(unittest.TestCase):
    def test_complete_synthetic_catalog_and_level_order_pass(self):
        image,expected=synthetic();self.assertEqual(len(image),22125);self.assertEqual(validate(expected,expected)['music'],21)
    def test_missing_last_track_effect_or_level_cannot_pass(self):
        for key in ['music','effects','level_tracks']:
            _,expected=synthetic();actual=copy.deepcopy(expected);actual[key].pop()
            with self.assertRaises(ValueError):validate(actual,expected)
    def test_duplicate_or_reordered_ids_cannot_pass(self):
        for key in ['music','effects']:
            _,expected=synthetic();actual=copy.deepcopy(expected);actual[key][0],actual[key][1]=actual[key][1],actual[key][0]
            with self.assertRaises(ValueError):validate(actual,expected)
            actual=copy.deepcopy(expected);actual[key][1]['index']=0
            with self.assertRaises(ValueError):validate(actual,expected)
    def test_changed_header_channel_bar_and_effect_fields_cannot_pass(self):
        for key in ['header_offset','sample_rate_factor','instrument_offset','wait_cycles','channel_offsets','first_bar_offsets']:
            _,expected=synthetic();actual=copy.deepcopy(expected);row=actual['music'][20]
            if isinstance(row[key],list):row[key][0]+=1
            else:row[key]+=1
            with self.assertRaises(ValueError):validate(actual,expected)
        actual=copy.deepcopy(expected);actual['effects'][17]['data_offset']+=1
        with self.assertRaises(ValueError):validate(actual,expected)
    def test_boolean_numbers_extra_fields_and_wrong_level_cycle_cannot_pass(self):
        _,expected=synthetic()
        for mutate in [lambda d:d['music'][0].update(index=False),lambda d:d.update(extra=1),lambda d:d['level_tracks'].__setitem__(20,d['level_tracks'][0])]:
            actual=copy.deepcopy(expected);mutate(actual)
            with self.assertRaises(ValueError):validate(actual,expected)
    def test_original_evidence_guard_runs_before_reading_assets(self):
        with self.assertRaisesRegex(ValueError,'ignored'):
            check_original(Path('/tmp/nonexistent-original'),Path('/tmp/public-adlib'))
