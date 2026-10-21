import copy
import unittest
from tools.g11_review import callback_summary, readiness


class G11ReviewTests(unittest.TestCase):
    def test_callback_budget_uses_exact_sixty_hz_and_rejects_invalid_samples(self):
        row = callback_summary([16_000_000, 16_666_666, 16_666_667])
        self.assertEqual(row['over_budget'], 1)
        self.assertEqual(row['max_ns'], 16_666_667)
        for values in [[], [0], [-1], [True], [float('nan')]]:
            with self.assertRaises(ValueError): callback_summary(values)

    def test_average_fps_and_zero_underruns_do_not_waive_callback_overruns(self):
        web = dict(callback_cost_ns=[[1], [22_600_000], [1]],
                   completion_deadline_observed=True, interval_memory_peak_observed=True)
        result = readiness(web)
        self.assertFalse(result['g11_ready'])
        self.assertEqual(result['unmet'], ['web-callback-budget'])

    def test_missing_deadline_and_peak_memory_evidence_remain_unmet(self):
        web = dict(callback_cost_ns=[[1], [1], [1]],
                   completion_deadline_observed=False, interval_memory_peak_observed=False)
        self.assertEqual(readiness(web)['unmet'], ['web-completion-deadline', 'web-interval-memory'])

    def test_all_required_observations_are_needed_for_a_ready_verdict(self):
        web = dict(callback_cost_ns=[[1], [1], [1]],
                   completion_deadline_observed=True, interval_memory_peak_observed=True)
        self.assertTrue(readiness(web)['g11_ready'])
        for values in [web['callback_cost_ns'][:2], [], [[], [1], [1]]]:
            bad = copy.deepcopy(web); bad['callback_cost_ns'] = values
            with self.assertRaises(ValueError): readiness(bad)

    def test_saved_ready_flag_cannot_override_computed_findings(self):
        from tools.g11_review import validate_verdict
        web = dict(callback_cost_ns=[[1], [22_600_000], [1]],
                   completion_deadline_observed=False, interval_memory_peak_observed=False)
        expected = readiness(web)
        validate_verdict(dict(expected, followup='https://github.com/edwardkim/ohmylemmings/issues/111'), web)
        for field, value in [('g11_ready', True), ('unmet', []), ('followup', '')]:
            bad = dict(expected, followup='https://github.com/edwardkim/ohmylemmings/issues/111'); bad[field] = value
            with self.assertRaises(ValueError): validate_verdict(bad, web)
