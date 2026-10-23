import hashlib
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
import zipfile

from tools.release_package import WEB_FILES, create_package, verify_package
from tools.web_build import SOURCE_REQUIRED


class ReleasePackageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.root = self.base / 'source'
        self.runtime = self.base / 'runtime'
        self.root.mkdir()
        for name in SOURCE_REQUIRED:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b'public source\n')
        (self.root / 'Cargo.toml').write_text('[workspace.package]\nversion = "0.1.0"\n')
        self.runtime.mkdir()
        for name in WEB_FILES:
            path = self.runtime / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b'\0asm\1\0\0\0' if name.endswith('.wasm') else b'web runtime\n')
            if not name.startswith('pkg/'):
                source = self.root / 'apps/web/www' / name
                source.parent.mkdir(parents=True, exist_ok=True)
                source.write_bytes(path.read_bytes())
        # A source distribution has no Git metadata and must stay rebuildable.
        names = sorted(p.relative_to(self.root).as_posix() for p in self.root.rglob('*') if p.is_file())
        (self.root / 'SOURCE-MANIFEST.json').write_text(json.dumps([dict(path=n) for n in names]))
        self.binary = self.base / 'lemmings-desktop'
        elf = bytearray(64)
        elf[:6] = b'\x7fELF\x02\x01'
        elf[18:20] = (62).to_bytes(2, 'little')
        self.binary.write_bytes(elf)

    def package(self, kind, directory='dist'):
        return create_package(kind, self.runtime if kind == 'web' else self.binary,
                              self.base / directory, root=self.root)

    def test_web_allowlist_excludes_originals_and_includes_verified_source_and_hashes(self):
        (self.runtime / 'MAIN.DAT').write_bytes(b'private original')
        (self.runtime / 'performance').mkdir()
        (self.runtime / 'performance/raw.json').write_bytes(b'private run')
        (self.root / 'assets/local').mkdir(parents=True)
        (self.root / 'assets/local/secret.dat').write_bytes(b'private original')
        path = self.package('web')
        result = verify_package(path, root=self.root)
        self.assertEqual(result['target'], 'web')
        with zipfile.ZipFile(path) as archive:
            names = archive.namelist()
            self.assertNotIn('MAIN.DAT', names)
            self.assertFalse(any(n.startswith('performance/') for n in names))
            self.assertIn('audio-source.tar.gz', names)
            self.assertIn('LGPL-2.1-or-later.txt', names)
            info = json.loads(archive.read('audio-source.json'))
            self.assertEqual(info['wasm_sha256'], hashlib.sha256(archive.read('pkg/lemmings_web_bg.wasm')).hexdigest())
        self.assertTrue(result['development'])
        self.assertTrue(result['performance_qualification_deferred'])

    def test_new_hd_packages_deliver_complete_profile_on_both_targets(self):
        from tools.tests.test_hd_delivery import install_fixture
        install_fixture(self.root, self.runtime)
        for path in [self.runtime/'index.html',self.root/'apps/web/www/index.html']:
            path.write_bytes(b'<!doctype html><html><head></head><body></body></html>')
        # Source-export inventory must describe the fixture's newly added public files.
        names = sorted(p.relative_to(self.root).as_posix() for p in self.root.rglob('*')
                       if p.is_file() and 'assets/local/' not in p.relative_to(self.root).as_posix()
                       and p.name != 'SOURCE-MANIFEST.json')
        (self.root/'SOURCE-MANIFEST.json').write_text(json.dumps([dict(path=n) for n in names]))
        for kind in ['linux','web']:
            path=self.package(kind,'hd-'+kind)
            self.assertEqual(verify_package(path,root=self.root)['status'],'PASS')
            if kind == 'web':
                with zipfile.ZipFile(path) as archive: files={n:archive.read(n) for n in archive.namelist()}
                self.assertIn(b'name="game-presentation" content="hd"', files['index.html'])
            else:
                with tarfile.open(path) as archive: files={n:archive.extractfile(n).read() for n in archive.getnames()}
            self.assertEqual(files['HD-PRESENTATION'],b'hd\n')
            self.assertEqual(json.loads(files['HD-ASSETS.json'])['logical_keys'],6)

    def test_both_targets_deliver_project_license_and_all_third_party_notices(self):
        for kind in ['linux', 'web']:
            path = self.package(kind, 'licensed-' + kind)
            if kind == 'web':
                with zipfile.ZipFile(path) as archive:
                    files = {n: archive.read(n) for n in archive.namelist()}
            else:
                with tarfile.open(path) as archive:
                    files = {n: archive.extractfile(n).read() for n in archive.getnames()}
            self.assertEqual(json.loads(files['PACKAGE-MANIFEST.json'])['project_license'], 'MIT')
            for name in ['LICENSE', 'THIRD_PARTY_NOTICES.txt']:
                self.assertEqual(files[name], (self.root / name).read_bytes())

    def test_missing_notice_cannot_produce_either_package(self):
        (self.root / 'THIRD_PARTY_NOTICES.txt').unlink()
        for kind in ['linux', 'web']:
            with self.assertRaises(ValueError): self.package(kind, 'missing-' + kind)

    def test_rehashed_notice_replacement_still_fails_against_reviewed_source(self):
        from tools.release_package import archive_bytes, encoded, sha
        path = self.package('web')
        with zipfile.ZipFile(path) as archive:
            files = {name: archive.read(name) for name in archive.namelist()}
        files['THIRD_PARTY_NOTICES.txt'] = b'replacement that removed upstream terms'
        manifest = json.loads(files['PACKAGE-MANIFEST.json'])
        for pin in manifest['files']:
            pin.update(bytes=len(files[pin['path']]), sha256=sha(files[pin['path']]))
        files['PACKAGE-MANIFEST.json'] = encoded(manifest)
        files['SHA256SUMS'] = ''.join(f'{sha(b)}  {n}\n' for n,b in sorted(files.items()) if n != 'SHA256SUMS').encode()
        path.write_bytes(archive_bytes(files, 'web'))
        Path(str(path)+'.sha256').write_text(f'{sha(path.read_bytes())}  {path.name}\n')
        with self.assertRaisesRegex(ValueError, 'license differs'): verify_package(path, root=self.root)

    def test_same_inputs_produce_identical_archives_for_both_targets(self):
        for kind in ['linux', 'web']:
            first = self.package(kind, 'first-' + kind)
            second = self.package(kind, 'second-' + kind)
            self.assertEqual(first.read_bytes(), second.read_bytes())

    def test_native_binary_is_executable_and_source_is_delivered(self):
        path = self.package('linux')
        self.assertEqual(verify_package(path, root=self.root)['target'], 'linux-x86_64')
        with tarfile.open(path) as archive:
            self.assertEqual(archive.getmember('lemmings-desktop').mode, 0o755)
            self.assertIn('source.tar.gz', archive.getnames())

    def test_missing_symlinked_stale_or_wrong_architecture_runtime_fails(self):
        target = self.runtime / 'app.js'
        original = target.read_bytes()
        target.unlink()
        with self.assertRaises(ValueError): self.package('web')
        target.symlink_to(self.root / 'apps/web/www/app.js')
        with self.assertRaises(ValueError): self.package('web')
        target.unlink()
        target.write_bytes(original + b'stale')
        with self.assertRaises(ValueError): self.package('web')
        self.binary.write_bytes(b'not an ELF')
        with self.assertRaises(ValueError): self.package('linux')

    def test_existing_evidence_is_never_overwritten(self):
        path = self.package('web')
        data = path.read_bytes()
        with self.assertRaises(FileExistsError): self.package('web')
        self.assertEqual(path.read_bytes(), data)

    def test_changed_archive_bytes_fail_checksum(self):
        path = self.package('web')
        path.write_bytes(path.read_bytes() + b'tampered')
        with self.assertRaises(ValueError): verify_package(path, root=self.root)

    def test_internal_corruption_or_extra_files_fail_even_with_updated_archive_checksum(self):
        for change in ['app.js', 'MAIN.DAT']:
            path = self.package('web', 'changed-' + change)
            with zipfile.ZipFile(path) as archive:
                files = {name: archive.read(name) for name in archive.namelist()}
            files[change] = b'changed bytes'
            with zipfile.ZipFile(path, 'w') as archive:
                for name, data in files.items():
                    entry = zipfile.ZipInfo(name)
                    entry.create_system = 3
                    entry.external_attr = 0o100644 << 16
                    archive.writestr(entry, data)
            Path(str(path) + '.sha256').write_text(f'{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}\n')
            with self.assertRaises(ValueError): verify_package(path, root=self.root)

    def test_catalog_package_omits_duplicate_full_pack_and_reports_archive_identity(self):
        from unittest.mock import patch
        members={'catalog.hdi':b'catalog','atlas-000.hdp':b'page'}
        (self.runtime/'hd-assets').mkdir()
        (self.runtime/'hd-assets.hdp').write_bytes(b'whole')
        for name,data in members.items():(self.runtime/'hd-assets'/name).write_bytes(data)
        with patch('tools.hd_catalog.runtime_catalog',return_value=members),patch('tools.hd_assets.runtime_pack',return_value=b'whole'):
            path=self.package('web','catalog-dist')
            result=verify_package(path,root=self.root)
        raw=path.read_bytes();self.assertEqual(result['archive_bytes'],len(raw));self.assertEqual(result['sha256'],hashlib.sha256(raw).hexdigest())
        with zipfile.ZipFile(path) as archive:
            self.assertNotIn('hd-assets.hdp',archive.namelist())
            self.assertEqual(archive.read('hd-assets/catalog.hdi'),b'catalog')
