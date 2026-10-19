import copy
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_skill_interactions import matrix_digest, matrix_keys, compare_states


class SkillInteractionsTests(unittest.TestCase):
    def rows(self):
        return [dict(initial=a, skill=s, direction=d, variant=v, accepted=False,
                     remaining=1, action=a, y=100, frame=0, end=False, fallen=0,
                     new_digger=False, bricks=0, climber=False, floater=False,
                     float_index=0, walking_once=False, timer=0,
                     map_hash='0' * 16) for a, s, d, v in matrix_keys()]

    def test_complete_matrix_has_2640_distinct_ordered_cases(self):
        rows = self.rows()
        self.assertEqual(len(rows), 2640)
        self.assertEqual(len(matrix_digest(rows)), 64)
        changed = copy.deepcopy(rows)
        changed[0]['accepted'] = True
        self.assertNotEqual(matrix_digest(rows), matrix_digest(changed))

    def test_matrix_rejects_truncation_reordering_and_duplicate_keys(self):
        rows = self.rows()
        for malformed in [rows[:-1], rows + rows[:1], [rows[1], rows[0]] + rows[2:], rows[:1] + rows[:-1]]:
            with self.assertRaises(ValueError):
                matrix_digest(malformed)

    def test_matrix_rejects_wrong_schema_types_and_hash(self):
        for patch in [dict(accepted=1), dict(remaining=True), dict(timer=-1),
                      dict(map_hash='z' * 16), dict(map_hash=None), dict(direction=True),
                      dict(action='Unknown'), dict(extra=1)]:
            rows = self.rows()
            # True compares equal to the expected +1; identity checks alone miss it.
            rows[11 if 'direction' in patch else 0].update(patch)
            with self.assertRaises(ValueError):
                matrix_digest(rows)

    def test_comparison_rejects_missing_boundaries_and_reports_first_difference(self):
        compare_states([{'frame': 0}], [{'frame': 0}], 'fixture')
        with self.assertRaisesRegex(AssertionError, 'length'):
            compare_states([{}], [], 'fixture')
        with self.assertRaisesRegex(AssertionError, 'fixture.*boundary 1'):
            compare_states([{}, {'frame': 0}], [{}, {'frame': 1}], 'fixture')
