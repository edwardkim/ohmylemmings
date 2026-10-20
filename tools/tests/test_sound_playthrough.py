import copy
import math
import struct
import tempfile
import unittest
import wave
from pathlib import Path
from tools import sound_playthrough as check


class SoundPlaythroughTests(unittest.TestCase):
    def test_pcm_metrics_reject_truncation_and_find_real_energy(self):
        with self.assertRaises(ValueError):
            check.metrics(b'\0', 44100)
        with self.assertRaises(ValueError):
            check.metrics(b'\0\0', True)
        data=struct.pack('<4h',0,100,-100,0)
        row=check.metrics(data,8000)
        self.assertEqual((row['frames'],row['peak'],row['nonzero']),(4,100,2))
        self.assertAlmostEqual(row['rms'],math.sqrt(5000))

    def test_trace_rejects_missing_reset_lost_commands_and_wrong_sample_clock(self):
        rows=[]
        for wall in range(1,581):
            generation=int(wall>=360)
            rows.append(dict(wall=wall,end=wall*44100*156250//2190197,
                sounds=dict(generation=generation,dropped=0,sequence_exhausted=False,events=[]),
                state=dict(completion=dict(phase='Lost' if 250<=wall<360 else 'Playing'))))
        for wall,command in [(18,['effect',2]),(38,['effect',1]),(58,['music',5]),(90,['effect',0]),(91,['effect',3]),(161,['effect',4]),(240,['effect',11]),(250,['stop'])]:
            rows[wall-1]['sounds']['events'].append(dict(sequence=wall,command=command,mapping_error=None))
        check.validate_trace(rows)
        for mutate in [lambda r:r.pop(),lambda r:r[359]['sounds'].update(generation=0),lambda r:r[54].update(end=1),lambda r:r[14]['sounds'].update(dropped=1),lambda r:r[90]['sounds']['events'].clear()]:
            bad=copy.deepcopy(rows);mutate(bad)
            with self.assertRaises(ValueError):
                check.validate_trace(bad)

    def test_wave_preserves_pcm_rate_and_channel_without_normalization(self):
        raw=struct.pack('<5h',0,1,-1,32767,-32768)
        with tempfile.TemporaryDirectory() as folder:
            path=Path(folder)/'clip.wav';row=check.write_wave(path,raw,48000)
            with wave.open(str(path),'rb') as clip:
                self.assertEqual((clip.getnchannels(),clip.getsampwidth(),clip.getframerate()),(1,2,48000))
                self.assertEqual(clip.readframes(5),raw)
            self.assertEqual(row['frames'],5)

    def test_dos_left_channel_is_bit_exact_without_mixing_or_gain(self):
        stereo=struct.pack('<6h',-32768,12000,32767,-12000,123,-999)
        self.assertEqual(check.left_channel(stereo,2),struct.pack('<3h',-32768,32767,123))
        for raw,channels in [(b'\0',2),(b'\0\0',2),(b'',0),(b'',True)]:
            with self.assertRaises(ValueError):check.left_channel(raw,channels)

    def test_listening_files_require_ignored_destination(self):
        with self.assertRaises(ValueError):
            check.guard_original(check.ROOT/'tools/listening')


if __name__=='__main__':
    unittest.main()
