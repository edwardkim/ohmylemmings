import unittest

from tools.audio_timing import cadence, decode_audio_sample


class AudioTimingTests(unittest.TestCase):
    def test_uses_monotonic_irq_counter_instead_of_game_budget(self):
        raw = bytearray(0x2200)
        raw[0x1c] = 247
        raw[0x1fcd] = 3
        raw[0x2010] = 255
        sample = decode_audio_sample(raw)
        self.assertEqual(sample['audio_irq_counter_u8'], 247)
        self.assertEqual(sample['irq_counter_u8'], 3)

    def test_wraps_monotonic_counter_while_game_budget_resets(self):
        samples = [dict(released_u8=1, adlib_selected_u8=255,
                        pc_flags=[0] * 5, host_elapsed_ns=i * 100_000_000,
                        audio_irq_counter_u8=(250 + 7 * i) & 255,
                        irq_counter_u8=i % 5) for i in range(91)]
        result = cadence(samples)
        self.assertEqual(result['irqs'], 630)
        self.assertEqual(result['measured_hz'], 70)
        samples[1]['adlib_selected_u8'] = 0
        with self.assertRaises(ValueError):
            cadence(samples)

    def test_requires_sufficient_bounded_intervals(self):
        samples = [dict(released_u8=1, adlib_selected_u8=255,
                        pc_flags=[0] * 5, host_elapsed_ns=i * 1_000_000_000,
                        audio_irq_counter_u8=i * 70 & 255) for i in range(30)]
        with self.assertRaises(ValueError):
            cadence(samples)
