import json
from pathlib import Path
import shutil
import tempfile
import unittest

from tools.license_policy import check

ROOT = Path(__file__).resolve().parents[2]


class LicensePolicyTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        import tomllib
        members = tomllib.loads((ROOT / 'Cargo.toml').read_text())['workspace']['members']
        names = ['Cargo.toml', 'Cargo.lock', 'LICENSE', 'THIRD_PARTY_NOTICES.txt',
                 'assets/manifests/license_inventory.json', 'assets/manifests/reference_sources.json',
                 'crates/lemmings-opl/LICENSE', 'tools/audio/reference/LICENSE'] + [n + '/Cargo.toml' for n in members]
        names += [p.relative_to(ROOT).as_posix() for p in (ROOT / 'third_party').rglob('*') if p.is_file()]
        for name in names:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / name, path)

    def test_authorized_mit_and_all_existing_dependency_terms_are_preserved(self):
        result = check(self.root)
        self.assertEqual(result['project_license'], 'MIT')
        self.assertEqual(result['inherited_crates'], 8)
        self.assertEqual(result['cargo_dependencies'], 18)

    def test_missing_or_modified_third_party_notice_is_rejected(self):
        path = self.root / 'third_party/lemmix__Licence.txt'
        path.write_bytes(path.read_bytes() + b'changed')
        with self.assertRaises(ValueError): check(self.root)

    def test_dependency_change_requires_reviewed_inventory_update(self):
        path = self.root / 'Cargo.lock'
        path.write_text(path.read_text().replace('name = "sdl2"', 'name = "unreviewed-sdl2"'))
        with self.assertRaises(ValueError): check(self.root)

    def test_lgpl_crate_cannot_be_silently_relicensed(self):
        path = self.root / 'crates/lemmings-opl/Cargo.toml'
        path.write_text(path.read_text().replace('LGPL-2.1-or-later', 'MIT'))
        with self.assertRaises(ValueError): check(self.root)

    def test_bundled_or_root_license_cannot_drift_from_reviewed_bytes(self):
        for name in ['LICENSE', 'THIRD_PARTY_NOTICES.txt']:
            path = self.root / name
            original = path.read_bytes()
            path.write_bytes(original + b'changed')
            with self.assertRaises(ValueError): check(self.root)
            path.write_bytes(original)
