import unittest
from tools.web_deadlines import frame_summary, memory_summary


class WebDeadlineTests(unittest.TestCase):
    def test_short_callback_can_miss_timestamp_based_completion_budget(self):
        row = frame_summary([[100, 115, 118], [200, 201, 203]])
        self.assertEqual(row['callback_over_budget'], 0)
        self.assertEqual(row['completion_misses'], 1)
        self.assertAlmostEqual(row['start_delay_ms']['max'], 15)
        self.assertAlmostEqual(row['completion_late_ms']['max'], 18-1000/60)

    def test_long_gaps_are_separate_from_callback_completion(self):
        row = frame_summary([[0, 1, 2], [50, 51, 52]])
        self.assertEqual(row['completion_misses'], 0)
        self.assertEqual(row['timestamp_gaps_over_two_periods'], 1)
        for rows in [[], [[0, 3, 2]], [[0, 0, float('nan')]], [[10, 10, 11], [5, 5, 6]]]:
            with self.assertRaises(ValueError): frame_summary(rows)

    def test_sampled_memory_peak_is_not_replaced_by_end_heap(self):
        row = memory_summary([{'at_ms':0,'usedSize':10,'totalSize':30},
                              {'at_ms':100,'usedSize':25,'totalSize':40},
                              {'at_ms':200,'usedSize':15,'totalSize':40}], 100, 200, 64)
        self.assertEqual(row['js_used_sampled_peak_bytes'], 25)
        self.assertEqual(row['js_used_last_sample_bytes'], 15)
        self.assertEqual(row['max_sample_gap_ms'], 100)
        self.assertFalse(row['continuous_js_peak_observed'])
        self.assertEqual(row['wasm_committed_peak_bytes'], 200)
        self.assertEqual(row['recorder_reserved_bytes'], 64)

    def test_missing_or_backwards_memory_evidence_cannot_pass(self):
        for samples in [[], [{'at_ms':0,'usedSize':10,'totalSize':5}],
                        [{'at_ms':1,'usedSize':1,'totalSize':2},{'at_ms':0,'usedSize':1,'totalSize':2}]]:
            with self.assertRaises(ValueError): memory_summary(samples,100,200,64)
        with self.assertRaises(ValueError): memory_summary([{'at_ms':0,'usedSize':1,'totalSize':2}],200,100,64)

    def test_recorded_ready_flag_cannot_erase_a_completion_miss(self):
        from tools.web_deadlines import validate_ready
        good=frame_summary([[0,1,2]])
        bad=frame_summary([[0,15,18]])
        rows=[good,bad,good]
        validate_ready(rows, {'web_deadline_ready':False,'followup':'https://github.com/edwardkim/ohmylemmings/issues/113'})
        with self.assertRaises(ValueError):validate_ready(rows, {'web_deadline_ready':True})
        with self.assertRaises(ValueError):validate_ready([good,good], {'web_deadline_ready':True})
        for field in ['g11_ready','g12_entry_ready']:
            with self.assertRaises(ValueError):
                validate_ready(rows, {'web_deadline_ready':False,'followup':'https://github.com/edwardkim/ohmylemmings/issues/113',field:True})
