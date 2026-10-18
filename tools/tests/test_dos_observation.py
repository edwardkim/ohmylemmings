import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location('dos_observation', Path(__file__).parents[1] / 'dos_observation.py')
dos = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(dos)


class DosObservationTests(unittest.TestCase):
    def test_exact_doubling_is_verified_before_reducing(self):
        rgb = bytes([40, 80, 120]) * (320 * 200)
        doubled = b''.join(b''.join(row[x:x+3]*2 for x in range(0, 960, 3))*2
                           for row in [rgb[y*960:(y+1)*960] for y in range(200)])
        self.assertEqual(dos.game_frame(640, 400, doubled), dos.game_frame(320, 200, rgb))
        bad = bytearray(doubled)
        bad[3] ^= 1
        with self.assertRaises(ValueError):
            dos.game_frame(640, 400, bytes(bad))
        hud = bytearray(doubled)
        hud[320*640*3] ^= 1
        self.assertEqual(dos.game_frame(640, 400, bytes(hud)), dos.game_frame(320, 200, rgb))

    def test_bad_dimensions_and_truncated_frames_are_rejected(self):
        for w, h, b in [(640, 350, bytes(640*350*3)), (320, 200, b''), (0, 0, b'')]:
            with self.subTest(w=w, h=h), self.assertRaises(ValueError):
                dos.game_frame(w, h, b)

    def test_only_hud_is_excluded_and_rgb565_is_explicit(self):
        rgb = bytes([255, 129, 66]) * (320*200)
        self.assertEqual(dos.game_frame(320, 200, rgb), bytes([31, 32, 8]) * (320*160))
        other = rgb[:320*160*3] + bytes(320*40*3)
        self.assertEqual(dos.game_frame(320, 200, rgb), dos.game_frame(320, 200, other))

    def test_pixel_difference_counts_pixels_and_reports_first_and_bounds(self):
        ref = bytes(320*160*3)
        got = bytearray(ref)
        got[(4*320+2)*3:(4*320+2)*3+3] = b'\1\2\3'
        got[(7*320+9)*3] = 1
        result = dos.pixel_difference(ref, bytes(got))
        self.assertEqual(result, dict(pixels=2, first=[2, 4], bounds=[2, 4, 10, 8]))
        self.assertEqual(dos.pixel_difference(ref, ref), dict(pixels=0, first=None, bounds=None))
        with self.assertRaises(ValueError):
            dos.pixel_difference(ref, b'')

    def test_visual_match_preserves_order_and_does_not_claim_internal_state(self):
        result = dos.match_anchors([(0, 'initial'), (54, 'fall'), (60, 'walk')],
                                   [(4, 'initial'), (8, 'initial'), (100, 'fall'), (124, 'walk')])
        self.assertEqual(result['status'], 'MATCH_VISUAL_ANCHORS')
        self.assertEqual([a['dos_frames'] for a in result['anchors']], [[4, 8], [100], [124]])
        self.assertIn('internal_state', result['unverified'])
        self.assertIn('tick_rate', result['unverified'])

    def test_missing_or_reversed_anchor_cannot_pass(self):
        for observed in [[(1, 'walk'), (2, 'fall')], [(2, 'fall')], []]:
            r = dos.match_anchors([(54, 'fall'), (60, 'walk')], observed)
            self.assertEqual(r['status'], 'UNVERIFIED_VISUAL_ANCHORS')
        with self.assertRaises(ValueError):
            dos.match_anchors([(60, 'walk'), (54, 'fall')], [])

    def test_repeated_runs_must_each_match_and_settings_must_be_identical(self):
        passed = dict(settings_sha256='settings', status='MATCH_VISUAL_ANCHORS', scene_sha256=['a', 'b'])
        self.assertEqual(dos.repeat_verdict([passed.copy() for _ in range(3)]), 'MATCH_THREE_RUNS')
        for rows in [[passed], [passed, passed, passed | dict(settings_sha256='other')],
                     [passed, passed, passed | dict(status='UNVERIFIED_VISUAL_ANCHORS')],
                     [passed, passed, passed | dict(scene_sha256=['a', 'c'])]]:
            self.assertEqual(dos.repeat_verdict(rows), 'UNVERIFIED')

    def test_upper_scope_never_hides_a_full_playfield_difference(self):
        frame = bytes(320*160*3)
        changed = bytearray(frame)
        changed[(110*320+250)*3] = 1
        a, b = dos.scene_signatures(frame), dos.scene_signatures(bytes(changed))
        self.assertEqual(a['upper_320x100'], b['upper_320x100'])
        self.assertNotEqual(a['full_320x160'], b['full_320x160'])
        with self.assertRaises(ValueError):
            dos.scene_signatures(b'')

    def test_scope_verdicts_are_recomputed_from_current_anchor_comparisons(self):
        full = dict(status='MATCH_VISUAL_ANCHORS', anchors=[dict(scene_sha256='full')])
        rows = [dict(settings_sha256='settings', status='MATCH_VISUAL_ANCHORS',
                     scene_sha256=['upper'], full_playfield=full) for _ in range(3)]
        self.assertEqual(dos.scope_verdicts(rows), dict(upper_view_status='MATCH_THREE_RUNS',
                                                      full_playfield_status='MATCH_THREE_RUNS'))
        rows[2] = rows[2] | dict(full_playfield=full | dict(status='UNVERIFIED_VISUAL_ANCHORS'))
        self.assertEqual(dos.scope_verdicts(rows), dict(upper_view_status='MATCH_THREE_RUNS',
                                                      full_playfield_status='UNVERIFIED'))
