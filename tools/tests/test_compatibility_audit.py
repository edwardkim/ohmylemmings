"""Reject incomplete or stale compatibility certificates without original assets."""
import copy
import json
from pathlib import Path
import tempfile
import unittest

from tools.compatibility_audit import check_pin, read_coverage, validate_coverage, validate_requirements


class CompatibilityAuditTests(unittest.TestCase):
    def test_missing_or_mutated_evidence_is_rejected(self):
        import hashlib
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / 'proof').write_bytes(b'proof')
            pin = dict(path='proof', bytes=5, sha256=hashlib.sha256(b'proof').hexdigest())
            check_pin(root, pin)
            for key, value in [('bytes', 4), ('sha256', '0' * 64), ('path', 'absent')]:
                with self.subTest(key=key), self.assertRaises(ValueError):
                    check_pin(root, dict(pin, **{key: value}))

    def test_all_comp_details_need_evidence_and_current_test_anchors(self):
        from tools.compatibility_audit import REQUIRED_DETAILS
        details = [dict(id=key, evidence=['e'], tests=[dict(path='test.rs', name='boundary')],
                        product=['product.rs'], verdict='PASS-SCOPED') for key in REQUIRED_DETAILS]
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / 'test.rs').write_text('#[test]\nfn boundary() {}')
            (root / 'product.rs').write_text('product')
            validate_requirements(details, {'e'}, {'test.rs', 'product.rs'}, root)
            mutations = [details[:-1], details + [details[0]]]
            for field, value in [('evidence', []), ('tests', []), ('product', []),
                                 ('verdict', 'pending'), ('evidence', ['unknown']),
                                 ('tests', [dict(path='test.rs', name='missing')])]:
                broken = copy.deepcopy(details)
                broken[0][field] = value
                mutations.append(broken)
            for broken in mutations:
                with self.assertRaises(ValueError):
                    validate_requirements(broken, {'e'}, {'test.rs', 'product.rs'}, root)

    def test_full_catalog_requires_each_success_loss_and_three_real_restarts(self):
        audit = json.loads(Path('assets/manifests/compatibility_audit.json').read_text())
        validate_coverage(read_coverage(audit['coverage'], Path('.')), Path('.'))
        for field in ['solutions', 'initial']:
            broken = copy.deepcopy(read_coverage(audit['coverage'], Path('.')))
            broken[field].pop()
            with self.assertRaises(ValueError):
                validate_coverage(broken, Path('.'))
        for field in ['native', 'web', 'restarts']:
            broken = copy.deepcopy(read_coverage(audit['coverage'], Path('.')))
            broken['solutions'][0][field].pop()
            with self.assertRaises(ValueError):
                validate_coverage(broken, Path('.'))

    def test_changed_input_goal_state_or_restart_attempt_cannot_pass(self):
        audit = json.loads(Path('assets/manifests/compatibility_audit.json').read_text())
        for field, value in [('input_sha256', '0' * 64), ('state_maps_sha256', '0' * 64),
                             ('final', {}), ('steps', 1)]:
            broken = copy.deepcopy(read_coverage(audit['coverage'], Path('.')))
            broken['solutions'][0][field] = value
            with self.assertRaises(ValueError):
                validate_coverage(broken, Path('.'))
        broken = copy.deepcopy(read_coverage(audit['coverage'], Path('.')))
        broken['solutions'][0]['web'][0]['attempts'].pop()
        with self.assertRaises(ValueError):
            validate_coverage(broken, Path('.'))

    def test_wrong_restart_level_or_different_second_attempt_is_rejected(self):
        coverage = read_coverage(json.loads(Path('assets/manifests/compatibility_audit.json').read_text())['coverage'], Path('.'))
        broken = copy.deepcopy(coverage)
        broken['solutions'][0]['restarts'][0]['transitions'][0]['target'] = 30
        with self.assertRaises(ValueError):
            validate_coverage(broken, Path('.'))
        broken = copy.deepcopy(coverage)
        broken['solutions'][0]['web'][0]['attempts'][1]['frame_hashes_sha256'] = '0' * 64
        with self.assertRaises(ValueError):
            validate_coverage(broken, Path('.'))
        broken = copy.deepcopy(coverage)
        broken['totals']['native_replays'] -= 1
        with self.assertRaises(ValueError):
            validate_coverage(broken, Path('.'))
