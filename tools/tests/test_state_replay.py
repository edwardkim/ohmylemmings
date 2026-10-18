import copy
import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location('state_replay', Path(__file__).parents[1] / 'state_replay.py')
replay = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(replay)


class StateReplayTests(unittest.TestCase):
    def config(self):
        return dict(width=32, height=32, minutes=5, total=1, rate=50,
                    entrances=[[0, 0]], pause_glitch=True, faller_three=True,
                    steps=70, inputs=[], level='synthetic')

    def test_canonical_ascii_sorted_compact_and_no_newline(self):
        self.assertEqual(replay.canonical({'z': '한', 'a': [1, True]}), b'{"a":[1,true],"z":"\\ud55c"}')
        with self.assertRaises(ValueError):
            replay.canonical({'x': 1.0})

    def test_input_order_types_commands_and_future_boundaries(self):
        c = self.config()
        c['inputs'] = [dict(control_step=4, sequence=0, command='Pause')]
        replay.validate_config(c)
        for invalid in [
            [dict(control_step=4, sequence=0, command='Unknown')],
            [dict(control_step=70, sequence=0, command='Pause')],
            [dict(control_step=True, sequence=0, command='Pause')],
            c['inputs'] * 2,
        ]:
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                replay.validate_config(dict(c, inputs=invalid))

    def test_mask_size_and_binary_values_are_checked(self):
        c = self.config()
        with self.assertRaises(ValueError):
            replay.make_header(c, b'\0', {'engine': 'test'})
        with self.assertRaises(ValueError):
            replay.make_header(c, bytes([2]) * 1024, {'engine': 'test'})

    def trace(self):
        c = self.config()
        c['steps'] = 0
        h = replay.make_header(c, bytes(1024), {'engine': 'test'})
        state = dict(control_step=0, iteration=0, paused=False, seconds_remaining=300,
                     clock_frame=0, glitch_pause_iterations=0, release_rate=50,
                     spawn_countdown=20, entrances_open=False, released=0,
                     alive=0, removed=0, actors=[], events=[])
        return replay.make_trace(h, [state])

    def test_match_is_explicitly_limited_to_supported_fields(self):
        t = self.trace()
        result = replay.compare(t, copy.deepcopy(t))
        self.assertEqual(result['status'], 'MATCH_SUPPORTED')
        self.assertIn('objects', result['unverified'])

    def test_missing_or_bad_hash_is_unverified_not_a_match(self):
        for change in ['missing', 'hash', 'count', 'type']:
            t = self.trace()
            if change == 'missing':
                del t['header']['input_sha256']
            elif change == 'hash':
                t['header']['input_sha256'] = '0' * 64
            elif change == 'count':
                t['snapshots'] = []
            else:
                t['snapshots'][0]['iteration'] = False
            with self.subTest(change=change):
                self.assertEqual(replay.compare(self.trace(), t)['status'], 'UNVERIFIED')

    def test_first_difference_reports_boundary_and_field(self):
        t = self.trace()
        other = copy.deepcopy(t)
        other['snapshots'][0]['spawn_countdown'] = 19
        result = replay.compare(t, other)
        self.assertEqual(result['status'], 'DIFFERENT')
        self.assertEqual(result['control_step'], 0)
        self.assertEqual(result['field'], '/snapshots/0/spawn_countdown')

    def test_mask_difference_reports_coordinate_and_keeps_full_masks(self):
        t = self.trace()
        mask = bytearray(1024)
        mask[35] = 1
        other = copy.deepcopy(t)
        other['header'] = replay.make_header(self.config() | {'steps': 0}, bytes(mask), {'engine': 'test'})
        result = replay.compare(t, other)
        self.assertEqual(result['coordinate'], [3, 1])
        self.assertEqual(result['field'], '/header/terrain')
        self.assertIn('reference_mask_base64', result)
        self.assertIn('candidate_mask_base64', result)

    def test_engine_provenance_is_required_but_not_compared_as_state(self):
        t = self.trace()
        other = copy.deepcopy(t)
        other['header']['provenance'] = {'engine': 'another'}
        self.assertEqual(replay.compare(t, other)['status'], 'MATCH_SUPPORTED')
        del other['header']['provenance']
        self.assertEqual(replay.compare(t, other)['status'], 'UNVERIFIED')

    def test_first_difference_after_pause_includes_neighboring_states(self):
        t = self.trace()
        config = self.config() | {'steps': 2, 'inputs': [
            dict(control_step=0, sequence=0, command='Pause'),
            dict(control_step=1, sequence=0, command='Resume')]}
        t['header'] = replay.make_header(config, bytes(1024), {'engine': 'test'})
        initial = t['snapshots'][0]
        t['snapshots'] += [initial | dict(control_step=1, paused=True),
                           initial | dict(control_step=2, iteration=1, clock_frame=1)]
        other = copy.deepcopy(t)
        other['snapshots'][2]['clock_frame'] = 2
        result = replay.compare(t, other)
        self.assertEqual(result['control_step'], 2)
        self.assertEqual(result['field'], '/snapshots/2/clock_frame')
        self.assertEqual(len(result['neighboring_snapshots']['reference']), 2)

    def test_input_file_identity_mismatch_is_unverified(self):
        t = self.trace()
        other = copy.deepcopy(t)
        other['header']['input_files'] = [dict(name='LEVEL009.DAT', size=12, sha256='1'*64)]
        self.assertEqual(replay.compare(t, other)['status'], 'UNVERIFIED')


if __name__ == '__main__':
    unittest.main()
