import copy
import unittest
from tools.performance import summarize, validate_samples, budgets, select_cases


class PerformanceTests(unittest.TestCase):
    def test_thirty_samples_use_nearest_rank_p95_and_per_operation_units(self):
        result = summarize(list(range(1,31)), operations=2)
        self.assertEqual(result['median_ns_per_op'], 7.75)
        self.assertEqual(result['p95_ns_per_op'], 14.5)
        self.assertEqual(result['max_ns_per_op'], 15)
        self.assertAlmostEqual(result['operations_per_second'], 1e9/7.75)

    def test_bad_or_insufficient_samples_cannot_be_certified(self):
        for values in [[], [1]*29, [1]*31, [1]*29+[0], [1]*29+[-1], [1]*29+[True], [1]*29+[float('nan')]]:
            with self.assertRaises(ValueError): summarize(values, 1)
        for operations in [0,-1,True]:
            with self.assertRaises(ValueError): summarize([1]*30,operations)

    def test_each_measured_run_requires_identical_work_and_output(self):
        rows=[dict(ns=100, operations=10, digest='a') for _ in range(33)]
        self.assertEqual(len(validate_samples(rows,10)),30)
        for key,value in [('digest','changed'),('operations',9),('ns',0)]:
            broken=copy.deepcopy(rows);broken[12][key]=value
            with self.assertRaises(ValueError):validate_samples(broken,10)
        with self.assertRaises(ValueError):validate_samples(rows[:-1],10)

    def test_dos_and_reference_wall_rates_are_not_game_clock_seconds(self):
        value=budgets(60,44100,512)
        self.assertAlmostEqual(value['dos_tick_ns'],1e9*156250/2190197)
        self.assertAlmostEqual(value['reference_tick_ns'],1e9/17)
        self.assertAlmostEqual(value['audio_quantum_ns'],1e9*512/44100)
        for args in [(0,44100,512),(60,0,512),(60,44100,0)]:
            with self.assertRaises(ValueError):budgets(*args)

    def test_workloads_keep_original_ids_and_bounded_recorded_inputs(self):
        cases=select_cases()
        self.assertTrue({'fun-01','fun-22','tricky-14','taxing-15','mayhem-22'} <= {c['id'] for c in cases})
        for c in cases:
            self.assertGreater(c['steps'],0)
            self.assertLessEqual(c['start']+c['steps'],c['final_control'])
            self.assertEqual(c['phase'],'Won')

    def test_ci_timer_preserves_exit_status_and_records_whole_command_scope(self):
        import subprocess
        import sys
        import json
        result=subprocess.run([sys.executable,'tools/performance/ci_shell.py','-c','exit 7'],capture_output=True,text=True)
        self.assertEqual(result.returncode,7)
        row=json.loads(result.stdout.removeprefix('PERF_COMMAND '))
        self.assertEqual(row['returncode'],7)
        self.assertGreater(row['wall_ns'],0)
        self.assertIn('capture',row['scope'])

    def test_make_uses_timed_shell_without_changing_command_output(self):
        import subprocess
        import sys
        result=subprocess.run(['make','-f','-','SHELL='+sys.executable,'.SHELLFLAGS=tools/performance/ci_shell.py -c'],input='check:\n\t@echo timed-shell-smoke\n',text=True,capture_output=True)
        self.assertEqual(result.returncode,0,result.stderr)
        self.assertIn('timed-shell-smoke',result.stdout)
        self.assertIn('PERF_COMMAND ',result.stdout)
