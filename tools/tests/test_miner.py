from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from check_miner import collect, replay_text
from observe_miner_dos import decode_miner, decode_miner_sample
from verify_miner import latest_complete_state


class MinerTests(unittest.TestCase):
    def test_truncated_memory_is_only_terminal_after_child_exit_and_cannot_invent_a_state(self):
        previous=dict(remaining_miners=0)
        self.assertIs(decode_miner_sample(b'',previous,True),previous)
        for last,exited in [(previous,False),(None,True),(None,False)]:
            with self.assertRaisesRegex(ValueError,'incomplete live DOS sample'):
                decode_miner_sample(b'',last,exited)
        with self.assertRaisesRegex(ValueError,'incomplete live DOS sample'):
            decode_miner_sample(b'partial',previous,True)
    def test_live_observer_uses_complete_lines_during_a_concurrent_trace_write(self):
        with tempfile.TemporaryDirectory() as temporary:
            path=Path(temporary)/'trace.jsonl'
            path.write_text('{"control_step":')
            self.assertIsNone(latest_complete_state(path))
            path.write_text('{"control_step":80}\n{"control_step":81,"actors":[')
            self.assertEqual(latest_complete_state(path),dict(control_step=80))
            path.write_text('{"control_step":80}\n{"control_step":81}\n')
            self.assertEqual(latest_complete_state(path),dict(control_step=81))
    def test_miner_trace_retains_deletion_validation_when_inventory_is_added(self):
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary)
            (root/'trace.jsonl').write_text('{"control_step":0}\n{"control_step":1}\n')
            (root/'basher.jsonl').write_text('{"remaining":0}\n{"remaining":0}\n')
            (root/'miner.jsonl').write_text('{"remaining":1}\n{"remaining":0}\n')
            (root/'terrain-initial.bin').write_bytes(b'\x01\x00')
            (root/'terrain-final.bin').write_bytes(b'\x00\x00')
            for deleted in ['1','0,0']:
                (root/'digger.jsonl').write_text('{"cleared":[]}\n{"cleared":['+deleted+']}\n')
                with self.assertRaisesRegex(ValueError,'invalid or repeated terrain deletion'):
                    collect(root)
            (root/'digger.jsonl').write_text('{"cleared":[]}\n{"cleared":[0]}\n')
            self.assertEqual(collect(root)[1]['miner'],dict(remaining=0))

    def test_replay_keeps_pause_and_three_job_replacements_in_boundary_order(self):
        inputs=[(80,0,'Pause',0),(80,1,'AssignMiner',3),(80,2,'AssignDigger',3),(80,3,'AssignBasher',3),(90,0,'Resume',0)]
        self.assertEqual(replay_text(inputs),'80 0 Pause\n80 1 AssignMiner 3\n80 2 AssignDigger 3\n80 3 AssignBasher 3\n90 0 Resume\n')

    def test_missing_inventory_boundary_cannot_be_treated_as_complete_replay(self):
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary)
            for name,text in [('trace.jsonl','{"control_step":0}\n'),('digger.jsonl','{"cleared":[]}\n'),('basher.jsonl','{"remaining":0}\n'),('miner.jsonl','')]:
                (root/name).write_text(text)
            (root/'terrain-initial.bin').write_bytes(b'\x01')
            (root/'terrain-final.bin').write_bytes(b'\x01')
            with self.assertRaisesRegex(ValueError,'Miner inventory'):
                collect(root)
            (root/'miner.jsonl').write_text('{"remaining":1}\n')
            self.assertEqual(collect(root)[0]['miner'],dict(remaining=1))

    def test_read_only_decoder_distinguishes_all_three_skill_quantities(self):
        data=bytearray(0x2200);data[0x79],data[0x7c],data[0x7f]=2,1,7
        result=decode_miner(data)
        self.assertEqual(result['remaining_miners'],1)
        self.assertEqual(result['remaining_bashers'],2)
        self.assertEqual(result['remaining_diggers'],7)


if __name__=='__main__':
    unittest.main()
