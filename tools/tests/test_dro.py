import struct
import unittest
from tools import dro


def recording(pairs, duration=10):
    return struct.pack('<8sHHIIBBBBBB',b'DBRAWOPL',2,0,len(pairs),duration,0,0,0,2,3,2)+bytes([0xa0,0xb0])+bytes(v for pair in pairs for v in pair)


class DroTests(unittest.TestCase):
    def test_decodes_millisecond_delays_and_bank_without_inventing_writes(self):
        pairs=[(0,7),(1,33),(2,9),(0,8),(1,1),(1,33)]
        row=dro.parse(recording(pairs))
        self.assertEqual(row['writes'],[[0,160,7],[0,176,33],[10,160,8],[10,176,1],[10,176,33]])
        self.assertEqual(dro.parse(recording([(128,9),(3,1)],512))['writes'],[[0,416,9]])

    def test_rejects_truncated_invalid_version_index_duration_and_trailing_data(self):
        good=recording([(0,7),(2,9)])
        for data in [good[:25],good[:-1],good+b'\0',good[:8]+b'\1'+good[9:],recording([(4,7)],0),recording([(2,0)],10)]:
            with self.assertRaises(ValueError):dro.parse(data)

    def test_note_comparison_preserves_repeats_and_rejects_wrong_notes_or_timing(self):
        actual=[[1000,160,7],[1000,176,33],[1010,176,1],[1010,160,8],[1010,176,33]]
        reference=[[0,160,7],[0,176,33],[14,176,1],[14,160,8],[14,176,33]]
        result=dro.compare_notes(actual,reference,channels=(0,),tolerance_ms=5)
        self.assertEqual(result['matched_notes'],2)
        self.assertEqual(result['channels'][0]['max_relative_error_ms'],4)
        for data in [reference[:-1],reference[:-1]+[[14,176,34]],[[ms*2,reg,value] for ms,reg,value in reference]]:
            with self.assertRaises(ValueError):dro.compare_notes(actual,data,channels=(0,),tolerance_ms=5)


if __name__=='__main__':unittest.main()
