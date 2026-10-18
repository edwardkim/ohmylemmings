import importlib.util
from pathlib import Path
from fractions import Fraction
import unittest

SPEC = importlib.util.spec_from_file_location('dos_timing', Path(__file__).parents[1] / 'dos_timing.py')
timing = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(timing)


class DosTimingTests(unittest.TestCase):
    def test_vga_and_free_running_pit_are_distinct_timer_paths(self):
        video = Fraction(2190197, 31250)
        gui = timing.control_rate(False, 13450, video)
        unlocked = timing.control_rate(True, 16384, video)
        self.assertEqual(gui, Fraction(2190197, 156250))
        self.assertEqual(unlocked, Fraction(1193182, 81920))
        self.assertNotEqual(gui, unlocked)
        with self.assertRaises(ValueError):
            timing.control_rate(True, 0, video)

    def test_video_quantization_accepts_observed_pit_cadence_but_rejects_drift(self):
        anchors = [(35, 244), (54, 335), (60, 364), (100, 557), (170, 894), (340, 1712)]
        video = Fraction(2190197, 31250)
        rate = timing.control_rate(True, 16384, video)
        self.assertEqual(timing.check_cadence(anchors, video, rate)['status'], 'MATCH_VIDEO_QUANTIZATION')
        self.assertEqual(timing.check_cadence(anchors, video, video/5)['status'], 'DIFFERENT')
        self.assertEqual(timing.check_cadence([], video, rate)['status'], 'UNVERIFIED')

    def test_exact_five_frame_spacing_has_zero_residual(self):
        anchors = [(35, 224), (54, 319), (60, 349), (100, 549), (170, 899), (340, 1749)]
        result = timing.check_cadence(anchors, Fraction(2190197, 31250), Fraction(2190197, 156250))
        self.assertEqual(result['status'], 'MATCH_VIDEO_QUANTIZATION')
        self.assertEqual(result['maximum_residual_video_frames'], '0')
        # The measured 5000-cycle GUI run has one extra video frame by step 340.
        # Preserve that observation as a difference, rather than widening a pass.
        delayed = anchors[:-1]+[(340, 1750)]
        self.assertEqual(timing.check_cadence(delayed, Fraction(2190197, 31250),
                                            Fraction(2190197, 156250))['status'], 'DIFFERENT')
