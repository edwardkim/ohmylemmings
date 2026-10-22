import json
from pathlib import Path
import tempfile
import unittest

from tools.release_version import parse_version, workspace_version, package_name, check_repository


class ReleaseVersionTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name)
        (self.root/'Cargo.toml').write_text('[workspace]\nmembers=["app"]\n[workspace.package]\nversion="0.1.0"\n')
        (self.root/'app').mkdir()
        (self.root/'app/Cargo.toml').write_text('[package]\nname="game"\nversion.workspace=true\n')
        (self.root/'mydocs/releases').mkdir(parents=True)
        (self.root/'mydocs/releases/issue-1_v0.1.0.md').write_text('Release notes')
        (self.root/'Cargo.lock').write_text('[[package]]\nname="game"\nversion="0.1.0"\n')

    def test_stable_and_numbered_preview_versions_are_distinct(self):
        self.assertFalse(parse_version('0.1.0')['prerelease'])
        for label in ['alpha','beta','rc']:
            self.assertTrue(parse_version('1.2.3-'+label+'.1')['prerelease'])
        for bad in ['v0.1.0','01.2.3','1.2','1.2.3+build','1.2.3-rc','1.2.3-rc.0','1.2.3-rc.01','1.2.3-dev.1','1.2.3/../bad']:
            with self.assertRaises(ValueError):parse_version(bad)

    def test_tag_requires_exact_workspace_version(self):
        self.assertEqual(check_repository(self.root,'v0.1.0')['version'],'0.1.0')
        for tag in ['0.1.0','v0.1.1','v0.1.0-rc.1']:
            with self.assertRaises(ValueError):check_repository(self.root,tag)

    def test_all_package_paths_derive_from_workspace_version(self):
        self.assertEqual(workspace_version(self.root),'0.1.0')
        expected={'linux-x86_64':'tar.gz','web':'zip','windows-x86_64':'zip','macos-universal2':'zip'}
        for target,suffix in expected.items():
            self.assertEqual(package_name(target,'0.2.0-rc.1'),f'ohmylemmings-0.2.0-rc.1-{target}.{suffix}')
        with self.assertRaises(ValueError):package_name('../bad','0.1.0')

    def test_missing_or_ambiguous_versioned_release_notes_are_rejected(self):
        notes=self.root/'mydocs/releases/issue-1_v0.1.0.md'
        notes.unlink()
        with self.assertRaises(ValueError):check_repository(self.root)
        notes.write_text('notes')
        notes.with_name('issue-2_v0.1.0.md').write_text('other')
        with self.assertRaises(ValueError):check_repository(self.root)

    def test_independent_member_or_stale_lock_version_is_rejected(self):
        (self.root/'app/Cargo.toml').write_text('[package]\nname="game"\nversion="0.1.0"\n')
        with self.assertRaises(ValueError):check_repository(self.root)
        (self.root/'app/Cargo.toml').write_text('[package]\nname="game"\nversion.workspace=true\n')
        (self.root/'Cargo.lock').write_text('[[package]]\nname="game"\nversion="0.0.9"\n')
        with self.assertRaises(ValueError):check_repository(self.root)
