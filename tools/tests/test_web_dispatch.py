import unittest
from tools.web_dispatch import correlate_trace, summarize_pairs


class WebDispatchTests(unittest.TestCase):
    def test_delayed_ack_is_separate_from_renderer_queue_and_callback_cost(self):
        events = [
            dict(name='issue113-before-run',pid=1,tid=2,ts=100000,args=dict(data=dict(startTime=100))),
            dict(name='Scheduler::BeginImplFrame',pid=1,tid=3,ts=200100,args=dict(args=dict(sequence_number=7,frame_time_us=200000))),
            dict(name='ProxyImpl::DidReceiveCompositorFrameAckOnImplThread',pid=1,tid=3,ts=215000),
            dict(name='SendBeginMainFrame',ph='I',pid=1,tid=3,ts=215100,args=dict(main_frame_pipeline=dict(main_frame_id=8,begin_frame_id=dict(sequence_number=7)))),
            dict(name='ProxyMain::BeginMainFrame',ph='X',pid=1,tid=2,ts=215200,dur=3500,args=dict(main_frame_pipeline=dict(main_frame_id=8))),
            dict(name='FireAnimationFrame',ph='X',pid=1,tid=2,ts=215900,dur=2700),
            dict(name='MinorGC',ph='X',pid=9,tid=2,ts=210000,dur=4000),
        ]
        row=correlate_trace([[0,16,18.5]],events,200)[0]
        self.assertAlmostEqual(row['send_delay_ms'],15.1)
        self.assertAlmostEqual(row['renderer_queue_ms'],0.1)
        self.assertAlmostEqual(row['pre_callback_ms'],0.8)
        self.assertAlmostEqual(row['ack_to_send_ms'],0.1)
        self.assertEqual(row['gc_overlap_ms'],0)
        self.assertGreater(row['completion_late_ms'],0)

    def test_missing_trace_match_is_reported_instead_of_guessed(self):
        events=[dict(name='issue113-before-run',pid=1,tid=2,ts=100000,args=dict(data=dict(startTime=100)))]
        row=correlate_trace([[-1,18,19]],events,200)[0]
        self.assertFalse(row['matched'])
        self.assertTrue(row['pre_window_timestamp'])

    def run_record(self, miss=False):
        return dict(seconds=60.01,ticks=841,hashes=['same']*5,underruns=0,visibility_events=0,
                    callback_ms=[[0,15,18] if miss else [0,1,2]],presents=842)

    def test_three_pairs_keep_baseline_misses_and_require_all_candidate_deadlines(self):
        before=[self.run_record(True) for _ in range(3)]
        after=[self.run_record() for _ in range(3)]
        result=summarize_pairs(before,after)
        self.assertEqual(result['before_misses'],[1,1,1])
        self.assertTrue(result['web_deadline_ready'])
        after[2]=self.run_record(True)
        self.assertFalse(summarize_pairs(before,after)['web_deadline_ready'])

    def test_missing_pairs_or_changed_results_cannot_pass(self):
        before=[self.run_record() for _ in range(3)]
        for after in [before[:2],[self.run_record() for _ in range(3)]]:
            if len(after)==3:after[1]['hashes'][0]='changed'
            with self.assertRaises(ValueError):summarize_pairs(before,after)
