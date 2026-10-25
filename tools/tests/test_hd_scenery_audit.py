import copy
import unittest

from tools.hd_scenery_audit import inspect_catalog, load_manifests, pixel_stats


class SceneryAuditTests(unittest.TestCase):
    def test_active_repository_catalog_has_all_world_materials_and_object_phases(self):
        result = inspect_catalog(load_manifests())
        self.assertEqual((result['object_profiles'], result['object_frames'], result['materials'], result['specials']),
                         (54, 540, 5, 4))
    def fixture(self):
        return [dict(atlases=[dict(path='art.png', width=4, height=4, alpha_cutoff=8)],
                     frames=[dict(key={'object': [0, 2, n]}, atlas=0, rect=[0, 0, 4, 4],
                                  display=[8, 8], anchor=[0, 0]) for n in range(2)])]

    def test_every_object_phase_is_required_and_duplicates_are_rejected(self):
        manifests = self.fixture()
        before = copy.deepcopy(manifests)
        result = inspect_catalog(manifests, {(0, 2): 2}, complete_world=False)
        self.assertEqual(result['object_profiles'], 1)
        self.assertEqual(result['object_frames'], 2)
        self.assertEqual(manifests, before)
        for edit in ['missing', 'duplicate', 'unknown']:
            bad = copy.deepcopy(manifests)
            if edit == 'missing':
                bad[0]['frames'].pop()
            elif edit == 'duplicate':
                bad[0]['frames'].append(copy.deepcopy(bad[0]['frames'][0]))
            else:
                bad[0]['frames'][0]['key']['object'][1] = 15
            with self.assertRaises(ValueError):
                inspect_catalog(bad, {(0, 2): 2}, complete_world=False)

    def test_invalid_crop_or_display_is_rejected_and_density_is_reported(self):
        result = inspect_catalog(self.fixture(), {(0, 2): 2}, complete_world=False)
        self.assertEqual(result['rows'][0]['density'], .5)
        for rect, display in [([3, 0, 2, 4], [8, 8]), ([0, 0, 4, 4], [0, 8])]:
            bad = self.fixture()
            bad[0]['frames'][0].update(rect=rect, display=display)
            with self.assertRaises(ValueError):
                inspect_catalog(bad, {(0, 2): 2}, complete_world=False)

    def test_pixel_audit_distinguishes_flat_paint_from_shading_and_transparency(self):
        flat = bytes([132, 150, 166, 255] * 16)
        self.assertEqual(pixel_stats(4, flat, [0, 0, 4, 4], 8)['colors'], 1)
        shaded = bytes(v for n in range(16) for v in [100, 100, 100, n*17])
        self.assertGreater(pixel_stats(4, shaded, [0, 0, 4, 4], 8)['colors'], 10)
        self.assertEqual(pixel_stats(4, bytes(64), [0, 0, 4, 4], 8)['visible_samples'], 0)
